// TJ-ARCH-MOB-001 compliant
//! Process-lifetime agent state: the config backend (platform-selected), the
//! memory/graph-RAG store (always embedded SurrealDB — a separate concern from
//! which config-DB engine is in use, see `init`'s doc comment), and the
//! A2uiEvent broadcast channel every chat run publishes to. One instance per
//! process — callers `init()` once at startup (after migrations), then `send()`
//! and `subscribe()` freely.
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use gen_ui_db_graph::GraphStore;
use gen_ui_mcp::McpRegistry;
use gen_ui_types::events::A2uiEvent;
use gen_ui_types::inference::InferenceProvider;
use once_cell::sync::OnceCell;
use serde_json::Value as JsonValue;
use tokio::sync::broadcast;

use crate::config::ConfigBackend;
use crate::error::AgentError;

/// Ring-buffer depth for the A2uiEvent channel. Chat turns are short-lived and
/// consumed live; a slow/absent subscriber only misses backlog, never blocks
/// the sender (broadcast::Sender::send never awaits).
const EVENT_CHANNEL_CAPACITY: usize = 256;

pub struct AgentState {
    pub config: ConfigBackend,
    pub memory: Arc<GraphStore>,
    /// Local-inference engine, when this platform has one. `None` on surfaces
    /// with no local lane (web drives WebLLM from JS; a build without the
    /// engine feature), in which case a `"local"` model pref resolves to
    /// `AgentError::NoLocalEngine` rather than silently answering from cloud —
    /// a user who asked for on-device inference must never be quietly switched
    /// to a network provider.
    pub inference: Option<Arc<dyn InferenceProvider>>,
    /// Complete in-process UAR kernel for transport-free mobile agent runs.
    /// Desktop/web may intentionally leave this absent and use the HTTP facade.
    pub embedded_uar: Option<Arc<crate::uar::UarEmbeddedRuntime>>,
    /// Registered MCP servers (C-108). Empty is the normal state — an install with no
    /// MCP server configured sends the model no tool definitions and behaves exactly as
    /// it did before tools existed. Always present (not Option) because "no servers" and
    /// "no registry" are the same thing to every caller.
    pub mcp: McpRegistry,
    events: broadcast::Sender<A2uiEvent>,
    event_history: parking_lot::Mutex<std::collections::BTreeMap<String, Vec<A2uiEvent>>>,
    run_cancellations: parking_lot::Mutex<std::collections::BTreeMap<String, Arc<AtomicBool>>>,
    uar_run_mappings: parking_lot::Mutex<std::collections::BTreeMap<String, UarRunMapping>>,
    pending_tool_confirmations:
        parking_lot::Mutex<std::collections::BTreeMap<String, PendingToolConfirmation>>,
}

#[derive(Debug, Clone)]
pub(crate) struct PendingToolConfirmation {
    pub approval_id: String,
    pub source_run_id: String,
    pub tool_use_id: String,
    pub tool_name: String,
    pub arguments: JsonValue,
}

#[derive(Debug, Clone)]
pub(crate) struct UarRunMapping {
    pub knowme_run_id: String,
    pub uar_run_id: String,
}

static STATE: OnceCell<AgentState> = OnceCell::new();

/// Initialise process-lifetime agent state with the platform's config backend
/// and its (always SurrealDB) memory/graph-RAG store. Call once at startup,
/// after the config DB's own migrations have run. `memory` is a separate
/// GraphStore instance from `ConfigBackend::Surreal` (mobile's config store) —
/// desktop has a Postgres config backend but STILL needs its own GraphStore
/// here, since memory/graph-RAG is a SurrealDB-only concern on every platform
/// (see references/arch-standard.md's "SurrealDB embedded" core component).
///
/// Calling twice is a programming error (typestate would be nicer, but both
/// callers — the Tauri plugin's `setup` and the FFI's `run_migrations` — are
/// naturally single-shot per process already). A second call is a no-op: the
/// first-set state (and its broadcast channel, which may already have live
/// subscribers) keeps running rather than being silently replaced — but it is
/// now logged loudly, since silently swallowing it previously masked exactly
/// this kind of programming error during development.
pub fn init(config: ConfigBackend, memory: Arc<GraphStore>) {
    init_with_inference(config, memory, None)
}

/// `init`, plus a local-inference engine for platforms that have one (desktop
/// passes an engine such as `gen_ui_inference::LlamaCppEngine`). Kept as a separate constructor
/// rather than a fourth `init` parameter: the engine is genuinely optional and
/// platform-specific, and every existing `init` call site stays correct.
pub fn init_with_inference(
    config: ConfigBackend,
    memory: Arc<GraphStore>,
    inference: Option<Arc<dyn InferenceProvider>>,
) {
    init_with_inference_and_uar(config, memory, inference, None)
}

/// `init_with_inference`, plus a fully constructed embedded UAR kernel.
/// Mobile boot uses this composition root so chat never has to discover or
/// construct runtime dependencies after a user submits a message.
pub fn init_with_inference_and_uar(
    config: ConfigBackend,
    memory: Arc<GraphStore>,
    inference: Option<Arc<dyn InferenceProvider>>,
    embedded_uar: Option<Arc<crate::uar::UarEmbeddedRuntime>>,
) {
    let (tx, _rx) = broadcast::channel(EVENT_CHANNEL_CAPACITY);
    let state = AgentState {
        config,
        memory,
        inference,
        embedded_uar,
        mcp: McpRegistry::new(),
        events: tx,
        event_history: parking_lot::Mutex::new(Default::default()),
        run_cancellations: parking_lot::Mutex::new(Default::default()),
        uar_run_mappings: parking_lot::Mutex::new(Default::default()),
        pending_tool_confirmations: parking_lot::Mutex::new(Default::default()),
    };
    if STATE.set(state).is_err() {
        tracing::warn!(
            "gen_ui_agent::state::init called more than once in this process; \
             ignoring the second call and keeping the original config backend, \
             memory store, and event broadcast channel"
        );
    }
}

pub(crate) fn get() -> Result<&'static AgentState, AgentError> {
    STATE.get().ok_or(AgentError::NotInitialised)
}

/// Subscribe to the live A2uiEvent feed. Each platform leaf (frb StreamSink,
/// Tauri emit) owns forwarding these into its own transport.
pub fn subscribe() -> Result<broadcast::Receiver<A2uiEvent>, AgentError> {
    Ok(get()?.events.subscribe())
}

/// Atomically subscribe to future events and snapshot any events already
/// published for a run. This closes the send-then-subscribe race at FFI/HTTP
/// boundaries without making producers wait for a UI listener.
pub fn subscribe_run(
    run_id: &str,
) -> Result<(Vec<A2uiEvent>, broadcast::Receiver<A2uiEvent>), AgentError> {
    let state = get()?;
    let history = state.event_history.lock();
    let receiver = state.events.subscribe();
    let backlog = history.get(run_id).cloned().unwrap_or_default();
    drop(history);
    Ok((backlog, receiver))
}

pub(crate) fn publish(event: A2uiEvent) {
    if let Ok(state) = get() {
        {
            let mut history = state.event_history.lock();
            let events = history.entry(event.run_id().to_owned()).or_default();
            if events.len() == EVENT_CHANNEL_CAPACITY {
                events.remove(0);
            }
            events.push(event.clone());
            while history.len() > 32 {
                if let Some(key) = history.keys().next().cloned() {
                    history.remove(&key);
                }
            }
        }
        // No subscribers is not an error — a run started before any UI attached
        // (or with devtools closed) still completes; it just has no listener.
        let _ = state.events.send(event);
    }
}

pub(crate) fn register_run(run_id: &str) -> Arc<AtomicBool> {
    let token = Arc::new(AtomicBool::new(false));
    if let Ok(state) = get() {
        state
            .run_cancellations
            .lock()
            .insert(run_id.to_owned(), token.clone());
    }
    token
}

pub fn cancel_run(run_id: &str) -> Result<bool, AgentError> {
    let state = get()?;
    let token = state.run_cancellations.lock().get(run_id).cloned();
    if let Some(token) = token {
        token.store(true, Ordering::Release);
        // The token alone is not enough for a local run. Callers poll it at
        // phase boundaries, but on-device generation is a single blocking call
        // into the engine — on Android a JNI `generate_text` that can run for
        // tens of seconds. Without signalling the engine itself, a stop tap
        // does nothing until generation finishes on its own, which is what the
        // device certification caught: the run completed with 1506 chars after
        // cancel was tapped.
        //
        // `cancel_generation` is best-effort and idempotent: engines with no
        // interruptible generation ignore it, and the token still governs the
        // phase-boundary checks, so this only ever shortens a run.
        if let Ok(inference) = inference() {
            inference.cancel_generation();
        }
        Ok(true)
    } else {
        Ok(false)
    }
}

pub(crate) fn finish_run(run_id: &str) {
    if let Ok(state) = get() {
        state.run_cancellations.lock().remove(run_id);
        state
            .uar_run_mappings
            .lock()
            .retain(|_, mapping| mapping.knowme_run_id != run_id);
    }
}

pub(crate) fn register_uar_run(conversation_id: &str, knowme_run_id: &str, uar_run_id: &str) {
    if let Ok(state) = get() {
        state.uar_run_mappings.lock().insert(
            conversation_id.to_owned(),
            UarRunMapping {
                knowme_run_id: knowme_run_id.to_owned(),
                uar_run_id: uar_run_id.to_owned(),
            },
        );
    }
}

pub(crate) fn uar_run_for_conversation(
    conversation_id: &str,
) -> Result<Option<UarRunMapping>, AgentError> {
    Ok(get()?.uar_run_mappings.lock().get(conversation_id).cloned())
}

pub(crate) fn register_pending_tool_confirmation(pending: PendingToolConfirmation) {
    if let Ok(state) = get() {
        state
            .pending_tool_confirmations
            .lock()
            .insert(pending.approval_id.clone(), pending);
    }
}

pub(crate) fn take_pending_tool_confirmation(
    approval_id: &str,
) -> Result<PendingToolConfirmation, AgentError> {
    let state = get()?;
    state
        .pending_tool_confirmations
        .lock()
        .remove(approval_id)
        .ok_or_else(|| AgentError::Interaction(format!("no pending tool approval '{approval_id}'")))
}

pub(crate) fn config() -> Result<&'static ConfigBackend, AgentError> {
    Ok(&get()?.config)
}

pub(crate) fn memory() -> Result<&'static Arc<GraphStore>, AgentError> {
    Ok(&get()?.memory)
}

/// The local-inference engine, or `NoLocalEngine` when this build/platform has
/// none. Deliberately an error rather than a cloud fallback — see the
/// `inference` field's doc comment.
pub(crate) fn inference() -> Result<&'static Arc<dyn InferenceProvider>, AgentError> {
    get()?.inference.as_ref().ok_or(AgentError::NoLocalEngine)
}

/// The transport-free embedded UAR kernel registered by the platform leaf.
pub(crate) fn embedded_uar() -> Result<&'static Arc<crate::uar::UarEmbeddedRuntime>, AgentError> {
    get()?
        .embedded_uar
        .as_ref()
        .ok_or_else(|| AgentError::Config("embedded UAR runtime is not available".into()))
}

/// The MCP registry. Always present; empty until a server is registered.
pub fn mcp() -> Result<&'static McpRegistry, AgentError> {
    Ok(&get()?.mcp)
}
