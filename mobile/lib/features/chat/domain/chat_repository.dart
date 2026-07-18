// TJ-ARCH-MOB-001 compliant
import 'entities/chat_conversation.dart';
import 'entities/chat_message.dart';
import '../../../bridge/a2ui/a2ui_event.dart';
import 'package:gen_ui_widgets/gen_ui_widgets.dart';

abstract interface class ChatRepository {
  Future<List<ChatConversation>> listConversations();
  Future<ChatConversation> createConversation(String title);
  Future<void> saveConversation(ChatConversation conversation);
  Future<void> deleteConversation(String id);
  Future<List<ChatMessage>> listMessages(String conversationId);
  Future<void> saveMessage(String conversationId, ChatMessage message);
  Future<String> startTurn({
    required String threadId,
    required String message,
    required List<String> history,
    List<String> attachmentPaths,
  });
  Stream<A2uiEvent> events(String runId);
  Future<String> submitInteraction({
    required String threadId,
    required String interactionJson,
    required List<String> history,
  });
  Future<ResolvedRemoteContent> resolveRemoteContent(
    String url,
    RemoteContentKind kind,
  );
}
