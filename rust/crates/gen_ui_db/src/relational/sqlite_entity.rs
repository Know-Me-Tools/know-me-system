// TJ-ARCH-MOB-001 compliant
//! Mobile SQLite implementation of the shared entity-envelope boundary.
//! Dart sees intent-level CRUD only; SQL and storage lifecycle remain in Rust.

use std::path::Path;
use std::str::FromStr;

use gen_ui_types::transport::{EntityRecord, EntityTransport, ListResult};
use gen_ui_types::view::ViewDescriptor;
use gen_ui_types::{CoreError, CoreResult};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Row, SqlitePool};

#[derive(Clone)]
pub struct SqliteEntityStore {
    pool: SqlitePool,
}

impl SqliteEntityStore {
    pub async fn open(path: impl AsRef<Path>) -> CoreResult<Self> {
        let url = format!("sqlite://{}", path.as_ref().display());
        let options = SqliteConnectOptions::from_str(&url)
            .map_err(database_error)?
            .create_if_missing(true)
            .foreign_keys(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .map_err(database_error)?;
        sqlx::raw_sql(
            "CREATE TABLE IF NOT EXISTS entity_records (
                 entity_type TEXT NOT NULL,
                 id TEXT NOT NULL,
                 data_json TEXT NOT NULL CHECK (json_valid(data_json)),
                 updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 PRIMARY KEY (entity_type, id)
             );
             CREATE INDEX IF NOT EXISTS entity_records_updated
                 ON entity_records (entity_type, updated_at DESC);",
        )
        .execute(&pool)
        .await
        .map_err(database_error)?;
        Ok(Self { pool })
    }
}

#[async_trait::async_trait]
impl EntityTransport for SqliteEntityStore {
    async fn list(&self, view: &ViewDescriptor) -> CoreResult<ListResult> {
        let limit = i64::from(view.limit.unwrap_or(200).min(1_000));
        let rows = sqlx::query(
            "SELECT id, entity_type, data_json FROM entity_records \
             WHERE entity_type = ?1 ORDER BY updated_at DESC LIMIT ?2",
        )
        .bind(&view.entity_type)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;
        Ok(ListResult {
            items: rows
                .into_iter()
                .map(|row| EntityRecord {
                    id: row.get("id"),
                    entity_type: row.get("entity_type"),
                    data_json: row.get("data_json"),
                })
                .collect(),
            next_cursor: None,
        })
    }

    async fn get(&self, entity_type: &str, id: &str) -> CoreResult<Option<EntityRecord>> {
        let row = sqlx::query(
            "SELECT id, entity_type, data_json FROM entity_records \
             WHERE entity_type = ?1 AND id = ?2",
        )
        .bind(entity_type)
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;
        Ok(row.map(|row| EntityRecord {
            id: row.get("id"),
            entity_type: row.get("entity_type"),
            data_json: row.get("data_json"),
        }))
    }

    async fn create(&self, record: &EntityRecord) -> CoreResult<EntityRecord> {
        sqlx::query(
            "INSERT INTO entity_records (entity_type, id, data_json, updated_at) \
             VALUES (?1, ?2, ?3, CURRENT_TIMESTAMP)",
        )
        .bind(&record.entity_type)
        .bind(&record.id)
        .bind(&record.data_json)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;
        Ok(record.clone())
    }

    async fn update(&self, record: &EntityRecord) -> CoreResult<EntityRecord> {
        sqlx::query(
            "INSERT INTO entity_records (entity_type, id, data_json, updated_at) \
             VALUES (?1, ?2, ?3, CURRENT_TIMESTAMP) \
             ON CONFLICT (entity_type, id) DO UPDATE SET \
                 data_json = excluded.data_json, updated_at = CURRENT_TIMESTAMP",
        )
        .bind(&record.entity_type)
        .bind(&record.id)
        .bind(&record.data_json)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;
        Ok(record.clone())
    }

    async fn delete(&self, entity_type: &str, id: &str) -> CoreResult<()> {
        sqlx::query("DELETE FROM entity_records WHERE entity_type = ?1 AND id = ?2")
            .bind(entity_type)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(database_error)?;
        Ok(())
    }
}

fn database_error(error: impl std::fmt::Display) -> CoreError {
    CoreError::Transient(format!("mobile entity database: {error}"))
}
