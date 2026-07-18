// TJ-ARCH-MOB-001 compliant
// ChatNotifier owns ALL ContentBlock mutations. It sends via the FFI facade,
// then folds the returned A2uiEvent stream into the assistant message using
// A2uiContentDriver. Riverpod 3: guard ref.mounted after awaits; the send flow
// is driven by the Mutations API in the screen. No FFI-driven provider retries.
import 'dart:async';

import 'package:gen_ui_widgets/gen_ui_widgets.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:uuid/uuid.dart';

import '../../../../bridge/a2ui/a2ui_content_driver.dart';
import '../../../../bridge/a2ui/a2ui_event.dart';
import '../../../../bridge/rust_bridge_provider.dart' as bridge;
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
  bool _loaded = false;
  ChatRepository get _repository => ref.read(chatRepositoryProvider);

  @override
  ChatState build() {
    ref.onDispose(() {
      for (final d in _drivers.values) {
        d.dispose();
      }
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
  }

  Future<void> createConversation() async {
    final conversation =
        await _repository.createConversation('New conversation');
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
    var conversations =
        state.conversations.where((item) => item.id != id).toList();
    if (conversations.isEmpty) {
      conversations = [
        await _repository.createConversation('New conversation'),
      ];
    }
    final threadId =
        id == state.threadId ? conversations.first.id : state.threadId;
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
            m.copyWith(content: [...m.content, block])
          else
            m,
      ],
    );
  }

  void finalizeMessage(String messageId) {
    state = state.copyWith(
      messages: [
        for (final m in state.messages)
          if (m.id == messageId) m.copyWith(isStreaming: false) else m,
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
      unawaited(_repository.saveMessage(state.threadId, message));
    }
  }

  /// Send a user message, open the assistant message, and fold its event stream.
  /// Terminal on Rust domain error (retry is off). Called from a Mutation.
  Future<void> sendMessage(String text) async {
    await load();
    if (!ref.mounted || state.threadId.isEmpty) return;
    final history = _historyForRust(state.messages);
    final userMsg = ChatMessage(
      id: _uuid.v4(),
      role: 'user',
      content: [TextBlock(text)],
      createdAt: DateTime.now().toUtc(),
    );
    final assistantId = _uuid.v4();
    final assistantMsg = ChatMessage(
      id: assistantId,
      role: 'assistant',
      content: const [],
      createdAt: DateTime.now().toUtc(),
      isStreaming: true,
    );
    state =
        state.copyWith(messages: [...state.messages, userMsg, assistantMsg]);
    await _repository.saveMessage(state.threadId, userMsg);
    await _touchConversation(text);

    final runId = await bridge.chatSend(state.threadId, text, history);
    if (!ref.mounted) return; // provider paused/disposed while awaiting

    final driver = A2uiContentDriver(
      messageId: assistantId,
      onBlock: streamBlock,
      onFinalize: finalizeMessage,
      onError: (id, msg) =>
          streamBlock(messageId: id, block: TextBlock('⚠️ $msg')),
    )..connect(bridge.chatEvents(runId));
    _drivers[assistantId] = driver;
    ref.onDispose(() => driver.dispose());
  }

  List<String> _historyForRust(List<ChatMessage> messages) => [
        for (final message in messages) ...[
          message.role,
          message.content
              .whereType<TextBlock>()
              .map((block) => block.text)
              .join(),
        ],
      ];

  Future<void> _touchConversation(String firstMessage) async {
    final index =
        state.conversations.indexWhere((item) => item.id == state.threadId);
    if (index < 0) return;
    final current = state.conversations[index];
    final title = current.title == 'New conversation'
        ? firstMessage.length > 42
            ? '${firstMessage.substring(0, 42)}…'
            : firstMessage
        : current.title;
    final updated =
        current.copyWith(title: title, updatedAt: DateTime.now().toUtc());
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
