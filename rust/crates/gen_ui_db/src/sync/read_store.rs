//! Provider positions and local projections advance in one transaction.
use super::local_store::{apply_changes, check_table, storage_error, PgLocalStore};
use super::seam::{CommandView, RowChange, RowOp};
use gen_ui_types::{CoreError, CoreResult};
use gen_ui_types::sync::{ReadCursor, ReadPoll};
use sqlx::{AssertSqlSafe, Row};

impl PgLocalStore {
    pub(super) async fn load_read_cursor(&self, shape: &str) -> CoreResult<Option<ReadCursor>> {
        let row: Option<(Option<String>, Option<String>)> = sqlx::query_as("SELECT handle, offset_value FROM sync_read_cursors WHERE scope = $1 AND shape_id = $2")
            .bind(&self.scope).bind(shape).fetch_optional(&self.pool).await.map_err(storage_error)?;
        Ok(row.and_then(|(handle, offset)| handle.zip(offset)).map(|(handle, offset)| ReadCursor { handle, offset }))
    }

    pub(super) async fn apply_read(&self, shape: &str, table: &str, previous: Option<&ReadCursor>, poll: &ReadPoll) -> CoreResult<bool> {
        let table = check_table(table)?;
        let mut tx = self.pool.begin().await.map_err(storage_error)?;
        self.lock_projection(&mut tx).await?;
        sqlx::query("INSERT INTO sync_read_cursors (scope, shape_id, table_name, status) VALUES ($1, $2, $3, 'syncing') ON CONFLICT (scope, shape_id) DO NOTHING")
            .bind(&self.scope).bind(shape).bind(table).execute(&mut *tx).await.map_err(storage_error)?;
        let row: (Option<String>, Option<String>, String) = sqlx::query_as("SELECT handle, offset_value, table_name FROM sync_read_cursors WHERE scope = $1 AND shape_id = $2 FOR UPDATE")
            .bind(&self.scope).bind(shape).fetch_one(&mut *tx).await.map_err(storage_error)?;
        if row.2 != table { return Err(CoreError::Terminal("shape cannot change its local table binding".into())); }
        let current = row.0.zip(row.1).map(|(handle, offset)| ReadCursor { handle, offset });
        if current.as_ref() != previous { return Ok(false); }
        let (cursor, state, reason) = match poll {
            ReadPoll::Batch { changes, cursor, caught_up } => {
                if !changes.is_empty() && cursor.is_none() {
                    return Err(CoreError::Terminal("provider changed rows without a complete cursor".into()));
                }
                let rows: Vec<_> = changes.iter().map(|change| RowChange {
                    table: table.to_owned(), key: change.id.clone(),
                    op: if change.deleted { RowOp::Delete } else { RowOp::Update },
                    value_json: change.data.to_string(),
                }).collect();
                apply_changes(&mut tx, &rows).await?;
                (cursor.as_ref().or(previous), if *caught_up { "live" } else { "syncing" }, None)
            }
            ReadPoll::Resnapshot => {
                sqlx::query(AssertSqlSafe(format!("DELETE FROM {table} WHERE NOT deleted")))
                    .execute(&mut *tx).await.map_err(storage_error)?;
                sqlx::query("DELETE FROM entity_records WHERE entity_type = $1 AND data_json->'deleted' IS DISTINCT FROM 'true'::jsonb")
                    .bind(table).execute(&mut *tx).await.map_err(storage_error)?;
                self.overlay_pending(&mut tx).await?;
                (None, "resnapshot", None)
            }
            ReadPoll::Revoked { reason } => {
                // Revocation clears disclosed read projections, including tombstones.
                // Private pending intent stays in the command log, never overlaid.
                sqlx::query(AssertSqlSafe(format!("DELETE FROM {table}")))
                    .execute(&mut *tx).await.map_err(storage_error)?;
                sqlx::query("DELETE FROM entity_records WHERE entity_type = $1")
                    .bind(table).execute(&mut *tx).await.map_err(storage_error)?;
                (None, "revoked", Some(reason.as_str()))
            }
        };
        sqlx::query("UPDATE sync_read_cursors SET handle = $3, offset_value = $4, status = $5, reason = $6, generation = generation + 1 WHERE scope = $1 AND shape_id = $2")
            .bind(&self.scope).bind(shape).bind(cursor.map(|c| c.handle.as_str())).bind(cursor.map(|c| c.offset.as_str()))
            .bind(state).bind(reason).execute(&mut *tx).await.map_err(storage_error)?;
        if state != "revoked" { self.overlay_pending(&mut tx).await?; }
        tx.commit().await.map_err(storage_error)?;
        Ok(true)
    }

    pub(super) async fn list_commands(&self) -> CoreResult<Vec<CommandView>> {
        let rows = sqlx::query("SELECT command_id, table_name, status, attempts, rejection FROM sync_outbox WHERE scope = $1 ORDER BY sequence DESC LIMIT 100")
            .bind(&self.scope).fetch_all(&self.pool).await.map_err(storage_error)?;
        rows.into_iter().map(|row| Ok(CommandView {
            command_id: row.try_get("command_id").map_err(storage_error)?,
            table: row.try_get("table_name").map_err(storage_error)?,
            status: row.try_get("status").map_err(storage_error)?,
            attempts: u32::try_from(row.try_get::<i64, _>("attempts").map_err(storage_error)?)
                .map_err(|_| CoreError::Terminal("invalid replay attempt count".into()))?,
            reason: row.try_get("rejection").map_err(storage_error)?,
        })).collect()
    }

    pub(super) async fn recover_command(&self, id: &str) -> CoreResult<()> {
        let mut tx = self.pool.begin().await.map_err(storage_error)?;
        self.lock_projection(&mut tx).await?;
        sqlx::query("UPDATE sync_outbox SET status = 'pending', attempts = 0, rejection = NULL, settled_at = NULL WHERE scope = $1 AND command_id = $2 AND status = 'poison'")
            .bind(&self.scope).bind(id).execute(&mut *tx).await.map_err(storage_error)?;
        tx.commit().await.map_err(storage_error)
    }
}
