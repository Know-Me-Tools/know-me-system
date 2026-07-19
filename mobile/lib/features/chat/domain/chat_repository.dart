// TJ-ARCH-MOB-001 compliant
import 'entities/chat_conversation.dart';
import 'entities/chat_message.dart';
import '../../../bridge/a2ui/a2ui_event.dart';
import 'package:gen_ui_widgets/gen_ui_widgets.dart';

class ChatHistoryEntry {
  const ChatHistoryEntry({
    required this.role,
    required this.content,
    this.name,
    this.toolCallId,
  });

  final String role;
  final String content;
  final String? name;
  final String? toolCallId;
}

abstract interface class ChatRepository {
  Future<List<ChatConversation>> listConversations();
  Future<ChatConversation> createConversation(String title);
  Future<void> saveConversation(ChatConversation conversation);
  Future<void> deleteConversation(String id);
  Future<List<ChatMessage>> listMessages(String conversationId);
  Future<void> saveMessage(String conversationId, ChatMessage message);
  Future<void> deleteMessages(List<String> ids);
  Future<String> startTurn({
    required String threadId,
    required String message,
    required List<ChatHistoryEntry> history,
    required String branchId,
    required String userMessageId,
    List<String> attachmentPaths,
  });
  Stream<A2uiEvent> events(String runId);
  Future<bool> cancelRun(String runId);
  Stream<Map<String, dynamic>> watchLocalModelStatus();
  Future<void> prepareLocalModel();
  Future<void> cancelModelOperation();
  Future<Map<String, dynamic>?> localRuntimeDiagnostics();
  Future<String> submitInteraction({
    required String threadId,
    required String interactionJson,
    required List<ChatHistoryEntry> history,
    required String branchId,
  });
  Future<ResolvedRemoteContent> resolveRemoteContent(
    String url,
    RemoteContentKind kind,
  );
}
