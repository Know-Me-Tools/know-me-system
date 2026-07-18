// TJ-ARCH-MOB-001 compliant
import 'dart:async';
import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_mermaid/flutter_mermaid.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:markdown_widget/markdown_widget.dart';
import 'package:shadcn_flutter/shadcn_flutter.dart' as shad;
import 'package:video_player/video_player.dart';

import 'a2ui_surface_view.dart';
import 'content_block.dart';

/// Exhaustive dispatcher for KnowMe's 11-variant A2UI projection.
///
/// Each branch delegates to a dedicated presentational widget. There is no
/// default branch: extending [ContentBlock] must break this switch until the
/// new block has an accessible renderer.
class ContentBlockView extends StatelessWidget {
  const ContentBlockView({
    required this.block,
    this.isStreaming = false,
    super.key,
  });

  final ContentBlock block;
  final bool isStreaming;

  @override
  Widget build(BuildContext context) => switch (block) {
    TextBlock(:final text) => ContentBlockText(
      text: text,
      isStreaming: isStreaming,
    ),
    ThinkingBlock(:final text) => ContentBlockThinking(
      text: text,
      isStreaming: isStreaming,
    ),
    CodeBlock(:final language, :final code) => ContentBlockCode(
      language: language,
      code: code,
    ),
    CitationBlock(:final source, :final quote) => ContentBlockCitation(
      source: source,
      quote: quote,
    ),
    MemoryBlock(:final operation, :final key, :final value) =>
      ContentBlockMemory(operation: operation, memoryKey: key, value: value),
    ToolUseBlock(:final id, :final name, :final inputJson) =>
      ContentBlockToolUse(id: id, name: name, inputJson: inputJson),
    ToolResultBlock(:final toolUseId, :final outputJson, :final isError) =>
      ContentBlockToolResult(
        toolUseId: toolUseId,
        outputJson: outputJson,
        isError: isError,
      ),
    SkillBlock(:final name, :final status) => ContentBlockSkill(
      name: name,
      status: status,
    ),
    ArtifactBlock(:final id, :final kind, :final content) =>
      ContentBlockArtifact(id: id, kind: kind, content: content),
    ImageBlock(:final url, :final dataBase64, :final mime) => ContentBlockImage(
      url: url,
      dataBase64: dataBase64,
      mime: mime,
    ),
    DividerBlock() => const ContentBlockDivider(),
  };
}

class ContentBlockText extends StatelessWidget {
  const ContentBlockText({
    required this.text,
    this.isStreaming = false,
    super.key,
  });

  final String text;
  final bool isStreaming;

  @override
  Widget build(BuildContext context) => Semantics(
    liveRegion: isStreaming,
    label: isStreaming ? 'Assistant response updating' : null,
    child: GenUiMarkdown(data: text),
  );
}

class ContentBlockThinking extends StatefulWidget {
  const ContentBlockThinking({
    required this.text,
    this.isStreaming = false,
    super.key,
  });

  final String text;
  final bool isStreaming;

  @override
  State<ContentBlockThinking> createState() => _ContentBlockThinkingState();
}

class _ContentBlockThinkingState extends State<ContentBlockThinking> {
  bool _expanded = false;

  @override
  Widget build(BuildContext context) {
    final colors = shad.Theme.of(context).colorScheme;
    return _BlockSurface(
      semanticLabel: 'Thinking',
      background: colors.secondary,
      header: Row(
        children: [
          Icon(Icons.psychology_outlined, size: 18, color: colors.primary),
          const SizedBox(width: 8),
          Expanded(
            child: Text(
              widget.isStreaming ? 'Thinking…' : 'Thinking',
              style: TextStyle(
                color: colors.foreground,
                fontWeight: FontWeight.w600,
              ),
            ),
          ),
          _CopyButton(value: widget.text, label: 'Copy thinking'),
          Semantics(
            button: true,
            expanded: _expanded,
            label: _expanded ? 'Collapse thinking' : 'Expand thinking',
            child: IconButton(
              tooltip: _expanded ? 'Collapse thinking' : 'Expand thinking',
              onPressed: () => setState(() => _expanded = !_expanded),
              icon: Icon(
                _expanded ? Icons.expand_less : Icons.expand_more,
                color: colors.mutedForeground,
              ),
            ),
          ),
        ],
      ),
      child: _expanded
          ? Semantics(
              liveRegion: widget.isStreaming,
              child: Padding(
                padding: const EdgeInsets.only(top: 8),
                child: GenUiMarkdown(data: widget.text),
              ),
            )
          : null,
    );
  }
}

class ContentBlockCode extends StatelessWidget {
  const ContentBlockCode({
    required this.language,
    required this.code,
    super.key,
  });

  final String language;
  final String code;

  @override
  Widget build(BuildContext context) => language.toLowerCase() == 'mermaid'
      ? ContentBlockMermaid(source: code)
      : _CodeSurface(language: language, code: code);
}

class ContentBlockCitation extends StatelessWidget {
  const ContentBlockCitation({
    required this.source,
    required this.quote,
    super.key,
  });

  final String source;
  final String quote;

  @override
  Widget build(BuildContext context) {
    final colors = shad.Theme.of(context).colorScheme;
    return _BlockSurface(
      semanticLabel: 'Citation from $source',
      background: colors.card,
      header: _BlockHeader(
        icon: Icons.menu_book_outlined,
        label: 'Citation · $source',
        copyValue: '$quote\n$source',
      ),
      child: Padding(
        padding: const EdgeInsets.only(top: 8),
        child: Text(
          quote,
          style: TextStyle(
            color: colors.mutedForeground,
            fontStyle: FontStyle.italic,
            height: 1.5,
          ),
        ),
      ),
    );
  }
}

class ContentBlockMemory extends StatelessWidget {
  const ContentBlockMemory({
    required this.operation,
    required this.memoryKey,
    this.value,
    super.key,
  });

  final String operation;
  final String memoryKey;
  final String? value;

  @override
  Widget build(BuildContext context) {
    final colors = shad.Theme.of(context).colorScheme;
    final normalized = operation.toLowerCase();
    final action = switch (normalized) {
      'read' || 'recall' => 'Recalled memory',
      'write' || 'create' => 'Saved memory',
      'update' => 'Updated memory',
      'reject' || 'rejected' => 'Memory rejected',
      _ => 'Memory · $operation',
    };
    return _BlockSurface(
      semanticLabel: action,
      background: colors.secondary,
      header: _BlockHeader(
        icon: Icons.memory,
        label: action,
        copyValue: [memoryKey, if (value != null) value!].join('\n'),
      ),
      child: Padding(
        padding: const EdgeInsets.only(top: 8),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              memoryKey,
              style: TextStyle(
                color: colors.foreground,
                fontWeight: FontWeight.w600,
              ),
            ),
            if (value case final memoryValue?) ...[
              const SizedBox(height: 4),
              Text(
                memoryValue,
                style: TextStyle(color: colors.mutedForeground, height: 1.45),
              ),
            ],
          ],
        ),
      ),
    );
  }
}

class ContentBlockToolUse extends StatelessWidget {
  const ContentBlockToolUse({
    required this.id,
    required this.name,
    required this.inputJson,
    super.key,
  });

  final String id;
  final String name;
  final String inputJson;

  @override
  Widget build(BuildContext context) => _DisclosureBlock(
    semanticLabel: 'Tool use $name',
    icon: Icons.build_outlined,
    title: name,
    status: 'Tool request',
    body: _prettyJson(inputJson),
  );
}

class ContentBlockToolResult extends StatelessWidget {
  const ContentBlockToolResult({
    required this.toolUseId,
    required this.outputJson,
    required this.isError,
    super.key,
  });

  final String toolUseId;
  final String outputJson;
  final bool isError;

  @override
  Widget build(BuildContext context) => _DisclosureBlock(
    semanticLabel: isError ? 'Tool failed' : 'Tool completed',
    icon: isError ? Icons.error_outline : Icons.check_circle_outline,
    title: isError ? 'Tool failed' : 'Tool completed',
    status: 'Call ${_shortId(toolUseId)}',
    body: _prettyJson(outputJson),
    isError: isError,
  );
}

class ContentBlockSkill extends StatelessWidget {
  const ContentBlockSkill({
    required this.name,
    required this.status,
    super.key,
  });

  final String name;
  final String status;

  @override
  Widget build(BuildContext context) {
    final colors = shad.Theme.of(context).colorScheme;
    return _BlockSurface(
      semanticLabel: 'Skill $name, $status',
      background: colors.secondary,
      header: _BlockHeader(
        icon: Icons.auto_awesome_outlined,
        label: name,
        copyValue: '$name · $status',
        status: status,
      ),
    );
  }
}

class ContentBlockArtifact extends StatelessWidget {
  const ContentBlockArtifact({
    required this.id,
    required this.kind,
    required this.content,
    super.key,
  });

  final String id;
  final String kind;
  final String content;

  @override
  Widget build(BuildContext context) {
    final normalized = kind.toLowerCase().trim();
    final child = normalized.startsWith('agui_')
        ? _AguiStructuredEvent(kind: normalized, content: content)
        : switch (normalized) {
            'markdown' || 'md' || 'document' => GenUiMarkdown(data: content),
            'mermaid' => ContentBlockMermaid(source: content),
            'svg' || 'image/svg+xml' => ContentBlockSvg(source: content),
            'video' ||
            'mp4' ||
            'webm' => ContentBlockVideo(source: _mediaUrl(content)),
            'opengraph' ||
            'open_graph' ||
            'link' => ContentBlockLinkPreview.fromJson(content),
            'code' => _CodeSurface(language: 'text', code: content),
            'pdf' || 'doc' || 'docx' => ContentBlockDocumentPreview(
              kind: normalized,
              content: content,
            ),
            'attachment' => ContentBlockDocumentPreview(
              kind: 'attachment',
              content: content,
            ),
            'a2ui' ||
            'application/a2ui+json' => A2uiSurfaceView(source: content),
            _ => SelectableText(content),
          };
    return _BlockSurface(
      semanticLabel: '$kind artifact',
      background: shad.Theme.of(context).colorScheme.secondary,
      header: _BlockHeader(
        icon: Icons.inventory_2_outlined,
        label: 'Artifact · $kind',
        copyValue: content,
      ),
      child: Padding(padding: const EdgeInsets.only(top: 8), child: child),
    );
  }
}

class _AguiStructuredEvent extends StatelessWidget {
  const _AguiStructuredEvent({required this.kind, required this.content});

  final String kind;
  final String content;

  @override
  Widget build(BuildContext context) {
    final decoded = _tryJson(content);
    if (kind == 'agui_confirmation') {
      return _ConfirmationRequest(value: decoded);
    }
    final (title, summary) = switch (kind) {
      'agui_state_snapshot' => (
        'Agent state synchronized',
        _mapSummary(decoded),
      ),
      'agui_state_delta' => ('Agent state updated', _patchSummary(decoded)),
      'agui_messages_snapshot' => (
        'Conversation synchronized',
        '${decoded is List ? decoded.length : 0} messages',
      ),
      final value when value.startsWith('agui_activity_') => (
        _activityTitle(value),
        _mapSummary(decoded),
      ),
      'agui_raw' => ('External agent event', 'Inspect event details'),
      _ => ('Agent event', kind.replaceAll('_', ' ')),
    };
    return _DisclosureBlock(
      semanticLabel: '$title. $summary',
      icon: Icons.sync_alt_outlined,
      title: title,
      status: summary,
      body: _prettyJson(content),
    );
  }
}

class _ConfirmationRequest extends StatelessWidget {
  const _ConfirmationRequest({required this.value});
  final dynamic value;

  @override
  Widget build(BuildContext context) {
    final map = value is Map
        ? Map<String, dynamic>.from(value as Map)
        : <String, dynamic>{};
    final title = map['title'] as String? ?? 'Confirmation required';
    final message =
        map['message'] as String? ??
        map['consequence'] as String? ??
        'Review this request before the agent continues.';
    final choices =
        map['choices'] as List<dynamic>? ?? const ['Cancel', 'Confirm'];
    return Semantics(
      container: true,
      label: '$title. $message',
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(title, style: const TextStyle(fontWeight: FontWeight.w700)),
          const SizedBox(height: 6),
          Text(message),
          const SizedBox(height: 12),
          Wrap(
            spacing: 8,
            runSpacing: 8,
            children: [
              for (final choice in choices)
                shad.OutlineButton(
                  // Response transport is intentionally disabled until a typed
                  // Rust confirmation command is connected by the host app.
                  onPressed: null,
                  child: Text(
                    choice is Map
                        ? (choice['label'] as String? ?? 'Choose')
                        : choice.toString(),
                  ),
                ),
            ],
          ),
        ],
      ),
    );
  }
}

dynamic _tryJson(String source) {
  try {
    return jsonDecode(source);
  } on FormatException {
    return source;
  }
}

String _mapSummary(dynamic value) {
  if (value is Map) {
    final keys = value.keys.take(3).join(', ');
    return value.isEmpty ? 'No visible fields' : 'Updated $keys';
  }
  if (value is List) return '${value.length} updates';
  return 'State available';
}

String _patchSummary(dynamic value) {
  if (value is! List) return 'State changed';
  final paths = value
      .whereType<Map>()
      .map((item) => item['path'])
      .whereType<String>()
      .take(3)
      .join(', ');
  return paths.isEmpty ? '${value.length} changes' : 'Changed $paths';
}

String _activityTitle(String kind) {
  final name = kind
      .replaceFirst('agui_activity_', '')
      .replaceFirst(RegExp(r'_delta$'), '')
      .replaceAll('_', ' ');
  return '${name.isEmpty ? 'Agent' : name[0].toUpperCase() + name.substring(1)} activity';
}

class ContentBlockImage extends StatelessWidget {
  const ContentBlockImage({
    required this.url,
    required this.dataBase64,
    required this.mime,
    this.alt = 'Generated image',
    super.key,
  });

  final String? url;
  final String? dataBase64;
  final String mime;
  final String alt;

  @override
  Widget build(BuildContext context) {
    Widget image;
    if (mime == 'image/svg+xml' && dataBase64 != null) {
      final source = utf8.decode(base64Decode(dataBase64!));
      image = SvgPicture.string(_sanitizeSvg(source), semanticsLabel: alt);
    } else if (dataBase64 != null) {
      image = Image.memory(
        base64Decode(dataBase64!),
        fit: BoxFit.contain,
        semanticLabel: alt,
        errorBuilder: (_, __, ___) => const _MediaError(label: 'Image'),
      );
    } else if (url != null) {
      image = Image.network(
        url!,
        fit: BoxFit.contain,
        semanticLabel: alt,
        errorBuilder: (_, __, ___) => const _MediaError(label: 'Image'),
      );
    } else {
      image = const _MediaError(label: 'Image');
    }
    return _BlockSurface(
      semanticLabel: alt,
      background: shad.Theme.of(context).colorScheme.secondary,
      header: _BlockHeader(
        icon: Icons.image_outlined,
        label: mime,
        copyValue: url ?? dataBase64 ?? '',
      ),
      child: Padding(
        padding: const EdgeInsets.only(top: 8),
        child: ConstrainedBox(
          constraints: const BoxConstraints(maxHeight: 420, minHeight: 120),
          child: Center(child: image),
        ),
      ),
    );
  }
}

class ContentBlockDivider extends StatelessWidget {
  const ContentBlockDivider({super.key});

  @override
  Widget build(BuildContext context) =>
      Semantics(label: 'Section break', child: const SizedBox(height: 20));
}

class GenUiMarkdown extends StatelessWidget {
  const GenUiMarkdown({required this.data, super.key});

  final String data;

  @override
  Widget build(BuildContext context) {
    final colors = shad.Theme.of(context).colorScheme;
    final dark = colors.background.computeLuminance() < 0.5;
    final base = dark
        ? MarkdownConfig.darkConfig
        : MarkdownConfig.defaultConfig;
    final config = base.copy(
      configs: [
        PConfig(
          textStyle: TextStyle(
            color: colors.foreground,
            fontSize: 16,
            height: 1.5,
          ),
        ),
        for (final heading in const [
          (MarkdownTag.h1, 30.0),
          (MarkdownTag.h2, 25.0),
          (MarkdownTag.h3, 22.0),
          (MarkdownTag.h4, 19.0),
          (MarkdownTag.h5, 17.0),
          (MarkdownTag.h6, 16.0),
        ])
          _FlatHeadingConfig(
            tag: heading.$1.name,
            style: TextStyle(
              color: colors.foreground,
              fontSize: heading.$2,
              height: 1.25,
              fontWeight: FontWeight.w700,
            ),
          ),
        LinkConfig(
          style: TextStyle(
            color: colors.primary,
            decoration: TextDecoration.underline,
            decorationColor: colors.primary,
          ),
        ),
        (dark ? PreConfig.darkConfig : const PreConfig()).copy(
          decoration: BoxDecoration(
            color: colors.background,
            borderRadius: BorderRadius.circular(12),
          ),
          textStyle: TextStyle(
            color: colors.foreground,
            fontFamily: 'monospace',
            fontSize: 13,
            height: 1.5,
          ),
          wrapper: (child, code, language) =>
              language.toLowerCase() == 'mermaid'
              ? ContentBlockMermaid(source: code)
              : _CodeFrame(language: language, code: code, child: child),
        ),
        ImgConfig(
          builder: (url, attributes) => _MarkdownImage(
            url: url,
            alt: attributes['alt'] ?? 'Generated image',
          ),
          errorBuilder: (_, alt, __) => _MediaError(label: alt),
        ),
        TableConfig(
          border: const TableBorder(),
          headerRowDecoration: BoxDecoration(color: colors.muted),
          bodyRowDecoration: BoxDecoration(color: colors.secondary),
          headerStyle: TextStyle(
            color: colors.foreground,
            fontWeight: FontWeight.w600,
          ),
          bodyStyle: TextStyle(color: colors.foreground),
          wrapper: (table) => SingleChildScrollView(
            scrollDirection: Axis.horizontal,
            child: table,
          ),
        ),
      ],
    );
    return MarkdownBlock(data: data, config: config, selectable: true);
  }
}

class _FlatHeadingConfig extends HeadingConfig {
  const _FlatHeadingConfig({required this.tag, required this.style});

  @override
  final String tag;

  @override
  final TextStyle style;
}

class ContentBlockMermaid extends StatelessWidget {
  const ContentBlockMermaid({required this.source, super.key});

  final String source;

  @override
  Widget build(BuildContext context) {
    final colors = shad.Theme.of(context).colorScheme;
    final dark = colors.background.computeLuminance() < 0.5;
    return _BlockSurface(
      semanticLabel: 'Mermaid diagram',
      background: colors.background,
      header: _BlockHeader(
        icon: Icons.account_tree_outlined,
        label: 'Mermaid diagram',
        copyValue: source,
      ),
      child: Padding(
        padding: const EdgeInsets.only(top: 8),
        child: Semantics(
          image: true,
          label: 'Diagram generated from Mermaid source',
          child: ConstrainedBox(
            constraints: const BoxConstraints(minHeight: 180, maxHeight: 420),
            child: InteractiveMermaidDiagram(
              code: source,
              style: dark ? MermaidStyle.dark() : MermaidStyle.neutral(),
            ),
          ),
        ),
      ),
    );
  }
}

class ContentBlockSvg extends StatelessWidget {
  const ContentBlockSvg({required this.source, super.key});

  final String source;

  @override
  Widget build(BuildContext context) => Semantics(
    image: true,
    label: 'Generated SVG artifact',
    child: ConstrainedBox(
      constraints: const BoxConstraints(minHeight: 120, maxHeight: 420),
      child: SvgPicture.string(
        _sanitizeSvg(source),
        fit: BoxFit.contain,
        semanticsLabel: 'Generated SVG artifact',
      ),
    ),
  );
}

class ContentBlockVideo extends StatefulWidget {
  const ContentBlockVideo({required this.source, super.key});

  final String source;

  @override
  State<ContentBlockVideo> createState() => _ContentBlockVideoState();
}

class _ContentBlockVideoState extends State<ContentBlockVideo> {
  VideoPlayerController? _controller;
  Object? _error;

  @override
  void initState() {
    super.initState();
    final uri = Uri.tryParse(widget.source);
    if (uri == null || !uri.hasScheme) {
      _error = FormatException('Video source is unavailable');
      return;
    }
    final controller = VideoPlayerController.networkUrl(uri);
    _controller = controller;
    unawaited(
      controller
          .initialize()
          .then((_) {
            if (mounted) setState(() {});
          })
          .catchError((Object error) {
            if (mounted) setState(() => _error = error);
          }),
    );
  }

  @override
  void dispose() {
    _controller?.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final controller = _controller;
    if (_error != null) return const _MediaError(label: 'Video');
    if (controller == null || !controller.value.isInitialized) {
      return const SizedBox(
        height: 160,
        child: Center(child: CircularProgressIndicator(strokeWidth: 2)),
      );
    }
    return Semantics(
      label: 'Video player',
      child: Column(
        children: [
          AspectRatio(
            aspectRatio: controller.value.aspectRatio,
            child: VideoPlayer(controller),
          ),
          Row(
            children: [
              IconButton(
                tooltip: controller.value.isPlaying
                    ? 'Pause video'
                    : 'Play video',
                onPressed: () async {
                  controller.value.isPlaying
                      ? await controller.pause()
                      : await controller.play();
                  if (mounted) setState(() {});
                },
                icon: Icon(
                  controller.value.isPlaying ? Icons.pause : Icons.play_arrow,
                ),
              ),
              Expanded(
                child: VideoProgressIndicator(controller, allowScrubbing: true),
              ),
              _CopyButton(value: widget.source, label: 'Copy video link'),
            ],
          ),
        ],
      ),
    );
  }
}

class ContentBlockDocumentPreview extends StatelessWidget {
  const ContentBlockDocumentPreview({
    required this.kind,
    required this.content,
    super.key,
  });

  final String kind;
  final String content;

  @override
  Widget build(BuildContext context) {
    final colors = shad.Theme.of(context).colorScheme;
    final metadata = _jsonMap(content);
    final name = metadata?['name']?.toString() ?? 'Document';
    final excerpt = metadata?['excerpt']?.toString();
    final size = metadata?['size']?.toString();
    return Semantics(
      label: '$kind document preview, $name',
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Icon(Icons.description_outlined, color: colors.primary, size: 32),
          const SizedBox(width: 12),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  name,
                  style: TextStyle(
                    color: colors.foreground,
                    fontWeight: FontWeight.w600,
                  ),
                ),
                Text(
                  [kind.toUpperCase(), if (size != null) size].join(' · '),
                  style: TextStyle(color: colors.mutedForeground, fontSize: 12),
                ),
                if (excerpt != null) ...[
                  const SizedBox(height: 8),
                  Text(
                    excerpt,
                    maxLines: 6,
                    overflow: TextOverflow.ellipsis,
                    style: TextStyle(
                      color: colors.mutedForeground,
                      height: 1.45,
                    ),
                  ),
                ],
              ],
            ),
          ),
        ],
      ),
    );
  }
}

class ContentBlockLinkPreview extends StatelessWidget {
  const ContentBlockLinkPreview({
    required this.url,
    this.title,
    this.description,
    this.siteName,
    this.imageUrl,
    super.key,
  });

  factory ContentBlockLinkPreview.fromJson(String content) {
    final json = _jsonMap(content);
    if (json == null) return ContentBlockLinkPreview(url: content);
    return ContentBlockLinkPreview(
      url: json['url']?.toString() ?? content,
      title: json['title']?.toString(),
      description: json['description']?.toString(),
      siteName: json['siteName']?.toString() ?? json['site_name']?.toString(),
      imageUrl: json['image']?.toString() ?? json['imageUrl']?.toString(),
    );
  }

  final String url;
  final String? title;
  final String? description;
  final String? siteName;
  final String? imageUrl;

  @override
  Widget build(BuildContext context) {
    final colors = shad.Theme.of(context).colorScheme;
    final host = Uri.tryParse(url)?.host;
    return Semantics(
      link: true,
      label: 'Link preview, ${title ?? host ?? url}',
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          if (imageUrl != null)
            ConstrainedBox(
              constraints: const BoxConstraints(maxHeight: 220),
              child: Image.network(
                imageUrl!,
                fit: BoxFit.cover,
                semanticLabel: 'Preview image for ${title ?? url}',
                errorBuilder: (_, __, ___) => const SizedBox.shrink(),
              ),
            ),
          Padding(
            padding: const EdgeInsets.all(12),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  siteName ?? host ?? 'Link',
                  style: TextStyle(
                    color: colors.primary,
                    fontSize: 12,
                    fontWeight: FontWeight.w600,
                  ),
                ),
                const SizedBox(height: 4),
                Text(
                  title ?? url,
                  style: TextStyle(
                    color: colors.foreground,
                    fontWeight: FontWeight.w600,
                  ),
                ),
                if (description != null) ...[
                  const SizedBox(height: 4),
                  Text(
                    description!,
                    maxLines: 3,
                    overflow: TextOverflow.ellipsis,
                    style: TextStyle(
                      color: colors.mutedForeground,
                      height: 1.4,
                    ),
                  ),
                ],
                Align(
                  alignment: AlignmentDirectional.centerEnd,
                  child: _CopyButton(value: url, label: 'Copy link'),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

class _DisclosureBlock extends StatefulWidget {
  const _DisclosureBlock({
    required this.semanticLabel,
    required this.icon,
    required this.title,
    required this.status,
    required this.body,
    this.isError = false,
  });

  final String semanticLabel;
  final IconData icon;
  final String title;
  final String status;
  final String body;
  final bool isError;

  @override
  State<_DisclosureBlock> createState() => _DisclosureBlockState();
}

class _DisclosureBlockState extends State<_DisclosureBlock> {
  bool _expanded = false;

  @override
  Widget build(BuildContext context) {
    final colors = shad.Theme.of(context).colorScheme;
    return _BlockSurface(
      semanticLabel: widget.semanticLabel,
      background: widget.isError
          ? colors.destructive.withValues(alpha: 0.12)
          : colors.secondary,
      header: Row(
        children: [
          Icon(
            widget.icon,
            size: 18,
            color: widget.isError ? colors.destructive : colors.primary,
          ),
          const SizedBox(width: 8),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  widget.title,
                  style: TextStyle(
                    color: colors.foreground,
                    fontWeight: FontWeight.w600,
                  ),
                ),
                Text(
                  widget.status,
                  style: TextStyle(color: colors.mutedForeground, fontSize: 12),
                ),
              ],
            ),
          ),
          _CopyButton(value: widget.body, label: 'Copy block'),
          IconButton(
            tooltip: _expanded ? 'Collapse details' : 'Expand details',
            onPressed: () => setState(() => _expanded = !_expanded),
            icon: Icon(
              _expanded ? Icons.expand_less : Icons.expand_more,
              color: colors.mutedForeground,
            ),
          ),
        ],
      ),
      child: _expanded
          ? Padding(
              padding: const EdgeInsets.only(top: 8),
              child: _CodeSurface(language: 'json', code: widget.body),
            )
          : null,
    );
  }
}

class _BlockSurface extends StatelessWidget {
  const _BlockSurface({
    required this.semanticLabel,
    required this.background,
    required this.header,
    this.child,
  });

  final String semanticLabel;
  final Color background;
  final Widget header;
  final Widget? child;

  @override
  Widget build(BuildContext context) => Semantics(
    container: true,
    label: semanticLabel,
    child: Container(
      width: double.infinity,
      margin: const EdgeInsets.symmetric(vertical: 6),
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: background,
        borderRadius: BorderRadius.circular(14),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [header, if (child != null) child!],
      ),
    ),
  );
}

class _BlockHeader extends StatelessWidget {
  const _BlockHeader({
    required this.icon,
    required this.label,
    required this.copyValue,
    this.status,
  });

  final IconData icon;
  final String label;
  final String copyValue;
  final String? status;

  @override
  Widget build(BuildContext context) {
    final colors = shad.Theme.of(context).colorScheme;
    return Row(
      children: [
        Icon(icon, size: 18, color: colors.primary),
        const SizedBox(width: 8),
        Expanded(
          child: Text(
            [label, if (status != null) status!].join(' · '),
            maxLines: 2,
            overflow: TextOverflow.ellipsis,
            style: TextStyle(
              color: colors.foreground,
              fontSize: 13,
              fontWeight: FontWeight.w600,
            ),
          ),
        ),
        _CopyButton(value: copyValue, label: 'Copy $label'),
      ],
    );
  }
}

class _CodeSurface extends StatelessWidget {
  const _CodeSurface({required this.language, required this.code});

  final String language;
  final String code;

  @override
  Widget build(BuildContext context) {
    final colors = shad.Theme.of(context).colorScheme;
    return _CodeFrame(
      language: language,
      code: code,
      child: SingleChildScrollView(
        scrollDirection: Axis.horizontal,
        child: SelectableText(
          code,
          style: TextStyle(
            color: colors.foreground,
            fontFamily: 'monospace',
            fontSize: 13,
            height: 1.5,
          ),
        ),
      ),
    );
  }
}

class _CodeFrame extends StatelessWidget {
  const _CodeFrame({
    required this.language,
    required this.code,
    required this.child,
  });

  final String language;
  final String code;
  final Widget child;

  @override
  Widget build(BuildContext context) {
    final colors = shad.Theme.of(context).colorScheme;
    return Semantics(
      container: true,
      label: '${language.isEmpty ? 'Code' : language} code block',
      child: Container(
        width: double.infinity,
        margin: const EdgeInsets.symmetric(vertical: 6),
        decoration: BoxDecoration(
          color: colors.background,
          borderRadius: BorderRadius.circular(12),
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Padding(
              padding: const EdgeInsets.fromLTRB(12, 4, 4, 0),
              child: Row(
                children: [
                  Expanded(
                    child: Text(
                      language.isEmpty ? 'code' : language,
                      style: TextStyle(
                        color: colors.mutedForeground,
                        fontSize: 12,
                        fontWeight: FontWeight.w600,
                      ),
                    ),
                  ),
                  _CopyButton(value: code, label: 'Copy code'),
                ],
              ),
            ),
            Padding(
              padding: const EdgeInsets.fromLTRB(12, 4, 12, 12),
              child: child,
            ),
          ],
        ),
      ),
    );
  }
}

class _MarkdownImage extends StatelessWidget {
  const _MarkdownImage({required this.url, required this.alt});

  final String url;
  final String alt;

  @override
  Widget build(BuildContext context) => _BlockSurface(
    semanticLabel: alt,
    background: shad.Theme.of(context).colorScheme.secondary,
    header: _BlockHeader(
      icon: Icons.image_outlined,
      label: alt,
      copyValue: url,
    ),
    child: Padding(
      padding: const EdgeInsets.only(top: 8),
      child: ConstrainedBox(
        constraints: const BoxConstraints(maxHeight: 420, minHeight: 120),
        child: Image.network(
          url,
          fit: BoxFit.contain,
          semanticLabel: alt,
          errorBuilder: (_, __, ___) => _MediaError(label: alt),
        ),
      ),
    ),
  );
}

class _CopyButton extends StatefulWidget {
  const _CopyButton({required this.value, required this.label});

  final String value;
  final String label;

  @override
  State<_CopyButton> createState() => _CopyButtonState();
}

class _CopyButtonState extends State<_CopyButton> {
  bool _copied = false;
  Timer? _timer;

  @override
  void dispose() {
    _timer?.cancel();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final colors = shad.Theme.of(context).colorScheme;
    return Semantics(
      button: true,
      label: _copied ? 'Copied' : widget.label,
      child: IconButton(
        tooltip: _copied ? 'Copied' : widget.label,
        constraints: const BoxConstraints(minWidth: 44, minHeight: 44),
        onPressed: widget.value.isEmpty
            ? null
            : () async {
                await Clipboard.setData(ClipboardData(text: widget.value));
                if (!mounted) return;
                setState(() => _copied = true);
                _timer?.cancel();
                _timer = Timer(const Duration(seconds: 2), () {
                  if (mounted) setState(() => _copied = false);
                });
              },
        icon: Icon(
          _copied ? Icons.check : Icons.copy_outlined,
          size: 18,
          color: colors.mutedForeground,
        ),
      ),
    );
  }
}

class _MediaError extends StatelessWidget {
  const _MediaError({required this.label});

  final String label;

  @override
  Widget build(BuildContext context) {
    final colors = shad.Theme.of(context).colorScheme;
    return Semantics(
      label: '$label could not be displayed',
      child: SizedBox(
        height: 120,
        child: Center(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Icon(Icons.broken_image_outlined, color: colors.mutedForeground),
              const SizedBox(height: 6),
              Text(
                '$label unavailable',
                style: TextStyle(color: colors.mutedForeground),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

String _prettyJson(String value) {
  try {
    return const JsonEncoder.withIndent('  ').convert(jsonDecode(value));
  } on FormatException {
    return value;
  }
}

Map<String, dynamic>? _jsonMap(String value) {
  try {
    final decoded = jsonDecode(value);
    return decoded is Map<String, dynamic> ? decoded : null;
  } on FormatException {
    return null;
  }
}

String _mediaUrl(String content) {
  final json = _jsonMap(content);
  return json?['url']?.toString() ?? content;
}

String _shortId(String value) =>
    value.length <= 8 ? value : value.substring(0, 8);

String _sanitizeSvg(String source) => source
    .replaceAll(
      RegExp(r'<script\b[^>]*>[\s\S]*?<\/script\s*>', caseSensitive: false),
      '',
    )
    .replaceAll(
      RegExp(
        r'''\s+on[a-z]+\s*=\s*(?:"[^"]*"|'[^']*'|[^\s>]+)''',
        caseSensitive: false,
      ),
      '',
    )
    .replaceAll(
      RegExp(
        r'''\s+(?:xlink:)?href\s*=\s*(?:"[^"]*"|'[^']*'|[^\s>]+)''',
        caseSensitive: false,
      ),
      '',
    );
