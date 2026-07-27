// TJ-ARCH-MOB-001 compliant
import 'dart:convert';

import 'package:gen_ui_widgets/gen_ui_widgets.dart';
import 'package:prometheus_entity_management/prometheus_entity_management.dart';
import 'package:uuid/uuid.dart';

import '../domain/chat_repository.dart';
import '../domain/entities/chat_conversation.dart';
import '../domain/entities/chat_message.dart';
import '../../../bridge/a2ui/a2ui_event.dart';
import '../../../bridge/rust_bridge_provider.dart' as bridge;
import '../../../bridge/third_party/gen_ui_types/inference.dart';

class PemChatRepository implements ChatRepository {
  const PemChatRepository(this.transport);

  final EntityTransport transport;
  static const _uuid = Uuid();
  static const _conversationType = 'chat_conversation';
  static const _messageType = 'chat_message';

  @override
  Future<StagedAttachment> stageAttachment({
    required String conversationId,
    required String sourcePath,
  }) async {
    final json = await bridge.stageAttachment(conversationId, sourcePath);
    return StagedAttachment.fromJson(jsonDecode(json) as Map<String, dynamic>);
  }

  @override
  Future<int> sweepOrphanAttachments(List<String> conversationIds) =>
      bridge.sweepOrphanAttachments(conversationIds);

  @override
  Future<String> startTurn({
    required String threadId,
    required String message,
    required List<ChatHistoryEntry> history,
    required String branchId,
    required String userMessageId,
    List<String> attachmentPaths = const [],
  }) => bridge.chatSend(
    threadId,
    message,
    history.map(_toInferenceMessage).toList(growable: false),
    attachmentPaths,
    branchId,
    userMessageId,
  );

  @override
  Stream<A2uiEvent> events(String runId) => bridge.chatEvents(runId);

  @override
  Future<bool> cancelRun(String runId) => bridge.chatCancel(runId);

  @override
  Stream<Map<String, dynamic>> watchLocalModelStatus() =>
      bridge.watchLocalModelStatus();

  @override
  Future<void> prepareLocalModel() => bridge.prepareLocalModel();

  @override
  Future<void> cancelModelOperation() => bridge.cancelModelOperation();

  @override
  Future<Map<String, dynamic>?> localRuntimeDiagnostics() =>
      bridge.localRuntimeDiagnostics();

  @override
  Future<String> submitInteraction({
    required String threadId,
    required String interactionJson,
    required List<ChatHistoryEntry> history,
    required String branchId,
  }) => bridge.chatAction(
    threadId,
    interactionJson,
    history.map(_toInferenceMessage).toList(growable: false),
    branchId,
  );

  @override
  Future<String> approveToolConfirmation({required String approvalId}) =>
      bridge.approveToolConfirmation(approvalId);

  static InferenceMessage _toInferenceMessage(ChatHistoryEntry entry) =>
      InferenceMessage(
        role: switch (entry.role) {
          'system' => InferenceRole.system,
          'assistant' => InferenceRole.assistant,
          'tool' => InferenceRole.tool,
          _ => InferenceRole.user,
        },
        content: entry.content,
        name: entry.name,
        toolCallId: entry.toolCallId,
      );

  @override
  Future<ResolvedRemoteContent> resolveRemoteContent(
    String url,
    RemoteContentKind kind,
  ) async => ResolvedRemoteContent.fromJson(
    jsonDecode(await bridge.resolveRemoteContent(url, kind.name))
        as Map<String, dynamic>,
  );

  @override
  Future<String> errorMessage(Object error) => bridge.bridgeErrorMessage(error);

  @override
  Future<List<ChatConversation>> listConversations() async {
    final result = await transport.list(
      const ViewDescriptor(entityType: _conversationType, limit: 1000),
    );
    return result.items.map(_decodeConversation).toList(growable: false)
      ..sort((a, b) => b.updatedAt.compareTo(a.updatedAt));
  }

  @override
  Future<ChatConversation> createConversation(String title) async {
    final now = DateTime.now().toUtc();
    final conversation = ChatConversation(
      id: _uuid.v4(),
      title: title,
      createdAt: now,
      updatedAt: now,
    );
    await transport.create(_conversationRecord(conversation));
    return conversation;
  }

  @override
  Future<void> saveConversation(ChatConversation conversation) async {
    await transport.update(_conversationRecord(conversation));
  }

  @override
  Future<void> deleteConversation(String id) async {
    await bridge.deleteConversationContext(id);
    final messages = await listMessages(id);
    for (final message in messages) {
      await transport.delete(_messageType, message.id);
    }
    await transport.delete(_conversationType, id);
  }

  @override
  Future<List<ChatMessage>> listMessages(String conversationId) async {
    final result = await transport.list(
      const ViewDescriptor(entityType: _messageType, limit: 1000),
    );
    final messages = result.items
        .map(_decodeMessage)
        .where((entry) => entry.$1 == conversationId)
        .map((entry) => entry.$2)
        .toList(growable: false);
    messages.sort((a, b) => a.createdAt.compareTo(b.createdAt));
    return messages;
  }

  @override
  Future<void> saveMessage(String conversationId, ChatMessage message) async {
    await transport.update(
      EntityRecord(
        id: message.id,
        entityType: _messageType,
        dataJson: jsonEncode({
          'conversationId': conversationId,
          'role': message.role,
          'createdAt': message.createdAt.toUtc().toIso8601String(),
          'updatedAt': message.updatedAt.toUtc().toIso8601String(),
          'runId': message.runId,
          'parentMessageId': message.parentMessageId,
          'branchId': message.branchId,
          'runStatus': message.runStatus,
          'runPhase': message.runPhase,
          'statusMessage': message.statusMessage,
          'error': message.error?.toJson(),
          'generationSettings': message.generationSettings,
          'runtimeDiagnostics': message.runtimeDiagnostics,
          'contextPlanVersion': message.contextPlanVersion,
          'attachmentPaths': message.attachmentPaths,
          'contextTelemetry': message.contextTelemetry,
          'provenance': message.provenance,
          'intent': message.intent,
          'content': message.content.map(_encodeBlock).toList(growable: false),
        }),
      ),
    );
  }

  @override
  Future<void> deleteMessages(List<String> ids) async {
    await bridge.deleteMessageContext(ids);
    for (final id in ids) {
      await transport.delete(_messageType, id);
    }
  }

  EntityRecord _conversationRecord(ChatConversation conversation) =>
      EntityRecord(
        id: conversation.id,
        entityType: _conversationType,
        dataJson: jsonEncode({
          'title': conversation.title,
          'createdAt': conversation.createdAt.toUtc().toIso8601String(),
          'updatedAt': conversation.updatedAt.toUtc().toIso8601String(),
        }),
      );

  ChatConversation _decodeConversation(EntityRecord record) {
    final data = jsonDecode(record.dataJson) as Map<String, dynamic>;
    return ChatConversation(
      id: record.id,
      title: data['title']?.toString() ?? 'Conversation',
      createdAt: DateTime.parse(data['createdAt'] as String),
      updatedAt: DateTime.parse(data['updatedAt'] as String),
    );
  }

  (String, ChatMessage) _decodeMessage(EntityRecord record) {
    final data = jsonDecode(record.dataJson) as Map<String, dynamic>;
    return (
      data['conversationId'] as String,
      ChatMessage(
        id: record.id,
        role: data['role'] as String,
        createdAt: DateTime.parse(data['createdAt'] as String),
        updatedAt: DateTime.tryParse(data['updatedAt']?.toString() ?? ''),
        runId: data['runId']?.toString(),
        parentMessageId: data['parentMessageId']?.toString(),
        branchId: data['branchId']?.toString() ?? 'main',
        runStatus: data['runStatus']?.toString() ?? 'complete',
        runPhase: data['runPhase']?.toString(),
        statusMessage: data['statusMessage']?.toString(),
        error: data['error'] is Map<String, dynamic>
            ? ChatRunError.fromJson(data['error'] as Map<String, dynamic>)
            : null,
        generationSettings:
            (data['generationSettings'] as Map<String, dynamic>?)
                ?.cast<String, Object?>() ??
            const {},
        runtimeDiagnostics:
            (data['runtimeDiagnostics'] as Map<String, dynamic>?)
                ?.cast<String, Object?>() ??
            const {},
        contextPlanVersion: data['contextPlanVersion']?.toString(),
        attachmentPaths:
            (data['attachmentPaths'] as List<dynamic>?)
                ?.map((value) => value.toString())
                .toList(growable: false) ??
            const [],
        contextTelemetry:
            (data['contextTelemetry'] as Map<String, dynamic>?)
                ?.cast<String, Object?>() ??
            const {},
        provenance:
            (data['provenance'] as Map<String, dynamic>?)
                ?.cast<String, Object?>() ??
            const {},
        intent: data['intent']?.toString(),
        content: (data['content'] as List<dynamic>)
            .cast<Map<String, dynamic>>()
            .map(ContentBlock.fromJson)
            .toList(growable: false),
      ),
    );
  }

  Map<String, Object?> _encodeBlock(ContentBlock block) => switch (block) {
    TextBlock(:final text) => {'type': 'text', 'text': text},
    ThinkingBlock(:final text) => {'type': 'thinking', 'text': text},
    CodeBlock(:final language, :final code) => {
      'type': 'code',
      'language': language,
      'code': code,
    },
    CitationBlock(:final source, :final quote) => {
      'type': 'citation',
      'source': source,
      'quote': quote,
    },
    MemoryBlock(:final operation, :final key, :final value) => {
      'type': 'memory',
      'operation': operation,
      'key': key,
      'value': value,
    },
    ToolUseBlock(:final id, :final name, :final inputJson) => {
      'type': 'toolUse',
      'id': id,
      'name': name,
      'inputJson': inputJson,
    },
    ToolResultBlock(:final toolUseId, :final outputJson, :final isError) => {
      'type': 'toolResult',
      'toolUseId': toolUseId,
      'outputJson': outputJson,
      'isError': isError,
    },
    SkillBlock(:final name, :final status) => {
      'type': 'skill',
      'name': name,
      'status': status,
    },
    ArtifactBlock(:final id, :final kind, :final content) => {
      'type': 'artifact',
      'id': id,
      'kind': kind,
      'content': content,
    },
    ImageBlock(:final url, :final dataBase64, :final mime) => {
      'type': 'image',
      'url': url,
      'dataBase64': dataBase64,
      'mime': mime,
    },
    DividerBlock() => {'type': 'divider'},
  };
}
