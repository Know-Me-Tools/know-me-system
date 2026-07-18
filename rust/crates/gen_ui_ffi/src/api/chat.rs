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
pub use gen_ui_db_graph::{MemoryHit, RelatedEntity};
pub use gen_ui_types::CoreError;

/// Start a chat turn; returns the run_id whose events arrive on chat_events(run_id).
/// `messages` is the selected conversation's role/text history. The thread id
/// remains transport metadata; persistence is owned by the entity repository.
pub async fn chat_send(
    thread_id: String,
    message: String,
    messages: Vec<String>,
    attachment_paths: Vec<String>,
) -> Result<String, CoreError> {
    let _ = thread_id;
    gen_ui_agent::chat::send_with_attachments(message, messages, attachment_paths)
        .await
        .map_err(Into::into)
}

/// Submit a typed A2UI/AG-UI user interaction and start the agent's response.
/// The JSON envelope is preserved verbatim so Rust remains the sole transport
/// boundary and the agent can correlate surface, component, and context IDs.
pub async fn chat_action(
    thread_id: String,
    interaction_json: String,
    messages: Vec<String>,
) -> Result<String, CoreError> {
    let _ = thread_id;
    gen_ui_agent::chat::send_interaction(interaction_json, messages)
        .await
        .map_err(Into::into)
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

/// Hybrid memory search (vector recall + graph expansion + BM25, RRF-fused in Rust).
pub async fn memory_search(query: String, k: u32) -> Result<Vec<MemoryHit>, CoreError> {
    gen_ui_agent::memory::search(query, k)
        .await
        .map_err(Into::into)
}

/// Expand the entity graph around a node to a given depth.
pub async fn graph_expand(entity_id: String, depth: u32) -> Result<Vec<RelatedEntity>, CoreError> {
    gen_ui_agent::memory::graph_expand(entity_id, depth)
        .await
        .map_err(Into::into)
}
