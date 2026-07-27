// TJ-ARCH-MOB-001 compliant
//! ContentBlock — the cross-platform UI contract. Every A2UI event maps to
//! exactly one variant. Dart/TS compilers enforce exhaustiveness at the match site.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum ContentBlock {
    Text {
        text: String,
    },
    Thinking {
        text: String,
    },
    Code {
        language: String,
        code: String,
    },
    Citation {
        source: String,
        quote: String,
    },
    Memory {
        operation: String,
        key: String,
        value: Option<String>,
    },
    ToolUse {
        id: String,
        name: String,
        input_json: String,
    },
    ToolResult {
        tool_use_id: String,
        output_json: String,
        is_error: bool,
    },
    Skill {
        name: String,
        status: String,
    },
    Artifact {
        id: String,
        kind: String,
        content: String,
    },
    /// Renderable media. Exactly one of `url`, `data_base64`, or `path` carries
    /// the bytes; the others are `None`.
    ///
    /// `path` points into app-owned storage (see `gen_ui_agent::attachment_store`),
    /// never at the path a file picker returned — a picked path is purgeable on
    /// Android and expires on iOS, so a transcript holding one renders during
    /// testing and breaks later.
    ///
    /// `alt` is required for any image a user can see: the a11y gate treats a
    /// missing accessible name as a failure, and a renderer has nothing to
    /// invent one from. Producers fall back to the file name.
    ///
    /// `width`/`height` are the intrinsic pixel dimensions when known. They
    /// exist so a renderer can reserve space before the bytes decode; without
    /// them every image entering a transcript shifts the layout under the
    /// reader. Optional because remote media reports them only after loading.
    #[serde(rename_all = "camelCase")]
    Image {
        url: Option<String>,
        data_base64: Option<String>,
        mime: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        path: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        alt: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        width: Option<u32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        height: Option<u32>,
    },
    Divider,
}

#[cfg(test)]
mod tests {
    use super::ContentBlock;

    #[test]
    fn structured_block_fields_serialize_in_the_cross_platform_camel_case_contract() {
        let value = serde_json::to_value(ContentBlock::ToolResult {
            tool_use_id: "call-1".into(),
            output_json: r#"{"ok":true}"#.into(),
            is_error: false,
        })
        .expect("tool result serializes");

        assert_eq!(value["type"], "toolResult");
        assert_eq!(value["toolUseId"], "call-1");
        assert_eq!(value["outputJson"], r#"{"ok":true}"#);
        assert_eq!(value["isError"], false);
        assert!(value.get("tool_use_id").is_none());
    }
}
