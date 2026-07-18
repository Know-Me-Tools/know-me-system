// TJ-ARCH-MOB-001 compliant
//! C-105 T12 live smoke test: proves `gen_ui_agent::chat::send` drives a REAL
//! on-device model end-to-end through the local lane — active_lane resolution
//! from the config DB -> InferenceProvider::load (downloading the GGUF from HF
//! on first run) -> generate -> streamed A2uiEvents. This is the same code path
//! tauri-plugin-gen-ui calls, exercised through the public API rather than by
//! poking the engine directly.
//!
//! Ignored by default: the first run downloads ~1GB from HuggingFace and
//! generation needs a Metal-capable machine. It is the only honest way to
//! verify the lane — compile-checks prove nothing about a model actually
//! loading, and the mistral.rs fork's own crates failed to build together at
//! the pinned SHA until a candle patch (see the C-105 design doc), which no
//! amount of type-checking would have caught.
//!
//! Run with:
//!   cargo test -p gen_ui_agent --test local_inference_live \
//!     --features test-local-mistral -- --ignored --nocapture
#![cfg(feature = "test-local-mistral")]

use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;
use gen_ui_agent::{state, ConfigBackend};
use gen_ui_db::relational::{ConfigStore, ModelPref, Provider, RelationalError, RelationalResult};
use gen_ui_db_graph::{
    Embedder, EmbeddingModelInfo, GraphError, GraphStore, GraphStoreConfig, EMBED_DIM,
};
use gen_ui_types::events::A2uiEvent;

/// Catalog id resolved by gen_ui_inference::mistral (Qwen2.5-1.5B, Q4_0 — NOT a
/// K-quant; see that module's note on why K-quants are broken for Qwen on the
/// pinned fork).
const LOCAL_MODEL: &str = "qwen2.5-1.5b-instruct-q4";

/// Memory/graph-RAG is never exercised here, so a real fastembed model would be
/// pure overhead. Same deterministic fake the ollama_live test uses.
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

/// Config store pinned to the local lane: `active_lane` = "local", and a
/// chat/local model_pref naming the catalog model. No provider — the local lane
/// has none, which is exactly the shape being tested.
struct LocalLaneConfigStore {
    model: String,
}

#[async_trait]
impl ConfigStore for LocalLaneConfigStore {
    async fn list_providers(&self) -> RelationalResult<Vec<Provider>> {
        Ok(vec![])
    }
    async fn upsert_provider(&self, _: &Provider) -> RelationalResult<()> {
        Err(RelationalError::Sync("read-only test store".into()))
    }
    async fn delete_provider(&self, _: &str) -> RelationalResult<()> {
        Err(RelationalError::Sync("read-only test store".into()))
    }
    async fn get_model_pref(
        &self,
        surface: &str,
        lane: &str,
    ) -> RelationalResult<Option<ModelPref>> {
        if surface != "chat" || lane != "local" {
            return Ok(None);
        }
        Ok(Some(ModelPref {
            surface: surface.into(),
            lane: lane.into(),
            provider_id: None,
            model_id: self.model.clone(),
            // Keep the run short and deterministic-ish: this asserts the pipe
            // works, not that the model is clever.
            params: serde_json::json!({ "temperature": 0.7, "max_tokens": 24 }),
        }))
    }
    async fn upsert_model_pref(&self, _: &ModelPref) -> RelationalResult<()> {
        Err(RelationalError::Sync("read-only test store".into()))
    }
    async fn get_setting(&self, key: &str) -> RelationalResult<Option<serde_json::Value>> {
        Ok(match key {
            "active_lane" => Some(serde_json::Value::String("local".into())),
            _ => None,
        })
    }
    async fn set_setting(&self, _: &str, _: serde_json::Value) -> RelationalResult<()> {
        Err(RelationalError::Sync("read-only test store".into()))
    }
}

async fn init_local_lane() {
    gen_ui_runtime::init(None);
    let memory = Arc::new(
        GraphStore::open(GraphStoreConfig {
            endpoint: "memory".to_string(),
            namespace: "test".to_string(),
            database: "local_inference".to_string(),
            embedder: Arc::new(ZeroEmbedder),
        })
        .await
        .expect("ephemeral in-memory GraphStore should open"),
    );
    let model = std::env::var("LOCAL_MODEL").unwrap_or_else(|_| LOCAL_MODEL.into());
    state::init_with_inference(
        ConfigBackend::Postgres(Arc::new(LocalLaneConfigStore { model })),
        memory,
        Some(Arc::new(gen_ui_inference::MistralEngine::new())),
    );
}

#[tokio::test]
#[ignore]
async fn chat_send_streams_a_real_local_response() {
    init_local_lane().await;

    let mut rx = state::subscribe().expect("subscribe after init");
    let started = Instant::now();

    // send() loads the model before returning, so this call absorbs the
    // (potentially minutes-long) first-run download by design.
    let run_id = gen_ui_agent::chat::send("Say hello in exactly one word.".into(), Vec::new())
        .await
        .expect("chat::send should resolve the local lane and start a run");
    eprintln!("model ready + run started in {:?}", started.elapsed());

    let mut text = String::new();
    let mut saw_started = false;
    let generation_started = Instant::now();

    loop {
        let event = tokio::time::timeout(std::time::Duration::from_secs(120), rx.recv())
            .await
            .expect("local generation should produce an event within 120s")
            .expect("event channel should stay open");

        match event {
            A2uiEvent::RunStarted { run_id: id } => {
                assert_eq!(id, run_id, "events must be scoped to this run");
                saw_started = true;
            }
            A2uiEvent::Block { block } => {
                if let gen_ui_types::ContentBlock::Text { text: delta } = block {
                    text.push_str(&delta);
                }
            }
            A2uiEvent::RunFinished { run_id: id } => {
                assert_eq!(id, run_id);
                break;
            }
            A2uiEvent::RunError { message } => panic!("local generation failed: {message}"),
        }
    }

    assert!(
        saw_started,
        "the run must announce RunStarted before its blocks"
    );
    assert!(
        !text.trim().is_empty(),
        "the model must produce actual text"
    );
    eprintln!(
        "local response in {:?}: {:?}",
        generation_started.elapsed(),
        text.trim()
    );

    // A non-empty assertion is NOT enough — that is exactly what let a broken
    // quantization pass unnoticed: Qwen2.5 Q4_K_M streamed a perfectly
    // well-formed run whose text was a repetition loop echoing the prompt back
    // ("Say hello in exactly word `` `` ``…"). Assert the output is coherent, not
    // merely present.
    let answer = text.trim();

    assert!(
        !answer.starts_with("Say hello in exactly"),
        "model echoed the prompt instead of answering it — the chat template is \
         not being applied, or the quantization is broken: {answer:?}"
    );

    // Degenerate loops repeat one short token forever. A real answer to "say
    // hello in one word" is a handful of distinct words at most.
    let words: Vec<&str> = answer.split_whitespace().collect();
    if words.len() > 3 {
        let unique: std::collections::HashSet<&&str> = words.iter().collect();
        let repetition_ratio = unique.len() as f32 / words.len() as f32;
        assert!(
            repetition_ratio > 0.4,
            "output looks like a degenerate repetition loop \
             ({}/{} unique words): {answer:?}",
            unique.len(),
            words.len()
        );
    }

    assert!(
        answer.to_lowercase().contains("hello") || answer.to_lowercase().contains("hi"),
        "model did not actually greet — expected a hello/hi somewhere in {answer:?}"
    );
}
