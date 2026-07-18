// TJ-ARCH-MOB-001 compliant
//! ContentBlock — the cross-platform UI contract. Every A2UI event maps to
//! exactly one variant. Dart/TS compilers enforce exhaustiveness at the match site.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "camelCase")]
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
    Image {
        url: Option<String>,
        data_base64: Option<String>,
        mime: String,
    },
    Divider,
}
