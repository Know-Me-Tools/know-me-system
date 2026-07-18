// TJ-ARCH-MOB-001 compliant
//! On-device text embedding behind a trait, so the store depends on the
//! *capability* not on fastembed. Native leaves enable `embed-native` for the
//! real ONNX model; wasm hosts inject a JS-backed embedder; tests inject a
//! deterministic fake and never touch the network.
use crate::error::GraphError;

/// Embedding width. 384 = all-MiniLM-L6-v2 / bge-small class. Standardised across
/// every engine (SQLite-vec, pgvector, SurrealDB HNSW) so vectors replicate cleanly
/// — the HNSW index in `schema.rs` is defined `DIMENSION 384` to match.
pub const EMBED_DIM: usize = 384;

/// Model provenance, surfaced so a store can refuse to mix embeddings from
/// different models in one index (silent dimension/space drift is a classic RAG bug).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmbeddingModelInfo {
    pub name: String,
    pub dim: usize,
}

/// The embedding capability the store needs. `embed` is batch-oriented because
/// fastembed and every ONNX backend amortise best over a batch; a single string
/// is just a one-element batch.
///
/// Implementors run CPU-bound inference — callers on the async path must invoke
/// them inside `gen_ui_runtime::spawn_blocking` (see `store::GraphStore::embed_blocking`).
pub trait Embedder: Send + Sync {
    fn model_info(&self) -> EmbeddingModelInfo;
    fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, GraphError>;
}

#[cfg(feature = "embed-native")]
mod native {
    use super::*;
    use fastembed::{EmbeddingModel, TextEmbedding, TextInitOptions};
    use std::sync::Mutex;

    /// fastembed-backed embedder (all-MiniLM-L6-v2, 384-dim). Downloads the ONNX
    /// model to an explicit application-data cache on first use, then runs fully offline.
    /// `TextEmbedding` is not `Sync`, so it sits behind a `Mutex`; embedding is
    /// short and always called off the async runtime via `spawn_blocking`.
    pub struct FastEmbedder {
        inner: Mutex<TextEmbedding>,
    }

    impl FastEmbedder {
        /// Load the default 384-dim model. Blocking (may download) — construct at
        /// startup off the async runtime.
        pub fn new(cache_dir: impl AsRef<std::path::Path>) -> Result<Self, GraphError> {
            let model = TextEmbedding::try_new(
                TextInitOptions::new(EmbeddingModel::AllMiniLML6V2)
                    .with_cache_dir(cache_dir.as_ref().to_path_buf()),
            )
            .map_err(|e| GraphError::Embedding(e.to_string()))?;
            Ok(Self {
                inner: Mutex::new(model),
            })
        }
    }

    impl Embedder for FastEmbedder {
        fn model_info(&self) -> EmbeddingModelInfo {
            EmbeddingModelInfo {
                name: "all-MiniLM-L6-v2".into(),
                dim: EMBED_DIM,
            }
        }

        fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, GraphError> {
            let mut model = self
                .inner
                .lock()
                .map_err(|_| GraphError::Embedding("embedder mutex poisoned".into()))?;
            let out = model
                .embed(texts, None)
                .map_err(|e| GraphError::Embedding(e.to_string()))?;
            for v in &out {
                if v.len() != EMBED_DIM {
                    return Err(GraphError::Embedding(format!(
                        "model returned dim {}, expected {EMBED_DIM}",
                        v.len()
                    )));
                }
            }
            Ok(out)
        }
    }
}

#[cfg(feature = "embed-native")]
pub use native::FastEmbedder;
