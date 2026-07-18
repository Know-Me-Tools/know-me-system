// TJ-ARCH-MOB-001 compliant
import 'entities/chat_conversation.dart';
import 'entities/chat_message.dart';

abstract interface class ChatRepository {
  Future<List<ChatConversation>> listConversations();
  Future<ChatConversation> createConversation(String title);
  Future<void> saveConversation(ChatConversation conversation);
  Future<void> deleteConversation(String id);
  Future<List<ChatMessage>> listMessages(String conversationId);
  Future<void> saveMessage(String conversationId, ChatMessage message);
}
