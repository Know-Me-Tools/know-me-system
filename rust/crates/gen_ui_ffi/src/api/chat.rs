// TJ-ARCH-MOB-001 compliant
//! Chat + memory/graph-RAG intent surface. Dart sends a turn and folds the
//! resulting A2uiEvent stream (see streams::chat_events) into ContentBlocks.
//! Memory/graph functions are intent-level (`memory_search`, `graph_expand`) —
//! never raw SurrealQL. All three delegate to gen_ui_agent, the SAME
//! implementation tauri-plugin-gen-ui's commands call — no duplicated
//! business logic between the mobile and desktop leaves.
// `pub use` (not `use`) — frb_generated.rs re-exports this module's types via
// `use crate::api::chat::*`, which only sees items visible through a public path.
//
// frb's Result<T,E> detection only matches a literal `Result<...>` return
// type — it does NOT resolve through a generic type alias (verified against
// flutter_rust_bridge_codegen 2.12.0: its alias-parsing filter drops any
// `type Foo<T> = ...` with generics before resolution runs). Every
// frb-exposed fn below spells out `Result<T, CoreError>` literally; using
// `CoreResult<T>` here would silently make Dart receive an opaque blob with
// no field/error access instead of a normal Future<T> that throws on Err.
pub use gen_ui_db_graph::{MemoryHit, MemoryProvenance, RelatedEntity};
pub use gen_ui_types::inference::{InferenceMessage, InferenceRole};
pub use gen_ui_types::CoreError;

/// Start a chat turn; returns the run_id whose events arrive on chat_events(run_id).
/// `messages` is the selected conversation's role/text history. The thread id
/// remains transport metadata; persistence is owned by the entity repository.
pub async fn chat_send(
    thread_id: String,
    message: String,
    messages: Vec<InferenceMessage>,
    attachment_paths: Vec<String>,
    branch_id: String,
    user_message_id: String,
) -> Result<String, CoreError> {
    gen_ui_agent::chat::send_with_attachments(
        message,
        messages,
        attachment_paths,
        thread_id,
        branch_id,
        user_message_id,
    )
    .await
    .map_err(Into::into)
}

/// Submit a typed A2UI/AG-UI user interaction and start the agent's response.
/// The JSON envelope is preserved verbatim so Rust remains the sole transport
/// boundary and the agent can correlate surface, component, and context IDs.
pub async fn chat_action(
    thread_id: String,
    interaction_json: String,
    messages: Vec<InferenceMessage>,
    branch_id: String,
) -> Result<String, CoreError> {
    gen_ui_agent::chat::send_interaction(interaction_json, messages, thread_id, branch_id)
        .await
        .map_err(Into::into)
}

/// Cancel an active run. Cancellation is cooperative: already received
/// semantic blocks remain persisted, and the stream terminates with a typed
/// `run_cancelled` event rather than fabricated assistant text.
pub fn chat_cancel(run_id: String) -> Result<bool, CoreError> {
    gen_ui_agent::state::cancel_run(&run_id).map_err(Into::into)
}

pub async fn prepare_local_model() -> Result<(), CoreError> {
    gen_ui_agent::chat::prepare_local_model()
        .await
        .map_err(Into::into)
}

pub fn local_model_status() -> Result<String, CoreError> {
    let status = gen_ui_agent::chat::local_model_status()?;
    serde_json::to_string(&status).map_err(|error| CoreError::Serde(error.to_string()))
}

pub fn cancel_model_operation() -> Result<(), CoreError> {
    gen_ui_agent::chat::cancel_model_operation().map_err(Into::into)
}

pub async fn local_runtime_diagnostics() -> Result<String, CoreError> {
    let diagnostics = gen_ui_agent::chat::local_runtime_diagnostics().await?;
    serde_json::to_string(&diagnostics).map_err(|error| CoreError::Serde(error.to_string()))
}

pub async fn theme_mode() -> Result<String, CoreError> {
    gen_ui_agent::chat::theme_mode().await.map_err(|error| {
        tracing::error!(%error, "theme preference read failed");
        error.into()
    })
}

pub async fn set_theme_mode(mode: String) -> Result<(), CoreError> {
    gen_ui_agent::chat::set_theme_mode(&mode)
        .await
        .map_err(|error| {
            tracing::error!(%error, %mode, "theme preference write failed");
            error.into()
        })
}

pub async fn resolve_remote_content(url: String, kind: String) -> Result<String, CoreError> {
    let resolved = gen_ui_agent::media::resolve(&url, &kind).await?;
    serde_json::to_string(&resolved).map_err(|error| CoreError::Serde(error.to_string()))
}

/// Ingest a note into memory (embeds on-device via fastembed, upserts into
/// SurrealDB). Returns the assigned record id.
pub async fn memory_ingest(text: String) -> Result<String, CoreError> {
    gen_ui_agent::memory::ingest(text).await.map_err(Into::into)
}

/// Extract a document with xberg and ingest its normalized content into graph-RAG.
pub async fn memory_ingest_attachment(path: String) -> Result<String, CoreError> {
    gen_ui_agent::memory::ingest_attachment(path)
        .await
        .map_err(Into::into)
}

/// Extract and ingest up to six documents with xberg, returning record ids in order.
pub async fn memory_ingest_attachments(paths: Vec<String>) -> Result<Vec<String>, CoreError> {
    gen_ui_agent::memory::ingest_attachments(paths)
        .await
        .map_err(Into::into)
}

/// Hybrid memory search (vector recall + graph expansion + BM25, RRF-fused in Rust).
pub async fn memory_search(query: String, k: u32) -> Result<Vec<MemoryHit>, CoreError> {
    gen_ui_agent::memory::search(query, k)
        .await
        .map_err(Into::into)
}

pub async fn memory_delete_conversation_context(
    conversation_id: String,
) -> Result<usize, CoreError> {
    gen_ui_agent::memory::delete_conversation_context(conversation_id)
        .await
        .map_err(Into::into)
}

pub async fn memory_delete_message_context(message_ids: Vec<String>) -> Result<usize, CoreError> {
    gen_ui_agent::memory::delete_message_context(message_ids)
        .await
        .map_err(Into::into)
}

pub async fn memory_promote_context(ids: Vec<String>) -> Result<usize, CoreError> {
    gen_ui_agent::memory::promote_context(ids)
        .await
        .map_err(Into::into)
}

/// Expand the entity graph around a node to a given depth.
pub async fn graph_expand(entity_id: String, depth: u32) -> Result<Vec<RelatedEntity>, CoreError> {
    gen_ui_agent::memory::graph_expand(entity_id, depth)
        .await
        .map_err(Into::into)
}
