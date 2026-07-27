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

/// A picked file after Rust has copied it into app-owned storage.
///
/// Every field is decided in Rust so both surfaces agree. Flutter deciding the
/// mime and kind for itself is what made a picked PNG render as a document
/// chip here while React made the same mistake separately.
class StagedAttachment {
  const StagedAttachment({
    required this.path,
    required this.fileName,
    required this.mime,
    required this.sizeBytes,
    required this.isImage,
    required this.isVideo,
    required this.isDocument,
    this.width,
    this.height,
  });

  factory StagedAttachment.fromJson(Map<String, dynamic> json) =>
      StagedAttachment(
        path: json['path'] as String,
        fileName: json['fileName'] as String,
        mime: json['mime'] as String,
        sizeBytes: (json['sizeBytes'] as num).toInt(),
        isImage: json['isImage'] as bool? ?? false,
        isVideo: json['isVideo'] as bool? ?? false,
        isDocument: json['isDocument'] as bool? ?? false,
        width: (json['width'] as num?)?.toInt(),
        height: (json['height'] as num?)?.toInt(),
      );

  /// Durable path. Safe to persist — unlike the picker's path, which lives in
  /// the purgeable cache on Android.
  final String path;
  final String fileName;
  final String mime;
  final int sizeBytes;
  final bool isImage;
  final bool isVideo;
  final bool isDocument;
  final int? width;
  final int? height;
}

abstract interface class ChatRepository {
  /// Copy a picked file into app-owned storage and describe it.
  ///
  /// Called at PICK time, not send time: the picker's path can be purged
  /// before the turn is even sent.
  Future<StagedAttachment> stageAttachment({
    required String conversationId,
    required String sourcePath,
  });

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
  Future<String> approveToolConfirmation({required String approvalId});
  Future<ResolvedRemoteContent> resolveRemoteContent(
    String url,
    RemoteContentKind kind,
  );
  Future<String> errorMessage(Object error);
}
