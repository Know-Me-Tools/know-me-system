// TJ-ARCH-MOB-001 compliant
//! Reciprocal-Rank Fusion in Rust — used for the graph-expansion lane, which
//! cannot be expressed as one SurrealQL statement (the DB's `search::rrf` fuses
//! the vector+BM25 lanes; the RELATE-neighbour lane is fused here).
//!
//! RRF score for an item = Σ over lists of `1 / (k + rank)`, rank 0-based. `k`
//! damps the contribution of low-ranked items; 60 is the canonical default.

/// Fusion tuning. `k` is the RRF smoothing constant; `limit` caps the output.
#[derive(Debug, Clone, Copy)]
pub struct RrfConfig {
    pub k: f32,
    pub limit: usize,
}

impl Default for RrfConfig {
    fn default() -> Self {
        Self { k: 60.0, limit: 20 }
    }
}

/// Fuse several ranked lists of ids into one, highest RRF score first.
/// Each inner slice is one lane already in rank order (best first). Ids may repeat
/// across lanes; their contributions sum. Ties break on id for determinism (so
/// snapshot tests are stable).
///
/// Only for lanes that can AGREE, where intra-lane position is meaningful — a
/// search lane's rank 0 really is its best hit, and summing contributions is what
/// makes an item ranked by several lanes outrank one favoured by a single lane.
///
/// Do NOT use it for lanes that are disjoint and ordered, where lane membership
/// alone is the contract: it cannot express "lane 0 always beats lane 1", since
/// rank 0 of lane 1 (1/60) outscores rank 1 of lane 0 (1/61). `graph_expand` hit
/// exactly that and now scores by hop distance instead — see the comment there.
pub fn rrf_fuse(lanes: &[Vec<String>], cfg: RrfConfig) -> Vec<(String, f32)> {
    use std::collections::HashMap;
    let mut scores: HashMap<&str, f32> = HashMap::new();
    for lane in lanes {
        for (rank, id) in lane.iter().enumerate() {
            *scores.entry(id.as_str()).or_insert(0.0) += 1.0 / (cfg.k + rank as f32);
        }
    }
    let mut fused: Vec<(String, f32)> = scores
        .into_iter()
        .map(|(id, s)| (id.to_string(), s))
        .collect();
    fused.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.0.cmp(&b.0))
    });
    fused.truncate(cfg.limit);
    fused
}
