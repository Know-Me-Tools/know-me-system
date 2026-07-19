// TJ-ARCH-MOB-001 compliant
//! gen_ui_protocol (L1) — A2UI/AG-UI adapters. Pure transformation over the L0
//! event enums; wasm-safe (no IO, no runtime dependency).
use gen_ui_types::content_block::ContentBlock;
use gen_ui_types::events::{A2uiEvent, AguiEvent, StreamEvent};
use gen_ui_types::lifecycle::{RunError, RunPhase};

/// StreamEvent -> A2uiEvent(s). Feature-complete adapter to be filled per the
/// ContentBlock contract; C-001 lands the seam + a working text path.
#[derive(Default)]
pub struct A2uiAdapter {
    run_id: String,
    reasoning: ReasoningParser,
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

#[derive(Default)]
struct ReasoningParser {
    mode: ReasoningMode,
    pending: String,
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum ReasoningMode {
    #[default]
    Answer,
    Thinking,
}

impl ReasoningParser {
    fn ingest(&mut self, delta: &str) -> Vec<ContentBlock> {
        self.pending.push_str(delta);
        let mut blocks = Vec::new();
        loop {
            let marker = match self.mode {
                ReasoningMode::Answer => "<think>",
                ReasoningMode::Thinking => "</think>",
            };
            if let Some(index) = self.pending.find(marker) {
                let text = self.pending[..index].to_owned();
                self.push_current(text, &mut blocks);
                self.pending.drain(..index + marker.len());
                self.mode = match self.mode {
                    ReasoningMode::Answer => ReasoningMode::Thinking,
                    ReasoningMode::Thinking => ReasoningMode::Answer,
                };
                continue;
            }

            // Keep only a suffix that could still become the delimiter after
            // the next token. Everything before it is provably ordinary text.
            let retained = longest_marker_prefix_suffix(&self.pending, marker);
            let emit_len = self.pending.len() - retained;
            if emit_len > 0 {
                let text = self.pending[..emit_len].to_owned();
                self.pending.drain(..emit_len);
                self.push_current(text, &mut blocks);
            }
            break;
        }
        blocks
    }

    fn flush(&mut self) -> Vec<ContentBlock> {
        let text = std::mem::take(&mut self.pending);
        let mut blocks = Vec::new();
        self.push_current(text, &mut blocks);
        blocks
    }

    fn is_thinking(&self) -> bool {
        self.mode == ReasoningMode::Thinking
    }

    fn push_current(&self, text: String, output: &mut Vec<ContentBlock>) {
        if text.is_empty() {
            return;
        }
        output.push(match self.mode {
            ReasoningMode::Answer => ContentBlock::Text { text },
            ReasoningMode::Thinking => ContentBlock::Thinking { text },
        });
    }
}

fn longest_marker_prefix_suffix(value: &str, marker: &str) -> usize {
    (1..marker.len())
        .rev()
        .find(|&length| value.ends_with(&marker[..length]))
        .unwrap_or(0)
}

impl A2uiAdapter {
    pub fn new(run_id: impl Into<String>) -> Self {
        Self {
            run_id: run_id.into(),
            reasoning: ReasoningParser::default(),
            pending_tools: Vec::new(),
        }
    }

    fn block_events(&self, blocks: Vec<ContentBlock>) -> Vec<A2uiEvent> {
        blocks
            .into_iter()
            .map(|block| A2uiEvent::Block {
                run_id: self.run_id.clone(),
                block,
            })
            .collect()
    }

    pub fn ingest(&mut self, ev: &StreamEvent) -> Vec<A2uiEvent> {
        match ev {
            StreamEvent::MessageStart => vec![A2uiEvent::RunStarted {
                run_id: self.run_id.clone(),
            }],
            StreamEvent::TextDelta { delta, .. } => {
                let blocks = self.reasoning.ingest(delta);
                self.block_events(blocks)
            }
            StreamEvent::ThinkingDelta { delta, .. } => {
                let pending = self.reasoning.flush();
                let mut events = self.block_events(pending);
                events.push(A2uiEvent::Block {
                    run_id: self.run_id.clone(),
                    block: ContentBlock::Thinking {
                        text: delta.clone(),
                    },
                });
                events
            }

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
                            run_id: self.run_id.clone(),
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
            StreamEvent::ToolResult {
                tool_use_id,
                output_json,
                is_error,
            } => vec![A2uiEvent::Block {
                run_id: self.run_id.clone(),
                block: ContentBlock::ToolResult {
                    tool_use_id: tool_use_id.clone(),
                    output_json: output_json.clone(),
                    is_error: *is_error,
                },
            }],

            StreamEvent::Custom {
                source,
                event_name,
                payload,
            } => vec![A2uiEvent::Block {
                run_id: self.run_id.clone(),
                block: ContentBlock::Artifact {
                    id: format!("{source}:{event_name}"),
                    kind: "provider_event".into(),
                    content: serde_json::json!({
                        "source": source,
                        "eventName": event_name,
                        "payload": payload,
                    })
                    .to_string(),
                },
            }],
            StreamEvent::Done => {
                let unclosed = self.reasoning.is_thinking();
                let pending = self.reasoning.flush();
                let mut events = self.block_events(pending);
                if unclosed {
                    events.push(A2uiEvent::RunError {
                        run_id: self.run_id.clone(),
                        error: RunError::protocol(
                            "The model stream ended before its reasoning section closed.",
                        ),
                    });
                } else {
                    events.push(A2uiEvent::RunFinished {
                        run_id: self.run_id.clone(),
                    });
                }
                events
            }
            StreamEvent::Error { message } => {
                let pending = self.reasoning.flush();
                let mut events = self.block_events(pending);
                events.push(A2uiEvent::RunError {
                    run_id: self.run_id.clone(),
                    error: RunError {
                        code: "inference_error".into(),
                        phase: RunPhase::Generation,
                        message: message.clone(),
                        retryable: true,
                        diagnostics: None,
                    },
                });
                events
            }
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
                run_id: _,
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

    fn rendered(events: &[A2uiEvent]) -> (String, String) {
        let mut answer = String::new();
        let mut thinking = String::new();
        for event in events {
            if let A2uiEvent::Block { block, .. } = event {
                match block {
                    ContentBlock::Text { text } => answer.push_str(text),
                    ContentBlock::Thinking { text } => thinking.push_str(text),
                    _ => {}
                }
            }
        }
        (answer, thinking)
    }

    fn parse_chunks(chunks: &[&str]) -> Vec<A2uiEvent> {
        let mut adapter = A2uiAdapter::new("run-reasoning");
        let mut output = Vec::new();
        for (index, chunk) in chunks.iter().enumerate() {
            output.extend(adapter.ingest(&StreamEvent::TextDelta {
                index: index as u32,
                delta: (*chunk).into(),
            }));
        }
        output.extend(adapter.ingest(&StreamEvent::Done));
        output
    }

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
                ..
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
                    ..
                } => {
                    assert_eq!(id, want_id);
                    assert_eq!(input_json, want_args, "calls must not share a buffer");
                }
                other => panic!("expected ToolUse, got {other:?}"),
            }
        }
    }

    #[test]
    fn tool_results_emit_tool_result_blocks() {
        let mut a = A2uiAdapter::new("run-1");
        let out = a.ingest(&StreamEvent::ToolResult {
            tool_use_id: "c1".into(),
            output_json: r#"{"ok":false}"#.into(),
            is_error: true,
        });

        assert_eq!(out.len(), 1);
        match &out[0] {
            A2uiEvent::Block {
                block:
                    ContentBlock::ToolResult {
                        tool_use_id,
                        output_json,
                        is_error,
                    },
                ..
            } => {
                assert_eq!(tool_use_id, "c1");
                assert_eq!(output_json, r#"{"ok":false}"#);
                assert!(*is_error);
            }
            other => panic!("expected ToolResult block, got {other:?}"),
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
            A2uiEvent::Block { block: ContentBlock::Text { text }, .. } if text == "hi"
        ));
    }

    #[test]
    fn reasoning_delimiters_survive_every_stream_boundary() {
        let input = "before<think>reasoning ✓</think>after";
        for split in 0..=input.len() {
            if !input.is_char_boundary(split) {
                continue;
            }
            let output = parse_chunks(&[&input[..split], &input[split..]]);
            assert_eq!(
                rendered(&output),
                ("beforeafter".into(), "reasoning ✓".into()),
                "split at byte {split}"
            );
            assert!(matches!(output.last(), Some(A2uiEvent::RunFinished { .. })));
        }
    }

    #[test]
    fn multiple_reasoning_sections_remain_separate_from_answer() {
        let output = parse_chunks(&["a<th", "ink>one</th", "ink>b<think>two</think>c"]);
        assert_eq!(rendered(&output), ("abc".into(), "onetwo".into()));
    }

    #[test]
    fn dedicated_reasoning_fields_emit_thinking_blocks() {
        let mut adapter = A2uiAdapter::new("run-1");
        let output = adapter.ingest(&StreamEvent::ThinkingDelta {
            index: 0,
            delta: "provider reasoning".into(),
        });
        assert!(matches!(
            &output[0],
            A2uiEvent::Block { block: ContentBlock::Thinking { text }, .. }
                if text == "provider reasoning"
        ));
    }

    #[test]
    fn unclosed_reasoning_is_preserved_and_fails_the_run() {
        let output = parse_chunks(&["answer<think>unfinished"]);
        assert_eq!(rendered(&output), ("answer".into(), "unfinished".into()));
        assert!(matches!(
            output.last(),
            Some(A2uiEvent::RunError { error, .. }) if error.code == "protocol_error"
        ));
    }

    #[test]
    fn unknown_provider_events_are_preserved_as_artifacts() {
        let mut adapter = A2uiAdapter::new("run-1");
        let output = adapter.ingest(&StreamEvent::Custom {
            source: "provider-x".into(),
            event_name: "new_chunk".into(),
            payload: serde_json::json!({"opaque": true}),
        });
        assert!(matches!(
            &output[0],
            A2uiEvent::Block {
                block: ContentBlock::Artifact { kind, content, .. }, ..
            } if kind == "provider_event" && content.contains("new_chunk")
        ));
    }
}
