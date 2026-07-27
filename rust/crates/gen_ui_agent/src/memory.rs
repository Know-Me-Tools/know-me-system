// TJ-ARCH-MOB-001 compliant
//! Shared memory/graph-RAG implementation. Called identically from gen_ui_ffi
//! (mobile, via frb) and tauri-plugin-gen-ui (desktop, via Tauri IPC) — both
//! platforms share the SAME embedded SurrealDB GraphStore instance shape (see
//! `state::init`'s doc comment on why this is not tied to `ConfigBackend`).
use gen_ui_db_graph::{MemoryHit, MemoryProvenance, MemoryRecord, RelatedEntity, SearchMode};

use crate::error::AgentError;
use crate::state;

/// Ingest a note into memory (embeds on-device via fastembed, upserts into
/// SurrealDB). Returns the assigned record id.
pub async fn ingest(text: String) -> Result<String, AgentError> {
    let store = state::memory()?;
    store
        .memory_ingest(MemoryRecord {
            id: None,
            text,
            kind: "note".to_string(),
            entity: None,
            provenance: None,
        })
        .await
        .map_err(|e| AgentError::Config(e.to_string()))
}

/// Extract a user-selected document with xberg, then ingest the normalized text
/// into graph-RAG. Chat and RAG intentionally share the same processor so a file
/// cannot be readable in one path and opaque in the other.
pub async fn ingest_attachment(path: String) -> Result<String, AgentError> {
    let document = gen_ui_document::process_path(&path)
        .await
        .map_err(|error| {
            AgentError::Attachment(format!("xberg could not process document: {error}"))
        })?;
    let text = format!("# {}\n\n{}", document.file_name, document.text);
    let store = state::memory()?;
    store
        .memory_ingest(MemoryRecord {
            id: None,
            text,
            kind: "document".to_string(),
            entity: None,
            provenance: None,
        })
        .await
        .map_err(|e| AgentError::Config(e.to_string()))
}

/// Extract and ingest multiple documents in order, returning their record ids.
pub async fn ingest_attachments(paths: Vec<String>) -> Result<Vec<String>, AgentError> {
    let mut ids = Vec::with_capacity(paths.len().min(6));
    for path in paths.into_iter().take(6) {
        ids.push(ingest_attachment(path).await?);
    }
    Ok(ids)
}

/// Persist a normalized chat attachment as stable, conversation-scoped chunks.
/// The same chunks feed immediate context and later RAG turns; callers must use
/// the explicit promotion path before they can become durable knowledge.
pub async fn ingest_conversation_document(
    document: &gen_ui_document::ProcessedDocument,
    conversation_id: &str,
    branch_id: &str,
    message_id: &str,
    run_id: &str,
) -> Result<Vec<String>, AgentError> {
    let provenance = gen_ui_document::DocumentProvenance {
        conversation_id: conversation_id.into(),
        branch_id: branch_id.into(),
        message_id: message_id.into(),
        run_id: run_id.into(),
        source_name: document.file_name.clone(),
    };
    let chunks = gen_ui_document::chunk_document(
        document,
        &provenance,
        gen_ui_document::ChunkStrategy::ParentChild {
            child_target_chars: 1_200,
        },
        4_800,
    );
    let store = state::memory()?;
    let mut ids = Vec::with_capacity(chunks.len());
    for chunk in chunks {
        ids.push(
            store
                .memory_ingest(MemoryRecord {
                    id: Some(chunk.id),
                    text: chunk.text,
                    kind: "document_chunk".into(),
                    entity: None,
                    provenance: Some(MemoryProvenance {
                        conversation_id: Some(conversation_id.into()),
                        branch_id: Some(branch_id.into()),
                        message_id: Some(message_id.into()),
                        run_id: Some(run_id.into()),
                        durable: false,
                    }),
                })
                .await
                .map_err(|error| AgentError::Config(error.to_string()))?,
        );
    }
    Ok(ids)
}

/// Hybrid semantic + lexical search over ingested memory, RRF-fused in the DB.
pub async fn search(query: String, k: u32) -> Result<Vec<MemoryHit>, AgentError> {
    search_with(query, k, SearchMode::Hybrid).await
}

/// `search`, choosing the retrieval lane (C-111 T3).
///
/// `SearchMode::Vector` is a diagnostic — it drops the BM25 lane and RRF so the UI can
/// show what fusion actually buys on the same query. Scores are comparable only within a
/// mode, so callers must never merge results from both.
pub async fn search_with(
    query: String,
    k: u32,
    mode: SearchMode,
) -> Result<Vec<MemoryHit>, AgentError> {
    let store = state::memory()?;
    store
        .memory_search_with(&query, k as usize, mode)
        .await
        .map_err(|e| AgentError::Config(e.to_string()))
}

pub async fn delete_conversation_context(conversation_id: String) -> Result<usize, AgentError> {
    // Attachment BYTES go with the conversation. Both UIs already call this on
    // conversation delete, and without it every stored file leaks forever: the
    // rows disappear from the transcript while the disk keeps growing.
    //
    // Deliberately best-effort. Failing the whole delete because a file could
    // not be unlinked would leave the user unable to remove a conversation at
    // all — the worse outcome — so the failure is logged and the memory
    // deletion below still runs.
    if let Err(error) = crate::attachment_store::remove_conversation(&conversation_id).await {
        tracing::warn!(
            conversation = %conversation_id,
            %error,
            "could not remove stored attachments; conversation delete continues"
        );
    }
    state::memory()?
        .delete_conversation_context(&conversation_id)
        .await
        .map_err(|error| AgentError::Config(error.to_string()))
}

pub async fn delete_message_context(message_ids: Vec<String>) -> Result<usize, AgentError> {
    state::memory()?
        .delete_message_context(&message_ids)
        .await
        .map_err(|error| AgentError::Config(error.to_string()))
}

pub async fn promote_context(ids: Vec<String>) -> Result<usize, AgentError> {
    state::memory()?
        .promote_context(&ids)
        .await
        .map_err(|error| AgentError::Config(error.to_string()))
}

/// Ingest the demo corpus (C-111). Idempotent — the notes carry stable ids, so the
/// `load_seeds` boot step can call this on every start without duplicating.
/// Returns the number of notes seeded.
pub async fn seed_demo_corpus() -> Result<usize, AgentError> {
    let store = state::memory()?;
    gen_ui_db_graph::seed_corpus(store)
        .await
        .map_err(|e| AgentError::Config(e.to_string()))
}

/// Expand the entity graph outward from `entity_id` up to `depth` RELATE hops.
pub async fn graph_expand(entity_id: String, depth: u32) -> Result<Vec<RelatedEntity>, AgentError> {
    let store = state::memory()?;
    let depth = depth.clamp(1, u8::MAX as u32) as u8;
    store
        .graph_expand(&entity_id, depth)
        .await
        .map_err(|e| AgentError::Config(e.to_string()))
}
