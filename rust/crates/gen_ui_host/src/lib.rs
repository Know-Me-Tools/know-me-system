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

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderCatalogEntry {
    pub id: String,
    pub display_name: String,
    pub default_base_url: Option<String>,
    pub requires_api_key: bool,
    pub supports_chat: bool,
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

    pub fn provider_catalog(&self) -> Result<Vec<ProviderCatalogEntry>, HostError> {
        gen_ui_agent::provider_admin::catalog()
            .map(|entries| {
                entries
                    .into_iter()
                    .map(|entry| ProviderCatalogEntry {
                        id: entry.id,
                        display_name: entry.display_name,
                        default_base_url: entry.default_base_url,
                        requires_api_key: entry.requires_api_key,
                        supports_chat: entry.supports_chat,
                    })
                    .collect()
            })
            .map_err(|error| HostError::Initialization(error.to_string()))
    }

    pub async fn start_chat(
        &self,
        user_message: String,
        history: Vec<String>,
    ) -> Result<String, HostError> {
        let receiver =
            gen_ui_agent::state::subscribe().map_err(|error| HostError::Chat(error.to_string()))?;
        let run_id = gen_ui_agent::chat::send(user_message, history)
            .await
            .map_err(|error| HostError::Chat(error.to_string()))?;
        self.pending_runs
            .insert(run_id.clone(), Arc::new(Mutex::new(Some(receiver))));
        Ok(run_id)
    }

    pub async fn start_chat_with_byok(
        &self,
        user_message: String,
        history: Vec<String>,
        config: EphemeralCloudConfig,
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

    gen_ui_agent::state::init_with_inference(
        ConfigBackend::Postgres(Arc::new(config_store)),
        Arc::new(memory),
        inference,
    );
    tracing::info!(path = %config.data_dir.display(), "shared application host ready");

    Ok(AppServices {
        data_dir: Arc::new(config.data_dir),
        pending_runs: Arc::new(DashMap::new()),
    })
}
