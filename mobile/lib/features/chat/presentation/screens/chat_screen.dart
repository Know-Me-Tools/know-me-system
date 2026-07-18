// TJ-ARCH-MOB-001 compliant
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:gen_ui_widgets/gen_ui_widgets.dart';
import 'package:shadcn_flutter/shadcn_flutter.dart' as shad;

import '../../../../shared/widgets/knowme_screen.dart';
import '../../../../shared/widgets/sync_chip.dart';
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
  bool _isSending = false;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      ref.read(chatProvider.notifier).load();
    });
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final state = ref.watch(chatProvider);
    final colors = shad.Theme.of(context).colorScheme;

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
            child: state.messages.isEmpty
                ? _ChatWelcome(colors: colors)
                : ListView.separated(
                    padding: const EdgeInsets.all(16),
                    itemCount: state.messages.length,
                    separatorBuilder: (_, __) => const SizedBox(height: 16),
                    itemBuilder: (context, i) {
                      final msg = state.messages[i];
                      final isUser = msg.role == 'user';
                      return Align(
                        alignment: isUser
                            ? Alignment.centerRight
                            : Alignment.centerLeft,
                        child: Container(
                          constraints: BoxConstraints(
                            maxWidth: MediaQuery.sizeOf(context).width * 0.84,
                          ),
                          padding: const EdgeInsets.all(14),
                          decoration: BoxDecoration(
                            color: isUser ? colors.muted : colors.card,
                            borderRadius: BorderRadius.only(
                              topLeft: const Radius.circular(18),
                              topRight: const Radius.circular(18),
                              bottomLeft: Radius.circular(isUser ? 18 : 5),
                              bottomRight: Radius.circular(isUser ? 5 : 18),
                            ),
                          ),
                          child: Column(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: [
                              for (final block in msg.content)
                                ContentBlockView(block: block),
                              if (msg.isStreaming)
                                const Padding(
                                  padding: EdgeInsets.only(top: 4),
                                  child: SizedBox(
                                    width: 14,
                                    height: 14,
                                    child: CircularProgressIndicator(
                                      strokeWidth: 2,
                                    ),
                                  ),
                                ),
                            ],
                          ),
                        ),
                      );
                    },
                  ),
          ),
          SafeArea(
            child: Padding(
              padding: const EdgeInsets.all(12),
              child: Row(
                children: [
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
                  shad.PrimaryButton(
                    onPressed: _isSending ? null : _send,
                    density: shad.ButtonDensity.icon,
                    child: _isSending
                        ? const SizedBox(
                            width: 18,
                            height: 18,
                            child: CircularProgressIndicator(strokeWidth: 2),
                          )
                        : const Icon(Icons.arrow_upward),
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
      builder: (sheetContext) => _ConversationSheet(
        close: () => shad.closeSheet(sheetContext),
      ),
    );
  }

  Future<void> _send() async {
    final text = _controller.text.trim();
    if (text.isEmpty) return;
    _controller.clear();
    setState(() => _isSending = true);
    try {
      await ref.read(chatProvider.notifier).sendMessage(text);
    } finally {
      if (mounted) setState(() => _isSending = false);
    }
  }
}

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
                      style:
                          TextStyle(fontSize: 22, fontWeight: FontWeight.w700),
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
                  child:
                      Icon(Icons.auto_awesome, color: colors.primary, size: 30),
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
