// TJ-ARCH-MOB-001 compliant
//! T10 live smoke test: proves gen_ui_agent::chat::send drives a real
//! liter-llm chat completion against a local Ollama instance end-to-end
//! (provider/model resolution from the config DB -> liter-llm ClientBuilder
//! -> streamed A2uiEvents) — the same code path both gen_ui_ffi (mobile) and
//! tauri-plugin-gen-ui (desktop) call. Ignored by default — requires Ollama
//! running locally with the model pulled, and writes a real (empty-string)
//! keychain entry for the test provider's api_key_ref, since
//! gen_ui_agent::secrets::resolve_api_key always reads the OS keychain
//! (there is no injectable SecretResolver in this design).
//!
//! Run with: OLLAMA_MODEL=llama3.2:1b cargo test -p gen_ui_agent --test
//! ollama_live -- --ignored --nocapture
use std::sync::Arc;

use async_trait::async_trait;
use gen_ui_agent::{state, ConfigBackend};
use gen_ui_db::relational::{ConfigStore, ModelPref, Provider, RelationalError, RelationalResult};
use gen_ui_db_graph::{
    Embedder, EmbeddingModelInfo, GraphError, GraphStore, GraphStoreConfig, EMBED_DIM,
};
use gen_ui_types::events::A2uiEvent;

const PROVIDER_ID: &str = "test-ollama";
const API_KEY_REF: &str = "test-ollama-noop";
const SERVICE: &str = "ai.prometheusags.knowme-poc";

/// This test never exercises memory/graph-RAG — a real fastembed model would
/// be pure overhead. A deterministic zero-vector fake keeps state::init's
/// required memory store cheap and dependency-free.
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

struct OllamaConfigStore {
    model_hint: String,
}

#[async_trait]
impl ConfigStore for OllamaConfigStore {
    async fn list_providers(&self) -> RelationalResult<Vec<Provider>> {
        Ok(vec![Provider {
            id: PROVIDER_ID.into(),
            kind: format!("ollama/{}", self.model_hint),
            base_url: std::env::var("OLLAMA_BASE_URL").ok(),
            api_key_ref: Some(API_KEY_REF.into()),
            enabled: true,
        }])
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
        if surface != "chat" || lane != "cloud" {
            return Ok(None);
        }
        Ok(Some(ModelPref {
            surface: surface.into(),
            lane: lane.into(),
            provider_id: Some(PROVIDER_ID.into()),
            model_id: format!("ollama/{}", self.model_hint),
            params: serde_json::Value::Null,
        }))
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

/// gen_ui_agent::secrets always reads the real OS keychain — write the
/// (empty, matching liter-llm's ollama/no-key convention) entry the test
/// provider references so `chat::send`'s real resolve_api_key call succeeds.
fn seed_test_keychain_entry() {
    let entry = keyring::Entry::new(SERVICE, API_KEY_REF).expect("keychain entry");
    entry
        .set_password("")
        .expect("seed empty api key for ollama test provider");
}

#[tokio::test]
#[ignore]
async fn chat_send_streams_a_real_ollama_response() {
    gen_ui_runtime::init(None);
    seed_test_keychain_entry();

    let model_hint = std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "llama3.2:1b".into());
    let memory = Arc::new(
        GraphStore::open(GraphStoreConfig {
            endpoint: "memory".to_string(),
            namespace: "test".to_string(),
            database: "ollama_live".to_string(),
            embedder: Arc::new(ZeroEmbedder),
        })
        .await
        .expect("ephemeral in-memory GraphStore should open"),
    );
    state::init(
        ConfigBackend::Postgres(Arc::new(OllamaConfigStore { model_hint })),
        memory,
    );

    let run_id = gen_ui_agent::chat::send("Say hello in exactly one word.".into(), Vec::new())
        .await
        .expect("chat::send should succeed against a running Ollama instance");

    let mut rx = state::subscribe().expect("agent state should be initialised");

    let mut saw_run_started = false;
    let mut saw_text = false;
    let mut saw_terminal = false;
    let mut collected_text = String::new();

    while let Ok(Ok(event)) =
        tokio::time::timeout(std::time::Duration::from_secs(30), rx.recv()).await
    {
        let is_this_run = match &event {
            A2uiEvent::RunStarted { run_id: id, .. }
            | A2uiEvent::RunFinished { run_id: id, .. } => *id == run_id,
            // Block/RunError carry no run_id in this PoC's single-turn-at-a-time
            // design (see gen_ui_ffi::api::streams's doc comment) — accept them
            // since no other run can be concurrently in flight in this test.
            _ => true,
        };
        if !is_this_run {
            continue;
        }
        match event {
            A2uiEvent::RunStarted { .. } => saw_run_started = true,
            A2uiEvent::Block { block } => {
                if let gen_ui_types::content_block::ContentBlock::Text { text } = block {
                    saw_text = true;
                    collected_text.push_str(&text);
                }
            }
            A2uiEvent::RunFinished { .. } => {
                saw_terminal = true;
                break;
            }
            A2uiEvent::RunError { message } => {
                panic!("chat run errored: {message}");
            }
        }
    }

    assert!(saw_run_started, "expected a RunStarted event");
    assert!(saw_text, "expected at least one text Block event");
    assert!(saw_terminal, "expected a terminal RunFinished event");
    assert!(
        !collected_text.trim().is_empty(),
        "collected response text should be non-empty"
    );
    println!("Ollama response: {collected_text:?}");
}
