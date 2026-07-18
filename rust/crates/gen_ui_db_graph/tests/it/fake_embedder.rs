// TJ-ARCH-MOB-001 compliant
//! Deterministic, network-free embedder for boundary tests. Hashes each token into
//! a 384-dim bag-of-words vector and L2-normalises it, so texts that share words
//! land near each other under cosine distance — enough to exercise the HNSW lane
//! without an ONNX model.
use gen_ui_db_graph::{Embedder, EmbeddingModelInfo, EMBED_DIM};

pub struct HashEmbedder;

impl Embedder for HashEmbedder {
    fn model_info(&self) -> EmbeddingModelInfo {
        EmbeddingModelInfo {
            name: "test-hash".into(),
            dim: EMBED_DIM,
        }
    }

    fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, gen_ui_db_graph::GraphError> {
        Ok(texts.iter().map(|t| embed_one(t)).collect())
    }
}

fn embed_one(text: &str) -> Vec<f32> {
    let mut v = vec![0.0f32; EMBED_DIM];
    for token in text.to_lowercase().split_whitespace() {
        let mut h: u64 = 1469598103934665603; // FNV-1a offset basis
        for b in token.bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(1099511628211);
        }
        v[(h as usize) % EMBED_DIM] += 1.0;
    }
    let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for x in &mut v {
            *x /= norm;
        }
    }
    v
}
