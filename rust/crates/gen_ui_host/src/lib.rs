// TJ-ARCH-MOB-001 compliant
//! Host-neutral application composition shared by Tauri and Axum.
//!
//! Transport leaves call this intent surface. They do not open databases,
//! select inference providers, or reproduce agent behavior themselves.
#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};
use std::sync::Arc;

use dashmap::DashMap;
use gen_ui_agent::ConfigBackend;
use gen_ui_db_graph::{FastEmbedder, GraphStore, GraphStoreConfig};
use gen_ui_types::events::A2uiEvent;
use gen_ui_types::inference::InferenceProvider;
use tokio::sync::{broadcast, Mutex, OnceCell};

pub use gen_ui_agent::control_plane::ChatControlPlaneCatalog;
pub use gen_ui_agent::provider_admin::{
    ConfiguredCloudModel, ConfiguredProvider, ProviderCatalogEntry, ProviderTestResult,
    SaveCloudModelRequest, SaveProviderRequest,
};

/// Process configuration for the shared application host.
#[derive(Debug, Clone)]
pub struct HostConfig {
    pub data_dir: PathBuf,
    pub namespace: String,
    pub database: String,
}

impl HostConfig {
    pub fn knowme(data_dir: impl Into<PathBuf>) -> Self {
        Self {
            data_dir: data_dir.into(),
            namespace: "knowme".to_string(),
            database: "poc".to_string(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum HostError {
    #[error("host initialization failed: {0}")]
    Initialization(String),
    #[error("chat failed: {0}")]
    Chat(String),
    #[error("unknown or already-consumed run: {0}")]
    UnknownRun(String),
}

type PendingEvents = Arc<Mutex<Option<broadcast::Receiver<A2uiEvent>>>>;

/// Transport-neutral, request-scoped hosted-web credentials. Secrets carried
/// here are never retained by `AppServices`.
pub struct EphemeralCloudConfig {
    pub provider: String,
    pub base_url: Option<String>,
    pub api_key: String,
    pub model: String,
}

/// Cloneable application service handle. Receivers are registered before a run
/// starts so an HTTP client cannot lose early events between POST and SSE attach.
#[derive(Clone)]
pub struct AppServices {
    data_dir: Arc<PathBuf>,
    pending_runs: Arc<DashMap<String, PendingEvents>>,
}

static SERVICES: OnceCell<AppServices> = OnceCell::const_new();

impl AppServices {
    /// Open stores, run migrations, initialize memory/RAG, and install the
    /// platform-selected inference provider exactly once per process.
    pub async fn bootstrap(
        config: HostConfig,
        inference: Option<Arc<dyn InferenceProvider>>,
    ) -> Result<Self, HostError> {
        SERVICES
            .get_or_try_init(|| async move { bootstrap_inner(config, inference).await })
            .await
            .cloned()
    }

    pub fn data_dir(&self) -> &Path {
        self.data_dir.as_path()
    }

    pub fn is_ready(&self) -> bool {
        gen_ui_agent::state::subscribe().is_ok()
    }

    pub async fn active_lane(&self) -> Result<String, HostError> {
        gen_ui_agent::chat::active_lane()
            .await
            .map_err(|error| HostError::Chat(error.to_string()))
    }

    pub async fn provider_catalog(&self) -> Result<Vec<ProviderCatalogEntry>, HostError> {
        gen_ui_agent::provider_admin::catalog()
            .await
            .map_err(|error| HostError::Initialization(error.to_string()))
    }

    pub async fn providers(&self) -> Result<Vec<ConfiguredProvider>, HostError> {
        gen_ui_agent::provider_admin::list()
            .await
            .map_err(|error| HostError::Initialization(error.to_string()))
    }

    pub async fn save_provider(&self, request: SaveProviderRequest) -> Result<(), HostError> {
        gen_ui_agent::provider_admin::save(request)
            .await
            .map_err(|error| HostError::Initialization(error.to_string()))
    }

    pub async fn delete_provider(&self, id: &str) -> Result<(), HostError> {
        gen_ui_agent::provider_admin::delete(id)
            .await
            .map_err(|error| HostError::Initialization(error.to_string()))
    }

    pub async fn test_provider(
        &self,
        id: &str,
        model: Option<&str>,
    ) -> Result<ProviderTestResult, HostError> {
        gen_ui_agent::provider_admin::test(id, model)
            .await
            .map_err(|error| HostError::Initialization(error.to_string()))
    }

    pub async fn cloud_model(&self) -> Result<Option<ConfiguredCloudModel>, HostError> {
        gen_ui_agent::provider_admin::get_cloud_model()
            .await
            .map_err(|error| HostError::Initialization(error.to_string()))
    }

    pub async fn save_cloud_model(&self, request: SaveCloudModelRequest) -> Result<(), HostError> {
        gen_ui_agent::provider_admin::save_cloud_model(request)
            .await
            .map_err(|error| HostError::Initialization(error.to_string()))
    }

    pub async fn chat_control_catalog(&self) -> Result<ChatControlPlaneCatalog, HostError> {
        gen_ui_agent::control_plane::catalog()
            .await
            .map_err(|error| HostError::Initialization(error.to_string()))
    }

    pub async fn conversation_policy(
        &self,
        conversation_id: &str,
    ) -> Result<serde_json::Value, HostError> {
        gen_ui_agent::control_plane::conversation_policy(conversation_id)
            .await
            .map_err(|error| HostError::Initialization(error.to_string()))
    }

    pub async fn save_conversation_policy(
        &self,
        conversation_id: &str,
        policy: &serde_json::Value,
    ) -> Result<serde_json::Value, HostError> {
        gen_ui_agent::control_plane::save_conversation_policy(conversation_id, policy)
            .await
            .map_err(|error| HostError::Initialization(error.to_string()))
    }

    pub async fn start_chat(
        &self,
        user_message: String,
        history: Vec<gen_ui_types::inference::InferenceMessage>,
        conversation_id: String,
        branch_id: String,
    ) -> Result<String, HostError> {
        let receiver =
            gen_ui_agent::state::subscribe().map_err(|error| HostError::Chat(error.to_string()))?;
        let run_id = gen_ui_agent::chat::send(user_message, history, conversation_id, branch_id)
            .await
            .map_err(|error| HostError::Chat(error.to_string()))?;
        self.pending_runs
            .insert(run_id.clone(), Arc::new(Mutex::new(Some(receiver))));
        Ok(run_id)
    }

    pub async fn start_chat_with_byok(
        &self,
        user_message: String,
        history: Vec<gen_ui_types::inference::InferenceMessage>,
        config: EphemeralCloudConfig,
        conversation_id: String,
        branch_id: String,
    ) -> Result<String, HostError> {
        let receiver =
            gen_ui_agent::state::subscribe().map_err(|error| HostError::Chat(error.to_string()))?;
        let run_id = gen_ui_agent::chat::send_cloud_ephemeral(
            user_message,
            history,
            gen_ui_agent::chat::EphemeralCloudConfig {
                provider: config.provider,
                base_url: config.base_url,
                api_key: config.api_key,
                model: config.model,
            },
            conversation_id,
            branch_id,
        )
        .map_err(|error| HostError::Chat(error.to_string()))?;
        self.pending_runs
            .insert(run_id.clone(), Arc::new(Mutex::new(Some(receiver))));
        Ok(run_id)
    }

    /// Claim a run's event receiver. A run has exactly one SSE consumer.
    pub async fn take_run_events(
        &self,
        run_id: &str,
    ) -> Result<broadcast::Receiver<A2uiEvent>, HostError> {
        let (_, slot) = self
            .pending_runs
            .remove(run_id)
            .ok_or_else(|| HostError::UnknownRun(run_id.to_string()))?;
        let receiver = slot
            .lock()
            .await
            .take()
            .ok_or_else(|| HostError::UnknownRun(run_id.to_string()))?;
        Ok(receiver)
    }
}

async fn bootstrap_inner(
    config: HostConfig,
    inference: Option<Arc<dyn InferenceProvider>>,
) -> Result<AppServices, HostError> {
    std::fs::create_dir_all(&config.data_dir)
        .map_err(|error| HostError::Initialization(error.to_string()))?;

    // Attachments live under the app's own data dir, never the OS cache: a
    // picked file's path is purgeable (Android clears it out from under us),
    // so a transcript that points at it renders now and breaks later.
    // Configured before any store opens, because the first chat_send after
    // bootstrap can already need it.
    gen_ui_agent::attachment_store::set_root(config.data_dir.join("attachments"));

    let config_store = gen_ui_db::relational::PgliteStore::open(config.data_dir.join("config-db"))
        .await
        .map_err(|error| HostError::Initialization(error.to_string()))?;
    config_store
        .migrate()
        .await
        .map_err(|error| HostError::Initialization(error.to_string()))?;

    let embed_cache = config.data_dir.join("model-cache/fastembed");
    std::fs::create_dir_all(&embed_cache)
        .map_err(|error| HostError::Initialization(error.to_string()))?;
    let embedder = gen_ui_runtime::spawn_blocking(move || FastEmbedder::new(embed_cache))
        .await
        .map_err(|error| HostError::Initialization(error.to_string()))?
        .map_err(|error| HostError::Initialization(error.to_string()))?;
    let memory = GraphStore::open(GraphStoreConfig {
        endpoint: format!("rocksdb://{}", config.data_dir.join("memory-db").display()),
        namespace: config.namespace,
        database: config.database,
        embedder: Arc::new(embedder),
    })
    .await
    .map_err(|error| HostError::Initialization(error.to_string()))?;

    // macOS runs the same in-process MLX engine iOS does, so it hosts the
    // embedded UAR kernel too (no service URL) — matching mobile. Windows/Linux
    // stay service clients (no embedded_uar). The embedded runtime is built from
    // the same inference provider and the shared MLX provider config (macOS reuses
    // the iOS Qwen3-4B MLX id), then handed to state alongside the provider.
    #[cfg(target_os = "macos")]
    let embedded_uar = match inference.clone() {
        Some(provider) => {
            const MACOS_MLX_MODEL_ID: &str = "qwen3.5-4b-mlx-4bit";
            let built = gen_ui_agent::uar::UarEmbeddedRuntime::build_local(
                provider,
                gen_ui_types::inference::LocalModelSpec {
                    model: MACOS_MLX_MODEL_ID.to_string(),
                    context_len: Some(8_192),
                },
                gen_ui_agent::uar::ios_qwen35_mlx_provider_config(MACOS_MLX_MODEL_ID),
                // A sibling of memory-db, not a shared file: two SurrealDB
                // instances keep their schemas independent, and consolidating
                // later is easier than untangling one store two systems write.
                Some(&config.data_dir.join("uar-db")),
            )
            .await
            .map_err(|error| HostError::Initialization(error.to_string()))?;
            Some(Arc::new(built))
        }
        None => None,
    };
    #[cfg(not(target_os = "macos"))]
    let embedded_uar = None;

    gen_ui_agent::state::init_with_inference_and_uar(
        ConfigBackend::Postgres(Arc::new(config_store)),
        Arc::new(memory),
        inference,
        embedded_uar,
    );
    // First desktop local-lane tool; registration failure degrades chat, not
    // the app, so it must never take down host boot. Known limitation: the
    // llama.cpp engine lacks continue_with_tool_results, so on Windows/Linux
    // local-lane tool calls fail at continuation (macOS runs MLXC, which
    // supports it; the UAR service lane covers Windows/Linux agent runs).
    if let Err(error) = gen_ui_agent::web_fetch::register_fetch_mcp().await {
        tracing::warn!(%error, "web fetch MCP registration failed; local fetch tool unavailable");
    }
    tracing::info!(path = %config.data_dir.display(), "shared application host ready");

    Ok(AppServices {
        data_dir: Arc::new(config.data_dir),
        pending_runs: Arc::new(DashMap::new()),
    })
}
