// TJ-ARCH-MOB-001 compliant
import 'dart:convert';

import 'package:gen_ui_widgets/gen_ui_widgets.dart';
import 'package:prometheus_entity_management/prometheus_entity_management.dart';
import 'package:uuid/uuid.dart';

import '../domain/chat_repository.dart';
import '../domain/entities/chat_conversation.dart';
import '../domain/entities/chat_message.dart';

class PemChatRepository implements ChatRepository {
  const PemChatRepository(this.transport);

  final EntityTransport transport;
  static const _uuid = Uuid();
  static const _conversationType = 'chat_conversation';
  static const _messageType = 'chat_message';

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
  Future<void> saveMessage(
    String conversationId,
    ChatMessage message,
  ) async {
    await transport.update(
      EntityRecord(
        id: message.id,
        entityType: _messageType,
        dataJson: jsonEncode({
          'conversationId': conversationId,
          'role': message.role,
          'createdAt': message.createdAt.toUtc().toIso8601String(),
          'content': message.content.map(_encodeBlock).toList(growable: false),
        }),
      ),
    );
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
        ToolResultBlock(:final toolUseId, :final outputJson, :final isError) =>
          {
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
