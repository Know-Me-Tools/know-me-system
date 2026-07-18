// TJ-ARCH-MOB-001 compliant
//! The demo corpus seeds and is searchable (C-111 T2).
//!
//! **Its own test binary, deliberately.** `GraphStore::open` is a process-wide
//! singleton, so every test in a binary shares one DB. Seeding ~20 corpus notes into
//! the shared `tests/it` DB swamped the other tests' small-`k` searches and turned a
//! green suite red — verified by stashing: without this test, `it` passes 5/5; with it,
//! runs failed 3-in-5 and the failure moved around depending on execution order.
//!
//! A separate binary gets its own process, its own singleton, and its own DB. That is
//! cheaper and more honest than teaching every other test to tolerate a corpus it did
//! not ask for. (`gen_ui_db_graph` is the one crate where an extra test binary is worth
//! its link cost — see the crate docs on surrealdb-core's rebuild behaviour.)
// Shared with `tests/it` rather than duplicated — one fake, one behaviour.
#[path = "it/fake_embedder.rs"]
mod fake_embedder;

use fake_embedder::HashEmbedder;
use gen_ui_db_graph::{GraphStore, GraphStoreConfig};
use std::sync::Arc;

/// Closes the gap between "seed_corpus compiles and its unit tests pass" and "a fresh
/// install's search actually returns something".
///
/// This change's history earns the paranoia: `memory_search` was declared verified while
/// failing at parse time on every call, because the verification ran an adjacent path.
///
/// Uses the HashEmbedder fake, so this proves the SEEDING and RETRIEVAL plumbing, not
/// semantic ranking quality — that needs real embeddings and a human reading results.
#[tokio::test]
async fn demo_corpus_seeds_and_is_searchable() {
    // The store embeds via gen_ui_runtime::spawn_blocking, which reads the crate's
    // global runtime OnceCell — #[tokio::test]'s own runtime is invisible to it.
    gen_ui_runtime::init(Some(2));

    let store = GraphStore::open(GraphStoreConfig {
        endpoint: "memory".into(),
        namespace: "test".into(),
        database: "corpus".into(),
        embedder: Arc::new(HashEmbedder),
    })
    .await
    .expect("store opens and applies schema");

    let n = gen_ui_db_graph::seed_corpus(&store)
        .await
        .expect("corpus seeds");
    assert_eq!(
        n,
        gen_ui_db_graph::corpus_len(),
        "seed_corpus should report every note"
    );
    assert!(n > 0, "corpus must not be empty");

    // Idempotent: stable ids are what let `load_seeds` run on every app start.
    let again = gen_ui_db_graph::seed_corpus(&store)
        .await
        .expect("re-seed is safe");
    assert_eq!(again, n, "re-seeding reports the same count");

    // "BossFang" is a coined product term appearing in exactly one seed — the kind of
    // rare token the BM25 lane exists to catch, and one no embedding would place well.
    let hits = store
        .memory_search("BossFang", 10)
        .await
        .expect("search the seeded corpus");
    assert!(
        hits.iter().any(|h| h.text.contains("BossFang")),
        "the BossFang seed should be retrievable for its own rare term; got {} hits: {:?}",
        hits.len(),
        hits.iter().map(|h| &h.text).collect::<Vec<_>>()
    );
}
