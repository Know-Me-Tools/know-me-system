// TJ-ARCH-MOB-001 compliant
import 'package:flutter/material.dart';
import 'content_block.dart';

/// Renders one ContentBlock. Exhaustive over all 11 variants: Dart's switch on a
/// sealed class is a compile error if a case is missing — no default branch.
/// Presentational only (no FFI calls, no providers). Feed it blocks a
/// ChatNotifier folded from the chatEvents stream.
class ContentBlockView extends StatelessWidget {
  final ContentBlock block;
  const ContentBlockView({required this.block, super.key});

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return switch (block) {
      TextBlock(:final text) => Text(text, style: theme.textTheme.bodyMedium),
      ThinkingBlock(:final text) => Opacity(
          opacity: 0.6,
          child: Text(text, style: theme.textTheme.bodySmall?.copyWith(fontStyle: FontStyle.italic)),
        ),
      CodeBlock(:final language, :final code) => Container(
          padding: const EdgeInsets.all(12),
          decoration: BoxDecoration(
            color: theme.colorScheme.surfaceContainerHighest,
            borderRadius: BorderRadius.circular(8),
          ),
          child: Text('[$language]\n$code', style: const TextStyle(fontFamily: 'monospace')),
        ),
      CitationBlock(:final source, :final quote) => Padding(
          padding: const EdgeInsets.only(left: 12),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(quote, style: theme.textTheme.bodyMedium?.copyWith(fontStyle: FontStyle.italic)),
              Text(source, style: theme.textTheme.labelSmall),
            ],
          ),
        ),
      MemoryBlock(:final key, :final value) => ListTile(
          dense: true,
          leading: const Icon(Icons.memory),
          title: Text(key),
          subtitle: value != null ? Text(value) : null,
        ),
      ToolUseBlock(:final name) => Chip(
          avatar: const Icon(Icons.build, size: 16),
          label: Text(name),
        ),
      ToolResultBlock(:final outputJson, :final isError) => Container(
          padding: const EdgeInsets.all(8),
          color: isError ? theme.colorScheme.errorContainer : theme.colorScheme.surfaceContainer,
          child: Text(outputJson),
        ),
      SkillBlock(:final name, :final status) => Chip(
          avatar: const Icon(Icons.extension, size: 16),
          label: Text('$name · $status'),
        ),
      ArtifactBlock(:final kind, :final content) => Card(
          child: Padding(
            padding: const EdgeInsets.all(12),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [Text(kind, style: theme.textTheme.labelMedium), Text(content)],
            ),
          ),
        ),
      ImageBlock(:final url) => url != null
          ? Image.network(url)
          : const SizedBox.shrink(),
      DividerBlock() => const Divider(),
    };
  }
}
