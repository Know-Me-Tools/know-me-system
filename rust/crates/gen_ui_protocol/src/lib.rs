// TJ-ARCH-MOB-001 compliant
//! gen_ui_protocol (L1) — A2UI/AG-UI adapters. Pure transformation over the L0
//! event enums; wasm-safe (no IO, no runtime dependency).
use gen_ui_types::content_block::ContentBlock;
use gen_ui_types::events::{A2uiEvent, AguiEvent, StreamEvent};

/// StreamEvent -> A2uiEvent(s). Feature-complete adapter to be filled per the
/// ContentBlock contract; C-001 lands the seam + a working text path.
#[derive(Default)]
pub struct A2uiAdapter {
    run_id: String,
    /// In-flight tool calls, keyed by call id (C-108).
    ///
    /// Tool arguments arrive as a *stream of string fragments* — the model emits
    /// `{"pat`, `h": "/tm`, `p"}` across chunks — so a ToolUse block can only be emitted
    /// once the arguments are complete. Holding the name here is what makes
    /// ToolCallComplete (which carries only an id) able to name its own block.
    pending_tools: Vec<PendingTool>,
}

struct PendingTool {
    id: String,
    name: String,
    /// Accumulated argument fragments. Not parsed as JSON until complete — a partial
    /// fragment is not valid JSON, and treating it as such would fail every time.
    args: String,
}

impl A2uiAdapter {
    pub fn new(run_id: impl Into<String>) -> Self {
        Self {
            run_id: run_id.into(),
            pending_tools: Vec::new(),
        }
    }

    pub fn ingest(&mut self, ev: &StreamEvent) -> Vec<A2uiEvent> {
        match ev {
            StreamEvent::MessageStart => vec![A2uiEvent::RunStarted {
                run_id: self.run_id.clone(),
            }],
            StreamEvent::TextDelta { delta, .. } => vec![A2uiEvent::Block {
                block: ContentBlock::Text {
                    text: delta.clone(),
                },
            }],

            // A tool call announces itself, then dribbles its arguments in. Nothing is
            // emitted until Complete: a half-built ToolUse block would render arguments
            // that are not yet valid JSON.
            StreamEvent::ToolCallStarted { id, name } => {
                self.pending_tools.push(PendingTool {
                    id: id.clone(),
                    name: name.clone(),
                    args: String::new(),
                });
                vec![]
            }
            StreamEvent::ToolCallDelta { id, delta } => {
                if let Some(t) = self.pending_tools.iter_mut().find(|t| &t.id == id) {
                    t.args.push_str(delta);
                }
                // A delta for an unknown id means Started was dropped or reordered.
                // Silently ignoring it is right: the alternative is inventing a tool
                // call whose name we do not know.
                vec![]
            }
            StreamEvent::ToolCallComplete { id } => {
                match self.pending_tools.iter().position(|t| &t.id == id) {
                    Some(i) => {
                        let t = self.pending_tools.remove(i);
                        vec![A2uiEvent::Block {
                            block: ContentBlock::ToolUse {
                                id: t.id,
                                name: t.name,
                                input_json: t.args,
                            },
                        }]
                    }
                    None => vec![],
                }
            }

            StreamEvent::Done => vec![A2uiEvent::RunFinished {
                run_id: self.run_id.clone(),
            }],
            StreamEvent::Error { message } => vec![A2uiEvent::RunError {
                message: message.clone(),
            }],
            _ => vec![],
        }
    }
}

/// A2uiEvent -> AguiEvent(s), bidirectional-capable.
pub struct AguiAdapter {
    thread_id: String,
    /// Retained for the bidirectional path (client→agent) filled in a later lane.
    #[allow(dead_code)]
    run_id: String,
}
impl AguiAdapter {
    pub fn new(thread_id: impl Into<String>, run_id: impl Into<String>) -> Self {
        Self {
            thread_id: thread_id.into(),
            run_id: run_id.into(),
        }
    }
    pub fn translate(&mut self, ev: &A2uiEvent) -> Vec<AguiEvent> {
        match ev {
            A2uiEvent::RunStarted { run_id } => vec![AguiEvent::RunStarted {
                thread_id: self.thread_id.clone(),
                run_id: run_id.clone(),
            }],
            A2uiEvent::Block {
                block: ContentBlock::Text { text },
            } => vec![AguiEvent::TextMessageContent {
                delta: text.clone(),
            }],
            A2uiEvent::RunFinished { run_id } => vec![AguiEvent::RunFinished {
                run_id: run_id.clone(),
            }],
            _ => vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tool arguments arrive as a stream of fragments that are individually invalid
    /// JSON. The adapter must hold them until complete, then emit ONE ToolUse block —
    /// not one per fragment, and not a block carrying half an object.
    #[test]
    fn tool_call_fragments_accumulate_into_one_block() {
        let mut a = A2uiAdapter::new("run-1");

        assert!(
            a.ingest(&StreamEvent::ToolCallStarted {
                id: "c1".into(),
                name: "forge__read".into()
            })
            .is_empty(),
            "a started call emits nothing until its arguments land"
        );

        for frag in [r#"{"pa"#, r#"th":"/t"#, r#"mp"}"#] {
            assert!(
                a.ingest(&StreamEvent::ToolCallDelta {
                    id: "c1".into(),
                    delta: frag.into()
                })
                .is_empty(),
                "argument fragments must not emit blocks — {frag:?} is not valid JSON"
            );
        }

        let out = a.ingest(&StreamEvent::ToolCallComplete { id: "c1".into() });
        assert_eq!(out.len(), 1, "completion emits exactly one block");
        match &out[0] {
            A2uiEvent::Block {
                block:
                    ContentBlock::ToolUse {
                        id,
                        name,
                        input_json,
                    },
            } => {
                assert_eq!(id, "c1");
                assert_eq!(name, "forge__read");
                assert_eq!(
                    input_json, r#"{"path":"/tmp"}"#,
                    "fragments must reassemble in order"
                );
            }
            other => panic!("expected a ToolUse block, got {other:?}"),
        }
    }

    /// Two calls in one turn must not interleave their arguments.
    #[test]
    fn concurrent_tool_calls_keep_their_arguments_separate() {
        let mut a = A2uiAdapter::new("run-1");
        a.ingest(&StreamEvent::ToolCallStarted {
            id: "c1".into(),
            name: "s__a".into(),
        });
        a.ingest(&StreamEvent::ToolCallStarted {
            id: "c2".into(),
            name: "s__b".into(),
        });
        a.ingest(&StreamEvent::ToolCallDelta {
            id: "c1".into(),
            delta: r#"{"x":1"#.into(),
        });
        a.ingest(&StreamEvent::ToolCallDelta {
            id: "c2".into(),
            delta: r#"{"y":2"#.into(),
        });
        a.ingest(&StreamEvent::ToolCallDelta {
            id: "c1".into(),
            delta: "}".into(),
        });
        a.ingest(&StreamEvent::ToolCallDelta {
            id: "c2".into(),
            delta: "}".into(),
        });

        let o1 = a.ingest(&StreamEvent::ToolCallComplete { id: "c1".into() });
        let o2 = a.ingest(&StreamEvent::ToolCallComplete { id: "c2".into() });

        for (out, want_id, want_args) in [(o1, "c1", r#"{"x":1}"#), (o2, "c2", r#"{"y":2}"#)] {
            match &out[0] {
                A2uiEvent::Block {
                    block: ContentBlock::ToolUse { id, input_json, .. },
                } => {
                    assert_eq!(id, want_id);
                    assert_eq!(input_json, want_args, "calls must not share a buffer");
                }
                other => panic!("expected ToolUse, got {other:?}"),
            }
        }
    }

    /// A delta for an id we never saw started means Started was dropped or reordered.
    /// Dropping it is right — the alternative is inventing a call whose name is unknown.
    #[test]
    fn orphan_tool_deltas_are_dropped_not_guessed() {
        let mut a = A2uiAdapter::new("run-1");
        assert!(a
            .ingest(&StreamEvent::ToolCallDelta {
                id: "ghost".into(),
                delta: "{}".into()
            })
            .is_empty());
        assert!(a
            .ingest(&StreamEvent::ToolCallComplete { id: "ghost".into() })
            .is_empty());
    }

    /// The text path must be unchanged by the tool work.
    #[test]
    fn text_deltas_still_emit_text_blocks() {
        let mut a = A2uiAdapter::new("run-1");
        let out = a.ingest(&StreamEvent::TextDelta {
            index: 0,
            delta: "hi".into(),
        });
        assert!(matches!(
            &out[0],
            A2uiEvent::Block { block: ContentBlock::Text { text } } if text == "hi"
        ));
    }
}
