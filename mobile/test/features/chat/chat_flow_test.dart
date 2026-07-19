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

  @override
  Future<void> saveConversation(ChatConversation conversation) async {}

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

  @override
  Future<String> submitInteraction({
    required String threadId,
    required String interactionJson,
    required List<ChatHistoryEntry> history,
    required String branchId,
  }) async => 'interaction-run';

  @override
  Stream<Map<String, dynamic>> watchLocalModelStatus() => const Stream.empty();
}
