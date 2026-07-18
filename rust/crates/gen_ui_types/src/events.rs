// TJ-ARCH-MOB-001 compliant
//! Raw stream + protocol event enums. Pure data — transformation logic is in
//! gen_ui_protocol (which depends on this crate).
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamEvent {
    MessageStart,
    TextDelta { index: u32, delta: String },
    ThinkingDelta { index: u32, delta: String },
    ToolCallStarted { id: String, name: String },
    ToolCallDelta { id: String, delta: String },
    ToolCallComplete { id: String },
    Error { message: String },
    Done,
}

/// A2UI event surface (subset shown; full 27-variant set filled in gen_ui_protocol
/// consumers). Kept as an open enum contract here.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum A2uiEvent {
    RunStarted {
        run_id: String,
    },
    Block {
        block: crate::content_block::ContentBlock,
    },
    RunFinished {
        run_id: String,
    },
    RunError {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AguiEvent {
    RunStarted { thread_id: String, run_id: String },
    TextMessageContent { delta: String },
    ToolCallStart { id: String, name: String },
    StateSnapshot { snapshot_json: String },
    RunFinished { run_id: String },
}
