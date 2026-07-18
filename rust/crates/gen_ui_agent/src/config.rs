// TJ-ARCH-MOB-001 compliant
//! Platform-agnostic config-DB access. Desktop (Tauri) reads through gen_ui_db's
//! `ConfigStore` trait (pglite-oxide); mobile reads through gen_ui_db_graph's
//! `GraphStore` inherent methods (embedded SurrealDB) — different storage engines,
//! same intent-level shape, so callers (chat::send) never branch on platform.
use std::sync::Arc;

use gen_ui_db::relational::{ConfigStore, ModelPref as PgModelPref, Provider as PgProvider};
use gen_ui_db_graph::{ModelPref as SurrealModelPref, Provider as SurrealProvider};

use crate::error::AgentError;

/// A provider row, normalized across both storage engines.
#[derive(Debug, Clone)]
pub struct Provider {
    pub id: String,
    pub kind: String,
    pub base_url: Option<String>,
    pub api_key_ref: Option<String>,
    pub enabled: bool,
}

impl From<PgProvider> for Provider {
    fn from(p: PgProvider) -> Self {
        Self {
            id: p.id,
            kind: p.kind,
            base_url: p.base_url,
            api_key_ref: p.api_key_ref,
            enabled: p.enabled,
        }
    }
}

impl From<SurrealProvider> for Provider {
    fn from(p: SurrealProvider) -> Self {
        Self {
            id: p.id,
            kind: p.kind,
            base_url: p.base_url,
            api_key_ref: p.api_key_ref,
            enabled: p.enabled,
        }
    }
}

/// A model preference row, normalized across both storage engines.
#[derive(Debug, Clone)]
pub struct ModelPref {
    pub provider_id: Option<String>,
    pub model_id: String,
    /// Engine-specific sampling knobs (temperature, top_p, max_tokens,
    /// context_len). Free-form because the two lanes want different keys and the
    /// cloud lane mostly defers to the provider's server-side defaults; the local
    /// lane reads what it needs and falls back per-key.
    pub params: serde_json::Value,
}

impl From<PgModelPref> for ModelPref {
    fn from(p: PgModelPref) -> Self {
        Self {
            provider_id: p.provider_id,
            model_id: p.model_id,
            params: p.params,
        }
    }
}

impl From<SurrealModelPref> for ModelPref {
    fn from(p: SurrealModelPref) -> Self {
        Self {
            provider_id: p.provider_id,
            model_id: p.model_id,
            params: p.params,
        }
    }
}

/// One config backend per platform, selected once at startup and held for the
/// process lifetime (see `crate::state`).
pub enum ConfigBackend {
    Postgres(Arc<dyn ConfigStore>),
    Surreal(Arc<gen_ui_db_graph::GraphStore>),
}

impl ConfigBackend {
    pub async fn list_providers(&self) -> Result<Vec<Provider>, AgentError> {
        match self {
            ConfigBackend::Postgres(store) => Ok(store
                .list_providers()
                .await
                .map_err(|e| AgentError::Config(e.to_string()))?
                .into_iter()
                .map(Provider::from)
                .collect()),
            ConfigBackend::Surreal(store) => Ok(store
                .list_providers()
                .await
                .map_err(|e| AgentError::Config(e.to_string()))?
                .into_iter()
                .map(Provider::from)
                .collect()),
        }
    }

    pub async fn upsert_provider(&self, provider: &Provider) -> Result<(), AgentError> {
        match self {
            ConfigBackend::Postgres(store) => store
                .upsert_provider(&PgProvider {
                    id: provider.id.clone(),
                    kind: provider.kind.clone(),
                    base_url: provider.base_url.clone(),
                    api_key_ref: provider.api_key_ref.clone(),
                    enabled: provider.enabled,
                })
                .await
                .map_err(|e| AgentError::Config(e.to_string())),
            ConfigBackend::Surreal(store) => store
                .upsert_provider(&SurrealProvider {
                    id: provider.id.clone(),
                    kind: provider.kind.clone(),
                    base_url: provider.base_url.clone(),
                    api_key_ref: provider.api_key_ref.clone(),
                    enabled: provider.enabled,
                })
                .await
                .map_err(|e| AgentError::Config(e.to_string())),
        }
    }

    pub async fn delete_provider(&self, id: &str) -> Result<(), AgentError> {
        match self {
            ConfigBackend::Postgres(store) => store
                .delete_provider(id)
                .await
                .map_err(|e| AgentError::Config(e.to_string())),
            ConfigBackend::Surreal(store) => store
                .delete_provider(id)
                .await
                .map_err(|e| AgentError::Config(e.to_string())),
        }
    }

    pub async fn get_model_pref(
        &self,
        surface: &str,
        lane: &str,
    ) -> Result<Option<ModelPref>, AgentError> {
        match self {
            ConfigBackend::Postgres(store) => Ok(store
                .get_model_pref(surface, lane)
                .await
                .map_err(|e| AgentError::Config(e.to_string()))?
                .map(ModelPref::from)),
            ConfigBackend::Surreal(store) => Ok(store
                .get_model_pref(surface, lane)
                .await
                .map_err(|e| AgentError::Config(e.to_string()))?
                .map(ModelPref::from)),
        }
    }

    pub async fn upsert_model_pref(
        &self,
        surface: &str,
        lane: &str,
        pref: &ModelPref,
    ) -> Result<(), AgentError> {
        match self {
            ConfigBackend::Postgres(store) => store
                .upsert_model_pref(&PgModelPref {
                    surface: surface.to_owned(),
                    lane: lane.to_owned(),
                    provider_id: pref.provider_id.clone(),
                    model_id: pref.model_id.clone(),
                    params: pref.params.clone(),
                })
                .await
                .map_err(|e| AgentError::Config(e.to_string())),
            ConfigBackend::Surreal(store) => store
                .upsert_model_pref(&SurrealModelPref {
                    surface: surface.to_owned(),
                    lane: lane.to_owned(),
                    provider_id: pref.provider_id.clone(),
                    model_id: pref.model_id.clone(),
                    params: pref.params.clone(),
                })
                .await
                .map_err(|e| AgentError::Config(e.to_string())),
        }
    }

    /// Read an app setting. `None` when unset — callers apply their own default
    /// rather than this layer inventing one.
    pub async fn get_setting(&self, key: &str) -> Result<Option<serde_json::Value>, AgentError> {
        match self {
            ConfigBackend::Postgres(store) => store
                .get_setting(key)
                .await
                .map_err(|e| AgentError::Config(e.to_string())),
            ConfigBackend::Surreal(store) => store
                .get_setting(key)
                .await
                .map_err(|e| AgentError::Config(e.to_string())),
        }
    }

    pub async fn set_setting(&self, key: &str, value: serde_json::Value) -> Result<(), AgentError> {
        match self {
            ConfigBackend::Postgres(store) => store
                .set_setting(key, value)
                .await
                .map_err(|e| AgentError::Config(e.to_string())),
            ConfigBackend::Surreal(store) => store
                .set_setting(key, value)
                .await
                .map_err(|e| AgentError::Config(e.to_string())),
        }
    }

    pub async fn get_provider(&self, id: &str) -> Result<Option<Provider>, AgentError> {
        match self {
            ConfigBackend::Postgres(store) => Ok(store
                .list_providers()
                .await
                .map_err(|e| AgentError::Config(e.to_string()))?
                .into_iter()
                .find(|p| p.id == id)
                .map(Provider::from)),
            ConfigBackend::Surreal(store) => Ok(store
                .list_providers()
                .await
                .map_err(|e| AgentError::Config(e.to_string()))?
                .into_iter()
                .find(|p| p.id == id)
                .map(Provider::from)),
        }
    }
}
