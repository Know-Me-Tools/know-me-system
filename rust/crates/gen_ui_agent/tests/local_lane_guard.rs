// TJ-ARCH-MOB-001 compliant
//! C-105 T12: the lane toggle must reject `local` when this build has no engine,
//! rather than let the next turn silently answer from the cloud — a user who
//! asked for on-device inference must never be quietly served by a network
//! provider. This is the mobile/no-engine shape (`state::init`, not
//! `init_with_inference`).
//!
//! Its own test binary, deliberately: `state::init` is a process-wide OnceCell
//! (calling it twice is documented as a programming error), so a no-engine test
//! cannot share a process with `local_inference_live`'s with-engine one —
//! whichever ran second would inherit the first's state. One live test per
//! binary is the convention `ollama_live.rs` already follows.
//!
//! Cheap — no model, no download. Ignored only because it needs the engine
//! feature compiled in to be a meaningful negative.
//!
//! Run with:
//!   cargo test -p gen_ui_agent --test local_lane_guard \
//!     --features test-local-mistral -- --ignored --nocapture
#![cfg(feature = "test-local-mistral")]

use std::sync::Arc;

use async_trait::async_trait;
use gen_ui_agent::{state, ConfigBackend};
use gen_ui_db::relational::{ConfigStore, ModelPref, Provider, RelationalError, RelationalResult};
use gen_ui_db_graph::{
    Embedder, EmbeddingModelInfo, GraphError, GraphStore, GraphStoreConfig, EMBED_DIM,
};

struct ZeroEmbedder;
impl Embedder for ZeroEmbedder {
    fn model_info(&self) -> EmbeddingModelInfo {
        EmbeddingModelInfo {
            name: "zero-fake".into(),
            dim: EMBED_DIM,
        }
    }
    fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, GraphError> {
        Ok(texts.iter().map(|_| vec![0.0; EMBED_DIM]).collect())
    }
}

/// Empty store: no providers, no prefs, no settings. Nothing here should be
/// reached — the guard must fire before any config lookup matters.
struct EmptyConfigStore;

#[async_trait]
impl ConfigStore for EmptyConfigStore {
    async fn list_providers(&self) -> RelationalResult<Vec<Provider>> {
        Ok(vec![])
    }
    async fn upsert_provider(&self, _: &Provider) -> RelationalResult<()> {
        Err(RelationalError::Sync("read-only test store".into()))
    }
    async fn delete_provider(&self, _: &str) -> RelationalResult<()> {
        Err(RelationalError::Sync("read-only test store".into()))
    }
    async fn get_model_pref(&self, _: &str, _: &str) -> RelationalResult<Option<ModelPref>> {
        Ok(None)
    }
    async fn upsert_model_pref(&self, _: &ModelPref) -> RelationalResult<()> {
        Err(RelationalError::Sync("read-only test store".into()))
    }
    async fn get_setting(&self, _: &str) -> RelationalResult<Option<serde_json::Value>> {
        Ok(None)
    }
    async fn set_setting(&self, _: &str, _: serde_json::Value) -> RelationalResult<()> {
        Err(RelationalError::Sync("read-only test store".into()))
    }
}

#[tokio::test]
#[ignore]
async fn set_active_lane_rejects_local_without_an_engine() {
    gen_ui_runtime::init(None);
    let memory = Arc::new(
        GraphStore::open(GraphStoreConfig {
            endpoint: "memory".to_string(),
            namespace: "test".to_string(),
            database: "no_engine".to_string(),
            embedder: Arc::new(ZeroEmbedder),
        })
        .await
        .expect("ephemeral in-memory GraphStore should open"),
    );

    // init, NOT init_with_inference — the shape mobile and any engine-less build
    // get.
    state::init(ConfigBackend::Postgres(Arc::new(EmptyConfigStore)), memory);

    assert!(
        !gen_ui_agent::chat::has_local_engine(),
        "a build initialised without an engine must not claim to have one"
    );

    let err = gen_ui_agent::chat::set_active_lane("local")
        .await
        .expect_err("selecting local without an engine must fail loudly, not fall back to cloud");
    assert!(
        err.to_string().contains("no local inference engine"),
        "unexpected error: {err}"
    );

    // The lane must be unchanged — a rejected switch must not half-apply.
    assert_eq!(
        gen_ui_agent::chat::active_lane()
            .await
            .expect("active_lane"),
        "cloud",
        "a rejected switch must leave the lane where it was"
    );

    // Cloud must still be selectable (the guard is specific to `local`), and an
    // unknown lane must be rejected rather than persisted.
    gen_ui_agent::chat::set_active_lane("cloud")
        .await
        .expect_err("this read-only test store rejects writes, proving the attempt got that far");
    let err = gen_ui_agent::chat::set_active_lane("banana")
        .await
        .expect_err("an unknown lane must be rejected");
    assert!(
        err.to_string().contains("unknown lane"),
        "unexpected error: {err}"
    );
}
