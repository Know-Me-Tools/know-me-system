// TJ-ARCH-MOB-001 compliant
import 'package:gen_ui_widgets/gen_ui_widgets.dart';

import 'chat_conversation.dart';
import '../../../../bridge/a2ui/a2ui_event.dart';

const _unset = Object();

/// One chat message = an ordered list of ContentBlocks folded from a run's
/// A2uiEvent stream. Immutable; ChatNotifier produces new instances on each edit.
class ChatMessage {
  ChatMessage({
    required this.id,
    required this.role,
    required this.content,
    DateTime? createdAt,
    DateTime? updatedAt,
    this.isStreaming = false,
    this.runId,
    this.parentMessageId,
    this.branchId = 'main',
    this.runStatus = 'complete',
    this.runPhase,
    this.statusMessage,
    this.error,
    this.generationSettings = const {},
    this.runtimeDiagnostics = const {},
    this.contextPlanVersion,
    this.attachmentPaths = const [],
    this.contextTelemetry = const {},
    this.provenance = const {},
  }) : createdAt =
           createdAt ?? DateTime.fromMillisecondsSinceEpoch(0, isUtc: true),
       updatedAt =
           updatedAt ??
           createdAt ??
           DateTime.fromMillisecondsSinceEpoch(0, isUtc: true);

  final String id;
  final String role; // 'user' | 'assistant'
  final List<ContentBlock> content;
  final DateTime createdAt;
  final DateTime updatedAt;
  final bool isStreaming;
  final String? runId;
  final String? parentMessageId;
  final String branchId;
  final String runStatus;
  final String? runPhase;
  final String? statusMessage;
  final ChatRunError? error;
  final Map<String, Object?> generationSettings;
  final Map<String, Object?> runtimeDiagnostics;
  final String? contextPlanVersion;
  final List<String> attachmentPaths;
  final Map<String, Object?> contextTelemetry;

  /// Which agent + provider + model produced this assistant turn, projected from
  /// the run's `effective_run_policy`. Keys: `agentId`, `providerId`, `modelId`
  /// (any may be null). Empty for user messages or before the run resolves.
  final Map<String, Object?> provenance;

  ChatMessage copyWith({
    List<ContentBlock>? content,
    bool? isStreaming,
    String? runId,
    String? runStatus,
    Object? runPhase = _unset,
    Object? statusMessage = _unset,
    Object? error = _unset,
    DateTime? updatedAt,
    List<String>? attachmentPaths,
    Map<String, Object?>? contextTelemetry,
    Map<String, Object?>? provenance,
    String? contextPlanVersion,
  }) => ChatMessage(
    id: id,
    role: role,
    content: content ?? this.content,
    createdAt: createdAt,
    updatedAt: updatedAt ?? DateTime.now().toUtc(),
    isStreaming: isStreaming ?? this.isStreaming,
    runId: runId ?? this.runId,
    parentMessageId: parentMessageId,
    branchId: branchId,
    runStatus: runStatus ?? this.runStatus,
    runPhase: identical(runPhase, _unset) ? this.runPhase : runPhase as String?,
    statusMessage: identical(statusMessage, _unset)
        ? this.statusMessage
        : statusMessage as String?,
    error: identical(error, _unset) ? this.error : error as ChatRunError?,
    generationSettings: generationSettings,
    runtimeDiagnostics: runtimeDiagnostics,
    contextPlanVersion: contextPlanVersion ?? this.contextPlanVersion,
    attachmentPaths: attachmentPaths ?? this.attachmentPaths,
    contextTelemetry: contextTelemetry ?? this.contextTelemetry,
    provenance: provenance ?? this.provenance,
  );
}

class ChatState {
  const ChatState({
    this.messages = const [],
    this.conversations = const [],
    this.threadId = '',
    this.isLoading = false,
    this.modelOperation = const {'state': 'not_installed'},
  });
  final List<ChatMessage> messages;
  final List<ChatConversation> conversations;
  final String threadId;
  final bool isLoading;
  final Map<String, Object?> modelOperation;

  const ChatState.initial()
    : messages = const [],
      conversations = const [],
      threadId = '',
      isLoading = false,
      modelOperation = const {'state': 'not_installed'};

  ChatState copyWith({
    List<ChatMessage>? messages,
    List<ChatConversation>? conversations,
    String? threadId,
    bool? isLoading,
    Map<String, Object?>? modelOperation,
  }) => ChatState(
    messages: messages ?? this.messages,
    conversations: conversations ?? this.conversations,
    threadId: threadId ?? this.threadId,
    isLoading: isLoading ?? this.isLoading,
    modelOperation: modelOperation ?? this.modelOperation,
  );
}
