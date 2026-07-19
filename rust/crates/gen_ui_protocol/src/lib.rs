// TJ-ARCH-MOB-001 compliant
//! gen_ui_protocol (L1) — A2UI/AG-UI adapters. Pure transformation over the L0
//! event enums; wasm-safe (no IO, no runtime dependency).
use gen_ui_types::content_block::ContentBlock;
use gen_ui_types::events::{A2uiEvent, AguiEvent, StreamEvent};
use gen_ui_types::inference::{InferenceRequest, InferenceTool};
use gen_ui_types::lifecycle::{RunError, RunPhase};
use serde::{Deserialize, Serialize};

/// UAR-compatible provider payload derived from KnowMe's canonical local
/// inference request.
///
/// This intentionally avoids depending on the UAR crate: mobile can build this
/// bridge today, and the future embedded UAR provider can adapt the same
/// payload into `universal_agent_runtime::llm::LlmRequest` mechanically.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UarLlmRequestPayload {
    pub messages: Vec<serde_json::Value>,
    pub tools: Vec<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extra_params: Option<serde_json::Value>,
}

/// Convert KnowMe's structured local-provider request into the message/tool
/// shape UAR's `LlmDriver` expects.
pub fn inference_request_to_uar_llm_payload(request: &InferenceRequest) -> UarLlmRequestPayload {
    let messages = request
        .effective_messages()
        .into_iter()
        .map(|message| {
            let mut value = serde_json::json!({
                "role": message.role.as_str(),
                "content": message.content,
            });
            if let serde_json::Value::Object(ref mut object) = value {
                if let Some(name) = message.name {
                    object.insert("name".into(), serde_json::Value::String(name));
                }
                if let Some(tool_call_id) = message.tool_call_id {
                    object.insert(
                        "tool_call_id".into(),
                        serde_json::Value::String(tool_call_id),
                    );
                }
            }
            value
        })
        .collect();

    let tools = request
        .tools
        .iter()
        .map(inference_tool_to_uar_tool)
        .collect();
    let extra_params = Some(serde_json::json!({
        "temperature": request.generation_settings.temperature,
        "top_p": request.generation_settings.top_p,
        "top_k": request.generation_settings.top_k,
        "min_p": request.generation_settings.min_p,
        "presence_penalty": request.generation_settings.presence_penalty,
        "max_tokens": request.generation_settings.max_tokens,
        "grammar": request.grammar,
        "originating_plan": request.originating_plan,
    }))
    .filter(|value| {
        value
            .as_object()
            .map(|object| !object.is_empty())
            .unwrap_or(true)
    });

    UarLlmRequestPayload {
        messages,
        tools,
        extra_params,
    }
}

fn inference_tool_to_uar_tool(tool: &InferenceTool) -> serde_json::Value {
    serde_json::json!({
        "type": "function",
        "function": {
            "name": tool.name,
            "description": tool.description,
            "parameters": tool.parameters,
        },
        "annotations": tool.annotations,
    })
}

/// Normalize a UAR `uar.agui/1` JSON event into KnowMe's provider-neutral
/// `StreamEvent` contract.
///
/// This is intentionally narrow and side-effect free: runtime execution still
/// belongs behind the future UAR facade, but the projection seam can be tested
/// offline now. Unknown UAR payloads are preserved as `StreamEvent::Custom`
/// rather than dropped.
pub fn normalize_uar_agui_event(payload: &serde_json::Value) -> StreamEvent {
    let event_type = json_string(payload, &["type"]).unwrap_or("CUSTOM");
    match event_type {
        "RUN_STARTED" => StreamEvent::MessageStart,
        "TEXT_MESSAGE_CONTENT" => StreamEvent::TextDelta {
            index: json_u32(payload, &["index"]).unwrap_or(0),
            delta: first_json_string(
                payload,
                &[
                    &["delta"][..],
                    &["content"][..],
                    &["text"][..],
                    &["message", "content"][..],
                ],
            )
            .unwrap_or_default()
            .to_owned(),
        },
        "REASONING_MESSAGE_CONTENT" => StreamEvent::ThinkingDelta {
            index: json_u32(payload, &["index"]).unwrap_or(0),
            delta: first_json_string(
                payload,
                &[
                    &["delta"][..],
                    &["content"][..],
                    &["text"][..],
                    &["reasoning"][..],
                ],
            )
            .unwrap_or_default()
            .to_owned(),
        },
        "TOOL_CALL_START" => StreamEvent::ToolCallStarted {
            id: first_json_string(
                payload,
                &[&["toolCallId"][..], &["id"][..], &["tool_call_id"][..]],
            )
            .unwrap_or("unknown-tool-call")
            .to_owned(),
            name: first_json_string(
                payload,
                &[&["name"][..], &["toolName"][..], &["tool", "name"][..]],
            )
            .unwrap_or("unknown_tool")
            .to_owned(),
        },
        "TOOL_CALL_ARGS" => StreamEvent::ToolCallDelta {
            id: first_json_string(
                payload,
                &[&["toolCallId"][..], &["id"][..], &["tool_call_id"][..]],
            )
            .unwrap_or("unknown-tool-call")
            .to_owned(),
            delta: first_json_string(
                payload,
                &[
                    &["delta"][..],
                    &["args"][..],
                    &["arguments"][..],
                    &["argumentsDelta"][..],
                ],
            )
            .map(str::to_owned)
            .unwrap_or_else(|| {
                payload
                    .get("arguments")
                    .map(json_payload_to_string)
                    .unwrap_or_default()
            }),
        },
        "TOOL_CALL_END" => StreamEvent::ToolCallComplete {
            id: first_json_string(
                payload,
                &[&["toolCallId"][..], &["id"][..], &["tool_call_id"][..]],
            )
            .unwrap_or("unknown-tool-call")
            .to_owned(),
        },
        "TOOL_CALL_RESULT" => StreamEvent::ToolResult {
            tool_use_id: first_json_string(
                payload,
                &[&["toolCallId"][..], &["id"][..], &["tool_call_id"][..]],
            )
            .unwrap_or("unknown-tool-call")
            .to_owned(),
            output_json: payload
                .get("result")
                .or_else(|| payload.get("output"))
                .or_else(|| payload.get("content"))
                .map(json_payload_to_string)
                .unwrap_or_else(|| "{}".to_owned()),
            is_error: json_bool(payload, &["isError"])
                .or_else(|| json_bool(payload, &["is_error"]))
                .unwrap_or(false),
        },
        "CUSTOM" => normalize_uar_custom_event(payload),
        "RUN_FINISHED" => StreamEvent::Done,
        "RUN_ERROR" => StreamEvent::Error {
            message: first_json_string(
                payload,
                &[
                    &["message"][..],
                    &["error", "message"][..],
                    &["value", "message"][..],
                ],
            )
            .unwrap_or("UAR run failed")
            .to_owned(),
        },
        _ => StreamEvent::Custom {
            source: "uar".into(),
            event_name: event_type.to_owned(),
            payload: payload.clone(),
        },
    }
}

fn normalize_uar_custom_event(payload: &serde_json::Value) -> StreamEvent {
    let name = json_string(payload, &["name"]).unwrap_or("CUSTOM");
    if name == "uar.skill.activated" {
        let value = payload.get("value").unwrap_or(payload);
        return StreamEvent::SkillActivation {
            name: first_json_string(
                value,
                &[
                    &["skill", "id"][..],
                    &["skill", "name"][..],
                    &["skill"][..],
                    &["name"][..],
                ],
            )
            .unwrap_or("unknown_skill")
            .to_owned(),
            status: first_json_string(value, &[&["status"][..], &["phase"][..]])
                .unwrap_or("activated")
                .to_owned(),
        };
    }

    StreamEvent::Custom {
        source: "uar".into(),
        event_name: name.to_owned(),
        payload: payload.clone(),
    }
}

fn json_string<'a>(value: &'a serde_json::Value, path: &[&str]) -> Option<&'a str> {
    let mut cursor = value;
    for segment in path {
        cursor = cursor.get(*segment)?;
    }
    cursor.as_str()
}

fn first_json_string<'a>(value: &'a serde_json::Value, paths: &[&[&str]]) -> Option<&'a str> {
    paths.iter().find_map(|path| json_string(value, path))
}

fn json_bool(value: &serde_json::Value, path: &[&str]) -> Option<bool> {
    let mut cursor = value;
    for segment in path {
        cursor = cursor.get(*segment)?;
    }
    cursor.as_bool()
}

fn json_u32(value: &serde_json::Value, path: &[&str]) -> Option<u32> {
    let mut cursor = value;
    for segment in path {
        cursor = cursor.get(*segment)?;
    }
    cursor.as_u64().and_then(|value| u32::try_from(value).ok())
}

fn json_payload_to_string(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(value) => value.clone(),
        other => other.to_string(),
    }
}

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
            StreamEvent::SkillActivation { name, status } => vec![A2uiEvent::Block {
                run_id: self.run_id.clone(),
                block: ContentBlock::Skill {
                    name: name.clone(),
                    status: status.clone(),
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
            A2uiEvent::Block {
                run_id: _,
                block: ContentBlock::Thinking { text },
            } => vec![AguiEvent::ReasoningMessageContent {
                delta: text.clone(),
            }],
            A2uiEvent::Block {
                run_id: _,
                block:
                    ContentBlock::ToolUse {
                        id,
                        name,
                        input_json,
                    },
            } => vec![
                AguiEvent::ToolCallStart {
                    id: id.clone(),
                    name: name.clone(),
                },
                AguiEvent::ToolCallArgs {
                    id: id.clone(),
                    delta: input_json.clone(),
                },
                AguiEvent::ToolCallEnd { id: id.clone() },
            ],
            A2uiEvent::Block {
                run_id: _,
                block:
                    ContentBlock::ToolResult {
                        tool_use_id,
                        output_json,
                        is_error,
                    },
            } => vec![AguiEvent::ToolCallResult {
                id: tool_use_id.clone(),
                output_json: output_json.clone(),
                is_error: *is_error,
            }],
            A2uiEvent::Block {
                run_id: _,
                block: ContentBlock::Skill { name, status },
            } => vec![AguiEvent::Custom {
                name: "uar.skill.activated".into(),
                value: serde_json::json!({
                    "skill": {"id": name},
                    "status": status,
                    "source": "knowme",
                }),
            }],
            A2uiEvent::Block {
                run_id: _,
                block: ContentBlock::Artifact { id, kind, content },
            } => vec![AguiEvent::Custom {
                name: "knowme.artifact".into(),
                value: serde_json::json!({
                    "id": id,
                    "kind": kind,
                    "content": content,
                }),
            }],
            A2uiEvent::Block { run_id: _, block } => vec![AguiEvent::Custom {
                name: "knowme.content_block".into(),
                value: serde_json::to_value(block).unwrap_or_else(|error| {
                    serde_json::json!({
                        "serializationError": error.to_string(),
                    })
                }),
            }],
            A2uiEvent::RunPhaseChanged {
                run_id,
                phase,
                status,
                message,
            } => vec![AguiEvent::Custom {
                name: "knowme.run_phase_changed".into(),
                value: serde_json::json!({
                    "runId": run_id,
                    "phase": phase,
                    "status": status,
                    "message": message,
                }),
            }],
            A2uiEvent::ContextTelemetry { run_id, payload } => vec![AguiEvent::Custom {
                name: "knowme.context_telemetry".into(),
                value: serde_json::json!({
                    "runId": run_id,
                    "payload": payload,
                }),
            }],
            A2uiEvent::RunFinished { run_id } => vec![AguiEvent::RunFinished {
                run_id: run_id.clone(),
            }],
            A2uiEvent::RunCancelled {
                run_id, message, ..
            } => vec![AguiEvent::RunCancelled {
                run_id: run_id.clone(),
                message: message.clone(),
            }],
            A2uiEvent::RunError { run_id, error } => vec![AguiEvent::RunError {
                run_id: run_id.clone(),
                error_json: serde_json::to_string(error).unwrap_or_else(|serialization_error| {
                    serde_json::json!({
                        "code": "run_error_serialization_failed",
                        "message": serialization_error.to_string(),
                    })
                    .to_string()
                }),
            }],
        }
    }
}

/// Stateful local-provider bridge for the future embedded UAR provider lane.
///
/// Local engines already emit KnowMe `StreamEvent`s. UAR consumers need AG-UI
/// style events. This bridge composes the existing `A2uiAdapter` and
/// `AguiAdapter` so reasoning parsing, tool-argument accumulation, unclosed
/// reasoning errors, and unknown-event preservation stay identical to the UI
/// path.
pub struct UarLocalProviderEventBridge {
    a2ui: A2uiAdapter,
    agui: AguiAdapter,
}

impl UarLocalProviderEventBridge {
    pub fn new(thread_id: impl Into<String>, run_id: impl Into<String>) -> Self {
        let run_id = run_id.into();
        Self {
            a2ui: A2uiAdapter::new(run_id.clone()),
            agui: AguiAdapter::new(thread_id, run_id),
        }
    }

    pub fn ingest(&mut self, event: &StreamEvent) -> Vec<AguiEvent> {
        self.a2ui
            .ingest(event)
            .into_iter()
            .flat_map(|event| self.agui.translate(&event))
            .collect()
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
    fn skill_activations_emit_skill_blocks() {
        let mut adapter = A2uiAdapter::new("run-1");
        let output = adapter.ingest(&StreamEvent::SkillActivation {
            name: "document_ingest".into(),
            status: "running".into(),
        });
        assert!(matches!(
            &output[0],
            A2uiEvent::Block { block: ContentBlock::Skill { name, status }, .. }
                if name == "document_ingest" && status == "running"
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

    #[test]
    fn agui_adapter_round_trips_reasoning_through_uar_normalizer() {
        let mut agui = AguiAdapter::new("thread-1", "run-1");
        let events = agui.translate(&A2uiEvent::Block {
            run_id: "run-1".into(),
            block: ContentBlock::Thinking {
                text: "plain reasoning".into(),
            },
        });

        assert_eq!(
            events,
            vec![AguiEvent::ReasoningMessageContent {
                delta: "plain reasoning".into()
            }]
        );

        let stream = normalize_uar_agui_event(&serde_json::to_value(&events[0]).unwrap());
        assert!(matches!(
            stream,
            StreamEvent::ThinkingDelta { delta, .. } if delta == "plain reasoning"
        ));
    }

    #[test]
    fn agui_adapter_round_trips_tool_use_lifecycle() {
        let mut agui = AguiAdapter::new("thread-1", "run-1");
        let events = agui.translate(&A2uiEvent::Block {
            run_id: "run-1".into(),
            block: ContentBlock::ToolUse {
                id: "tool-1".into(),
                name: "calendar.find".into(),
                input_json: r#"{"query":"tomorrow"}"#.into(),
            },
        });

        assert_eq!(
            events,
            vec![
                AguiEvent::ToolCallStart {
                    id: "tool-1".into(),
                    name: "calendar.find".into(),
                },
                AguiEvent::ToolCallArgs {
                    id: "tool-1".into(),
                    delta: r#"{"query":"tomorrow"}"#.into(),
                },
                AguiEvent::ToolCallEnd {
                    id: "tool-1".into(),
                },
            ]
        );

        let normalized: Vec<StreamEvent> = events
            .iter()
            .map(|event| normalize_uar_agui_event(&serde_json::to_value(event).unwrap()))
            .collect();
        assert!(matches!(
            &normalized[0],
            StreamEvent::ToolCallStarted { id, name }
                if id == "tool-1" && name == "calendar.find"
        ));
        assert!(matches!(
            &normalized[1],
            StreamEvent::ToolCallDelta { id, delta }
                if id == "tool-1" && delta == r#"{"query":"tomorrow"}"#
        ));
        assert!(matches!(
            &normalized[2],
            StreamEvent::ToolCallComplete { id } if id == "tool-1"
        ));
    }

    #[test]
    fn agui_adapter_preserves_skill_activation_as_custom_uar_event() {
        let mut agui = AguiAdapter::new("thread-1", "run-1");
        let events = agui.translate(&A2uiEvent::Block {
            run_id: "run-1".into(),
            block: ContentBlock::Skill {
                name: "office_writer".into(),
                status: "activated".into(),
            },
        });

        let stream = normalize_uar_agui_event(&serde_json::to_value(&events[0]).unwrap());
        assert!(matches!(
            stream,
            StreamEvent::SkillActivation { name, status }
                if name == "office_writer" && status == "activated"
        ));
    }

    #[test]
    fn agui_adapter_preserves_errors_and_unknown_blocks() {
        let mut agui = AguiAdapter::new("thread-1", "run-1");
        let error_events = agui.translate(&A2uiEvent::RunError {
            run_id: "run-1".into(),
            error: RunError::protocol("bad stream"),
        });
        assert!(matches!(
            &error_events[0],
            AguiEvent::RunError { run_id, error_json }
                if run_id == "run-1" && error_json.contains("bad stream")
        ));

        let artifact_events = agui.translate(&A2uiEvent::Block {
            run_id: "run-1".into(),
            block: ContentBlock::Artifact {
                id: "future-1".into(),
                kind: "provider_event".into(),
                content: r#"{"opaque":true}"#.into(),
            },
        });
        assert!(matches!(
            &artifact_events[0],
            AguiEvent::Custom { name, value }
                if name == "knowme.artifact" && value["id"] == "future-1"
        ));
    }

    #[test]
    fn inference_request_becomes_uar_driver_payload_without_flattening_roles() {
        let request = InferenceRequest {
            prompt: String::new(),
            messages: vec![
                gen_ui_types::inference::InferenceMessage {
                    role: gen_ui_types::inference::InferenceRole::System,
                    content: "Be concise".into(),
                    name: None,
                    tool_call_id: None,
                },
                gen_ui_types::inference::InferenceMessage {
                    role: gen_ui_types::inference::InferenceRole::User,
                    content: "Find tomorrow's meetings".into(),
                    name: Some("traveler".into()),
                    tool_call_id: None,
                },
                gen_ui_types::inference::InferenceMessage {
                    role: gen_ui_types::inference::InferenceRole::Tool,
                    content: r#"{"count":2}"#.into(),
                    name: None,
                    tool_call_id: Some("tool-1".into()),
                },
            ],
            tools: vec![InferenceTool {
                name: "calendar.find".into(),
                description: "Find calendar events".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {"query": {"type": "string"}},
                    "required": ["query"]
                }),
                annotations: Some(gen_ui_types::inference::InferenceToolAnnotations {
                    read_only_hint: Some(true),
                    ..Default::default()
                }),
            }],
            images: Vec::new(),
            generation_settings: gen_ui_types::inference::SampleParams {
                temperature: 0.2,
                max_tokens: 256,
                ..Default::default()
            },
            grammar: None,
            originating_plan: None,
        };

        let payload = inference_request_to_uar_llm_payload(&request);
        assert_eq!(payload.messages[0]["role"], "system");
        assert_eq!(payload.messages[1]["role"], "user");
        assert_eq!(payload.messages[1]["name"], "traveler");
        assert_eq!(payload.messages[2]["role"], "tool");
        assert_eq!(payload.messages[2]["tool_call_id"], "tool-1");
        assert_eq!(payload.tools[0]["type"], "function");
        assert_eq!(payload.tools[0]["function"]["name"], "calendar.find");
        assert_eq!(
            payload.tools[0]["function"]["parameters"]["required"][0],
            "query"
        );
        assert_eq!(
            payload.tools[0]["annotations"]["readOnlyHint"],
            serde_json::Value::Bool(true)
        );
        assert!(
            (payload.extra_params.as_ref().unwrap()["temperature"]
                .as_f64()
                .unwrap()
                - 0.2)
                .abs()
                < 0.000_001
        );
        assert_eq!(payload.extra_params.as_ref().unwrap()["max_tokens"], 256);
    }

    #[test]
    fn local_provider_event_bridge_uses_same_reasoning_and_tool_semantics() {
        let mut bridge = UarLocalProviderEventBridge::new("thread-1", "run-1");
        let mut output = Vec::new();
        for event in [
            StreamEvent::MessageStart,
            StreamEvent::TextDelta {
                index: 0,
                delta: "answer<think>private".into(),
            },
            StreamEvent::TextDelta {
                index: 1,
                delta: "</think>done".into(),
            },
            StreamEvent::ToolCallStarted {
                id: "tool-1".into(),
                name: "docs.search".into(),
            },
            StreamEvent::ToolCallDelta {
                id: "tool-1".into(),
                delta: r#"{"q":"uar"}"#.into(),
            },
            StreamEvent::ToolCallComplete {
                id: "tool-1".into(),
            },
            StreamEvent::Done,
        ] {
            output.extend(bridge.ingest(&event));
        }

        assert!(matches!(
            &output[0],
            AguiEvent::RunStarted { thread_id, run_id }
                if thread_id == "thread-1" && run_id == "run-1"
        ));
        assert!(output.iter().any(|event| matches!(
            event,
            AguiEvent::ReasoningMessageContent { delta } if delta == "private"
        )));
        assert!(output.iter().any(|event| matches!(
            event,
            AguiEvent::ToolCallStart { id, name } if id == "tool-1" && name == "docs.search"
        )));
        assert!(
            matches!(output.last(), Some(AguiEvent::RunFinished { run_id }) if run_id == "run-1")
        );
    }

    #[test]
    fn uar_agui_reasoning_maps_to_thinking_block() {
        let event = normalize_uar_agui_event(&serde_json::json!({
            "type": "REASONING_MESSAGE_CONTENT",
            "profile": "uar.agui/1",
            "runId": "run-1",
            "sequence": 2,
            "delta": "visible reasoning"
        }));
        let mut adapter = A2uiAdapter::new("run-1");
        let output = adapter.ingest(&event);
        assert!(matches!(
            &output[0],
            A2uiEvent::Block { block: ContentBlock::Thinking { text }, .. }
                if text == "visible reasoning"
        ));
    }

    #[test]
    fn uar_skill_activation_maps_to_skill_block() {
        let event = normalize_uar_agui_event(&serde_json::json!({
            "type": "CUSTOM",
            "profile": "uar.agui/1",
            "name": "uar.skill.activated",
            "value": {
                "skill": {"id": "office_writer", "name": "Office Writer"},
                "selectionMethod": "routing"
            },
            "runId": "run-1",
            "sequence": 4
        }));
        let mut adapter = A2uiAdapter::new("run-1");
        let output = adapter.ingest(&event);
        assert!(matches!(
            &output[0],
            A2uiEvent::Block { block: ContentBlock::Skill { name, status }, .. }
                if name == "office_writer" && status == "activated"
        ));
    }

    #[test]
    fn uar_tool_lifecycle_maps_to_tool_use_block() {
        let events = [
            normalize_uar_agui_event(&serde_json::json!({
                "type": "TOOL_CALL_START",
                "toolCallId": "call-1",
                "name": "calendar.find"
            })),
            normalize_uar_agui_event(&serde_json::json!({
                "type": "TOOL_CALL_ARGS",
                "toolCallId": "call-1",
                "delta": "{\"query\":\"tomorrow\"}"
            })),
            normalize_uar_agui_event(&serde_json::json!({
                "type": "TOOL_CALL_END",
                "toolCallId": "call-1"
            })),
        ];
        let mut adapter = A2uiAdapter::new("run-1");
        let mut output = Vec::new();
        for event in events {
            output.extend(adapter.ingest(&event));
        }
        assert!(matches!(
            &output[0],
            A2uiEvent::Block {
                block: ContentBlock::ToolUse { id, name, input_json }, ..
            } if id == "call-1" && name == "calendar.find" && input_json == "{\"query\":\"tomorrow\"}"
        ));
    }

    #[test]
    fn unknown_uar_custom_events_are_preserved_as_artifacts() {
        let event = normalize_uar_agui_event(&serde_json::json!({
            "type": "CUSTOM",
            "profile": "uar.agui/1",
            "name": "uar.future.event",
            "value": {"still": "kept"}
        }));
        let mut adapter = A2uiAdapter::new("run-1");
        let output = adapter.ingest(&event);
        assert!(matches!(
            &output[0],
            A2uiEvent::Block {
                block: ContentBlock::Artifact { kind, content, .. }, ..
            } if kind == "provider_event"
                && content.contains("uar.future.event")
                && content.contains("still")
        ));
    }
}
