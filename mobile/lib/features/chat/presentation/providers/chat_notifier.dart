// TJ-ARCH-MOB-001 compliant
// ChatNotifier owns ALL ContentBlock mutations. It sends via the FFI facade,
// then folds the returned A2uiEvent stream into the assistant message using
// A2uiContentDriver. Riverpod 3: guard ref.mounted after awaits; the send flow
// is driven by the Mutations API in the screen. No FFI-driven provider retries.
import 'dart:async';
import 'dart:convert';

import 'package:collection/collection.dart';
import 'package:gen_ui_widgets/gen_ui_widgets.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:uuid/uuid.dart';

import '../../../../bridge/a2ui/a2ui_content_driver.dart';
import '../../../../bridge/a2ui/a2ui_event.dart';
import '../../domain/entities/chat_message.dart';
import '../../domain/chat_repository.dart';
import 'chat_repository_provider.dart';

part 'chat_notifier.g.dart';

const _uuid = Uuid();

// ChatNotifier.build() is synchronous (never throws), so provider-level retry
// does not apply here. The FFI terminality that matters lives on the async
// FFI-backed providers (entityList/entity/syncStatus) and on the Mutation's
// own error handling — sendMessage runs inside a Mutation, not a build().
@riverpod
class ChatNotifier extends _$ChatNotifier {
  final _drivers = <String, A2uiContentDriver>{};
  final _checkpointTimers = <String, Timer>{};
  StreamSubscription<Map<String, dynamic>>? _modelStatusSubscription;
  bool _loaded = false;
  ChatRepository get _repository => ref.read(chatRepositoryProvider);

  @override
  ChatState build() {
    ref.onDispose(() {
      for (final d in _drivers.values) {
        d.dispose();
      }
      for (final timer in _checkpointTimers.values) {
        timer.cancel();
      }
      unawaited(_modelStatusSubscription?.cancel());
    });
    return const ChatState.initial();
  }

  Future<void> load() async {
    if (_loaded || state.isLoading) return;
    state = state.copyWith(isLoading: true);
    var conversations = await _repository.listConversations();
    if (conversations.isEmpty) {
      conversations = [
        await _repository.createConversation('New conversation'),
      ];
    }
    final threadId = conversations.first.id;
    final messages = await _repository.listMessages(threadId);
    if (!ref.mounted) return;
    _loaded = true;
    state = state.copyWith(
      conversations: conversations,
      threadId: threadId,
      messages: messages,
      isLoading: false,
    );
    _modelStatusSubscription ??= _repository.watchLocalModelStatus().listen(
      (operation) {
        if (!ref.mounted) return;
        state = state.copyWith(
          modelOperation: operation.cast<String, Object?>(),
        );
      },
      onError: (Object error, StackTrace stackTrace) {
        if (!ref.mounted) return;
        state = state.copyWith(
          modelOperation: {
            'state': 'failed',
            'error': {
              'code': 'model_status_stream_failed',
              'phase': 'model_loading',
              'message': error.toString(),
              'retryable': true,
              'diagnostics': stackTrace.toString(),
            },
          },
        );
      },
    );
  }

  Future<void> cancelModelOperation() => _repository.cancelModelOperation();

  Future<void> createConversation() async {
    final conversation = await _repository.createConversation(
      'New conversation',
    );
    state = state.copyWith(
      conversations: [conversation, ...state.conversations],
      threadId: conversation.id,
      messages: const [],
    );
  }

  Future<void> selectConversation(String id) async {
    if (id == state.threadId) return;
    state = state.copyWith(isLoading: true);
    final messages = await _repository.listMessages(id);
    if (!ref.mounted) return;
    state = state.copyWith(threadId: id, messages: messages, isLoading: false);
  }

  Future<void> deleteConversation(String id) async {
    await _repository.deleteConversation(id);
    var conversations = state.conversations
        .where((item) => item.id != id)
        .toList();
    if (conversations.isEmpty) {
      conversations = [
        await _repository.createConversation('New conversation'),
      ];
    }
    final threadId = id == state.threadId
        ? conversations.first.id
        : state.threadId;
    final messages = id == state.threadId
        ? await _repository.listMessages(threadId)
        : state.messages;
    if (!ref.mounted) return;
    state = state.copyWith(
      conversations: conversations,
      threadId: threadId,
      messages: messages,
    );
  }

  /// Append a ContentBlock to a streaming assistant message (sync — no await).
  void streamBlock({required String messageId, required ContentBlock block}) {
    state = state.copyWith(
      messages: [
        for (final m in state.messages)
          if (m.id == messageId)
            m.copyWith(content: _appendOrMerge(m.content, block))
          else
            m,
      ],
    );
    _scheduleCheckpoint(messageId);
  }

  void updateRunPhase(String messageId, RunPhaseChanged phase) {
    state = state.copyWith(
      messages: [
        for (final message in state.messages)
          if (message.id == messageId)
            message.copyWith(
              runId: phase.runId,
              runStatus: phase.status,
              runPhase: phase.phase,
              statusMessage: phase.message,
              isStreaming: true,
            )
          else
            message,
      ],
    );
    _scheduleCheckpoint(messageId);
  }

  void markRunError(String messageId, ChatRunError error) {
    state = state.copyWith(
      messages: [
        for (final message in state.messages)
          if (message.id == messageId)
            message.copyWith(
              runStatus: 'error',
              runPhase: error.phase,
              statusMessage: error.message,
              error: error,
              isStreaming: false,
            )
          else
            message,
      ],
    );
    _checkpointNow(messageId);
  }

  void markRunCancelled(String messageId, RunCancelled event) {
    state = state.copyWith(
      messages: [
        for (final message in state.messages)
          if (message.id == messageId)
            message.copyWith(
              runStatus: 'cancelled',
              runPhase: event.phase,
              statusMessage: event.message,
              isStreaming: false,
            )
          else
            message,
      ],
    );
    _checkpointNow(messageId);
  }

  void updateContextTelemetry(String messageId, ContextTelemetry event) {
    state = state.copyWith(
      messages: [
        for (final message in state.messages)
          if (message.id == messageId)
            message.copyWith(
              contextPlanVersion: event.payload['planVersion']?.toString(),
              contextTelemetry: event.payload.cast<String, Object?>(),
            )
          else
            message,
      ],
    );
    _scheduleCheckpoint(messageId);
  }

  List<ContentBlock> _appendOrMerge(
    List<ContentBlock> content,
    ContentBlock block,
  ) => switch (block) {
    TextBlock(:final text) => switch (content.lastOrNull) {
      TextBlock(text: final previous) => [
        ...content.sublist(0, content.length - 1),
        TextBlock('$previous$text'),
      ],
      _ => [...content, block],
    },
    ThinkingBlock(:final text) => switch (content.lastOrNull) {
      ThinkingBlock(text: final previous) => [
        ...content.sublist(0, content.length - 1),
        ThinkingBlock('$previous$text'),
      ],
      _ => [...content, block],
    },
    CodeBlock() ||
    CitationBlock() ||
    MemoryBlock() ||
    ToolUseBlock() ||
    ToolResultBlock() ||
    SkillBlock() ||
    ArtifactBlock() ||
    ImageBlock() ||
    DividerBlock() => [...content, block],
  };

  void finalizeMessage(String messageId) {
    state = state.copyWith(
      messages: [
        for (final m in state.messages)
          if (m.id == messageId)
            m.copyWith(
              isStreaming: false,
              runStatus: switch (m.runStatus) {
                'error' || 'cancelled' => m.runStatus,
                _ => 'complete',
              },
              runPhase: 'persistence',
              statusMessage: switch (m.runStatus) {
                'error' || 'cancelled' => m.statusMessage,
                _ => null,
              },
            )
          else
            m,
      ],
    );
    ChatMessage? message;
    for (final item in state.messages) {
      if (item.id == messageId) {
        message = item;
        break;
      }
    }
    if (message != null && state.threadId.isNotEmpty) {
      _checkpointTimers.remove(messageId)?.cancel();
      unawaited(_repository.saveMessage(state.threadId, message));
    }
  }

  /// Send a user message, open the assistant message, and fold its event stream.
  /// Terminal on Rust domain error (retry is off). Called from a Mutation.
  Future<void> sendMessage(
    String text, {
    List<ContentBlock> attachments = const [],
    List<String> attachmentPaths = const [],
    String branchId = 'main',
  }) async {
    await load();
    if (!ref.mounted || state.threadId.isEmpty) return;
    final history = _historyForRust(state.messages, branchId);
    final userMsg = ChatMessage(
      id: _uuid.v4(),
      role: 'user',
      content: [...attachments, TextBlock(text)],
      createdAt: DateTime.now().toUtc(),
      branchId: branchId,
      attachmentPaths: attachmentPaths,
    );
    final assistantId = _uuid.v4();
    final assistantMsg = ChatMessage(
      id: assistantId,
      role: 'assistant',
      content: const [],
      createdAt: DateTime.now().toUtc(),
      isStreaming: true,
      parentMessageId: userMsg.id,
      branchId: userMsg.branchId,
      runStatus: 'queued',
      runPhase: 'queued',
      statusMessage: 'Queued',
    );
    state = state.copyWith(
      messages: [...state.messages, userMsg, assistantMsg],
    );
    await _repository.saveMessage(state.threadId, userMsg);
    await _touchConversation(text);

    late final String runId;
    try {
      runId = await _repository.startTurn(
        threadId: state.threadId,
        message: text,
        history: history,
        attachmentPaths: attachmentPaths,
        branchId: branchId,
        userMessageId: userMsg.id,
      );
    } catch (error, stackTrace) {
      markRunError(
        assistantId,
        ChatRunError(
          code: 'run_start_failed',
          phase: 'queued',
          message: error.toString(),
          retryable: true,
          diagnostics: stackTrace.toString(),
        ),
      );
      return;
    }
    if (!ref.mounted) return; // provider paused/disposed while awaiting
    updateRunPhase(
      assistantId,
      RunPhaseChanged(runId, 'queued', 'preparing', 'Preparing response'),
    );

    final driver = A2uiContentDriver(
      messageId: assistantId,
      onBlock: streamBlock,
      onFinalize: finalizeMessage,
      onPhase: updateRunPhase,
      onCancelled: markRunCancelled,
      onTelemetry: updateContextTelemetry,
      onError: markRunError,
    )..connect(_repository.events(runId));
    _drivers[assistantId] = driver;
    ref.onDispose(() => driver.dispose());
  }

  Future<void> resubmitMessage(String messageId) async {
    final message = state.messages
        .where((item) => item.id == messageId)
        .firstOrNull;
    if (message == null || message.role != 'user' || message.isStreaming) {
      return;
    }
    final text = message.content
        .whereType<TextBlock>()
        .map((block) => block.text)
        .join();
    final attachments = message.content
        .where((block) => block is! TextBlock)
        .toList();
    await sendMessage(
      text,
      attachments: attachments,
      attachmentPaths: message.attachmentPaths,
      branchId: _uuid.v4(),
    );
  }

  Future<void> cancelRun(String messageId) async {
    final message = state.messages
        .where((item) => item.id == messageId)
        .firstOrNull;
    final runId = message?.runId;
    if (message == null || !message.isStreaming || runId == null) return;
    updateRunPhase(
      messageId,
      RunPhaseChanged(
        runId,
        message.runPhase ?? 'generation',
        'preparing',
        'Cancelling',
      ),
    );
    final accepted = await _repository.cancelRun(runId);
    if (!ref.mounted || accepted) return;
    markRunError(
      messageId,
      const ChatRunError(
        code: 'run_not_active',
        phase: 'generation',
        message: 'This run already ended and could not be cancelled.',
        retryable: false,
      ),
    );
  }

  Future<void> deleteResponse(String messageId) async {
    final selected = state.messages
        .where((item) => item.id == messageId)
        .firstOrNull;
    if (selected == null ||
        selected.role != 'assistant' ||
        selected.isStreaming) {
      return;
    }
    final selectedIndex = state.messages.indexWhere(
      (item) => item.id == messageId,
    );
    final ids = state.messages
        .skip(selectedIndex)
        .where((item) => item.branchId == selected.branchId)
        .map((item) => item.id)
        .toList(growable: false);
    await _repository.deleteMessages(ids);
    if (!ref.mounted) return;
    state = state.copyWith(
      messages: state.messages.where((item) => !ids.contains(item.id)).toList(),
    );
  }

  Future<void> regenerateResponse(String messageId) async {
    final assistant = state.messages
        .where((item) => item.id == messageId)
        .firstOrNull;
    if (assistant == null ||
        assistant.role != 'assistant' ||
        assistant.isStreaming) {
      return;
    }
    final user = state.messages
        .where((item) => item.id == assistant.parentMessageId)
        .firstOrNull;
    if (user == null) return;
    final text = user.content
        .whereType<TextBlock>()
        .map((block) => block.text)
        .join();
    await deleteResponse(messageId);
    if (!ref.mounted) return;
    await _startResponseForSavedUser(
      user,
      text,
      attachmentPaths: user.attachmentPaths,
    );
  }

  Future<void> _startResponseForSavedUser(
    ChatMessage user,
    String text, {
    required List<String> attachmentPaths,
  }) async {
    final userIndex = state.messages.indexWhere((item) => item.id == user.id);
    if (userIndex < 0) return;
    final assistantId = _uuid.v4();
    final assistant = ChatMessage(
      id: assistantId,
      role: 'assistant',
      content: const [],
      createdAt: DateTime.now().toUtc(),
      isStreaming: true,
      parentMessageId: user.id,
      branchId: user.branchId,
      runStatus: 'queued',
      runPhase: 'queued',
      statusMessage: 'Queued',
    );
    final history = _historyForRust(
      state.messages.take(userIndex).toList(),
      user.branchId,
    );
    state = state.copyWith(messages: [...state.messages, assistant]);
    late final String runId;
    try {
      runId = await _repository.startTurn(
        threadId: state.threadId,
        message: text,
        history: history,
        attachmentPaths: attachmentPaths,
        branchId: user.branchId,
        userMessageId: user.id,
      );
    } catch (error, stackTrace) {
      markRunError(
        assistantId,
        ChatRunError(
          code: 'regenerate_start_failed',
          phase: 'queued',
          message: error.toString(),
          retryable: true,
          diagnostics: stackTrace.toString(),
        ),
      );
      return;
    }
    if (!ref.mounted) return;
    updateRunPhase(
      assistantId,
      RunPhaseChanged(runId, 'queued', 'preparing', 'Preparing response'),
    );
    final driver = A2uiContentDriver(
      messageId: assistantId,
      onBlock: streamBlock,
      onFinalize: finalizeMessage,
      onPhase: updateRunPhase,
      onCancelled: markRunCancelled,
      onTelemetry: updateContextTelemetry,
      onError: markRunError,
    )..connect(_repository.events(runId));
    _drivers[assistantId] = driver;
    ref.onDispose(driver.dispose);
  }

  /// Submit an A2UI or AG-UI confirmation interaction through the same typed
  /// Repository -> Rust path as a normal turn and stream the agent's response.
  Future<void> submitInteraction(Map<String, Object?> interaction) async {
    await load();
    if (!ref.mounted || state.threadId.isEmpty) return;
    final payload = jsonEncode(interaction);
    final userMsg = ChatMessage(
      id: _uuid.v4(),
      role: 'user',
      content: [ArtifactBlock(_uuid.v4(), 'client_interaction', payload)],
      createdAt: DateTime.now().toUtc(),
    );
    final assistantId = _uuid.v4();
    final assistantMsg = ChatMessage(
      id: assistantId,
      role: 'assistant',
      content: const [],
      createdAt: DateTime.now().toUtc(),
      isStreaming: true,
      parentMessageId: userMsg.id,
      branchId: userMsg.branchId,
      runStatus: 'queued',
      runPhase: 'queued',
      statusMessage: 'Queued',
    );
    final history = _historyForRust(state.messages, userMsg.branchId);
    state = state.copyWith(
      messages: [...state.messages, userMsg, assistantMsg],
    );
    await _repository.saveMessage(state.threadId, userMsg);
    final runId = await _repository.submitInteraction(
      threadId: state.threadId,
      interactionJson: payload,
      history: history,
      branchId: userMsg.branchId,
    );
    if (!ref.mounted) return;
    final driver = A2uiContentDriver(
      messageId: assistantId,
      onBlock: streamBlock,
      onFinalize: finalizeMessage,
      onPhase: updateRunPhase,
      onCancelled: markRunCancelled,
      onTelemetry: updateContextTelemetry,
      onError: markRunError,
    )..connect(_repository.events(runId));
    _drivers[assistantId] = driver;
    ref.onDispose(driver.dispose);
  }

  Future<void> approveToolConfirmation(
    String sourceMessageId,
    String approvalId,
  ) async {
    await load();
    if (!ref.mounted || state.threadId.isEmpty || approvalId.isEmpty) return;
    final source = state.messages
        .where((item) => item.id == sourceMessageId)
        .firstOrNull;
    if (source == null || source.role != 'assistant' || source.isStreaming) {
      return;
    }

    final assistantId = _uuid.v4();
    final assistant = ChatMessage(
      id: assistantId,
      role: 'assistant',
      content: const [],
      createdAt: DateTime.now().toUtc(),
      isStreaming: true,
      parentMessageId: source.id,
      branchId: source.branchId,
      runStatus: 'queued',
      runPhase: 'tool_execution',
      statusMessage: 'Approving tool call',
    );
    state = state.copyWith(messages: [...state.messages, assistant]);

    late final String runId;
    try {
      runId = await _repository.approveToolConfirmation(approvalId: approvalId);
    } catch (error, stackTrace) {
      markRunError(
        assistantId,
        ChatRunError(
          code: 'tool_approval_start_failed',
          phase: 'tool_execution',
          message: error.toString(),
          retryable: false,
          diagnostics: stackTrace.toString(),
        ),
      );
      return;
    }
    if (!ref.mounted) return;
    updateRunPhase(
      assistantId,
      RunPhaseChanged(
        runId,
        'tool_execution',
        'preparing',
        'Executing approved tool call',
      ),
    );
    final driver = A2uiContentDriver(
      messageId: assistantId,
      onBlock: streamBlock,
      onFinalize: finalizeMessage,
      onPhase: updateRunPhase,
      onCancelled: markRunCancelled,
      onTelemetry: updateContextTelemetry,
      onError: markRunError,
    )..connect(_repository.events(runId));
    _drivers[assistantId] = driver;
    ref.onDispose(driver.dispose);
  }

  Future<ResolvedRemoteContent> resolveRemoteContent(
    String url,
    RemoteContentKind kind,
  ) => _repository.resolveRemoteContent(url, kind);

  List<ChatHistoryEntry> _historyForRust(
    List<ChatMessage> messages,
    String branchId,
  ) => [
    for (final message in messages)
      if (message.branchId == branchId) ...[
        if (message.content
            .whereType<TextBlock>()
            .map((block) => block.text)
            .join()
            .trim()
            .isNotEmpty)
          ChatHistoryEntry(
            role: message.role,
            content: message.content
                .whereType<TextBlock>()
                .map((block) => block.text)
                .join(),
          ),
        for (final result in message.content.whereType<ToolResultBlock>())
          ChatHistoryEntry(
            role: 'tool',
            content: result.outputJson,
            toolCallId: result.toolUseId,
          ),
      ],
  ];

  Future<void> _touchConversation(String firstMessage) async {
    final index = state.conversations.indexWhere(
      (item) => item.id == state.threadId,
    );
    if (index < 0) return;
    final current = state.conversations[index];
    final title = current.title == 'New conversation'
        ? firstMessage.length > 42
              ? '${firstMessage.substring(0, 42)}…'
              : firstMessage
        : current.title;
    final updated = current.copyWith(
      title: title,
      updatedAt: DateTime.now().toUtc(),
    );
    await _repository.saveConversation(updated);
    if (!ref.mounted) return;
    state = state.copyWith(
      conversations: [
        updated,
        for (final item in state.conversations)
          if (item.id != updated.id) item,
      ],
    );
  }

  void _scheduleCheckpoint(String messageId) {
    _checkpointTimers.remove(messageId)?.cancel();
    _checkpointTimers[messageId] = Timer(
      const Duration(milliseconds: 250),
      () => _checkpointNow(messageId),
    );
  }

  void _checkpointNow(String messageId) {
    _checkpointTimers.remove(messageId)?.cancel();
    if (state.threadId.isEmpty) return;
    final message = state.messages
        .where((item) => item.id == messageId)
        .firstOrNull;
    if (message != null) {
      unawaited(_repository.saveMessage(state.threadId, message));
    }
  }

  /// Test/utility seam: fold an already-open stream (no FFI send). Used by the
  /// boundary test to drive canned Rust-shaped events through the real fold path.
  Future<void> foldStream(String messageId, Stream<A2uiEvent> stream) async {
    if (!state.messages.any((m) => m.id == messageId)) {
      state = state.copyWith(
        messages: [
          ...state.messages,
          ChatMessage(
            id: messageId,
            role: 'assistant',
            content: const [],
            isStreaming: true,
          ),
        ],
      );
    }
    final driver = A2uiContentDriver(
      messageId: messageId,
      onBlock: streamBlock,
      onFinalize: finalizeMessage,
    );
    _drivers[messageId] = driver;
    await driver.connect(stream);
  }
}
