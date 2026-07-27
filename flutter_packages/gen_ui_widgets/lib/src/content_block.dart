// TJ-ARCH-MOB-001 compliant
/// ContentBlock — the cross-platform UI contract, mirrored from gen_ui_types.
/// In the host app this is the frb-generated union; this sealed hierarchy lets
/// the widget package compile and be tested standalone. Dart's exhaustive
/// switch on a sealed type is a compile error if a variant is unhandled — the
/// same guarantee the Rust match site has.
///
/// fromJson mirrors gen_ui_types::content_block::ContentBlock's serde shape
/// (`#[serde(tag = "type", rename_all = "camelCase")]`, applied to both
/// variant and field names) — needed because the FFI stream carrying this
/// type sends JSON, not a frb-mirrored class (see gen_ui_ffi::api::streams's
/// doc comment on why enums nested only inside a `StreamSink<T>` stay opaque).
sealed class ContentBlock {
  const ContentBlock();

  factory ContentBlock.fromJson(Map<String, dynamic> json) =>
      switch (json['type'] as String) {
        'text' => TextBlock(json['text'] as String),
        'thinking' => ThinkingBlock(json['text'] as String),
        'code' => CodeBlock(json['language'] as String, json['code'] as String),
        'citation' => CitationBlock(
          json['source'] as String,
          json['quote'] as String,
        ),
        'memory' => MemoryBlock(
          json['operation'] as String,
          json['key'] as String,
          json['value'] as String?,
        ),
        'toolUse' => ToolUseBlock(
          _jsonString(json, 'id'),
          _jsonString(json, 'name'),
          _jsonString(json, 'inputJson', snakeKey: 'input_json'),
        ),
        'toolResult' => ToolResultBlock(
          _jsonString(json, 'toolUseId', snakeKey: 'tool_use_id'),
          _jsonString(json, 'outputJson', snakeKey: 'output_json'),
          _jsonBool(json, 'isError', snakeKey: 'is_error'),
        ),
        'skill' => SkillBlock(json['name'] as String, json['status'] as String),
        'artifact' => ArtifactBlock(
          json['id'] as String,
          json['kind'] as String,
          json['content'] as String,
        ),
        'image' => ImageBlock(
          json['url'] as String?,
          _jsonNullableString(json, 'dataBase64', snakeKey: 'data_base64'),
          json['mime'] as String,
          path: json['path'] as String?,
          alt: json['alt'] as String?,
          width: (json['width'] as num?)?.toInt(),
          height: (json['height'] as num?)?.toInt(),
        ),
        'divider' => const DividerBlock(),
        final type => throw FormatException('Unknown ContentBlock type: $type'),
      };
}

String _jsonString(
  Map<String, dynamic> json,
  String key, {
  String? snakeKey,
  String defaultValue = '',
}) =>
    (json[key] ?? (snakeKey == null ? null : json[snakeKey]))?.toString() ??
    defaultValue;

String? _jsonNullableString(
  Map<String, dynamic> json,
  String key, {
  String? snakeKey,
}) => (json[key] ?? (snakeKey == null ? null : json[snakeKey]))?.toString();

bool _jsonBool(
  Map<String, dynamic> json,
  String key, {
  String? snakeKey,
  bool defaultValue = false,
}) =>
    (json[key] ?? (snakeKey == null ? null : json[snakeKey])) as bool? ??
    defaultValue;

class TextBlock extends ContentBlock {
  final String text;
  const TextBlock(this.text);
}

class ThinkingBlock extends ContentBlock {
  final String text;
  const ThinkingBlock(this.text);
}

class CodeBlock extends ContentBlock {
  final String language;
  final String code;
  const CodeBlock(this.language, this.code);
}

class CitationBlock extends ContentBlock {
  final String source;
  final String quote;
  const CitationBlock(this.source, this.quote);
}

class MemoryBlock extends ContentBlock {
  final String operation;
  final String key;
  final String? value;
  const MemoryBlock(this.operation, this.key, this.value);
}

class ToolUseBlock extends ContentBlock {
  final String id;
  final String name;
  final String inputJson;
  const ToolUseBlock(this.id, this.name, this.inputJson);
}

class ToolResultBlock extends ContentBlock {
  final String toolUseId;
  final String outputJson;
  final bool isError;
  const ToolResultBlock(this.toolUseId, this.outputJson, this.isError);
}

class SkillBlock extends ContentBlock {
  final String name;
  final String status;
  const SkillBlock(this.name, this.status);
}

class ArtifactBlock extends ContentBlock {
  final String id;
  final String kind;
  final String content;
  const ArtifactBlock(this.id, this.kind, this.content);
}

class ImageBlock extends ContentBlock {
  final String? url;
  final String? dataBase64;
  final String mime;

  /// A file in app-owned storage, for user attachments. Never a picker path —
  /// those are purgeable on Android and expire on iOS.
  final String? path;

  /// Accessible description. Falls back to the file name at attach time, so a
  /// screen reader says "holiday.png" rather than "image/png".
  final String? alt;

  /// Intrinsic pixel size when known, so the layout does not shift as the
  /// image decodes.
  final int? width;
  final int? height;

  const ImageBlock(
    this.url,
    this.dataBase64,
    this.mime, {
    this.path,
    this.alt,
    this.width,
    this.height,
  });
}

class DividerBlock extends ContentBlock {
  const DividerBlock();
}

/// Stable plain-text projection used by message-level copy/export actions.
/// Exhaustive by design so new A2UI block variants cannot silently disappear.
String contentBlockCopyText(ContentBlock block) => switch (block) {
  TextBlock(:final text) || ThinkingBlock(:final text) => text,
  CodeBlock(:final language, :final code) => '```$language\n$code\n```',
  CitationBlock(:final source, :final quote) => '$quote\n— $source',
  MemoryBlock(:final operation, :final key, :final value) => [
    'Memory · $operation · $key',
    if (value != null) value,
  ].join('\n'),
  ToolUseBlock(:final name, :final inputJson) => 'Tool · $name\n$inputJson',
  ToolResultBlock(:final outputJson) => outputJson,
  SkillBlock(:final name, :final status) => 'Skill · $name · $status',
  ArtifactBlock(:final kind, :final content) => 'Artifact · $kind\n$content',
  ImageBlock(:final url, :final dataBase64, :final mime) =>
    url ?? dataBase64 ?? 'Image · $mime',
  DividerBlock() => '---',
};

/// Coalesces transport deltas that were persisted as adjacent text blocks by
/// older clients. Semantic block boundaries (code, citation, tools, etc.) are
/// always preserved.
List<ContentBlock> coalesceContentBlocks(Iterable<ContentBlock> blocks) {
  final result = <ContentBlock>[];
  for (final block in blocks) {
    final previous = result.isEmpty ? null : result.last;
    if (block case TextBlock(:final text)) {
      if (previous case TextBlock(text: final existing)) {
        result[result.length - 1] = TextBlock('$existing$text');
        continue;
      }
    }
    if (block case ThinkingBlock(:final text)) {
      if (previous case ThinkingBlock(text: final existing)) {
        result[result.length - 1] = ThinkingBlock('$existing$text');
        continue;
      }
    }
    result.add(block);
  }
  return result;
}
