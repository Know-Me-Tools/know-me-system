// TJ-ARCH-MOB-001 compliant
// DemoChatRepository is intentionally gated by KNOWME_DEMO_SKIP_MOBILE_BOOT.
// It keeps the iOS Simulator demo shell operable when production mobile boot is
// bypassed, without changing the production FFI-backed repository path.
import 'dart:async';
import 'dart:convert';

import 'package:gen_ui_widgets/gen_ui_widgets.dart';
import 'package:uuid/uuid.dart';

import '../../../bridge/a2ui/a2ui_event.dart';
import '../domain/chat_repository.dart';
import '../domain/entities/chat_conversation.dart';
import '../domain/entities/chat_message.dart';

class DemoChatRepository implements ChatRepository {
  DemoChatRepository();

  static const _uuid = Uuid();
  static final _conversations = <ChatConversation>[];
  static final _messagesByConversation = <String, List<ChatMessage>>{};
  static final _runPrompts = <String, String>{};

  @override
  Future<List<ChatConversation>> listConversations() async =>
      List<ChatConversation>.unmodifiable(_conversations);

  @override
  Future<ChatConversation> createConversation(String title) async {
    final now = DateTime.now().toUtc();
    final conversation = ChatConversation(
      id: _uuid.v4(),
      title: title,
      createdAt: now,
      updatedAt: now,
    );
    _conversations.insert(0, conversation);
    _messagesByConversation[conversation.id] = <ChatMessage>[];
    return conversation;
  }

  @override
  Future<void> saveConversation(ChatConversation conversation) async {
    final index = _conversations.indexWhere(
      (item) => item.id == conversation.id,
    );
    if (index < 0) {
      _conversations.insert(0, conversation);
    } else {
      _conversations[index] = conversation;
    }
  }

  @override
  Future<void> deleteConversation(String id) async {
    _conversations.removeWhere((item) => item.id == id);
    _messagesByConversation.remove(id);
  }

  @override
  Future<List<ChatMessage>> listMessages(String conversationId) async =>
      List<ChatMessage>.unmodifiable(
        _messagesByConversation[conversationId] ?? const <ChatMessage>[],
      );

  @override
  Future<void> saveMessage(String conversationId, ChatMessage message) async {
    final messages = _messagesByConversation.putIfAbsent(
      conversationId,
      () => <ChatMessage>[],
    );
    final index = messages.indexWhere((item) => item.id == message.id);
    if (index < 0) {
      messages.add(message);
    } else {
      messages[index] = message;
    }
  }

  @override
  Future<void> deleteMessages(List<String> ids) async {
    final toRemove = ids.toSet();
    for (final messages in _messagesByConversation.values) {
      messages.removeWhere((message) => toRemove.contains(message.id));
    }
  }

  @override
  Future<StagedAttachment> stageAttachment({
    required String conversationId,
    required String sourcePath,
  }) async {
    // The demo repository has no Rust behind it, so nothing is copied. It
    // reports the source path back and classifies from the extension — enough
    // for the demo transcript to render, and honest that no storage happened.
    final name = sourcePath.split('/').last;
    final extension = name.contains('.')
        ? name.split('.').last.toLowerCase()
        : '';
    const imageExtensions = {'png', 'jpg', 'jpeg', 'gif', 'webp', 'svg'};
    const videoExtensions = {'mp4', 'webm', 'mov'};
    return StagedAttachment(
      path: sourcePath,
      fileName: name,
      mime: _mimeForExtension(extension),
      sizeBytes: 0,
      isImage: imageExtensions.contains(extension),
      isVideo: videoExtensions.contains(extension),
      isDocument:
          !imageExtensions.contains(extension) &&
          !videoExtensions.contains(extension),
    );
  }

  static String _mimeForExtension(String extension) => switch (extension) {
    'png' => 'image/png',
    'jpg' || 'jpeg' => 'image/jpeg',
    'gif' => 'image/gif',
    'webp' => 'image/webp',
    'svg' => 'image/svg+xml',
    'pdf' => 'application/pdf',
    'md' => 'text/markdown',
    'txt' => 'text/plain',
    'mp4' => 'video/mp4',
    'webm' => 'video/webm',
    'mov' => 'video/quicktime',
    _ => 'application/octet-stream',
  };

  @override
  Future<String> startTurn({
    required String threadId,
    required String message,
    required List<ChatHistoryEntry> history,
    required String branchId,
    required String userMessageId,
    List<String> attachmentPaths = const [],
  }) async {
    final runId = 'demo-run-${_uuid.v4()}';
    _runPrompts[runId] = message;
    return runId;
  }

  @override
  Stream<A2uiEvent> events(String runId) async* {
    final prompt = _runPrompts[runId] ?? 'your prompt';
    yield RunStarted(runId);
    yield RunPhaseChanged(
      runId,
      'demo',
      'preparing',
      'Using simulator demo runtime',
    );
    await Future<void>.delayed(const Duration(milliseconds: 180));
    yield BlockEvent(
      runId,
      const ThinkingBlock(
        'Demo note: production mobile boot is intentionally bypassed for this simulator build.',
      ),
    );
    await Future<void>.delayed(const Duration(milliseconds: 180));
    yield RunPhaseChanged(runId, 'demo', 'generating', 'Rendering response');
    yield BlockEvent(
      runId,
      TextBlock(
        'KnowMe simulator demo is running. I received: "$prompt".\n\n'
        '- Desktop Tauri build is launched.\n'
        '- iOS Simulator shell is launched.\n'
        '- Local inference/runtime certification is tracked as future-release work.',
      ),
    );
    yield BlockEvent(runId, const SkillBlock('demo-runtime', 'complete'));
    yield RunFinished(runId);
  }

  @override
  Future<bool> cancelRun(String runId) async => true;

  @override
  Stream<Map<String, dynamic>> watchLocalModelStatus() => Stream.value({
    'state': 'ready',
    'runtime': 'demo',
    'message': 'Simulator demo runtime',
  });

  @override
  Future<void> prepareLocalModel() async {}

  @override
  Future<void> cancelModelOperation() async {}

  @override
  Future<Map<String, dynamic>?> localRuntimeDiagnostics() async => {
    'runtimeKind': 'demo',
    'platform': 'ios-simulator',
    'localInferenceCertified': false,
  };

  @override
  Future<String> submitInteraction({
    required String threadId,
    required String interactionJson,
    required List<ChatHistoryEntry> history,
    required String branchId,
  }) async {
    final runId = 'demo-interaction-${_uuid.v4()}';
    _runPrompts[runId] = 'A2UI interaction: $interactionJson';
    return runId;
  }

  @override
  Future<String> approveToolConfirmation({required String approvalId}) async =>
      approvalId;

  @override
  Future<ResolvedRemoteContent> resolveRemoteContent(
    String url,
    RemoteContentKind kind,
  ) async => ResolvedRemoteContent(
    url: url,
    mime: switch (kind) {
      RemoteContentKind.image => 'image/*',
      RemoteContentKind.video => 'video/*',
      RemoteContentKind.audio => 'audio/*',
      RemoteContentKind.opengraph => 'application/json',
    },
    title: kind == RemoteContentKind.opengraph ? url : null,
    description: kind == RemoteContentKind.opengraph
        ? jsonEncode({'demo': true, 'url': url})
        : null,
  );

  @override
  Future<String> errorMessage(Object error) async =>
      error.toString().replaceFirst('Exception: ', '');
}
