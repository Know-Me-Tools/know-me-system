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
/// doc comment on why enums nested only inside a StreamSink<T> stay opaque).
sealed class ContentBlock {
  const ContentBlock();

  factory ContentBlock.fromJson(Map<String, dynamic> json) =>
      switch (json['type'] as String) {
        'text' => TextBlock(json['text'] as String),
        'thinking' => ThinkingBlock(json['text'] as String),
        'code' => CodeBlock(
            json['language'] as String,
            json['code'] as String,
          ),
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
            json['id'] as String,
            json['name'] as String,
            json['inputJson'] as String,
          ),
        'toolResult' => ToolResultBlock(
            json['toolUseId'] as String,
            json['outputJson'] as String,
            json['isError'] as bool,
          ),
        'skill' => SkillBlock(
            json['name'] as String,
            json['status'] as String,
          ),
        'artifact' => ArtifactBlock(
            json['id'] as String,
            json['kind'] as String,
            json['content'] as String,
          ),
        'image' => ImageBlock(
            json['url'] as String?,
            json['dataBase64'] as String?,
            json['mime'] as String,
          ),
        'divider' => const DividerBlock(),
        final type => throw FormatException('Unknown ContentBlock type: $type'),
      };
}

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
  const ImageBlock(this.url, this.dataBase64, this.mime);
}

class DividerBlock extends ContentBlock {
  const DividerBlock();
}
