// TJ-ARCH-MOB-001 compliant
import 'dart:convert';

import 'package:file_picker/file_picker.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:gen_ui_widgets/gen_ui_widgets.dart';
import 'package:shadcn_flutter/shadcn_flutter.dart' as shad;

import '../../../../core/theme/tokens.dart';
import '../../../../shared/widgets/knowme_screen.dart';
import '../../../../shared/widgets/sync_chip.dart';
import '../../domain/entities/chat_message.dart';
import '../providers/chat_notifier.dart';

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
    _scrollController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final state = ref.watch(chatProvider);
    final colors = shad.Theme.of(context).colorScheme;
    ref.listen<int>(
      chatProvider.select(
        (value) => value.messages.fold<int>(
          0,
          (total, message) =>
              total + message.content.length + (message.isStreaming ? 1 : 0),
        ),
      ),
      (_, __) => _scheduleScrollToLatest(),
    );

    return KnowMeScreen(
      title: 'Chat',
      trailing: [
        shad.GhostButton(
          onPressed: _openConversations,
          density: shad.ButtonDensity.icon,
          child: const Icon(Icons.forum_outlined),
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
                    separatorBuilder: (_, __) => const SizedBox(height: 16),
                    itemBuilder: (context, i) => _MessageBubble(
                      message: state.messages[i],
                      onInteraction: (interaction) => ref
                          .read(chatProvider.notifier)
                          .submitInteraction(interaction),
                      remoteResolver: (url, kind) => ref
                          .read(chatProvider.notifier)
                          .resolveRemoteContent(url, kind),
                    ),
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
                          onPressed: _isSending ? null : _pickAttachments,
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
                        child: shad.TextField(
                          controller: _controller,
                          placeholder: const Text('Ask KnowMe anything…'),
                          minLines: 1,
                          maxLines: 5,
                          onSubmitted: _isSending ? null : (_) => _send(),
                        ),
                      ),
                      const SizedBox(width: 8),
                      Tooltip(
                        message: 'Send message',
                        child: shad.PrimaryButton(
                          onPressed: _isSending ? null : _send,
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
                                  label: 'Send message',
                                  child: const Icon(Icons.arrow_upward),
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
    shad.openSheetOverlay<void>(
      context: context,
      position: shad.OverlayPosition.left,
      constraints: const BoxConstraints(maxWidth: 360),
      builder: (sheetContext) =>
          _ConversationSheet(close: () => shad.closeSheet(sheetContext)),
    );
  }

  Future<void> _send() async {
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
    final attachmentPaths = [
      for (final file in _attachments)
        if (file.path case final path?) path,
    ];
    _controller.clear();
    setState(() {
      _attachments.clear();
      _attachmentError = null;
      _isSending = true;
      _isNearBottom = true;
    });
    try {
      await ref
          .read(chatProvider.notifier)
          .sendMessage(
            text.isEmpty ? 'Please review the attached files.' : text,
            attachments: attachmentBlocks,
            attachmentPaths: attachmentPaths,
          );
    } finally {
      if (mounted) setState(() => _isSending = false);
    }
  }

  Future<void> _pickAttachments() async {
    final result = await FilePicker.platform.pickFiles(
      allowMultiple: true,
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
    if (!mounted || result == null) return;
    const maxBytes = 25 * 1024 * 1024;
    final tooLarge = result.files
        .where((file) => file.size > maxBytes)
        .toList();
    final unavailable = result.files
        .where((file) => file.path == null)
        .toList();
    setState(() {
      _attachments.addAll(
        result.files
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

class _MessageBubble extends StatelessWidget {
  const _MessageBubble({
    required this.message,
    this.onInteraction,
    this.remoteResolver,
  });

  final ChatMessage message;
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
    return Align(
      alignment: isUser ? Alignment.centerRight : Alignment.centerLeft,
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
                child: Tooltip(
                  message: 'Copy entire message',
                  child: IconButton(
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
              ),
            ],
          ),
        ),
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

class _ConversationSheet extends ConsumerWidget {
  const _ConversationSheet({required this.close});

  final Future<void> Function() close;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final state = ref.watch(chatProvider);
    final colors = shad.Theme.of(context).colorScheme;
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
                  shad.PrimaryButton(
                    onPressed: () async {
                      await ref
                          .read(chatProvider.notifier)
                          .createConversation();
                      await close();
                    },
                    density: shad.ButtonDensity.icon,
                    child: const Icon(Icons.add),
                  ),
                ],
              ),
              const SizedBox(height: 18),
              Expanded(
                child: ListView.separated(
                  itemCount: state.conversations.length,
                  separatorBuilder: (_, __) => const SizedBox(height: 8),
                  itemBuilder: (context, index) {
                    final conversation = state.conversations[index];
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
                                await close();
                              },
                              child: Text(
                                conversation.title,
                                maxLines: 2,
                                overflow: TextOverflow.ellipsis,
                              ),
                            ),
                          ),
                          shad.GhostButton(
                            onPressed: () => ref
                                .read(chatProvider.notifier)
                                .deleteConversation(conversation.id),
                            density: shad.ButtonDensity.icon,
                            child: const Icon(Icons.delete_outline),
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
