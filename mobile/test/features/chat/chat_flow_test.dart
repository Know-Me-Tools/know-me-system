// TJ-ARCH-MOB-001 compliant
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:gen_ui_widgets/gen_ui_widgets.dart';

import 'package:knowme_poc/bridge/a2ui/a2ui_event.dart';
import 'package:knowme_poc/features/chat/domain/chat_repository.dart';
import 'package:knowme_poc/features/chat/domain/entities/chat_conversation.dart';
import 'package:knowme_poc/features/chat/domain/entities/chat_message.dart';
import 'package:knowme_poc/features/chat/presentation/providers/chat_repository_provider.dart';
import 'package:knowme_poc/features/chat/presentation/providers/chat_notifier.dart';

// A fake ONLY at the FFI edge: Rust-shaped A2uiEvents. Nothing internal mocked.
Stream<A2uiEvent> _cannedRun() async* {
  yield const RunStarted('run-1');
  yield const BlockEvent('run-1', TextBlock('Hello '));
  yield const BlockEvent('run-1', TextBlock('world'));
  yield const RunFinished('run-1');
}

/// A run that starts and emits, but never finishes — the live-stream state the
/// conversation-switch guard exists to protect.
Stream<A2uiEvent> _neverFinishingRun() async* {
  yield const RunStarted('run-live');
  yield const BlockEvent('run-live', TextBlock('partial'));
}

Stream<A2uiEvent> _semanticRun() async* {
  yield const RunStarted('run-semantic');
  yield const BlockEvent('run-semantic', ThinkingBlock('reason '));
  yield const BlockEvent('run-semantic', ThinkingBlock('continues'));
  yield const BlockEvent(
    'run-semantic',
    CitationBlock('policy.md', 'Keep reasoning out of answers.'),
  );
  yield const BlockEvent('run-semantic', TextBlock('Final '));
  yield const BlockEvent('run-semantic', TextBlock('answer'));
  yield const RunFinished('run-semantic');
}

void main() {
  test(
    'legacy persisted stream deltas coalesce without crossing rich blocks',
    () {
      final blocks = coalesceContentBlocks(const [
        TextBlock('Hello '),
        TextBlock('world'),
        DividerBlock(),
        TextBlock('Next'),
      ]);
      expect(blocks, hasLength(3));
      expect((blocks.first as TextBlock).text, 'Hello world');
      expect(blocks[1], isA<DividerBlock>());
    },
  );

  test('ChatNotifier folds an A2uiEvent stream into ContentBlocks', () async {
    final container = ProviderContainer();
    addTearDown(container.dispose);

    final notifier = container.read(chatProvider.notifier);
    await notifier.foldStream('msg-1', _cannedRun());

    final ChatState state = container.read(chatProvider);
    final msg = state.messages.firstWhere((m) => m.id == 'msg-1');
    expect(
      msg.isStreaming,
      isFalse,
      reason: 'RunFinished finalizes the message',
    );
    expect(msg.content, hasLength(1));
    expect((msg.content.single as TextBlock).text, 'Hello world');
  });

  test(
    'ChatNotifier folds reasoning, citation, and answer chunks without flattening semantics',
    () async {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      final notifier = container.read(chatProvider.notifier);
      await notifier.foldStream('msg-semantic', _semanticRun());

      final ChatState state = container.read(chatProvider);
      final msg = state.messages.firstWhere((m) => m.id == 'msg-semantic');
      expect(msg.isStreaming, isFalse);
      expect(msg.content, hasLength(3));
      expect((msg.content[0] as ThinkingBlock).text, 'reason continues');
      expect((msg.content[1] as CitationBlock).source, 'policy.md');
      expect(
        (msg.content[1] as CitationBlock).quote,
        'Keep reasoning out of answers.',
      );
      expect((msg.content[2] as TextBlock).text, 'Final answer');
    },
  );

  test(
    'resubmit creates a new branch with the selected prefix history',
    () async {
      final repo = _FakeChatRepository(
        messages: [
          _message('u-1', 'user', 'Remember the alpha code.', branchId: 'main'),
          _message('a-1', 'assistant', 'Alpha is saved.', branchId: 'main'),
          ChatMessage(
            id: 'u-2',
            role: 'user',
            content: const [
              ArtifactBlock(
                'attachment-1',
                'attachment',
                '{"name":"brief.pdf"}',
              ),
              TextBlock('Use the saved code.'),
            ],
            branchId: 'main',
            attachmentPaths: const ['/tmp/brief.pdf'],
          ),
          _message('a-2', 'assistant', 'Old answer.', branchId: 'main'),
          _message('u-other', 'user', 'Other branch', branchId: 'other'),
        ],
      );
      final container = ProviderContainer(
        overrides: [chatRepositoryProvider.overrideWithValue(repo)],
      );
      final subscription = container.listen(chatProvider, (_, _) {});
      addTearDown(container.dispose);
      addTearDown(subscription.close);

      final notifier = container.read(chatProvider.notifier);
      await notifier.load();
      await notifier.resubmitMessage('u-2');
      await _flushAsyncEvents();

      expect(repo.startRequests, hasLength(1));
      final request = repo.startRequests.single;
      expect(request.message, 'Use the saved code.');
      expect(request.branchId, isNot('main'));
      expect(request.attachmentPaths, const ['/tmp/brief.pdf']);
      expect(request.history.map((entry) => '${entry.role}:${entry.content}'), [
        'user:Remember the alpha code.',
        'assistant:Alpha is saved.',
      ]);

      final state = container.read(chatProvider);
      expect(
        state.messages.where((message) => message.id == 'a-2'),
        hasLength(1),
      );
      final resubmittedUser = state.messages.firstWhere(
        (message) => message.id == request.userMessageId,
      );
      expect(resubmittedUser.branchId, request.branchId);
      expect(resubmittedUser.attachmentPaths, const ['/tmp/brief.pdf']);
    },
  );

  test(
    'deleteResponse removes only the selected assistant branch tail',
    () async {
      final repo = _FakeChatRepository(
        messages: [
          _message('u-1', 'user', 'First', branchId: 'main'),
          _message('a-1', 'assistant', 'First answer', branchId: 'main'),
          _message('u-other', 'user', 'Other branch', branchId: 'other'),
          _message('u-2', 'user', 'Second', branchId: 'main'),
          _message('a-2', 'assistant', 'Second answer', branchId: 'main'),
        ],
      );
      final container = ProviderContainer(
        overrides: [chatRepositoryProvider.overrideWithValue(repo)],
      );
      final subscription = container.listen(chatProvider, (_, _) {});
      addTearDown(container.dispose);
      addTearDown(subscription.close);

      final notifier = container.read(chatProvider.notifier);
      await notifier.load();
      await notifier.deleteResponse('a-1');

      expect(repo.deletedIds, ['a-1', 'u-2', 'a-2']);
      final remainingIds = container
          .read(chatProvider)
          .messages
          .map((message) => message.id)
          .toList();
      expect(remainingIds, ['u-1', 'u-other']);
    },
  );

  test(
    'regenerate deletes the answer tail and reruns the saved user',
    () async {
      final repo = _FakeChatRepository(
        messages: [
          _message('u-1', 'user', 'Remember beta.', branchId: 'main'),
          _message('a-1', 'assistant', 'Beta saved.', branchId: 'main'),
          ChatMessage(
            id: 'u-2',
            role: 'user',
            content: const [TextBlock('Use beta.')],
            branchId: 'main',
            attachmentPaths: const ['/tmp/context.txt'],
          ),
          _message(
            'a-2',
            'assistant',
            'Old beta answer.',
            branchId: 'main',
            parentMessageId: 'u-2',
          ),
        ],
      );
      final container = ProviderContainer(
        overrides: [chatRepositoryProvider.overrideWithValue(repo)],
      );
      final subscription = container.listen(chatProvider, (_, _) {});
      addTearDown(container.dispose);
      addTearDown(subscription.close);

      final notifier = container.read(chatProvider.notifier);
      await notifier.load();
      await notifier.regenerateResponse('a-2');
      await _flushAsyncEvents();

      expect(repo.deletedIds, ['a-2']);
      expect(repo.startRequests, hasLength(1));
      final request = repo.startRequests.single;
      expect(request.message, 'Use beta.');
      expect(request.branchId, 'main');
      expect(request.userMessageId, 'u-2');
      expect(request.attachmentPaths, const ['/tmp/context.txt']);
      expect(request.history.map((entry) => '${entry.role}:${entry.content}'), [
        'user:Remember beta.',
        'assistant:Beta saved.',
      ]);
      expect(
        container
            .read(chatProvider)
            .messages
            .where((message) => message.id == 'a-2'),
        isEmpty,
      );
    },
  );

  test(
    'provenance from a ContextTelemetry event lands on the message',
    () async {
      final container = ProviderContainer();
      addTearDown(container.dispose);
      final notifier = container.read(chatProvider.notifier);

      Stream<A2uiEvent> runWithProvenance() async* {
        yield const RunStarted('run-prov');
        yield const ContextTelemetry('run-prov', {
          'kind': 'provenance',
          'provenance': {
            'agentId': 'orchestrator-agent',
            'providerId': 'anthropic',
            'modelId': 'claude-opus-4-8',
          },
        });
        yield const BlockEvent('run-prov', TextBlock('Answer.'));
        yield const RunFinished('run-prov');
      }

      await notifier.foldStream('msg-prov', runWithProvenance());

      final msg = container
          .read(chatProvider)
          .messages
          .firstWhere((m) => m.id == 'msg-prov');
      expect(msg.provenance['agentId'], 'orchestrator-agent');
      expect(msg.provenance['providerId'], 'anthropic');
      expect(msg.provenance['modelId'], 'claude-opus-4-8');
      // A provenance-only telemetry event must not overwrite context telemetry.
      expect(msg.contextTelemetry, isEmpty);
    },
  );

  test('sendSteer marks the turn and tells the model to keep the work', () async {
    final repo = _FakeChatRepository();
    final container = ProviderContainer(
      overrides: [chatRepositoryProvider.overrideWithValue(repo)],
    );
    final subscription = container.listen(chatProvider, (_, _) {});
    addTearDown(container.dispose);
    addTearDown(subscription.close);

    final notifier = container.read(chatProvider.notifier);
    await notifier.load();
    await notifier.sendSteer('Focus on the cost side instead.');
    await _flushAsyncEvents();

    final request = repo.startRequests.single;
    // The correction has to survive verbatim AND be framed as a correction —
    // the tag rides in the content because it is the only carrier every lane
    // forwards unchanged.
    expect(request.message, contains('Focus on the cost side instead.'));
    expect(request.message, contains('stopped your previous response'));

    final sent = container
        .read(chatProvider)
        .messages
        .firstWhere((message) => message.id == request.userMessageId);
    expect(sent.intent, 'steer');
  });

  test('sendResume continues after the stopped partial turn', () async {
    final repo = _FakeChatRepository(
      messages: [
        _message('u-1', 'user', 'Write a long guide.', branchId: 'main'),
        ChatMessage(
          id: 'a-1',
          role: 'assistant',
          content: [const TextBlock('Chapter one, partially wri')],
          branchId: 'main',
          runStatus: 'cancelled',
          statusMessage: 'Cancelled by user',
        ),
      ],
    );
    final container = ProviderContainer(
      overrides: [chatRepositoryProvider.overrideWithValue(repo)],
    );
    final subscription = container.listen(chatProvider, (_, _) {});
    addTearDown(container.dispose);
    addTearDown(subscription.close);

    final notifier = container.read(chatProvider.notifier);
    await notifier.load();
    await notifier.sendResume();
    await _flushAsyncEvents();

    final request = repo.startRequests.single;
    // This is what makes Resume free of Rust changes: history filters on
    // branchId and non-empty text only — there is NO runStatus filter — so the
    // cancelled partial turn is still there for the model to continue from.
    expect(request.history.map((entry) => '${entry.role}:${entry.content}'), [
      'user:Write a long guide.',
      'assistant:Chapter one, partially wri',
    ]);
    expect(request.message, contains('Continue your previous response'));

    final sent = container
        .read(chatProvider)
        .messages
        .firstWhere((message) => message.id == request.userMessageId);
    expect(sent.intent, 'resume');
  });

  test('an ordinary send carries no intent', () async {
    final repo = _FakeChatRepository();
    final container = ProviderContainer(
      overrides: [chatRepositoryProvider.overrideWithValue(repo)],
    );
    final subscription = container.listen(chatProvider, (_, _) {});
    addTearDown(container.dispose);
    addTearDown(subscription.close);

    final notifier = container.read(chatProvider.notifier);
    await notifier.load();
    await notifier.sendMessage('Just a question.');
    await _flushAsyncEvents();

    final request = repo.startRequests.single;
    expect(request.message, 'Just a question.');
    final sent = container
        .read(chatProvider)
        .messages
        .firstWhere((message) => message.id == request.userMessageId);
    expect(sent.intent, isNull);
  });

  test('renameConversation persists the new title', () async {
    // The Flutter sheet showed the title as static text with only a delete
    // button beside it, so a conversation kept whatever its first message
    // auto-titled it forever. React has had rename since it shipped.
    final repo = _FakeChatRepository();
    final container = ProviderContainer(
      overrides: [chatRepositoryProvider.overrideWithValue(repo)],
    );
    final subscription = container.listen(chatProvider, (_, _) {});
    addTearDown(container.dispose);
    addTearDown(subscription.close);

    final notifier = container.read(chatProvider.notifier);
    await notifier.load();
    final id = container.read(chatProvider).threadId;

    await notifier.renameConversation(id, '  Trip planning  ');

    // Trimmed, because a title padded with spaces reads as misaligned in the list.
    expect(repo.savedConversations.single.title, 'Trip planning');
    expect(
      container.read(chatProvider).conversations.first.title,
      'Trip planning',
    );
  });

  test('renameConversation ignores an empty title', () async {
    // An empty title would leave a blank row the user cannot tell apart from
    // its neighbours.
    final repo = _FakeChatRepository();
    final container = ProviderContainer(
      overrides: [chatRepositoryProvider.overrideWithValue(repo)],
    );
    final subscription = container.listen(chatProvider, (_, _) {});
    addTearDown(container.dispose);
    addTearDown(subscription.close);

    final notifier = container.read(chatProvider.notifier);
    await notifier.load();
    final before = container.read(chatProvider).conversations.first.title;

    await notifier.renameConversation(container.read(chatProvider).threadId, '   ');

    expect(repo.savedConversations, isEmpty);
    expect(container.read(chatProvider).conversations.first.title, before);
  });

  test('selectConversation refuses to switch while a run is streaming', () async {
    // Switching swaps state.messages out from under the driver still writing
    // blocks to a message id no longer in the visible list — the tokens go
    // nowhere and the turn is lost. React has always guarded this.
    final repo = _FakeChatRepository();
    final container = ProviderContainer(
      overrides: [chatRepositoryProvider.overrideWithValue(repo)],
    );
    final subscription = container.listen(chatProvider, (_, _) {});
    addTearDown(container.dispose);
    addTearDown(subscription.close);

    final notifier = container.read(chatProvider.notifier);
    await notifier.load();
    final original = container.read(chatProvider).threadId;
    // Drive the streaming state directly. Sending would race the fake's canned
    // run to completion, and the guard would then be tested against a turn that
    // had already finished — a test that passes for the wrong reason.
    await notifier.foldStream('streaming-1', _neverFinishingRun());
    expect(
      container.read(chatProvider).messages.any((message) => message.isStreaming),
      isTrue,
      reason: 'the guard is only meaningful against a live run',
    );

    await notifier.selectConversation('some-other-conversation');

    expect(container.read(chatProvider).threadId, original);
  });

  test('a failed submitInteraction ends the turn instead of spinning', () async {
    // The assistant placeholder is pushed into state BEFORE the call. An
    // uncaught throw left that bubble streaming forever with no way back —
    // this was the only action in the notifier without the guard its siblings
    // have, so the failure had no surface at all.
    final repo = _FakeChatRepository()..failInteraction = true;
    final container = ProviderContainer(
      overrides: [chatRepositoryProvider.overrideWithValue(repo)],
    );
    final subscription = container.listen(chatProvider, (_, _) {});
    addTearDown(container.dispose);
    addTearDown(subscription.close);

    final notifier = container.read(chatProvider.notifier);
    await notifier.load();
    await notifier.submitInteraction({'action': 'submit', 'value': 'yes'});
    await _flushAsyncEvents();

    final assistant = container
        .read(chatProvider)
        .messages
        .lastWhere((message) => message.role == 'assistant');
    expect(assistant.isStreaming, isFalse);
    expect(assistant.runStatus, 'error');
    expect(assistant.error?.code, 'interaction_submit_failed');
    expect(assistant.error?.retryable, isTrue);
  });
}

ChatMessage _message(
  String id,
  String role,
  String text, {
  required String branchId,
  String? parentMessageId,
}) => ChatMessage(
  id: id,
  role: role,
  content: [TextBlock(text)],
  branchId: branchId,
  parentMessageId: parentMessageId,
);

Future<void> _flushAsyncEvents() async {
  await Future<void>.delayed(Duration.zero);
  await Future<void>.delayed(Duration.zero);
}

class _StartTurnRequest {
  const _StartTurnRequest({
    required this.threadId,
    required this.message,
    required this.history,
    required this.branchId,
    required this.userMessageId,
    required this.attachmentPaths,
  });

  final String threadId;
  final String message;
  final List<ChatHistoryEntry> history;
  final String branchId;
  final String userMessageId;
  final List<String> attachmentPaths;
}

class _FakeChatRepository implements ChatRepository {
  _FakeChatRepository({List<ChatMessage> messages = const []})
    : _messages = {for (final message in messages) message.id: message};

  final _conversation = ChatConversation(
    id: 'thread-1',
    title: 'Test',
    createdAt: DateTime.fromMillisecondsSinceEpoch(0, isUtc: true),
    updatedAt: DateTime.fromMillisecondsSinceEpoch(0, isUtc: true),
  );
  final Map<String, ChatMessage> _messages;
  final List<_StartTurnRequest> startRequests = [];
  final List<String> deletedIds = [];

  @override
  Future<String> approveToolConfirmation({required String approvalId}) async =>
      'approval-run';

  @override
  Future<bool> cancelRun(String runId) async => true;

  @override
  Future<void> cancelModelOperation() async {}

  @override
  Future<ChatConversation> createConversation(String title) async =>
      _conversation;

  @override
  Future<void> deleteConversation(String id) async {}

  @override
  Future<void> deleteMessages(List<String> ids) async {
    deletedIds.addAll(ids);
    for (final id in ids) {
      _messages.remove(id);
    }
  }

  @override
  Stream<A2uiEvent> events(String runId) => Stream.fromIterable([
    RunStarted(runId),
    BlockEvent(runId, TextBlock('response $runId')),
    RunFinished(runId),
  ]);

  @override
  Future<String> errorMessage(Object error) async => error.toString();

  @override
  Future<List<ChatConversation>> listConversations() async => [_conversation];

  @override
  Future<List<ChatMessage>> listMessages(String conversationId) async =>
      _messages.values.toList(growable: false);

  @override
  Future<Map<String, dynamic>?> localRuntimeDiagnostics() async => const {};

  @override
  Future<void> prepareLocalModel() async {}

  @override
  Future<ResolvedRemoteContent> resolveRemoteContent(
    String url,
    RemoteContentKind kind,
  ) async => ResolvedRemoteContent(url: url, mime: 'text/plain');

  final savedConversations = <ChatConversation>[];

  @override
  Future<void> saveConversation(ChatConversation conversation) async {
    savedConversations.add(conversation);
  }

  @override
  Future<void> saveMessage(String conversationId, ChatMessage message) async {
    _messages[message.id] = message;
  }

  @override
  Future<String> startTurn({
    required String threadId,
    required String message,
    required List<ChatHistoryEntry> history,
    required String branchId,
    required String userMessageId,
    List<String> attachmentPaths = const [],
  }) async {
    final runId = 'run-${startRequests.length + 1}';
    startRequests.add(
      _StartTurnRequest(
        threadId: threadId,
        message: message,
        history: List<ChatHistoryEntry>.of(history),
        branchId: branchId,
        userMessageId: userMessageId,
        attachmentPaths: List<String>.of(attachmentPaths),
      ),
    );
    return runId;
  }

  /// Set to make the next `submitInteraction` throw, standing in for a dead
  /// FFI bridge or a rejected run.
  bool failInteraction = false;

  @override
  Future<String> submitInteraction({
    required String threadId,
    required String interactionJson,
    required List<ChatHistoryEntry> history,
    required String branchId,
  }) async {
    if (failInteraction) throw StateError('bridge is gone');
    return 'interaction-run';
  }

  @override
  Stream<Map<String, dynamic>> watchLocalModelStatus() => const Stream.empty();
}
