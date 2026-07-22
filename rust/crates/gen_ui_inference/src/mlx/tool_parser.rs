// TJ-ARCH-MOB-001 compliant
//! Streaming parser that turns Qwen `<tool_call>…</tool_call>` blocks in the
//! model's text stream into canonical tool-call `StreamEvent`s, so the host
//! tool gate and the UAR driver bridge consume the same event shape they get
//! from every other engine.
//!
//! The parser is fed token deltas as they stream. Open/close tags can split
//! across deltas, so a bounded holdback buffer keeps just enough tail to detect
//! a tag that has only partially arrived; everything definitely-not-a-tag is
//! emitted immediately as text.

use gen_ui_types::events::StreamEvent;

const OPEN_TAG: &str = "<tool_call>";
const CLOSE_TAG: &str = "</tool_call>";

/// The most bytes we might need to hold back while waiting to see whether a
/// partial tag completes — one char less than the longer tag.
const MAX_HOLDBACK: usize = CLOSE_TAG.len() - 1;

#[derive(Debug, PartialEq)]
enum State {
    /// Passing text through, watching for `<tool_call>`.
    Text,
    /// Inside a tool call, accumulating its JSON body until `</tool_call>`.
    InToolCall,
}

pub(crate) struct ToolCallParser {
    state: State,
    /// Unmatched tail that might be the start of a tag (Text state) or the
    /// accumulating body / partial close tag (InToolCall state).
    buffer: String,
    next_call_index: usize,
}

impl ToolCallParser {
    pub(crate) fn new() -> Self {
        Self {
            state: State::Text,
            buffer: String::new(),
            next_call_index: 0,
        }
    }

    /// Feed a delta; returns the stream events it completes.
    pub(crate) fn push(&mut self, delta: &str) -> Vec<StreamEvent> {
        self.buffer.push_str(delta);
        let mut out = Vec::new();
        loop {
            match self.state {
                State::Text => {
                    if let Some(pos) = self.buffer.find(OPEN_TAG) {
                        // Emit text before the tag, drop the tag, switch state.
                        let text = self.buffer[..pos].to_string();
                        if !text.is_empty() {
                            out.push(StreamEvent::TextDelta {
                                index: 0,
                                delta: text,
                            });
                        }
                        self.buffer.drain(..pos + OPEN_TAG.len());
                        self.state = State::InToolCall;
                        continue;
                    }
                    // No complete open tag. Emit everything except a tail that
                    // could still become one; keep that tail buffered.
                    let safe = safe_emit_len(&self.buffer, OPEN_TAG);
                    if safe > 0 {
                        let text = self.buffer[..safe].to_string();
                        out.push(StreamEvent::TextDelta {
                            index: 0,
                            delta: text,
                        });
                        self.buffer.drain(..safe);
                    }
                    break;
                }
                State::InToolCall => {
                    if let Some(pos) = self.buffer.find(CLOSE_TAG) {
                        let body = self.buffer[..pos].to_string();
                        self.buffer.drain(..pos + CLOSE_TAG.len());
                        self.state = State::Text;
                        out.extend(self.emit_tool_call(&body));
                        continue;
                    }
                    // Wait for more — the whole body stays buffered until the
                    // close tag arrives.
                    break;
                }
            }
        }
        out
    }

    /// Flush at end of stream: emit any buffered text, or a best-effort tool
    /// call if a block was left open, so nothing is silently dropped.
    pub(crate) fn finish(&mut self) -> Vec<StreamEvent> {
        let mut out = Vec::new();
        match self.state {
            State::Text => {
                if !self.buffer.is_empty() {
                    out.push(StreamEvent::TextDelta {
                        index: 0,
                        delta: std::mem::take(&mut self.buffer),
                    });
                }
            }
            State::InToolCall => {
                // Stream ended mid-tool-call. Emit what we have as a best-effort
                // call rather than losing it; malformed JSON is handled by
                // emit_tool_call's fallback.
                let body = std::mem::take(&mut self.buffer);
                out.extend(self.emit_tool_call(&body));
                self.state = State::Text;
            }
        }
        out
    }

    /// Turn one `<tool_call>` body (expected `{"name":…,"arguments":…}`) into
    /// the canonical `ToolCallStarted`/`Delta`/`Complete` trio. Malformed JSON
    /// is surfaced as a `Custom` diagnostic plus the raw body as text, never a
    /// dropped turn.
    fn emit_tool_call(&mut self, body: &str) -> Vec<StreamEvent> {
        let index = self.next_call_index;
        self.next_call_index += 1;
        let id = format!("mlx-tool-{index}");

        let parsed = serde_json::from_str::<serde_json::Value>(body.trim());
        let (name, arguments) = match parsed.as_ref() {
            Ok(value) => {
                let name = value
                    .get("name")
                    .and_then(serde_json::Value::as_str)
                    .map(str::to_owned);
                let arguments = value
                    .get("arguments")
                    .map(|arguments| match arguments {
                        serde_json::Value::String(raw) => raw.clone(),
                        other => other.to_string(),
                    })
                    .unwrap_or_else(|| "{}".to_owned());
                (name, arguments)
            }
            Err(_) => (None, String::new()),
        };

        match name {
            Some(name) => vec![
                StreamEvent::ToolCallStarted {
                    id: id.clone(),
                    name,
                },
                StreamEvent::ToolCallDelta {
                    id: id.clone(),
                    delta: arguments,
                },
                StreamEvent::ToolCallComplete { id },
            ],
            None => vec![
                StreamEvent::Custom {
                    source: "mlx".into(),
                    event_name: "malformed_tool_call".into(),
                    payload: serde_json::json!({
                        "message": "model emitted a <tool_call> block that did not parse as {name, arguments}",
                        "raw": body,
                    }),
                },
                StreamEvent::TextDelta {
                    index: 0,
                    delta: body.to_string(),
                },
            ],
        }
    }
}

/// How many leading bytes of `buffer` can be emitted now without risking that
/// a suffix is the start of `tag`. Keeps back the longest suffix of `buffer`
/// that is a proper prefix of `tag` (bounded by `MAX_HOLDBACK`). All tag bytes
/// are ASCII, so a suffix can only match on a char boundary; non-boundary
/// offsets are skipped, which also avoids splitting a multi-byte char mid-way.
fn safe_emit_len(buffer: &str, tag: &str) -> usize {
    let max = buffer.len().min(MAX_HOLDBACK);
    for keep in (1..=max).rev() {
        let start = buffer.len() - keep;
        if !buffer.is_char_boundary(start) {
            continue;
        }
        if tag.starts_with(&buffer[start..]) {
            return start;
        }
    }
    buffer.len()
}
