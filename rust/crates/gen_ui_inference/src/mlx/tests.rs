use super::*;
use gen_ui_types::inference::{InferenceRequest, SampleParams};
use std::sync::Mutex;

trait LockExt<T> {
    fn lock_ok(&self) -> std::sync::MutexGuard<'_, T>;
}
impl<T> LockExt<T> for Mutex<T> {
    fn lock_ok(&self) -> std::sync::MutexGuard<'_, T> {
        self.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

/// A scripted bridge: `load` succeeds, `generate` replays a fixed set of
/// bridge events, and calls are recorded for assertions.
struct MockMlxBridge {
    events: Mutex<Vec<MlxBridgeEvent>>,
    load_calls: Mutex<Vec<(PathBuf, String)>>,
    generate_calls: Mutex<Vec<String>>,
    /// Counts `cancel` so a test can prove the engine actually reached the
    /// bridge. Before `cancel_generation` was overridden, the trait default
    /// swallowed the request and MLX generated to completion — a silent gap that
    /// only a call-count assertion catches.
    cancel_calls: Mutex<usize>,
    available: bool,
    fail_load: bool,
}

impl MockMlxBridge {
    fn with_events(events: Vec<MlxBridgeEvent>) -> Self {
        Self {
            events: Mutex::new(events),
            load_calls: Mutex::new(Vec::new()),
            generate_calls: Mutex::new(Vec::new()),
            cancel_calls: Mutex::new(0),
            available: true,
            fail_load: false,
        }
    }
}

impl MlxBridge for MockMlxBridge {
    fn load(&self, model_dir: &Path, config_json: &str) -> CoreResult<()> {
        self.load_calls
            .lock_ok()
            .push((model_dir.to_owned(), config_json.to_owned()));
        if self.fail_load {
            Err(CoreError::Terminal("scripted load failure".into()))
        } else {
            Ok(())
        }
    }

    fn generate(
        &self,
        request_json: &str,
        on_event: &mut dyn FnMut(MlxBridgeEvent),
    ) -> CoreResult<()> {
        self.generate_calls.lock_ok().push(request_json.to_owned());
        for event in self.events.lock_ok().drain(..) {
            on_event(event);
        }
        Ok(())
    }

    fn cancel(&self) {
        *self.cancel_calls.lock_ok() += 1;
    }
    fn close(&self) {}
    fn available(&self) -> bool {
        self.available
    }
}

fn text_request(user: &str) -> InferenceRequest {
    InferenceRequest {
        prompt: String::new(),
        messages: vec![gen_ui_types::inference::InferenceMessage {
            role: InferenceRole::User,
            content: user.into(),
            name: None,
            tool_call_id: None,
        }],
        tools: vec![],
        images: vec![],
        generation_settings: SampleParams::default(),
        grammar: None,
        originating_plan: None,
    }
}

/// The engine's generation path uses `gen_ui_runtime::spawn_blocking`, which
/// needs the process runtime initialized. Idempotent across tests.
fn init_runtime() {
    use std::sync::Once;
    static INIT: Once = Once::new();
    INIT.call_once(|| gen_ui_runtime::init(Some(2)));
}

async fn collect(stream: futures::stream::BoxStream<'static, StreamEvent>) -> Vec<StreamEvent> {
    stream.collect::<Vec<_>>().await
}

#[tokio::test]
async fn generation_wraps_deltas_with_message_start_and_done() {
    init_runtime();
    let bridge = Arc::new(MockMlxBridge::with_events(vec![
        MlxBridgeEvent::Delta {
            text: "Hello ".into(),
        },
        MlxBridgeEvent::Delta {
            text: "world".into(),
        },
        MlxBridgeEvent::Final,
    ]));
    let engine = MlxEngine::with_bridge("/tmp/knowme-mlx-test", bridge.clone());
    let events = collect(engine.generate_request(&text_request("hi")).await.unwrap()).await;

    assert!(matches!(events.first(), Some(StreamEvent::MessageStart)));
    assert!(matches!(events.last(), Some(StreamEvent::Done)));
    let text: String = events
        .iter()
        .filter_map(|event| match event {
            StreamEvent::TextDelta { delta, .. } => Some(delta.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(text, "Hello world");
    // The request the bridge saw carried the user message.
    assert!(bridge.generate_calls.lock_ok()[0].contains("\"hi\""));
}

#[tokio::test]
async fn tool_call_in_stream_becomes_canonical_events() {
    init_runtime();
    let bridge = Arc::new(MockMlxBridge::with_events(vec![
        MlxBridgeEvent::Delta {
            text: "Let me check. ".into(),
        },
        MlxBridgeEvent::Delta {
            text: "<tool_call>{\"name\":\"get_weather\",\"arguments\":{\"city\":\"Paris\"}}</tool_call>".into(),
        },
        MlxBridgeEvent::Final,
    ]));
    let engine = MlxEngine::with_bridge("/tmp/knowme-mlx-test", bridge);
    let events = collect(
        engine
            .generate_request(&text_request("weather?"))
            .await
            .unwrap(),
    )
    .await;

    let started = events.iter().find_map(|event| match event {
        StreamEvent::ToolCallStarted { name, .. } => Some(name.clone()),
        _ => None,
    });
    assert_eq!(started.as_deref(), Some("get_weather"));
    assert!(events.iter().any(|event| matches!(
        event,
        StreamEvent::ToolCallDelta { delta, .. } if delta.contains("Paris")
    )));
    assert!(events
        .iter()
        .any(|event| matches!(event, StreamEvent::ToolCallComplete { .. })));
    // Text before the tag survives; the tag itself does not leak as text.
    let text: String = events
        .iter()
        .filter_map(|event| match event {
            StreamEvent::TextDelta { delta, .. } => Some(delta.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(text, "Let me check. ");
    assert!(!text.contains("tool_call"));
}

#[tokio::test]
async fn unavailable_bridge_ends_cleanly_with_error() {
    init_runtime();
    let mut bridge = MockMlxBridge::with_events(vec![]);
    bridge.available = false;
    let engine = MlxEngine::with_bridge("/tmp/knowme-mlx-test", Arc::new(bridge));
    let events = collect(engine.generate_request(&text_request("hi")).await.unwrap()).await;
    assert!(events
        .iter()
        .any(|event| matches!(event, StreamEvent::Error { .. })));
    assert!(matches!(events.last(), Some(StreamEvent::Done)));
}

// -------- tool-call parser --------

fn parse_all(deltas: &[&str]) -> Vec<StreamEvent> {
    let mut parser = ToolCallParser::new();
    let mut out = Vec::new();
    for delta in deltas {
        out.extend(parser.push(delta));
    }
    out.extend(parser.finish());
    out
}

/// Parse with Qwen3 `<think>` reasoning splitting enabled.
fn parse_all_reasoning(deltas: &[&str]) -> Vec<StreamEvent> {
    use gen_ui_types::inference::ReasoningFormat;
    let mut parser = ToolCallParser::with_reasoning(ReasoningFormat::Qwen3Think);
    let mut out = Vec::new();
    for delta in deltas {
        out.extend(parser.push(delta));
    }
    out.extend(parser.finish());
    out
}

fn thinking_of(events: &[StreamEvent]) -> String {
    events
        .iter()
        .filter_map(|event| match event {
            StreamEvent::ThinkingDelta { delta, .. } => Some(delta.clone()),
            _ => None,
        })
        .collect()
}

fn text_of(events: &[StreamEvent]) -> String {
    events
        .iter()
        .filter_map(|event| match event {
            StreamEvent::TextDelta { delta, .. } => Some(delta.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn parser_passes_plain_text_through() {
    let events = parse_all(&["hello ", "there"]);
    assert_eq!(text_of(&events), "hello there");
    assert!(!events
        .iter()
        .any(|e| matches!(e, StreamEvent::ToolCallStarted { .. })));
}

#[test]
fn parser_handles_open_tag_split_across_deltas() {
    // "<tool" then "_call>{...}" then "</tool_call>" split awkwardly.
    let events = parse_all(&[
        "before <too",
        "l_call>{\"name\":\"f\",\"arguments\":{}}</tool",
        "_call> after",
    ]);
    assert_eq!(text_of(&events), "before  after");
    assert_eq!(
        events
            .iter()
            .filter(|e| matches!(e, StreamEvent::ToolCallStarted { .. }))
            .count(),
        1
    );
    // The split "<too"/"l_call>" must not leak as text.
    assert!(!text_of(&events).contains("tool"));
}

#[test]
fn parser_close_tag_split_across_deltas() {
    let events = parse_all(&[
        "<tool_call>{\"name\":\"g\",\"arguments\":{\"x\":1}}<",
        "/tool_call>done",
    ]);
    assert_eq!(text_of(&events), "done");
    assert!(events.iter().any(|e| matches!(
        e,
        StreamEvent::ToolCallDelta { delta, .. } if delta.contains("\"x\":1")
    )));
}

#[test]
fn parser_malformed_tool_call_falls_back_to_diagnostic_and_text() {
    let events = parse_all(&["<tool_call>not json at all</tool_call>"]);
    assert!(events.iter().any(|e| matches!(
        e,
        StreamEvent::Custom { event_name, .. } if event_name == "malformed_tool_call"
    )));
    assert!(text_of(&events).contains("not json"));
    assert!(!events
        .iter()
        .any(|e| matches!(e, StreamEvent::ToolCallStarted { .. })));
}

#[test]
fn parser_unterminated_tool_call_flushes_on_finish() {
    // Stream ends mid-tool-call; the body must still surface, not vanish.
    let events = parse_all(&["<tool_call>{\"name\":\"h\",\"arguments\":{}}"]);
    assert!(events.iter().any(|e| matches!(
        e,
        StreamEvent::ToolCallStarted { name, .. } if name == "h"
    )));
}

#[test]
fn parser_text_after_tool_call_is_preserved() {
    let events =
        parse_all(&["<tool_call>{\"name\":\"a\",\"arguments\":{}}</tool_call>trailing text"]);
    assert_eq!(text_of(&events), "trailing text");
    assert!(events
        .iter()
        .any(|e| matches!(e, StreamEvent::ToolCallStarted { .. })));
}

// -------- reasoning (`<think>`) splitting --------

#[test]
fn think_block_becomes_thinking_deltas_not_text() {
    // Pre-filled shape (device reality): reasoning body, then `</think>`, then
    // content — the opening `<think>` lives in the prompt, not the stream.
    let events = parse_all_reasoning(&["the user wants a greeting</think>Hello there!"]);
    assert_eq!(thinking_of(&events), "the user wants a greeting");
    assert_eq!(text_of(&events), "Hello there!");
    // Reasoning must never leak into content.
    assert!(!text_of(&events).contains("the user wants"));
}

#[test]
fn think_disabled_passes_tags_through_as_text() {
    // ReasoningFormat::None: a literal <think> is ordinary content.
    let events = parse_all(&["<think>kept</think> body"]);
    assert!(thinking_of(&events).is_empty());
    assert_eq!(text_of(&events), "<think>kept</think> body");
}

#[test]
fn think_close_split_across_deltas() {
    // Pre-filled reasoning whose closing `</think>` splits across deltas.
    let events = parse_all_reasoning(&["reason", "ing text</thi", "nk>answer"]);
    assert_eq!(thinking_of(&events), "reasoning text");
    assert_eq!(text_of(&events), "answer");
    // The split "</thi"/"nk>" must not leak as text.
    assert!(!text_of(&events).contains("think"));
}

#[test]
fn prefilled_think_close_only_splits_reasoning_from_content() {
    // The REAL device shape: Qwen3's chat template pre-fills the opening
    // `<think>`, so the generated stream starts inside reasoning and emits only
    // the closing `</think>`. Everything before it is reasoning, not content.
    // (This is the regression from the on-device screenshot: a bare `</think>`
    // and reasoning leaking into the answer bubble.)
    let events = parse_all_reasoning(&[
        "The user just wants a greeting.\n</think>\n\nHello! How can I assist you today?",
    ]);
    assert_eq!(
        thinking_of(&events).trim(),
        "The user just wants a greeting."
    );
    assert_eq!(
        text_of(&events).trim(),
        "Hello! How can I assist you today?"
    );
    // The bare close tag must never appear in content.
    assert!(!text_of(&events).contains("</think>"));
    assert!(!text_of(&events).contains("The user just wants"));
}

#[test]
fn prefilled_empty_think_yields_pure_content() {
    // Thinking pre-filled but the model reasons nothing: `</think>` comes first,
    // then content. No ThinkingDelta of substance; content is clean.
    let events = parse_all_reasoning(&["</think>Just the answer."]);
    assert!(thinking_of(&events).trim().is_empty());
    assert_eq!(text_of(&events).trim(), "Just the answer.");
}

#[test]
fn prefilled_close_split_across_deltas() {
    // The pre-filled `</think>` arrives split across token deltas; the partial
    // tag must not leak into reasoning or content.
    let events = parse_all_reasoning(&["weighing options</thi", "nk>final answer"]);
    assert_eq!(thinking_of(&events), "weighing options");
    assert_eq!(text_of(&events), "final answer");
    assert!(!text_of(&events).contains("think"));
    assert!(!thinking_of(&events).contains("think"));
}

#[test]
fn prefilled_then_tool_call_after_reasoning() {
    // Pre-filled reasoning, then a tool call in the content region.
    let events = parse_all_reasoning(&[
        "need the weather</think>",
        "<tool_call>{\"name\":\"weather\",\"arguments\":{\"city\":\"SF\"}}</tool_call>",
        "Here you go.",
    ]);
    assert_eq!(thinking_of(&events), "need the weather");
    assert_eq!(text_of(&events), "Here you go.");
    assert!(events.iter().any(|e| matches!(
        e,
        StreamEvent::ToolCallStarted { name, .. } if name == "weather"
    )));
}

#[test]
fn prefilled_tool_call_before_any_close_is_not_reasoning() {
    // Edge: thinking pre-filled, but the model goes straight to a tool call with
    // no `</think>`. The call must be parsed as a tool call, not swallowed as
    // reasoning body.
    let events = parse_all_reasoning(&[
        "<tool_call>{\"name\":\"weather\",\"arguments\":{\"city\":\"SF\"}}</tool_call>done",
    ]);
    assert!(events.iter().any(|e| matches!(
        e,
        StreamEvent::ToolCallStarted { name, .. } if name == "weather"
    )));
    assert_eq!(text_of(&events), "done");
    assert!(thinking_of(&events).is_empty());
}

#[test]
fn think_unterminated_flushes_as_thinking_on_finish() {
    // Stream ends still inside the pre-filled reasoning block (no `</think>`);
    // the remainder must surface as reasoning, not vanish or leak as content.
    let events = parse_all_reasoning(&["partial reasoning that never closes"]);
    assert_eq!(thinking_of(&events), "partial reasoning that never closes");
    assert!(text_of(&events).is_empty());
}

#[test]
fn explicit_think_open_is_still_absorbed() {
    // A model that emits its own leading `<think>` open (not pre-filled) is
    // handled identically: the open is stripped, body is reasoning, content
    // follows the close.
    let events = parse_all_reasoning(&["<think>explicit open</think>and the answer"]);
    assert_eq!(thinking_of(&events), "explicit open");
    assert_eq!(text_of(&events), "and the answer");
}

#[tokio::test]
async fn cancel_generation_reaches_the_bridge() {
    init_runtime();
    // The trait default for `cancel_generation` is a no-op, so an engine that
    // forgets to override it accepts a stop and keeps generating. Only the
    // bridge call count distinguishes "wired" from "silently ignored".
    let bridge = Arc::new(MockMlxBridge::with_events(vec![MlxBridgeEvent::Final]));
    let engine = MlxEngine::with_bridge("/tmp/knowme-mlx-test", bridge.clone());

    assert_eq!(*bridge.cancel_calls.lock_ok(), 0);
    engine.cancel_generation();
    assert_eq!(
        *bridge.cancel_calls.lock_ok(),
        1,
        "cancel_generation must ask the bridge to stop the in-flight generate"
    );
}

#[tokio::test]
async fn cancelled_generation_uses_the_shared_wording() {
    init_runtime();
    // One state, one phrase. The engine name has no place in what the user
    // reads, and the device certification asserts this exact string.
    let bridge = Arc::new(MockMlxBridge::with_events(vec![
        MlxBridgeEvent::Delta {
            text: "partial".into(),
        },
        MlxBridgeEvent::Cancelled,
    ]));
    let engine = MlxEngine::with_bridge("/tmp/knowme-mlx-test", bridge.clone());
    let events = collect(engine.generate_request(&text_request("hi")).await.unwrap()).await;

    let cancelled = events.iter().find_map(|event| match event {
        StreamEvent::Cancelled { message } => Some(message.clone()),
        _ => None,
    });
    assert_eq!(cancelled.as_deref(), Some("Cancelled by user"));
    // The partial text must survive the cancel — that is the whole point.
    assert_eq!(text_of(&events), "partial");
}
