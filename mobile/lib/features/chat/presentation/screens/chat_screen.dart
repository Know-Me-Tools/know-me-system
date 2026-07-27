// TJ-ARCH-MOB-001 compliant
import 'dart:async';
import 'dart:convert';

import 'package:file_picker/file_picker.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:gen_ui_widgets/gen_ui_widgets.dart';
import 'package:shadcn_flutter/shadcn_flutter.dart' as shad;

import '../../../../bridge/a2ui/a2ui_event.dart';
import '../../../../core/theme/tokens.dart';
import '../../../../shared/widgets/knowme_screen.dart';
import '../../../../shared/widgets/sync_chip.dart';
import '../../domain/entities/chat_message.dart';
import '../providers/chat_notifier.dart';
import '../widgets/chat_policy_toolbar.dart';

/// Chat surface. Renders each message's ContentBlocks with the shared
/// ContentBlockView (exhaustive over all 11 variants — a compile-time contract).
/// Send tracks its own local pending state — Riverpod 3.3.2's Mutation API exists
/// internally (riverpod/src/core/mutations.dart) but is NOT part of the public
/// export surface yet (still experimental per the Riverpod 3 release notes), so
/// this uses the always-available local-state pattern instead of an unstable API.

class ChatScreen extends ConsumerStatefulWidget {
  const ChatScreen({super.key});
  @override
  ConsumerState<ChatScreen> createState() => _ChatScreenState();
}

class _ChatScreenState extends ConsumerState<ChatScreen> {
  final _controller = TextEditingController();
  final _composerFocus = FocusNode();
  final _scrollController = ScrollController();
  final _attachments = <PlatformFile>[];
  bool _isSending = false;
  bool _isNearBottom = true;
  String? _attachmentError;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      ref.read(chatProvider.notifier).load();
    });
    _scrollController.addListener(_trackScrollPosition);
  }

  @override
  void dispose() {
    _controller.dispose();
    _composerFocus.dispose();
    _scrollController.dispose();
    super.dispose();
  }

  /// Set when the user taps Steer, cleared once that send goes out.
  ///
  /// Tapping Steer only focuses the composer; the intent has to survive until
  /// the user finishes typing and submits, otherwise the correction goes out as
  /// an anonymous new question and the model has no reason to treat the stopped
  /// turn as work to build on.
  bool _pendingSteer = false;

  /// Hand the composer to the user after they stop a run.
  ///
  /// Steering is the point of stopping, so put the caret where the next
  /// instruction goes rather than making them find it. The existing text is
  /// left alone — a half-typed thought is theirs to keep.
  void _focusComposerToSteer() {
    setState(() => _pendingSteer = true);
    _composerFocus.requestFocus();
    _scrollToLatest();
  }

  @override
  Widget build(BuildContext context) {
    final state = ref.watch(chatProvider);
    final colors = shad.Theme.of(context).colorScheme;
    final activeMessage = _activeAssistantMessage(state.messages);
    final modelBusy = _modelOperationBusy(state.modelOperation);
    ref.listen<int>(
      chatProvider.select(
        (value) => value.messages.fold<int>(
          0,
          (total, message) =>
              total + message.content.length + (message.isStreaming ? 1 : 0),
        ),
      ),
      (_, _) => _scheduleScrollToLatest(),
    );

    return KnowMeScreen(
      title: 'Chat',
      trailing: [
        // Icon-only, so it carried no accessible name at all: a screen reader
        // announced nothing, and a certification run had no way to find the one
        // control that opens the thread list.
        Semantics(
          button: true,
          label: 'Conversations',
          excludeSemantics: true,
          child: shad.GhostButton(
            onPressed: _openConversations,
            density: shad.ButtonDensity.icon,
            child: const Icon(Icons.forum_outlined),
          ),
        ),
        const SyncChip(),
      ],
      child: Column(
        children: [
          if (state.isLoading)
            const Padding(
              padding: EdgeInsets.all(8),
              child: SizedBox(
                width: 18,
                height: 18,
                child: CircularProgressIndicator(strokeWidth: 2),
              ),
            ),
          if (state.threadId.isNotEmpty)
            ChatPolicyToolbar(conversationId: state.threadId),
          Expanded(
            child: Stack(
              children: [
                if (state.messages.isEmpty)
                  _ChatWelcome(colors: colors)
                else
                  ListView.separated(
                    controller: _scrollController,
                    keyboardDismissBehavior:
                        ScrollViewKeyboardDismissBehavior.onDrag,
                    padding: const EdgeInsets.fromLTRB(12, 16, 12, 24),
                    itemCount: state.messages.length,
                    separatorBuilder: (_, _) => const SizedBox(height: 16),
                    itemBuilder: (context, i) {
                      final message = state.messages[i];
                      return _MessageBubble(
                        message: message,
                        onResubmit: () => ref
                            .read(chatProvider.notifier)
                            .resubmitMessage(message.id),
                        onDelete: () => ref
                            .read(chatProvider.notifier)
                            .deleteResponse(message.id),
                        onRegenerate: () => ref
                            .read(chatProvider.notifier)
                            .regenerateResponse(message.id),
                        onSteer: _focusComposerToSteer,
                        onResume: () =>
                            ref.read(chatProvider.notifier).sendResume(),
                        onInteraction: (interaction) {
                          if (interaction['kind'] == 'local_tool_approve') {
                            final approvalId = interaction['approvalId']
                                ?.toString();
                            if (approvalId == null || approvalId.isEmpty) {
                              return null;
                            }
                            return ref
                                .read(chatProvider.notifier)
                                .approveToolConfirmation(
                                  message.id,
                                  approvalId,
                                );
                          }
                          return ref
                              .read(chatProvider.notifier)
                              .submitInteraction(interaction);
                        },
                        remoteResolver: (url, kind) => ref
                            .read(chatProvider.notifier)
                            .resolveRemoteContent(url, kind),
                      );
                    },
                  ),
                if (!_isNearBottom && state.messages.isNotEmpty)
                  PositionedDirectional(
                    end: 16,
                    bottom: 8,
                    child: shad.SecondaryButton(
                      onPressed: _scrollToLatest,
                      child: const Row(
                        mainAxisSize: MainAxisSize.min,
                        children: [
                          Icon(Icons.arrow_downward, size: 16),
                          SizedBox(width: 6),
                          Text('Latest'),
                        ],
                      ),
                    ),
                  ),
              ],
            ),
          ),
          SafeArea(
            child: Padding(
              padding: const EdgeInsets.all(12),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  if (modelBusy ||
                      state.modelOperation['state'] == 'failed') ...[
                    _ModelOperationBanner(
                      operation: state.modelOperation,
                      onCancel: modelBusy
                          ? () => ref
                                .read(chatProvider.notifier)
                                .cancelModelOperation()
                          : null,
                    ),
                    const SizedBox(height: 8),
                  ],
                  if (_attachments.isNotEmpty) ...[
                    _AttachmentStrip(
                      files: _attachments,
                      onRemove: (file) => setState(() {
                        _attachments.remove(file);
                        _attachmentError = null;
                      }),
                    ),
                    const SizedBox(height: 8),
                  ],
                  if (_attachmentError != null) ...[
                    Semantics(
                      liveRegion: true,
                      child: Text(
                        _attachmentError!,
                        style: TextStyle(
                          color: colors.destructive,
                          fontSize: 12,
                        ),
                      ),
                    ),
                    const SizedBox(height: 6),
                  ],
                  Row(
                    crossAxisAlignment: CrossAxisAlignment.end,
                    children: [
                      Tooltip(
                        message: 'Add images or documents',
                        child: shad.GhostButton(
                          onPressed: _isSending || modelBusy
                              ? null
                              : _pickAttachments,
                          density: shad.ButtonDensity.icon,
                          child: Semantics(
                            button: true,
                            label: 'Add attachment',
                            child: const Icon(Icons.attach_file),
                          ),
                        ),
                      ),
                      const SizedBox(width: 8),
                      Expanded(
                        // The armed-steer state is otherwise carried only by the
                        // placeholder, which a screen reader does not re-announce
                        // — so name the mode on the field itself.
                        child: Semantics(
                          textField: true,
                          label: _pendingSteer
                              ? 'Steer the stopped response'
                              : 'Message KnowMe',
                          child: shad.TextField(
                            controller: _controller,
                            focusNode: _composerFocus,
                            // Name the mode while it is armed. Steering is a
                            // different act from asking, and the whole point of
                            // making it explicit is that a regular user can tell
                            // which one they are about to do.
                            placeholder: Text(
                              _pendingSteer
                                  ? 'Steer the stopped response…'
                                  : 'Ask KnowMe anything…',
                            ),
                            minLines: 1,
                            maxLines: 5,
                            enabled: !modelBusy,
                            // shadcn_flutter defaults contextMenuBuilder to null,
                            // which suppresses the platform selection toolbar — so
                            // long-press showed no Paste/Copy/Select All callout.
                            // Restore the native adaptive toolbar so paste works.
                            contextMenuBuilder:
                                shad.TextField.nativeContextMenuBuilder(),
                            onSubmitted: _isSending || modelBusy
                                ? null
                                : (_) => _send(),
                          ),
                        ),
                      ),
                      const SizedBox(width: 8),
                      Tooltip(
                        message: activeMessage == null
                            ? 'Send message'
                            : 'Stop generating',
                        child: shad.PrimaryButton(
                          onPressed: _isSending || modelBusy
                              ? null
                              : activeMessage == null
                              ? _send
                              : () => ref
                                    .read(chatProvider.notifier)
                                    .cancelRun(activeMessage.id),
                          density: shad.ButtonDensity.icon,
                          child: _isSending
                              ? const SizedBox(
                                  width: 18,
                                  height: 18,
                                  child: CircularProgressIndicator(
                                    strokeWidth: 2,
                                  ),
                                )
                              : Semantics(
                                  button: true,
                                  label: activeMessage == null
                                      ? 'Send message'
                                      : 'Stop generating',
                                  child: Icon(
                                    activeMessage == null
                                        ? Icons.arrow_upward
                                        : Icons.stop,
                                  ),
                                ),
                        ),
                      ),
                    ],
                  ),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }

  void _openConversations() {
    // Steer is armed against the turn the user just stopped, in the
    // conversation they were in. Every exit from this sheet either switches
    // threads or starts a new one, and carrying the arming across would
    // silently prefix the correction preamble onto a message in an unrelated
    // conversation — where no stopped turn exists for the model to build on.
    // React fixes this in the store; this is the Flutter equivalent, placed at
    // the one door into thread switching rather than on each row.
    if (_pendingSteer) {
      setState(() => _pendingSteer = false);
    }
    shad.openSheetOverlay<void>(
      context: context,
      position: shad.OverlayPosition.left,
      constraints: const BoxConstraints(maxWidth: 360),
      builder: (sheetContext) =>
          _ConversationSheet(close: () => shad.closeSheet(sheetContext)),
    );
  }

  Future<void> _send() async {
    if (_modelOperationBusy(ref.read(chatProvider).modelOperation)) return;
    final text = _controller.text.trim();
    if (text.isEmpty && _attachments.isEmpty) return;
    final attachmentBlocks = [
      for (final file in _attachments)
        ArtifactBlock(
          file.identifier ?? file.name,
          'attachment',
          jsonEncode({
            'name': file.name,
            'size': _formatBytes(file.size),
            'mime': _mimeFor(file.extension),
          }),
        ),
    ];
    final attachmentPaths = [for (final file in _attachments) ?file.path];
    final isSteer = _pendingSteer;
    _controller.clear();
    setState(() {
      _attachments.clear();
      _attachmentError = null;
      _isSending = true;
      _isNearBottom = true;
      _pendingSteer = false;
    });
    final body = text.isEmpty ? 'Please review the attached files.' : text;
    final notifier = ref.read(chatProvider.notifier);
    try {
      await (isSteer
          ? notifier.sendSteer(
              body,
              attachments: attachmentBlocks,
              attachmentPaths: attachmentPaths,
            )
          : notifier.sendMessage(
              body,
              attachments: attachmentBlocks,
              attachmentPaths: attachmentPaths,
            ));
    } finally {
      if (mounted) setState(() => _isSending = false);
    }
  }

  Future<void> _pickAttachments() async {
    final file = await FilePicker.pickFile(
      type: FileType.custom,
      allowedExtensions: const [
        'png',
        'jpg',
        'jpeg',
        'gif',
        'webp',
        'svg',
        'pdf',
        'txt',
        'md',
        'doc',
        'docx',
        'mp4',
        'webm',
      ],
    );
    if (!mounted || file == null) return;
    const maxBytes = 25 * 1024 * 1024;
    final selectedFiles = [file];
    final tooLarge = selectedFiles
        .where((file) => file.size > maxBytes)
        .toList();
    final unavailable = selectedFiles
        .where((file) => file.path == null)
        .toList();
    setState(() {
      _attachments.addAll(
        selectedFiles
            .where((file) => file.size <= maxBytes && file.path != null)
            .take(6 - _attachments.length),
      );
      _attachmentError = switch ((
        tooLarge.isNotEmpty,
        unavailable.isNotEmpty,
      )) {
        (true, true) => 'Some files were too large or unavailable to the app.',
        (true, false) => 'Files larger than 25 MB were not attached.',
        (false, true) => 'Some files were unavailable to the app.',
        (false, false) => null,
      };
    });
  }

  void _trackScrollPosition() {
    if (!_scrollController.hasClients) return;
    final distance =
        _scrollController.position.maxScrollExtent -
        _scrollController.position.pixels;
    final nearBottom = distance < 120;
    if (nearBottom != _isNearBottom && mounted) {
      setState(() => _isNearBottom = nearBottom);
    }
  }

  void _scheduleScrollToLatest() {
    if (!_isNearBottom) return;
    WidgetsBinding.instance.addPostFrameCallback((_) => _scrollToLatest());
  }

  void _scrollToLatest() {
    if (!_scrollController.hasClients) return;
    _scrollController.animateTo(
      _scrollController.position.maxScrollExtent,
      duration: MediaQuery.disableAnimationsOf(context)
          ? Duration.zero
          : const Duration(milliseconds: 180),
      curve: Curves.easeOut,
    );
    if (!_isNearBottom) setState(() => _isNearBottom = true);
  }
}

ChatMessage? _activeAssistantMessage(List<ChatMessage> messages) {
  for (final message in messages.reversed) {
    if (message.role == 'assistant' && message.isStreaming) return message;
  }
  return null;
}

bool _modelOperationBusy(Map<String, Object?> operation) =>
    switch (operation['state']) {
      'downloading' || 'verifying' || 'loading' || 'cancelling' => true,
      _ => false,
    };

class _MessageBubble extends StatelessWidget {
  const _MessageBubble({
    required this.message,
    required this.onResubmit,
    required this.onDelete,
    required this.onRegenerate,
    required this.onSteer,
    required this.onResume,
    this.onInteraction,
    this.remoteResolver,
  });

  final ChatMessage message;
  final Future<void> Function() onResubmit;
  final Future<void> Function() onDelete;
  final Future<void> Function() onRegenerate;
  final VoidCallback onSteer;
  final Future<void> Function() onResume;
  final AgentInteractionCallback? onInteraction;
  final RemoteContentResolver? remoteResolver;

  @override
  Widget build(BuildContext context) {
    final colors = shad.Theme.of(context).colorScheme;
    final palette = KnowMePalette.of(context);
    final isUser = message.role == 'user';
    final screenWidth = MediaQuery.sizeOf(context).width;
    final maxWidth = (screenWidth * 0.80).clamp(0.0, 720.0);
    final blocks = coalesceContentBlocks(message.content);
    final copyValue = blocks.map(contentBlockCopyText).join('\n\n');
    if (!isUser) {
      return Align(
        alignment: Alignment.centerLeft,
        child: ConstrainedBox(
          constraints: BoxConstraints(maxWidth: maxWidth),
          child: DefaultTextStyle(
            style: TextStyle(
              color: colors.foreground,
              fontSize: 16,
              height: 1.5,
            ),
            child: _AssistantTurn(
              message: message,
              blocks: blocks,
              onInteraction: onInteraction,
              remoteResolver: remoteResolver,
              onDelete: onDelete,
              onRegenerate: onRegenerate,
              onSteer: onSteer,
              onResume: onResume,
            ),
          ),
        ),
      );
    }
    return Align(
      alignment: Alignment.centerRight,
      child: Container(
        constraints: BoxConstraints(maxWidth: maxWidth),
        padding: const EdgeInsets.fromLTRB(14, 12, 8, 6),
        decoration: BoxDecoration(
          color: isUser ? palette.accentSoft : colors.card,
          borderRadius: BorderRadiusDirectional.only(
            topStart: const Radius.circular(18),
            topEnd: const Radius.circular(18),
            bottomStart: Radius.circular(isUser ? 18 : 5),
            bottomEnd: Radius.circular(isUser ? 5 : 18),
          ),
        ),
        child: DefaultTextStyle(
          style: TextStyle(color: colors.foreground, fontSize: 16, height: 1.5),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              for (var i = 0; i < blocks.length; i++)
                ContentBlockView(
                  block: blocks[i],
                  isStreaming: message.isStreaming && i == blocks.length - 1,
                  onInteraction: onInteraction,
                  remoteResolver: remoteResolver,
                ),
              if (message.isStreaming && message.content.isEmpty)
                Semantics(
                  liveRegion: true,
                  label: 'Assistant is responding',
                  child: Padding(
                    padding: const EdgeInsets.all(6),
                    child: SizedBox(
                      width: 18,
                      height: 18,
                      child: CircularProgressIndicator(
                        strokeWidth: 2,
                        color: colors.primary,
                      ),
                    ),
                  ),
                ),
              Align(
                alignment: AlignmentDirectional.centerEnd,
                child: Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Tooltip(
                      message: 'Resubmit message',
                      child: IconButton(
                        tooltip: 'Resubmit message',
                        constraints: const BoxConstraints(
                          minWidth: 48,
                          minHeight: 48,
                        ),
                        onPressed: message.isStreaming ? null : onResubmit,
                        icon: Icon(
                          Icons.replay_outlined,
                          size: 18,
                          color: colors.mutedForeground,
                        ),
                      ),
                    ),
                    Tooltip(
                      message: 'Copy entire message',
                      child: IconButton(
                        tooltip: 'Copy entire message',
                        constraints: const BoxConstraints(
                          minWidth: 48,
                          minHeight: 48,
                        ),
                        onPressed: () =>
                            Clipboard.setData(ClipboardData(text: copyValue)),
                        icon: Icon(
                          Icons.copy_outlined,
                          size: 18,
                          color: colors.mutedForeground,
                        ),
                      ),
                    ),
                  ],
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class _AssistantTurn extends StatelessWidget {
  const _AssistantTurn({
    required this.message,
    required this.blocks,
    required this.onDelete,
    required this.onRegenerate,
    required this.onSteer,
    required this.onResume,
    this.onInteraction,
    this.remoteResolver,
  });

  final ChatMessage message;
  final List<ContentBlock> blocks;
  final Future<void> Function() onDelete;
  final Future<void> Function() onRegenerate;

  /// Hand the user the composer, pre-focused, so they can redirect from what
  /// was already generated rather than starting over.
  final VoidCallback onSteer;
  final Future<void> Function() onResume;
  final AgentInteractionCallback? onInteraction;
  final RemoteContentResolver? remoteResolver;

  @override
  Widget build(BuildContext context) {
    final colors = shad.Theme.of(context).colorScheme;
    final segments = <Widget>[];
    final answer = <ContentBlock>[];

    void flushAnswer() {
      if (answer.isEmpty) return;
      final snapshot = List<ContentBlock>.of(answer);
      answer.clear();
      segments.add(
        Container(
          padding: const EdgeInsets.fromLTRB(14, 12, 10, 8),
          decoration: BoxDecoration(
            color: colors.card,
            borderRadius: BorderRadius.circular(18),
          ),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              for (var index = 0; index < snapshot.length; index++)
                ContentBlockView(
                  block: snapshot[index],
                  isStreaming:
                      message.isStreaming && index == snapshot.length - 1,
                  onInteraction: onInteraction,
                  remoteResolver: remoteResolver,
                ),
            ],
          ),
        ),
      );
    }

    for (var index = 0; index < blocks.length; index++) {
      final block = blocks[index];
      if (_isAnswerBlock(block)) {
        answer.add(block);
        continue;
      }
      flushAnswer();
      final answerHasBegun = blocks.take(index).any(_isAnswerBlock);
      segments.add(
        ContentBlockView(
          block: block,
          isStreaming:
              message.isStreaming && !answerHasBegun && block is ThinkingBlock,
          onInteraction: onInteraction,
          remoteResolver: remoteResolver,
        ),
      );
    }
    flushAnswer();

    final waiting =
        message.isStreaming &&
        (message.content.isEmpty ||
            !matchesGeneratingContent(message.runStatus));
    if (waiting && message.statusMessage != null) {
      segments.add(_RunStatusBubble(message: message));
    }
    if (message.runStatus == 'error' && message.error != null) {
      segments.add(_RunErrorBubble(error: message.error!));
    } else if (message.runStatus == 'cancelled') {
      segments.add(
        _StoppedTurnBar(
          // Matches the durable status Rust writes ("Cancelled by user",
          // chat.rs:151) so one state never shows two different words. Reached
          // only if a cancel arrives with no message at all.
          message: message.statusMessage ?? 'Cancelled by user',
          keptBlocks: blocks.length,
          onSteer: onSteer,
          onResume: onResume,
          onRegenerate: onRegenerate,
          onDelete: onDelete,
        ),
      );
    }

    final answerCopy = blocks
        .where(_isAnswerBlock)
        .map(contentBlockCopyText)
        .join('\n\n');
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        for (var index = 0; index < segments.length; index++) ...[
          segments[index],
          if (index != segments.length - 1) const SizedBox(height: 8),
        ],
        _ProvenanceChip(provenance: message.provenance),
        Row(
          mainAxisAlignment: MainAxisAlignment.end,
          children: [
            IconButton(
              tooltip: 'Copy answer',
              constraints: const BoxConstraints(minWidth: 48, minHeight: 48),
              onPressed: answerCopy.isEmpty
                  ? null
                  : () => Clipboard.setData(ClipboardData(text: answerCopy)),
              icon: const Icon(Icons.copy_outlined, size: 18),
            ),
            IconButton(
              tooltip: 'Regenerate response',
              constraints: const BoxConstraints(minWidth: 48, minHeight: 48),
              onPressed: message.isStreaming ? null : onRegenerate,
              icon: const Icon(Icons.refresh, size: 18),
            ),
            IconButton(
              tooltip: 'Delete response and later branch messages',
              constraints: const BoxConstraints(minWidth: 48, minHeight: 48),
              onPressed: message.isStreaming
                  ? null
                  : () async {
                      final confirmed = await showDialog<bool>(
                        context: context,
                        builder: (dialogContext) => AlertDialog(
                          title: const Text('Delete response?'),
                          content: const Text(
                            'This removes the response and later messages on this branch.',
                          ),
                          actions: [
                            TextButton(
                              onPressed: () =>
                                  Navigator.pop(dialogContext, false),
                              child: const Text('Keep'),
                            ),
                            TextButton(
                              onPressed: () =>
                                  Navigator.pop(dialogContext, true),
                              child: const Text('Delete'),
                            ),
                          ],
                        ),
                      );
                      if (confirmed == true) await onDelete();
                    },
              icon: const Icon(Icons.delete_outline, size: 18),
            ),
          ],
        ),
      ],
    );
  }
}

/// A subtle, text-first footer showing which agent + provider/model produced the
/// response. Renders nothing until provenance resolves. Status is text (not color
/// alone) and the label ellipsizes so it survives large Dynamic Type.
class _ProvenanceChip extends StatelessWidget {
  const _ProvenanceChip({required this.provenance});

  final Map<String, Object?> provenance;

  @override
  Widget build(BuildContext context) {
    final label = _provenanceLabel(provenance);
    if (label == null) return const SizedBox.shrink();
    final colors = shad.Theme.of(context).colorScheme;
    return Padding(
      padding: const EdgeInsets.only(left: 14, top: 4, bottom: 2),
      child: Row(
        children: [
          Icon(Icons.memory_outlined, size: 12, color: colors.mutedForeground),
          const SizedBox(width: 4),
          Expanded(
            child: Text(
              label,
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
              style: TextStyle(color: colors.mutedForeground, fontSize: 11),
            ),
          ),
        ],
      ),
    );
  }
}

/// Build the "agent · provider/model" label, or null when there is nothing to
/// show. The on-device provider id renders as "On-device"; missing parts are
/// omitted rather than shown as nulls.
String? _provenanceLabel(Map<String, Object?> provenance) {
  String? str(String key) {
    final value = provenance[key]?.toString().trim();
    return (value == null || value.isEmpty) ? null : value;
  }

  final agent = str('agentId');
  final provider = str('providerId');
  final model = str('modelId');

  final modelPart = switch ((provider, model)) {
    ('on-device', _) => 'On-device',
    (final p?, final m?) => '$p/$m',
    (final p?, null) => p,
    (null, final m?) => m,
    (null, null) => null,
  };

  final parts = [?agent, ?modelPart];
  return parts.isEmpty ? null : parts.join(' · ');
}

bool _isAnswerBlock(ContentBlock block) => switch (block) {
  TextBlock() ||
  CodeBlock() ||
  ArtifactBlock() ||
  ImageBlock() ||
  DividerBlock() => true,
  ThinkingBlock() ||
  CitationBlock() ||
  MemoryBlock() ||
  ToolUseBlock() ||
  ToolResultBlock() ||
  SkillBlock() => false,
};

bool matchesGeneratingContent(String status) =>
    status == 'thinking' || status == 'generating';

class _RunStatusBubble extends StatelessWidget {
  const _RunStatusBubble({required this.message});
  final ChatMessage message;

  @override
  Widget build(BuildContext context) {
    final colors = shad.Theme.of(context).colorScheme;
    return Semantics(
      liveRegion: true,
      label: message.statusMessage,
      child: Container(
        padding: const EdgeInsets.all(12),
        decoration: BoxDecoration(
          color: colors.secondary,
          borderRadius: BorderRadius.circular(14),
        ),
        child: Row(
          children: [
            SizedBox(
              width: 18,
              height: 18,
              child: CircularProgressIndicator(
                strokeWidth: 2,
                color: colors.primary,
              ),
            ),
            const SizedBox(width: 10),
            Expanded(
              child: Text(message.statusMessage ?? 'Preparing response'),
            ),
          ],
        ),
      ),
    );
  }
}

class _ModelOperationBanner extends StatelessWidget {
  const _ModelOperationBanner({required this.operation, this.onCancel});

  final Map<String, Object?> operation;
  final VoidCallback? onCancel;

  @override
  Widget build(BuildContext context) {
    final state = operation['state']?.toString() ?? 'not_installed';
    final received = (operation['bytes_received'] as num?)?.toDouble();
    final total = (operation['total_bytes'] as num?)?.toDouble();
    final progress = received != null && total != null && total > 0
        ? received / total
        : null;
    final error = operation['error'] is Map
        ? (operation['error'] as Map)['message']?.toString()
        : null;
    final label = switch (state) {
      'downloading' =>
        progress == null
            ? 'Downloading local model'
            : 'Downloading local model · ${(progress * 100).round()}%',
      'verifying' => 'Verifying local model',
      'loading' => 'Loading model into memory',
      'cancelling' => 'Cancelling model preparation',
      'failed' => error ?? 'Local model preparation failed',
      _ => state.replaceAll('_', ' '),
    };
    final colors = shad.Theme.of(context).colorScheme;
    return Semantics(
      liveRegion: true,
      label: label,
      child: Container(
        padding: const EdgeInsets.all(12),
        decoration: BoxDecoration(
          color: state == 'failed'
              ? colors.destructive.withValues(alpha: 0.1)
              : colors.secondary,
          borderRadius: BorderRadius.circular(14),
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Row(
              children: [
                Expanded(child: Text(label)),
                if (onCancel != null)
                  SizedBox(
                    height: 44,
                    child: shad.SecondaryButton(
                      onPressed: onCancel,
                      child: const Text('Cancel'),
                    ),
                  ),
              ],
            ),
            if (state == 'downloading') ...[
              const SizedBox(height: 8),
              LinearProgressIndicator(value: progress),
            ],
          ],
        ),
      ),
    );
  }
}

class _RunErrorBubble extends StatelessWidget {
  const _RunErrorBubble({required this.error});
  final ChatRunError error;

  @override
  Widget build(BuildContext context) {
    final palette = KnowMePalette.of(context);
    final message = error.displayMessage;
    return Semantics(
      liveRegion: true,
      label: 'Error: $message',
      child: Container(
        padding: const EdgeInsets.all(12),
        decoration: BoxDecoration(
          color: palette.destructive.withValues(alpha: 0.12),
          borderRadius: BorderRadius.circular(14),
        ),
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Icon(Icons.error_outline, color: palette.destructive),
            const SizedBox(width: 10),
            Expanded(child: Text(message)),
          ],
        ),
      ),
    );
  }
}

/// Shown when a run was stopped, in place of the old bare "cancelled" label.
///
/// Stopping is how users steer: they see the model heading the wrong way and
/// interrupt it, expecting the work so far to stay in play. This says so
/// explicitly — what was kept, and the four things they can do next — because
/// the previous label left regular users unsure whether their work survived.
///
/// The two actions that KEEP the user's work lead (Steer, Resume); the two that
/// throw it away are subordinate, so the destructive option stops competing for
/// the tap. Codex and Amp both treat steer and interrupt as separate intents
/// rather than one undifferentiated "send again", which is the distinction most
/// users are actually reaching for when they hit stop.
class _StoppedTurnBar extends StatelessWidget {
  const _StoppedTurnBar({
    required this.message,
    required this.keptBlocks,
    required this.onSteer,
    required this.onResume,
    required this.onRegenerate,
    required this.onDelete,
  });

  final String message;
  final int keptBlocks;
  final VoidCallback onSteer;
  final Future<void> Function() onResume;
  final Future<void> Function() onRegenerate;
  final Future<void> Function() onDelete;

  @override
  Widget build(BuildContext context) {
    final colors = shad.Theme.of(context).colorScheme;
    final kept = keptBlocks == 1 ? '1 block' : '$keptBlocks blocks';
    // Only claim work was kept when there is some; a stop before the first
    // block produced nothing, and saying otherwise would be a lie the user can
    // see through.
    final summary = keptBlocks > 0
        ? '$message — kept $kept, still in context'
        : message;

    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        // Flat 2.0: regions differ by background colour only — no border.
        color: colors.secondary,
        borderRadius: BorderRadius.circular(14),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Semantics(
            liveRegion: true,
            child: Row(
              children: [
                // Icon plus text: state is never carried by colour alone.
                const Icon(Icons.stop_circle_outlined),
                const SizedBox(width: 10),
                Expanded(child: Text(summary)),
              ],
            ),
          ),
          const SizedBox(height: 12),
          // Wrap so the actions reflow instead of clipping at large text scales.
          Wrap(
            spacing: 8,
            runSpacing: 8,
            children: [
              _StoppedAction(
                icon: Icons.edit_outlined,
                label: 'Steer',
                semanticLabel:
                    'Steer from here — write a new instruction, '
                    'keeping the work so far',
                onPressed: onSteer,
                emphasis: _ActionEmphasis.primary,
              ),
              // For the user who stopped and changed their mind. Only offered
              // when something was actually kept — there is nothing to continue
              // from otherwise.
              if (keptBlocks > 0)
                _StoppedAction(
                  icon: Icons.play_arrow_outlined,
                  label: 'Resume',
                  semanticLabel: 'Resume — continue from where it stopped',
                  onPressed: () => onResume(),
                  emphasis: _ActionEmphasis.primary,
                ),
              _StoppedAction(
                icon: Icons.refresh,
                // Named for what it does to the kept work: `regenerateResponse`
                // is destructive in place, so it discards the partial answer the
                // rest of this bar exists to preserve. Saying "replaces" is the
                // difference between an informed tap and a surprise.
                label: 'Retry',
                semanticLabel:
                    'Retry — replaces this response and discards the '
                    'work so far',
                onPressed: () => onRegenerate(),
                emphasis: _ActionEmphasis.subordinate,
              ),
              _StoppedAction(
                icon: Icons.delete_outline,
                label: 'Discard',
                semanticLabel: 'Discard this stopped response',
                onPressed: () => onDelete(),
                emphasis: _ActionEmphasis.subordinate,
              ),
            ],
          ),
        ],
      ),
    );
  }
}

/// How much visual weight an action carries.
///
/// Three tiers rather than a bool, because the bar now has to say which actions
/// keep the user's work (primary) and which throw it away (subordinate) without
/// relying on colour alone.
enum _ActionEmphasis { primary, secondary, subordinate }

class _StoppedAction extends StatelessWidget {
  const _StoppedAction({
    required this.icon,
    required this.label,
    required this.semanticLabel,
    required this.onPressed,
    this.emphasis = _ActionEmphasis.secondary,
  });

  final IconData icon;
  final String label;
  final String semanticLabel;
  final VoidCallback onPressed;
  final _ActionEmphasis emphasis;

  @override
  Widget build(BuildContext context) {
    final child = Row(
      mainAxisSize: MainAxisSize.min,
      children: [Icon(icon, size: 16), const SizedBox(width: 6), Text(label)],
    );
    return Semantics(
      button: true,
      label: semanticLabel,
      // Excluded so each control announces once rather than doubling up its
      // own visible label with the descriptive one.
      excludeSemantics: true,
      child: ConstrainedBox(
        constraints: const BoxConstraints(minHeight: 48),
        child: switch (emphasis) {
          _ActionEmphasis.primary => shad.PrimaryButton(
            onPressed: onPressed,
            child: child,
          ),
          _ActionEmphasis.secondary => shad.SecondaryButton(
            onPressed: onPressed,
            child: child,
          ),
          _ActionEmphasis.subordinate => shad.GhostButton(
            onPressed: onPressed,
            child: child,
          ),
        },
      ),
    );
  }
}

class _AttachmentStrip extends StatelessWidget {
  const _AttachmentStrip({required this.files, required this.onRemove});

  final List<PlatformFile> files;
  final ValueChanged<PlatformFile> onRemove;

  @override
  Widget build(BuildContext context) {
    final colors = shad.Theme.of(context).colorScheme;
    return SingleChildScrollView(
      scrollDirection: Axis.horizontal,
      child: Row(
        children: [
          for (final file in files)
            Padding(
              padding: const EdgeInsetsDirectional.only(end: 8),
              child: Semantics(
                label: '${file.name}, ${_formatBytes(file.size)} attachment',
                child: Container(
                  constraints: const BoxConstraints(maxWidth: 220),
                  padding: const EdgeInsetsDirectional.fromSTEB(10, 6, 4, 6),
                  decoration: BoxDecoration(
                    color: colors.secondary,
                    borderRadius: BorderRadius.circular(12),
                  ),
                  child: Row(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Icon(
                        Icons.description_outlined,
                        size: 18,
                        color: colors.primary,
                      ),
                      const SizedBox(width: 8),
                      Flexible(
                        child: Text(
                          file.name,
                          maxLines: 1,
                          overflow: TextOverflow.ellipsis,
                          style: TextStyle(
                            color: colors.foreground,
                            fontSize: 13,
                          ),
                        ),
                      ),
                      IconButton(
                        tooltip: 'Remove ${file.name}',
                        constraints: const BoxConstraints(
                          minWidth: 48,
                          minHeight: 48,
                        ),
                        onPressed: () => onRemove(file),
                        icon: Icon(
                          Icons.close,
                          size: 16,
                          color: colors.mutedForeground,
                        ),
                      ),
                    ],
                  ),
                ),
              ),
            ),
        ],
      ),
    );
  }
}

String _formatBytes(int bytes) {
  if (bytes >= 1024 * 1024) {
    return '${(bytes / (1024 * 1024)).toStringAsFixed(1)} MB';
  }
  if (bytes >= 1024) return '${(bytes / 1024).toStringAsFixed(1)} KB';
  return '$bytes B';
}

String _mimeFor(String? extension) => switch (extension?.toLowerCase()) {
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
  _ => 'application/octet-stream',
};

class _ConversationSheet extends ConsumerStatefulWidget {
  const _ConversationSheet({required this.close});

  final Future<void> Function() close;

  @override
  ConsumerState<_ConversationSheet> createState() => _ConversationSheetState();
}

class _ConversationSheetState extends ConsumerState<_ConversationSheet> {
  final _searchController = TextEditingController();
  String _query = '';

  @override
  void dispose() {
    _searchController.dispose();
    super.dispose();
  }

  /// Rename in place, from a dialog seeded with the current title.
  ///
  /// React has had rename since it shipped; here the title was static text, so
  /// a conversation kept whatever its first message auto-titled it forever.
  Future<void> _rename(String id, String currentTitle) async {
    final controller = TextEditingController(text: currentTitle);
    // A sheet rather than a dialog: it is the overlay primitive this app
    // already uses everywhere else, so focus, dismissal and theming behave the
    // way the rest of the surface does.
    // The overlay completer is not a Future, so the controller is disposed when
    // the sheet reports closed rather than on a bogus await.
    final sheet = shad.openSheetOverlay<void>(
      context: context,
      position: shad.OverlayPosition.bottom,
      builder: (sheetContext) => SafeArea(
        child: Padding(
          padding: const EdgeInsets.all(16),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              const Text('Rename conversation').semiBold(),
              const SizedBox(height: 12),
              shad.TextField(
                controller: controller,
                autofocus: true,
                placeholder: const Text('Conversation title'),
                onSubmitted: (value) {
                  shad.closeSheet(sheetContext);
                  unawaited(_submitRename(id, value));
                },
              ),
              const SizedBox(height: 12),
              Row(
                mainAxisAlignment: MainAxisAlignment.end,
                children: [
                  shad.SecondaryButton(
                    onPressed: () => shad.closeSheet(sheetContext),
                    child: const Text('Cancel'),
                  ),
                  const SizedBox(width: 8),
                  shad.PrimaryButton(
                    onPressed: () {
                      shad.closeSheet(sheetContext);
                      unawaited(_submitRename(id, controller.text));
                    },
                    child: const Text('Rename'),
                  ),
                ],
              ),
            ],
          ),
        ),
      ),
    );
    await sheet.future;
    controller.dispose();
  }

  Future<void> _submitRename(String id, String title) async {
    if (!mounted) return;
    await ref.read(chatProvider.notifier).renameConversation(id, title);
  }

  @override
  Widget build(BuildContext context) {
    final state = ref.watch(chatProvider);
    final colors = shad.Theme.of(context).colorScheme;
    // Case-insensitive substring on the title, matching React's thread search.
    // Body text is deliberately not searched on either surface.
    final query = _query.trim().toLowerCase();
    final conversations = query.isEmpty
        ? state.conversations
        : state.conversations
              .where(
                (conversation) =>
                    conversation.title.toLowerCase().contains(query),
              )
              .toList();
    return ColoredBox(
      color: colors.background,
      child: SafeArea(
        child: Padding(
          padding: const EdgeInsets.all(16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              Row(
                children: [
                  const Expanded(
                    child: Text(
                      'Conversations',
                      style: TextStyle(
                        fontSize: 22,
                        fontWeight: FontWeight.w700,
                      ),
                    ),
                  ),
                  Semantics(
                    button: true,
                    label: 'New conversation',
                    excludeSemantics: true,
                    child: shad.PrimaryButton(
                      onPressed: () async {
                        await ref
                            .read(chatProvider.notifier)
                            .createConversation();
                        await widget.close();
                      },
                      density: shad.ButtonDensity.icon,
                      child: const Icon(Icons.add),
                    ),
                  ),
                ],
              ),
              const SizedBox(height: 12),
              // Only offered when there is something to search — a filter over
              // an empty list is noise.
              if (state.conversations.isNotEmpty) ...[
                Semantics(
                  textField: true,
                  label: 'Search threads',
                  child: shad.TextField(
                    controller: _searchController,
                    placeholder: const Text('Search threads'),
                    features: const [shad.InputFeature.leading(Icon(Icons.search))],
                    onChanged: (value) => setState(() => _query = value),
                  ),
                ),
                const SizedBox(height: 12),
              ],
              Expanded(
                child: conversations.isEmpty
                    ? Center(
                        child: Text(
                          'No threads found',
                          style: TextStyle(color: colors.mutedForeground),
                        ),
                      )
                    : ListView.separated(
                        itemCount: conversations.length,
                        separatorBuilder: (_, _) => const SizedBox(height: 8),
                        itemBuilder: (context, index) {
                          final conversation = conversations[index];
                          final selected = conversation.id == state.threadId;
                          return ColoredBox(
                            color: selected ? colors.accent : colors.card,
                            child: Row(
                              children: [
                                Expanded(
                                  child: shad.GhostButton(
                                    alignment: Alignment.centerLeft,
                                    onPressed: () async {
                                      await ref
                                          .read(chatProvider.notifier)
                                          .selectConversation(conversation.id);
                                      await widget.close();
                                    },
                                    child: Text(
                                      conversation.title,
                                      maxLines: 2,
                                      overflow: TextOverflow.ellipsis,
                                    ),
                                  ),
                                ),
                                Semantics(
                                  button: true,
                                  label: 'Rename ${conversation.title}',
                                  excludeSemantics: true,
                                  child: shad.GhostButton(
                                    onPressed: () => _rename(conversation.id, conversation.title),
                                    density: shad.ButtonDensity.icon,
                                    child: const Icon(Icons.edit_outlined),
                                  ),
                                ),
                                Semantics(
                                  button: true,
                                  label: 'Delete ${conversation.title}',
                                  excludeSemantics: true,
                                  child: shad.GhostButton(
                                    onPressed: () => ref
                                        .read(chatProvider.notifier)
                                        .deleteConversation(conversation.id),
                                    density: shad.ButtonDensity.icon,
                                    child: const Icon(Icons.delete_outline),
                                  ),
                                ),
                              ],
                            ),
                          );
                        },
                      ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class _ChatWelcome extends StatelessWidget {
  const _ChatWelcome({required this.colors});
  final shad.ColorScheme colors;

  @override
  Widget build(BuildContext context) => Center(
    child: SingleChildScrollView(
      padding: const EdgeInsets.all(32),
      child: ConstrainedBox(
        constraints: const BoxConstraints(maxWidth: 520),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Container(
              width: 64,
              height: 64,
              decoration: BoxDecoration(
                color: colors.accent,
                borderRadius: BorderRadius.circular(18),
              ),
              child: Icon(Icons.auto_awesome, color: colors.primary, size: 30),
            ),
            const SizedBox(height: 20),
            const Text(
              'KnowMe',
              style: TextStyle(fontSize: 30, fontWeight: FontWeight.w700),
            ),
            const SizedBox(height: 10),
            const Text(
              'What would you like to understand?',
              textAlign: TextAlign.center,
              style: TextStyle(fontSize: 24, fontWeight: FontWeight.w700),
            ),
            const SizedBox(height: 10),
            Text(
              'Think out loud, revisit what matters, or connect the details you have shared—privately, on your device.',
              textAlign: TextAlign.center,
              style: TextStyle(color: colors.mutedForeground, height: 1.45),
            ),
          ],
        ),
      ),
    ),
  );
}
