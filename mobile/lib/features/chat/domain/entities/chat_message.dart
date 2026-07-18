// TJ-ARCH-MOB-001 compliant
import 'package:gen_ui_widgets/gen_ui_widgets.dart';

import 'chat_conversation.dart';

/// One chat message = an ordered list of ContentBlocks folded from a run's
/// A2uiEvent stream. Immutable; ChatNotifier produces new instances on each edit.
class ChatMessage {
  ChatMessage({
    required this.id,
    required this.role,
    required this.content,
    DateTime? createdAt,
    this.isStreaming = false,
  }) : createdAt =
            createdAt ?? DateTime.fromMillisecondsSinceEpoch(0, isUtc: true);

  final String id;
  final String role; // 'user' | 'assistant'
  final List<ContentBlock> content;
  final DateTime createdAt;
  final bool isStreaming;

  ChatMessage copyWith({List<ContentBlock>? content, bool? isStreaming}) =>
      ChatMessage(
        id: id,
        role: role,
        content: content ?? this.content,
        createdAt: createdAt,
        isStreaming: isStreaming ?? this.isStreaming,
      );
}

class ChatState {
  const ChatState({
    this.messages = const [],
    this.conversations = const [],
    this.threadId = '',
    this.isLoading = false,
  });
  final List<ChatMessage> messages;
  final List<ChatConversation> conversations;
  final String threadId;
  final bool isLoading;

  const ChatState.initial()
      : messages = const [],
        conversations = const [],
        threadId = '',
        isLoading = false;

  ChatState copyWith({
    List<ChatMessage>? messages,
    List<ChatConversation>? conversations,
    String? threadId,
    bool? isLoading,
  }) =>
      ChatState(
        messages: messages ?? this.messages,
        conversations: conversations ?? this.conversations,
        threadId: threadId ?? this.threadId,
        isLoading: isLoading ?? this.isLoading,
      );
}
