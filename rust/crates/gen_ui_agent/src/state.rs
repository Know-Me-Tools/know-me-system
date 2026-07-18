// TJ-ARCH-MOB-001 compliant
//! Process-lifetime agent state: the config backend (platform-selected), the
//! memory/graph-RAG store (always embedded SurrealDB — a separate concern from
//! which config-DB engine is in use, see `init`'s doc comment), and the
//! A2uiEvent broadcast channel every chat run publishes to. One instance per
//! process — callers `init()` once at startup (after migrations), then `send()`
//! and `subscribe()` freely.
use std::sync::Arc;

use gen_ui_db_graph::GraphStore;
use gen_ui_mcp::McpRegistry;
use gen_ui_types::events::A2uiEvent;
use gen_ui_types::inference::InferenceProvider;
use once_cell::sync::OnceCell;
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
    /// Registered MCP servers (C-108). Empty is the normal state — an install with no
    /// MCP server configured sends the model no tool definitions and behaves exactly as
    /// it did before tools existed. Always present (not Option) because "no servers" and
    /// "no registry" are the same thing to every caller.
    pub mcp: McpRegistry,
    events: broadcast::Sender<A2uiEvent>,
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
    let (tx, _rx) = broadcast::channel(EVENT_CHANNEL_CAPACITY);
    let state = AgentState {
        config,
        memory,
        inference,
        mcp: McpRegistry::new(),
        events: tx,
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

pub(crate) fn publish(event: A2uiEvent) {
    if let Ok(state) = get() {
        // No subscribers is not an error — a run started before any UI attached
        // (or with devtools closed) still completes; it just has no listener.
        let _ = state.events.send(event);
    }
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

/// The MCP registry. Always present; empty until a server is registered.
pub fn mcp() -> Result<&'static McpRegistry, AgentError> {
    Ok(&get()?.mcp)
}
