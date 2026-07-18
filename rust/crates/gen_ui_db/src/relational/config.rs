// TJ-ARCH-MOB-001 compliant
//! Config DB v1: provider/model administration for the liter-llm gateway.
//! Desktop (Tauri, via pglite-oxide) and web (PGlite) share this Postgres-dialect
//! schema (migrations/postgres/0002_config.sql) and this ConfigStore impl. Mobile
//! does NOT use this module — it stores config in the embedded SurrealDB instance
//! already wired for graph-RAG (see gen_ui_db_graph), keeping one embedded-DB
//! story per platform rather than adding sqlite as a third backend.
//! Intent-level API only — callers never see raw SQL (matches gen_ui_ffi's api
//! modules).
//!
//! `api_key_ref` is a reference into platform-secure storage (keychain on
//! desktop/mobile), never a plaintext secret — enforced by type (no plaintext
//! field exists on `Provider`).

use serde::{Deserialize, Serialize};

use super::RelationalResult;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSetting {
    pub key: String,
    pub value: serde_json::Value,
}

#[async_trait::async_trait]
pub trait ConfigStore: Send + Sync {
    async fn list_providers(&self) -> RelationalResult<Vec<Provider>>;
    async fn upsert_provider(&self, provider: &Provider) -> RelationalResult<()>;
    async fn delete_provider(&self, id: &str) -> RelationalResult<()>;

    async fn get_model_pref(
        &self,
        surface: &str,
        lane: &str,
    ) -> RelationalResult<Option<ModelPref>>;
    async fn upsert_model_pref(&self, pref: &ModelPref) -> RelationalResult<()>;

    async fn get_setting(&self, key: &str) -> RelationalResult<Option<serde_json::Value>>;
    async fn set_setting(&self, key: &str, value: serde_json::Value) -> RelationalResult<()>;
}

#[cfg(feature = "pg")]
mod postgres_impl {
    use sqlx::Row;

    use super::{ConfigStore, ModelPref, Provider, RelationalResult};
    use crate::relational::PostgresStore;

    #[async_trait::async_trait]
    impl ConfigStore for PostgresStore {
        async fn list_providers(&self) -> RelationalResult<Vec<Provider>> {
            let rows = sqlx::query(
                "SELECT id, kind, base_url, api_key_ref, enabled FROM providers ORDER BY id",
            )
            .fetch_all(self.pool())
            .await?;
            Ok(rows
                .into_iter()
                .map(|r| Provider {
                    id: r.get("id"),
                    kind: r.get("kind"),
                    base_url: r.get("base_url"),
                    api_key_ref: r.get("api_key_ref"),
                    enabled: r.get("enabled"),
                })
                .collect())
        }

        async fn upsert_provider(&self, provider: &Provider) -> RelationalResult<()> {
            sqlx::query(
                "INSERT INTO providers (id, kind, base_url, api_key_ref, enabled, updated_at) \
                 VALUES ($1, $2, $3, $4, $5, CURRENT_TIMESTAMP) \
                 ON CONFLICT (id) DO UPDATE SET kind = $2, base_url = $3, api_key_ref = $4, \
                 enabled = $5, updated_at = CURRENT_TIMESTAMP",
            )
            .bind(&provider.id)
            .bind(&provider.kind)
            .bind(&provider.base_url)
            .bind(&provider.api_key_ref)
            .bind(provider.enabled)
            .execute(self.pool())
            .await?;
            Ok(())
        }

        async fn delete_provider(&self, id: &str) -> RelationalResult<()> {
            sqlx::query("DELETE FROM providers WHERE id = $1")
                .bind(id)
                .execute(self.pool())
                .await?;
            Ok(())
        }

        async fn get_model_pref(
            &self,
            surface: &str,
            lane: &str,
        ) -> RelationalResult<Option<ModelPref>> {
            let row = sqlx::query(
                "SELECT surface, lane, provider_id, model_id, params FROM model_prefs \
                 WHERE surface = $1 AND lane = $2",
            )
            .bind(surface)
            .bind(lane)
            .fetch_optional(self.pool())
            .await?;
            Ok(row.map(|r| ModelPref {
                surface: r.get("surface"),
                lane: r.get("lane"),
                provider_id: r.get("provider_id"),
                model_id: r.get("model_id"),
                params: r.get("params"),
            }))
        }

        async fn upsert_model_pref(&self, pref: &ModelPref) -> RelationalResult<()> {
            sqlx::query(
                "INSERT INTO model_prefs (surface, lane, provider_id, model_id, params, updated_at) \
                 VALUES ($1, $2, $3, $4, $5, CURRENT_TIMESTAMP) \
                 ON CONFLICT (surface, lane) DO UPDATE SET provider_id = $3, model_id = $4, \
                 params = $5, updated_at = CURRENT_TIMESTAMP",
            )
            .bind(&pref.surface)
            .bind(&pref.lane)
            .bind(&pref.provider_id)
            .bind(&pref.model_id)
            .bind(&pref.params)
            .execute(self.pool())
            .await?;
            Ok(())
        }

        async fn get_setting(&self, key: &str) -> RelationalResult<Option<serde_json::Value>> {
            let row = sqlx::query("SELECT value FROM app_settings WHERE key = $1")
                .bind(key)
                .fetch_optional(self.pool())
                .await?;
            Ok(row.map(|r| r.get("value")))
        }

        async fn set_setting(&self, key: &str, value: serde_json::Value) -> RelationalResult<()> {
            sqlx::query(
                "INSERT INTO app_settings (key, value, updated_at) VALUES ($1, $2, CURRENT_TIMESTAMP) \
                 ON CONFLICT (key) DO UPDATE SET value = $2, updated_at = CURRENT_TIMESTAMP",
            )
            .bind(key)
            .bind(&value)
            .execute(self.pool())
            .await?;
            Ok(())
        }
    }

    #[cfg(feature = "pglite")]
    #[async_trait::async_trait]
    impl ConfigStore for crate::relational::PgliteStore {
        async fn list_providers(&self) -> RelationalResult<Vec<Provider>> {
            self.store().list_providers().await
        }
        async fn upsert_provider(&self, provider: &Provider) -> RelationalResult<()> {
            self.store().upsert_provider(provider).await
        }
        async fn delete_provider(&self, id: &str) -> RelationalResult<()> {
            self.store().delete_provider(id).await
        }
        async fn get_model_pref(
            &self,
            surface: &str,
            lane: &str,
        ) -> RelationalResult<Option<ModelPref>> {
            self.store().get_model_pref(surface, lane).await
        }
        async fn upsert_model_pref(&self, pref: &ModelPref) -> RelationalResult<()> {
            self.store().upsert_model_pref(pref).await
        }
        async fn get_setting(&self, key: &str) -> RelationalResult<Option<serde_json::Value>> {
            self.store().get_setting(key).await
        }
        async fn set_setting(&self, key: &str, value: serde_json::Value) -> RelationalResult<()> {
            self.store().set_setting(key, value).await
        }
    }
}
