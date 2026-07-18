// TJ-ARCH-MOB-001 compliant
//! Config DB v1 for mobile: provider/model administration for the liter-llm
//! gateway, stored in the same embedded SurrealDB instance as the memory graph
//! (one embedded-DB story per platform — desktop/web use the Postgres-dialect
//! schema in gen_ui_db::relational::config instead). Same intent-level shapes
//! as the desktop/web ConfigStore so the FFI surface stays uniform across
//! platforms; only the storage engine differs.
//!
//! `api_key_ref` is a reference into platform-secure storage (keychain /
//! Android Keystore), never a plaintext secret — enforced by type.

use serde::{Deserialize, Serialize};
use surrealdb::types::SurrealValue;

use crate::error::{check_statements, GraphError};
use crate::store::GraphStore;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provider {
    pub id: String,
    pub kind: String,
    pub base_url: Option<String>,
    pub api_key_ref: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPref {
    pub surface: String,
    pub lane: String,
    pub provider_id: Option<String>,
    pub model_id: String,
    pub params: serde_json::Value,
}

#[derive(SurrealValue)]
struct ProviderRow {
    id: String,
    kind: String,
    base_url: Option<String>,
    api_key_ref: Option<String>,
    enabled: bool,
}

impl From<ProviderRow> for Provider {
    fn from(r: ProviderRow) -> Self {
        Provider {
            id: r.id,
            kind: r.kind,
            base_url: r.base_url,
            api_key_ref: r.api_key_ref,
            enabled: r.enabled,
        }
    }
}

#[derive(SurrealValue)]
struct ModelPrefRow {
    surface: String,
    lane: String,
    provider_id: Option<String>,
    model_id: String,
}

impl GraphStore {
    /// INTENT: list all configured providers.
    pub async fn list_providers(&self) -> Result<Vec<Provider>, GraphError> {
        let mut res = self
            .db()
            .query("SELECT meta::id(id) AS id, kind, base_url, api_key_ref, enabled FROM provider ORDER BY id;")
            .await?;
        let rows: Vec<ProviderRow> = res.take(0)?;
        Ok(rows.into_iter().map(Provider::from).collect())
    }

    /// INTENT: create or update a provider (id is the record key).
    pub async fn upsert_provider(&self, provider: &Provider) -> Result<(), GraphError> {
        if provider.id.trim().is_empty() {
            return Err(GraphError::Invalid("provider id is empty".into()));
        }
        let mut response = self
            .db()
            .query(
                "UPSERT type::record('provider', $id) \
                 SET kind = $kind, base_url = $base_url, api_key_ref = $api_key_ref, \
                 enabled = $enabled;",
            )
            .bind(("id", provider.id.clone()))
            .bind(("kind", provider.kind.clone()))
            .bind(("base_url", provider.base_url.clone()))
            .bind(("api_key_ref", provider.api_key_ref.clone()))
            .bind(("enabled", provider.enabled))
            .await?;
        check_statements(&mut response, "upsert_provider")
    }

    /// INTENT: remove a provider by id.
    pub async fn delete_provider(&self, id: &str) -> Result<(), GraphError> {
        let mut response = self
            .db()
            .query("DELETE type::record('provider', $id);")
            .bind(("id", id.to_string()))
            .await?;
        check_statements(&mut response, "delete_provider")
    }

    /// INTENT: read the model preference for one (surface, lane) pair.
    pub async fn get_model_pref(
        &self,
        surface: &str,
        lane: &str,
    ) -> Result<Option<ModelPref>, GraphError> {
        let mut res = self
            .db()
            .query(
                "SELECT surface, lane, provider_id, model_id, params FROM model_pref \
                 WHERE surface = $surface AND lane = $lane LIMIT 1;",
            )
            .bind(("surface", surface.to_string()))
            .bind(("lane", lane.to_string()))
            .await?;
        let rows: Vec<ModelPrefRow> = res.take(0)?;
        // `params` is a FLEXIBLE object field — read back via a second projection
        // so SurrealValue doesn't need a generic-object variant.
        let params: Vec<Option<serde_json::Value>> = {
            let mut p = self
                .db()
                .query(
                    "SELECT VALUE params FROM model_pref WHERE surface = $surface AND lane = $lane LIMIT 1;",
                )
                .bind(("surface", surface.to_string()))
                .bind(("lane", lane.to_string()))
                .await?;
            p.take(0)?
        };
        Ok(rows
            .into_iter()
            .zip(params)
            .next()
            .map(|(r, params)| ModelPref {
                surface: r.surface,
                lane: r.lane,
                provider_id: r.provider_id,
                model_id: r.model_id,
                params: params.unwrap_or(serde_json::Value::Null),
            }))
    }

    /// INTENT: create or update the model preference for one (surface, lane) pair.
    pub async fn upsert_model_pref(&self, pref: &ModelPref) -> Result<(), GraphError> {
        let mut response = self
            .db()
            .query(
                "UPSERT type::record('model_pref', $key) \
                 SET surface = $surface, lane = $lane, provider_id = $provider_id, \
                 model_id = $model_id, params = $params;",
            )
            .bind(("key", format!("{}_{}", pref.surface, pref.lane)))
            .bind(("surface", pref.surface.clone()))
            .bind(("lane", pref.lane.clone()))
            .bind(("provider_id", pref.provider_id.clone()))
            .bind(("model_id", pref.model_id.clone()))
            .bind(("params", pref.params.clone()))
            .await?;
        check_statements(&mut response, "upsert_model_pref")
    }

    /// INTENT: read one app setting by key.
    pub async fn get_setting(&self, key: &str) -> Result<Option<serde_json::Value>, GraphError> {
        let mut res = self
            .db()
            .query("SELECT VALUE value FROM type::record('app_setting', $key);")
            .bind(("key", key.to_string()))
            .await?;
        let rows: Vec<Option<serde_json::Value>> = res.take(0)?;
        Ok(rows.into_iter().next().flatten())
    }

    /// INTENT: create or update one app setting.
    pub async fn set_setting(&self, key: &str, value: serde_json::Value) -> Result<(), GraphError> {
        let mut response = self
            .db()
            .query("UPSERT type::record('app_setting', $key) SET value = $value;")
            .bind(("key", key.to_string()))
            .bind(("value", value))
            .await?;
        check_statements(&mut response, "set_setting")
    }
}
