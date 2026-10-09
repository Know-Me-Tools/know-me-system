// TJ-ARCH-MOB-001 compliant
//! [`LocalStore`] over the local relational store (C-106 T6b).
//!
//! The read path lands here: the CDC/spine consumer decodes each server-side row change
//! into a [`RowChange`] and this applies it to the on-device Postgres — pglite-oxide on
//! desktop, cloud Postgres in tests. One transaction per `apply_batch` so the local DB
//! is only ever consistent at a change-batch boundary, never halfway through one.
//!
//! Why this is generic over "has a `PgPool`" rather than tied to `PgliteStore`: desktop
//! and cloud are the same wire protocol (pglite-oxide *is* Postgres 17 over the standard
//! protocol), so one impl serves both and the tests can run against either.
use async_trait::async_trait;
use gen_ui_types::error::{CoreError, CoreResult};
use sqlx::{AssertSqlSafe, PgPool, Postgres, Transaction};

use super::seam::{LocalStore, PendingWrite, RowChange, RowOp, WriteOutcome};

/// Tables this store is allowed to write.
///
/// **This is a security boundary, not a tidiness check.** `table` arrives from a decoded
/// server message, and Postgres cannot parameterise an identifier — a table name has to
/// be interpolated into the SQL string. Without an allow-list that is a SQL-injection
/// hole reachable from the wire. An unknown table is [`CoreError::Terminal`]: retrying
/// cannot make it known, and silently skipping would strand the row forever.
const SYNCED_TABLES: &[&str] = &["notes", "memories"];

pub(super) fn check_table(table: &str) -> CoreResult<&'static str> {
    SYNCED_TABLES
        .iter()
        .find(|t| **t == table)
        .copied()
        .ok_or_else(|| CoreError::Terminal(format!("table not syncable: {table}")))
}

/// Applies decoded row changes to a local Postgres-protocol database.
pub struct PgLocalStore {
    pub(super) pool: PgPool,
    pub(super) scope: String,
}

impl PgLocalStore {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self::scoped(pool, "local")
    }
    pub fn scoped(pool: PgPool, scope: impl Into<String>) -> Self {
        Self { pool, scope: scope.into() }
    }
}

#[async_trait]
impl LocalStore for PgLocalStore {
    /// Apply one batch atomically.
    ///
    /// Rows arrive as whole JSON objects, so rather than generate per-column SQL per
    /// table we hand the object to Postgres and let `jsonb_populate_record` shape it
    /// against the table's own rowtype. That keeps this impl schema-agnostic: adding a
    /// column to `notes` needs no change here.
    async fn apply_batch(&self, changes: &[RowChange]) -> CoreResult<()> {
        if changes.is_empty() {
            return Ok(());
        }
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| CoreError::Transient(format!("begin: {e}")))?;

        self.lock_projection(&mut tx).await?;
        apply_changes(&mut tx, changes).await?;
        self.overlay_pending(&mut tx).await?;

        tx.commit()
            .await
            .map_err(|e| CoreError::Transient(format!("commit: {e}")))
    }

    /// Wipe a table's synced rows so the consumer can re-materialise from scratch.
    ///
    /// Remove committed live rows on shape rotation, retaining tombstones and
    /// reapplying pending local intent atomically before resnapshot starts.
    async fn truncate_shape(&self, table: &str) -> CoreResult<()> {
        let table = check_table(table)?;
        let mut tx = self.pool.begin().await.map_err(storage_error)?;
        self.lock_projection(&mut tx).await?;
        sqlx::query(AssertSqlSafe(format!("DELETE FROM {table} WHERE NOT deleted")))
            .execute(&mut *tx).await.map_err(storage_error)?;
        sqlx::query("DELETE FROM entity_records WHERE entity_type = $1 AND data_json->'deleted' IS DISTINCT FROM 'true'::jsonb")
            .bind(table).execute(&mut *tx).await.map_err(storage_error)?;
        self.overlay_pending(&mut tx).await?;
        tx.commit().await.map_err(storage_error)
    }
    async fn initialize_outbox(&self) -> CoreResult<()> { self.open_outbox().await }
    async fn enqueue_pending(&self, write: &PendingWrite) -> CoreResult<()> { self.commit_intent(write).await }
    async fn queued_writes(&self, poison: bool) -> CoreResult<Vec<PendingWrite>> { self.read_intents(poison).await }
    async fn settle_write(&self, write: &PendingWrite, outcome: &WriteOutcome) -> CoreResult<()> { self.commit_outcome(write, outcome).await }

}

/// `SET col = EXCLUDED.col` for every column except the PK.
///
/// Hand-rolled per table rather than derived at runtime: the alternative is querying
/// `information_schema` on every upsert (a round-trip per row) or caching a schema map
/// (state to invalidate). These tables are defined in `infra/knowme-sync.sql` and change
/// about never; the `SYNCED_TABLES` allow-list already pins the set.
fn excluded_assignments(table: &str) -> &'static str {
    match table {
        "notes" => {
            "title = EXCLUDED.title, body = EXCLUDED.body, \
             updated_at = EXCLUDED.updated_at, deleted = EXCLUDED.deleted"
        }
        "memories" => {
            "text = EXCLUDED.text, kind = EXCLUDED.kind, entity = EXCLUDED.entity, \
             updated_at = EXCLUDED.updated_at, deleted = EXCLUDED.deleted"
        }
        // Unreachable: `check_table` gates every caller. Kept total rather than
        // `unreachable!()` so a future table added to SYNCED_TABLES without an
        // assignment list degrades to "PK-only upsert", not a panic in the sync loop.
        _ => "updated_at = EXCLUDED.updated_at",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unlisted_table_terminally() {
        // The injection guard: an attacker-controlled table name must not reach SQL,
        // and must not be retried forever either.
        let err = check_table("notes; DROP TABLE users --").unwrap_err();
        assert!(matches!(err, CoreError::Terminal(_)), "got {err:?}");
    }

    #[test]
    fn accepts_synced_tables() {
        assert_eq!(check_table("notes").unwrap(), "notes");
        assert_eq!(check_table("memories").unwrap(), "memories");
    }

    #[test]
    fn assignments_never_touch_the_primary_key() {
        // Upserting the PK would be a no-op at best; at worst it masks a key mismatch.
        for t in SYNCED_TABLES {
            let a = excluded_assignments(t);
            assert!(!a.contains("id ="), "{t} assigns the PK: {a}");
            assert!(a.contains("updated_at ="), "{t} must carry updated_at: {a}");
        }
    }
}

pub(super) fn storage_error(e: sqlx::Error) -> CoreError { CoreError::Transient(format!("local sync storage: {e}")) }
pub(super) async fn apply_changes(tx: &mut Transaction<'_, Postgres>, changes: &[RowChange]) -> CoreResult<()> {
        for change in changes {
            check_table(&change.table)?;
            let mut value = if change.op == RowOp::Delete { serde_json::json!({"id": change.key, "deleted": true}) }
                else { serde_json::from_str::<serde_json::Value>(&change.value_json).map_err(|e| CoreError::Serde(e.to_string()))? };
            value.as_object_mut().ok_or_else(|| CoreError::Terminal("sync row must be an object".into()))?
                .insert("id".into(), serde_json::json!(change.key));
            sqlx::query("INSERT INTO entity_records (entity_type, id, data_json, updated_at) VALUES ($1, $2, $3::jsonb, CURRENT_TIMESTAMP) ON CONFLICT (entity_type, id) DO UPDATE SET data_json = EXCLUDED.data_json, updated_at = EXCLUDED.updated_at")
                .bind(&change.table).bind(&change.key).bind(value.to_string()).execute(&mut **tx).await.map_err(storage_error)?;
            let table = check_table(&change.table)?;
            match change.op {
                // Insert and Update are the same statement on purpose. A sync stream is
                // not a reliable narrator of which one a row "is": a re-materialised
                // shape replays existing rows as inserts, and an update can arrive for a
                // row we never saw. Upsert is the only convergent choice.
                RowOp::Insert | RowOp::Update => {
                    let sql = format!(
                        "INSERT INTO {table} SELECT * FROM jsonb_populate_record(NULL::{table}, jsonb_build_object('title', '', 'body', '', 'text', '', 'kind', 'note', 'deleted', false, 'updated_at', CURRENT_TIMESTAMP) || $1::jsonb) \
                         ON CONFLICT (id) DO UPDATE SET {assignments}",
                        assignments = excluded_assignments(table),
                    );
                    sqlx::query(AssertSqlSafe(sql))
                        .bind(value.to_string())
                        .execute(&mut **tx)
                        .await
                        .map_err(|e| CoreError::Transient(format!("upsert {table}: {e}")))?;
                }
                // Soft delete: the server marks `deleted = true` and ships that as a row
                // change, so a hard DELETE here would diverge from what the next
                // re-materialisation replays. `value_json` is empty for deletes, so key
                // off `key` instead.
                RowOp::Delete => {
                    let sql = if table == "memories" {
                        "INSERT INTO memories (id, text, deleted) VALUES ($1, '', true) ON CONFLICT (id) DO UPDATE SET deleted = true".to_owned()
                    } else {
                        "INSERT INTO notes (id, deleted) VALUES ($1, true) ON CONFLICT (id) DO UPDATE SET deleted = true".to_owned()
                    };
                    sqlx::query(AssertSqlSafe(sql))
                        .bind(&change.key)
                        .execute(&mut **tx)
                        .await
                        .map_err(|e| CoreError::Transient(format!("delete {table}: {e}")))?;
                }
            }
        }

    Ok(())
}
