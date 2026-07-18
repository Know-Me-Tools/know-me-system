// TJ-ARCH-MOB-001 compliant
//! Boundary tests for gen_ui_db_graph — exercised at the public intent API over a
//! real in-memory SurrealDB (`memory` engine) with a deterministic fake embedder,
//! so no ONNX download and no flakiness. Per CLAUDE.md: 3–5 behaviour tests at the
//! API surface, no internal mocks, features-first.
mod fake_embedder;

use fake_embedder::HashEmbedder;
use gen_ui_db_graph::{GraphStore, GraphStoreConfig, MemoryRecord};
use std::sync::Arc;

/// Run a test body on the process-global runtime, NOT a per-test one.
///
/// Two process-wide singletons force this, and `#[tokio::test]` breaks both:
///   1. `gen_ui_runtime`'s global `RT` — `spawn_blocking` (the embedder path) reads it,
///      and a `#[tokio::test]` runtime is invisible to it ("runtime not initialised").
///   2. `GraphStore`'s global `GRAPH_STORE` — SurrealDB binds its router task to
///      whichever runtime called `connect()`. Under `#[tokio::test]` that is a
///      throwaway runtime: the FIRST test to open binds the router, and when that
///      test's runtime drops the router dies — while the singleton keeps handing the
///      dead handle to every later test. They then fail with
///      `Failed to send command: sending into a closed channel`, which reads like a
///      broken query but is really a dead router. It is order/timing dependent, so it
///      can pass once and fail the next run.
///
/// `gen_ui_runtime`'s `RT` is a `static OnceCell<Runtime>` that lives for the whole
/// process, so a router bound to it outlives every test. `init` is idempotent.
fn run_test<F: std::future::Future<Output = ()>>(body: F) {
    gen_ui_runtime::init(Some(2));
    gen_ui_runtime::handle().block_on(body);
}

async fn open_store() -> GraphStore {
    GraphStore::open(GraphStoreConfig {
        endpoint: "memory".into(),
        namespace: "test".into(),
        database: "graph".into(),
        embedder: Arc::new(HashEmbedder),
    })
    .await
    .expect("store opens and applies schema")
}

/// Ingest → hybrid search returns the ingested memory ranked first for its own text.
#[test]
fn ingest_then_search_finds_the_memory() {
    run_test(async {
        let store = open_store().await;
        let id = store
            .memory_ingest(MemoryRecord {
                id: None,
                text: "the octopus is a highly intelligent cephalopod".into(),
                kind: "note".into(),
                entity: None,
            })
            .await
            .expect("ingest succeeds");
        assert!(!id.is_empty());

        let hits = store
            .memory_search("octopus intelligent cephalopod", 5)
            .await
            .expect("search succeeds");
        assert!(!hits.is_empty(), "expected at least one hit");
        assert!(
            hits[0].text.contains("octopus"),
            "top hit should be the ingested memory, got {:?}",
            hits[0].text
        );
    });
}

/// BM25 lexical lane finds an exact-term match even when semantics are thin.
#[test]
fn lexical_lane_matches_rare_term() {
    run_test(async {
        let store = open_store().await;
        for text in [
            "quarterly revenue grew in the fiscal report",
            "the platypus lays eggs despite being a mammal",
        ] {
            store
                .memory_ingest(MemoryRecord {
                    id: None,
                    text: text.into(),
                    kind: "note".into(),
                    entity: None,
                })
                .await
                .expect("ingest");
        }
        let hits = store.memory_search("platypus", 5).await.expect("search");
        assert!(
            hits.iter().any(|h| h.text.contains("platypus")),
            "BM25 lane should surface the platypus memory"
        );
    });
}

/// Vector-only mode returns hits and is scored "higher = better", like hybrid.
///
/// This asserts the lane WORKS, not that it is worse — the fake HashEmbedder has no
/// real semantics, so a meaningful hybrid-beats-vector comparison isn't possible here.
/// That difference is what the UI toggle exists to show against real embeddings; a test
/// asserting it with a hash fake would be asserting the fake's arbitrary behaviour.
#[test]
fn vector_mode_returns_scored_hits() {
    run_test(async {
        use gen_ui_db_graph::SearchMode;
        let store = open_store().await;
        for text in [
            "the axolotl regenerates entire limbs",
            "quarterly revenue grew in the fiscal report",
        ] {
            store
                .memory_ingest(MemoryRecord {
                    id: None,
                    text: text.into(),
                    kind: "note".into(),
                    entity: None,
                })
                .await
                .expect("ingest");
        }

        let hits = store
            .memory_search_with("axolotl regenerates limbs", 5, SearchMode::Vector)
            .await
            .expect("vector search succeeds");

        assert!(!hits.is_empty(), "vector lane should return hits");
        // Scores are 1/(1+distance), so bounded (0,1] and descending.
        for h in &hits {
            assert!(
                h.score > 0.0 && h.score <= 1.0,
                "score out of range: {}",
                h.score
            );
        }
        assert!(
            hits.windows(2).all(|w| w[0].score >= w[1].score),
            "vector hits must be ranked best-first: {:?}",
            hits.iter().map(|h| h.score).collect::<Vec<_>>()
        );
    });
}

/// Empty inputs are rejected at the boundary (terminal, not a silent empty result).
#[test]
fn empty_inputs_are_rejected() {
    run_test(async {
        let store = open_store().await;
        assert!(store
            .memory_ingest(MemoryRecord {
                id: None,
                text: "   ".into(),
                kind: "note".into(),
                entity: None,
            })
            .await
            .is_err());
        assert!(store.memory_search("", 5).await.is_err());
        // depth 0 is a caller error, not an empty traversal.
        assert!(store.graph_expand("entity:anything", 0).await.is_err());
    });
}

/// RELATE edges are traversed and fused by graph_expand, nearer hops ranking higher.
#[test]
fn graph_expand_traverses_relate_edges() {
    run_test(async {
        let store = open_store().await;
        // Seed a small graph: a -> b -> c, plus a -> d directly.
        for (id, label) in [
            ("a", "Alpha"),
            ("b", "Beta"),
            ("c", "Gamma"),
            ("d", "Delta"),
        ] {
            store
                .create_entity(id, "node", label)
                .await
                .expect("create entity");
        }
        for (from, to) in [("a", "b"), ("b", "c"), ("a", "d")] {
            store.relate(from, to, "related").await.expect("relate");
        }
        let related = store
            .graph_expand("a", 2)
            .await
            .expect("graph_expand succeeds");
        let ids: Vec<&str> = related.iter().map(|r| r.id.as_str()).collect();
        assert!(
            ids.contains(&"b"),
            "1-hop neighbour b should be present: {ids:?}"
        );
        assert!(
            ids.contains(&"d"),
            "1-hop neighbour d should be present: {ids:?}"
        );
        assert!(
            ids.contains(&"c"),
            "2-hop neighbour c should be present: {ids:?}"
        );
        // b (1 hop) must outrank c (2 hops) under RRF.
        let rank = |want: &str| related.iter().position(|r| r.id == want).unwrap();
        assert!(
            rank("b") < rank("c"),
            "nearer hop b should outrank farther hop c"
        );
    });
}

#[test]
fn relate_rejected_by_db_surfaces_error() {
    run_test(async {
        let store = open_store().await;
        store
            .create_entity("real", "node", "Real")
            .await
            .expect("create entity");
        store
            .upsert_provider(&gen_ui_db_graph::Provider {
                id: "not-an-entity".into(),
                kind: "openai".into(),
                base_url: None,
                api_key_ref: None,
                enabled: true,
            })
            .await
            .expect("provider upsert");

        let error = store
            .relate_raw_for_test("provider", "not-an-entity", "real")
            .await
            .expect_err("relation type violation must surface");
        assert!(error.to_string().contains("relate"));
    });
}

#[test]
fn relate_binds_endpoint_ids_not_literal_param_names() {
    run_test(async {
        let store = open_store().await;
        for (id, label) in [("src", "Source"), ("dst", "Destination")] {
            store
                .create_entity(id, "node", label)
                .await
                .expect("create entity");
        }
        store.relate("src", "dst", "related").await.expect("relate");

        let edges = store.edge_endpoints_for_test().await.expect("read edges");
        assert!(edges.contains(&("src".to_string(), "dst".to_string())));
        assert!(!edges
            .iter()
            .any(|(from, to)| from.starts_with('$') || to.starts_with('$')));
    });
}
