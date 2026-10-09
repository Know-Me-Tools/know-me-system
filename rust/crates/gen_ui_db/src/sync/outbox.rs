//! C11 durable local intent and authoritative command-result reconciliation.
use super::local_store::{apply_changes, check_table, storage_error, PgLocalStore};
use super::seam::{PendingWrite, RowChange, RowOp, WriteOutcome};
use gen_ui_types::{CoreError, CoreResult};
use sqlx::{Postgres, Row, Transaction};

impl PgLocalStore {
    pub(super) async fn open_outbox(&self) -> CoreResult<()> {
        let mut tx = self.pool.begin().await.map_err(storage_error)?;
        sqlx::query("INSERT INTO sync_projection_scope (singleton, scope) VALUES (true, $1) ON CONFLICT DO NOTHING")
            .bind(&self.scope).execute(&mut *tx).await.map_err(storage_error)?;
        let active: String = sqlx::query_scalar("SELECT scope FROM sync_projection_scope WHERE singleton FOR UPDATE")
            .fetch_one(&mut *tx).await.map_err(storage_error)?;
        if active != self.scope {
            // A newly configured tenant/authority must not see the previous cache.
            // Previous intents remain inspectable, but are never replayed under it.
            sqlx::query("DELETE FROM notes").execute(&mut *tx).await.map_err(storage_error)?;
            sqlx::query("DELETE FROM memories").execute(&mut *tx).await.map_err(storage_error)?;
            sqlx::query("DELETE FROM entity_records WHERE entity_type IN ('notes', 'memories')")
                .execute(&mut *tx).await.map_err(storage_error)?;
            sqlx::query("UPDATE sync_projection_scope SET scope = $1 WHERE singleton")
                .bind(&self.scope).execute(&mut *tx).await.map_err(storage_error)?;
        }
        self.overlay_pending(&mut tx).await?;
        tx.commit().await.map_err(storage_error)
    }

    pub(super) async fn lock_projection(&self, tx: &mut Transaction<'_, Postgres>) -> CoreResult<()> {
        let active: Option<String> = sqlx::query_scalar("SELECT scope FROM sync_projection_scope WHERE singleton FOR UPDATE")
            .fetch_optional(&mut **tx).await.map_err(storage_error)?;
        if active.as_deref() != Some(self.scope.as_str()) {
            return Err(CoreError::Terminal("sync authority changed; detach the previous transport".into()));
        }
        Ok(())
    }

    pub(super) async fn commit_intent(&self, write: &PendingWrite) -> CoreResult<()> {
        let change = local_change(write)?;
        let mut tx = self.pool.begin().await.map_err(storage_error)?;
        self.lock_projection(&mut tx).await?;
        if self.read_revoked(&mut tx, &write.table).await? {
            return Err(CoreError::Terminal("selected read grant is revoked; authenticate before submitting".into()));
        }
        // Each new submission gets a fresh identity. Retries use the identity saved
        // in this transaction; identical edits separated in time are not collapsed.
        let key = if write.idempotency_key.is_empty() {
            sqlx::query_scalar::<_, String>("SELECT gen_random_uuid()::text")
                .fetch_one(&mut *tx).await.map_err(storage_error)?
        } else { write.idempotency_key.clone() };
        let inserted = sqlx::query("INSERT INTO sync_outbox (scope, command_id, table_name, change_json) VALUES ($1, $2, $3, $4) ON CONFLICT DO NOTHING")
            .bind(&self.scope).bind(&key).bind(&write.table).bind(&write.change_json)
            .execute(&mut *tx).await.map_err(storage_error)?.rows_affected();
        if inserted == 0 {
            let existing: (String, String) = sqlx::query_as("SELECT table_name, change_json FROM sync_outbox WHERE scope = $1 AND command_id = $2")
                .bind(&self.scope).bind(&key).fetch_one(&mut *tx).await.map_err(storage_error)?;
            if existing != (write.table.clone(), write.change_json.clone()) {
                return Err(CoreError::Terminal("command identity already belongs to another mutation".into()));
            }
            // A retry of an already settled submission must not undo newer edits.
        } else {
            apply_changes(&mut tx, &[change]).await?;
        }
        tx.commit().await.map_err(storage_error)
    }

    pub(super) async fn read_intents(&self, poison: bool) -> CoreResult<Vec<PendingWrite>> {
        let mut tx = self.pool.begin().await.map_err(storage_error)?;
        self.lock_projection(&mut tx).await?;
        let rows = sqlx::query("SELECT command_id, table_name, change_json, attempts FROM sync_outbox o WHERE scope = $1 AND status = $2 AND NOT EXISTS (SELECT 1 FROM sync_read_cursors c WHERE c.scope = o.scope AND c.table_name = o.table_name AND c.status = 'revoked') ORDER BY sequence")
            .bind(&self.scope).bind(if poison { "poison" } else { "pending" })
            .fetch_all(&mut *tx).await.map_err(storage_error)?;
        let writes = rows.into_iter().map(|row| Ok(PendingWrite {
            idempotency_key: row.try_get("command_id").map_err(storage_error)?,
            table: row.try_get("table_name").map_err(storage_error)?,
            change_json: row.try_get("change_json").map_err(storage_error)?,
            attempts: u32::try_from(row.try_get::<i64, _>("attempts").map_err(storage_error)?)
                .map_err(|_| CoreError::Terminal("invalid persisted replay attempt count".into()))?,
        })).collect::<CoreResult<Vec<_>>>()?;
        tx.commit().await.map_err(storage_error)?;
        Ok(writes)
    }

    pub(super) async fn overlay_pending(&self, tx: &mut Transaction<'_, Postgres>) -> CoreResult<()> {
        let rows = sqlx::query("SELECT command_id, table_name, change_json FROM sync_outbox o WHERE scope = $1 AND status = 'pending' AND NOT EXISTS (SELECT 1 FROM sync_read_cursors c WHERE c.scope = o.scope AND c.table_name = o.table_name AND c.status = 'revoked') ORDER BY sequence")
            .bind(&self.scope).fetch_all(&mut **tx).await.map_err(storage_error)?;
        for row in rows {
            let write = PendingWrite {
                idempotency_key: row.try_get("command_id").map_err(storage_error)?,
                table: row.try_get("table_name").map_err(storage_error)?,
                change_json: row.try_get("change_json").map_err(storage_error)?, attempts: 0,
            };
            apply_changes(tx, &[local_change(&write)?]).await?;
        }
        Ok(())
    }

    async fn read_revoked(&self, tx: &mut Transaction<'_, Postgres>, table: &str) -> CoreResult<bool> {
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM sync_read_cursors WHERE scope = $1 AND table_name = $2 AND status = 'revoked')")
            .bind(&self.scope).bind(table).fetch_one(&mut **tx).await.map_err(storage_error)
    }

    pub(super) async fn commit_outcome(&self, write: &PendingWrite, outcome: &WriteOutcome) -> CoreResult<()> {
        let mut tx = self.pool.begin().await.map_err(storage_error)?;
        self.lock_projection(&mut tx).await?;
        let (status, result, reason) = match outcome {
            WriteOutcome::Applied => ("committed", None, None),
            WriteOutcome::Committed { change } => {
                if !self.read_revoked(&mut tx, &write.table).await? {
                    apply_changes(&mut tx, std::slice::from_ref(change)).await?;
                }
                ("committed", Some(serde_json::json!({"table": change.table, "id": change.key, "op": if change.op == RowOp::Delete { "delete" } else { "upsert" }, "data": serde_json::from_str::<serde_json::Value>(&change.value_json).map_err(|e| CoreError::Serde(e.to_string()))?}).to_string()), None)
            }
            WriteOutcome::Retry => ("pending", None, None),
            WriteOutcome::Poison { reason } => {
                // A rejected optimistic edit is not authoritative. Drop that row,
                // reset the selected shape, and let its snapshot restore authority.
                let change = local_change(write)?;
                let table = check_table(&write.table)?;
                sqlx::query(sqlx::AssertSqlSafe(format!("DELETE FROM {table} WHERE id = $1")))
                    .bind(&change.key).execute(&mut *tx).await.map_err(storage_error)?;
                sqlx::query("DELETE FROM entity_records WHERE entity_type = $1 AND id = $2")
                    .bind(table).bind(&change.key).execute(&mut *tx).await.map_err(storage_error)?;
                sqlx::query("UPDATE sync_read_cursors SET handle = NULL, offset_value = NULL, status = 'resnapshot', generation = generation + 1 WHERE scope = $1 AND table_name = $2 AND status != 'revoked'")
                    .bind(&self.scope).bind(table).execute(&mut *tx).await.map_err(storage_error)?;
                ("poison", None, Some(reason.as_str()))
            },
        };
        sqlx::query("UPDATE sync_outbox SET status = $3, attempts = $4, result_json = $5::jsonb, rejection = $6, settled_at = CASE WHEN $3 = 'pending' THEN NULL ELSE CURRENT_TIMESTAMP END WHERE scope = $1 AND command_id = $2 AND status = 'pending'")
            .bind(&self.scope).bind(&write.idempotency_key).bind(status).bind(i64::from(write.attempts))
            .bind(result).bind(reason).execute(&mut *tx).await.map_err(storage_error)?;
        self.overlay_pending(&mut tx).await?;
        tx.commit().await.map_err(storage_error)
    }
}

fn local_change(write: &PendingWrite) -> CoreResult<RowChange> {
    check_table(&write.table)?;
    let body: serde_json::Value = serde_json::from_str(&write.change_json).map_err(|e| CoreError::Serde(e.to_string()))?;
    let key = body.get("id").and_then(serde_json::Value::as_str)
        .filter(|id| !id.is_empty()).ok_or_else(|| CoreError::Terminal("write requires a row identity".into()))?;
    let op = match body.get("op").and_then(serde_json::Value::as_str) {
        Some("insert" | "upsert") => RowOp::Insert,
        Some("update") => RowOp::Update,
        Some("delete") => RowOp::Delete,
        _ => return Err(CoreError::Terminal("unknown write operation".into())),
    };
    let mut value = body.get("data").cloned().unwrap_or_else(|| serde_json::json!({}));
    let object = value.as_object_mut().ok_or_else(|| CoreError::Terminal("write data must be an object".into()))?;
    object.insert("id".into(), serde_json::json!(key));
    Ok(RowChange { table: write.table.clone(), key: key.to_owned(), op, value_json: value.to_string() })
}
