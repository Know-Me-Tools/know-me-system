// TJ-ARCH-MOB-001 compliant
//! gen_ui_db_graph (L2) — SurrealDB 3.2 embedded hybrid graph-RAG store.
//!
//! Owns the knowledge-graph half of the data layer (relational + sync live in
//! `gen_ui_db`). SurrealDB is isolated here on purpose: surrealdb-core's build.rs
//! re-runs on any downstream change (surrealdb#6954), so keeping it in its own
//! crate keeps the rest of the workspace off that slow recompile path.
//!
//! Engines: `kv-rocksdb` on native (persistent, incl. iOS/Android), `kv-indxdb`
//! on wasm32 (the only embedded KV that links in the browser).
//!
//! The public surface is INTENT-LEVEL, never raw SurrealQL — there is no official
//! Dart SurrealDB SDK, so `gen_ui_ffi` re-exports these functions and Dart calls
//! `memory_search` / `graph_expand` / `memory_ingest`, not queries. Keeping SurrealQL
//! private also means the schema can change without breaking the FFI contract.
//!
//! Hybrid retrieval pipeline (`memory_search`):
//!   1. HNSW vector recall  — semantic nearest neighbours (384-dim embeddings)
//!   2. BM25 full-text lane — lexical matches the vector lane misses
//!   3. `search::rrf()`     — reciprocal-rank fusion of (1) and (2) IN the DB
//!   4. graph expansion     — RELATE-edge neighbours of the fused hits, re-fused
//!      in Rust (`rrf`) because it is not one SurrealQL statement
#![forbid(unsafe_code)]

mod config;
mod corpus;
mod embed;
mod error;
mod rrf;
mod schema;
mod store;

pub use config::{ModelPref, Provider};
pub use corpus::{corpus_len, seed_corpus};
pub use embed::{Embedder, EmbeddingModelInfo, EMBED_DIM};
pub use error::GraphError;
pub use rrf::{rrf_fuse, RrfConfig};
pub use store::{GraphStore, GraphStoreConfig, MemoryHit, MemoryRecord, RelatedEntity, SearchMode};

#[cfg(feature = "embed-native")]
pub use embed::FastEmbedder;

/// Result alias for graph-store operations. `GraphError` maps cleanly into the
/// workspace-wide `gen_ui_types::CoreError` via `From`, so callers at the FFI
/// boundary propagate one error taxonomy.
pub type GraphResult<T> = Result<T, GraphError>;
