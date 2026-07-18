// TJ-ARCH-MOB-001 compliant
//! Dev-only smoke-test wiring: an in-memory `ConfigStore` pointing at a local
//! Ollama instance, gated behind the `GEN_UI_DEV_OLLAMA_MODEL` env var so it
//! never activates unless explicitly opted into. This is NOT the real desktop
//! ConfigStore (that's pglite-oxide via `run_migrations`, see commands.rs) —
//! it exists solely to let `pnpm tauri dev` boot straight into a working
//! local, keyless provider round-trip without the real config-DB/keychain
//! admin UI (C-109) existing yet. When active, it replaces run_migrations's
//! normal `gen_ui_agent::state::init` call entirely (see setup() in lib.rs) —
//! delete this module once C-109's admin UI makes it redundant.
//!
//! liter-llm's Ollama support takes an `ollama/<model>` model-hint string and
//! an empty API key — gen_ui_agent::secrets::resolve_api_key always reads the
//! real OS keychain (no injectable resolver in this design), so this module
//! seeds a matching empty-string keychain entry for its own provider's
//! api_key_ref rather than bypassing that call.
use std::sync::Arc;

use async_trait::async_trait;
use gen_ui_agent::ConfigBackend;
use gen_ui_db::relational::{ConfigStore, ModelPref, Provider, RelationalError, RelationalResult};
use gen_ui_db_graph::{
    Embedder, EmbeddingModelInfo, GraphError, GraphStore, GraphStoreConfig, EMBED_DIM,
};

const DEV_PROVIDER_ID: &str = "dev-ollama";
const DEV_API_KEY_REF: &str = "dev-ollama-noop";
const KEYCHAIN_SERVICE: &str = "ai.prometheusags.knowme-poc";

#[derive(Debug)]
struct DevOllamaConfigStore {
    model_hint: String,
}

#[async_trait]
impl ConfigStore for DevOllamaConfigStore {
    async fn list_providers(&self) -> RelationalResult<Vec<Provider>> {
        Ok(vec![Provider {
            id: DEV_PROVIDER_ID.into(),
            kind: format!("ollama/{}", self.model_hint),
            base_url: std::env::var("OLLAMA_BASE_URL").ok(),
            api_key_ref: Some(DEV_API_KEY_REF.into()),
            enabled: true,
        }])
    }
    async fn upsert_provider(&self, _provider: &Provider) -> RelationalResult<()> {
        Err(RelationalError::Sync(
            "dev Ollama config store is read-only".into(),
        ))
    }
    async fn delete_provider(&self, _id: &str) -> RelationalResult<()> {
        Err(RelationalError::Sync(
            "dev Ollama config store is read-only".into(),
        ))
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
            provider_id: Some(DEV_PROVIDER_ID.into()),
            model_id: format!("ollama/{}", self.model_hint),
            params: serde_json::Value::Null,
        }))
    }
    async fn upsert_model_pref(&self, _pref: &ModelPref) -> RelationalResult<()> {
        Err(RelationalError::Sync(
            "dev Ollama config store is read-only".into(),
        ))
    }
    async fn get_setting(&self, _key: &str) -> RelationalResult<Option<serde_json::Value>> {
        Ok(None)
    }
    async fn set_setting(&self, _key: &str, _value: serde_json::Value) -> RelationalResult<()> {
        Err(RelationalError::Sync(
            "dev Ollama config store is read-only".into(),
        ))
    }
}

/// Dev shortcut's memory store needs no real embeddings — it exists purely to
/// let `state::init` (which now always requires a memory store, see
/// gen_ui_agent::state's doc comment) accept SOMETHING here without pulling in
/// a real fastembed model just to prove a chat round-trip.
struct ZeroEmbedder;
impl Embedder for ZeroEmbedder {
    fn model_info(&self) -> EmbeddingModelInfo {
        EmbeddingModelInfo {
            name: "dev-ollama-zero-fake".into(),
            dim: EMBED_DIM,
        }
    }
    fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, GraphError> {
        Ok(texts.iter().map(|_| vec![0.0; EMBED_DIM]).collect())
    }
}

/// Install the dev Ollama `ConfigStore` if `GEN_UI_DEV_OLLAMA_MODEL` is set
/// (e.g. `llama3.2:1b`). No-op otherwise — the process falls through to
/// `run_migrations`'s normal pglite-oxide-backed `state::init` as usual.
/// Returns whether it installed (callers use this to skip the real
/// `run_migrations` init when dev Ollama is active).
///
/// Called from `.setup()`, which is sync — `block_on` is acceptable here since
/// this is a rare, opt-in dev-only path, never the hot path.
pub fn install_if_requested() -> bool {
    let Ok(model_hint) = std::env::var("GEN_UI_DEV_OLLAMA_MODEL") else {
        return false;
    };
    log::info!("GEN_UI_DEV_OLLAMA_MODEL set — installing dev Ollama config (model: {model_hint})");

    if let Ok(entry) = keyring::Entry::new(KEYCHAIN_SERVICE, DEV_API_KEY_REF) {
        if let Err(e) = entry.set_password("") {
            log::warn!("failed to seed dev Ollama keychain entry: {e}");
        }
    }

    let memory = gen_ui_runtime::handle().block_on(async {
        GraphStore::open(GraphStoreConfig {
            endpoint: "memory".to_string(),
            namespace: "dev".to_string(),
            database: "ollama".to_string(),
            embedder: Arc::new(ZeroEmbedder),
        })
        .await
        .expect("ephemeral in-memory dev GraphStore should open")
    });

    gen_ui_agent::state::init(
        ConfigBackend::Postgres(Arc::new(DevOllamaConfigStore { model_hint })),
        Arc::new(memory),
    );
    true
}
