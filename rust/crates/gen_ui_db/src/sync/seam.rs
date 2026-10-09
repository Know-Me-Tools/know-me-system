// TJ-ARCH-MOB-001 compliant
//! Seams the sync engine writes through — so it depends on neither the concrete
//! relational store (C-003) nor the forge client (C-006), and PES can reuse them.
use async_trait::async_trait;
use gen_ui_types::error::CoreResult;
use serde::{Deserialize, Serialize};

/// A single decoded row change — from an FRF spine `EntityChange` (current) or an
/// Electric shape message (legacy lane). Substrate-neutral by design: it is the seam
/// both read lanes converge on before touching a [`LocalStore`].
#[derive(Debug, Clone, PartialEq)]
pub struct RowChange {
    pub table: String,
    pub op: RowOp,
    /// Primary-key value(s) as JSON (the shape's `key`).
    pub key: String,
    /// Full row value as a JSON object (empty for deletes).
    pub value_json: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowOp {
    Insert,
    Update,
    Delete,
}

/// Read-path sink: the shape consumer applies decoded rows here. C-003's sqlx store
/// implements this against SQLite (mobile) / pglite-oxide (desktop). A single
/// `apply_batch` per transaction keeps the local DB consistent with a shape
/// message boundary.
#[async_trait]
pub trait LocalStore: Send + Sync {
    /// Apply a batch of row changes atomically (one local transaction).
    async fn apply_batch(&self, changes: &[RowChange]) -> CoreResult<()>;

    /// Wipe a table's synced rows — used on `must-refetch` (shape rotation) so the
    /// consumer can re-materialise the shape from offset `-1` without duplicates.
    async fn truncate_shape(&self, table: &str) -> CoreResult<()>;

    /// Open the durable command log before starting replay. Stores without this
    /// capability must not silently advertise durable offline writes.
    async fn initialize_outbox(&self) -> CoreResult<()> {
        Err(gen_ui_types::CoreError::Terminal("local store has no durable outbox".into()))
    }

    /// Commit the local projection and command intent in the same transaction.
    async fn enqueue_pending(&self, _write: &PendingWrite) -> CoreResult<()> {
        Err(gen_ui_types::CoreError::Terminal("local store has no durable outbox".into()))
    }

    async fn queued_writes(&self, _poison: bool) -> CoreResult<Vec<PendingWrite>> {
        Err(gen_ui_types::CoreError::Terminal("local store has no durable outbox".into()))
    }

    /// Retain the committed result or refusal before removing work from replay.
    async fn settle_write(&self, _write: &PendingWrite, _outcome: &WriteOutcome) -> CoreResult<()> {
        Err(gen_ui_types::CoreError::Terminal("local store has no durable outbox".into()))
    }

    async fn read_cursor(&self, _shape: &str) -> CoreResult<Option<gen_ui_types::sync::ReadCursor>> {
        Err(gen_ui_types::CoreError::Terminal("local store has no durable read cursor".into()))
    }

    async fn commit_read(&self, _shape: &str, _table: &str, _previous: Option<&gen_ui_types::sync::ReadCursor>, _poll: &gen_ui_types::sync::ReadPoll) -> CoreResult<bool> {
        Err(gen_ui_types::CoreError::Terminal("local store has no durable read cursor".into()))
    }

    async fn command_history(&self) -> CoreResult<Vec<CommandView>> {
        Err(gen_ui_types::CoreError::Terminal("local store has no command history".into()))
    }

    async fn retry_command(&self, _id: &str) -> CoreResult<()> {
        Err(gen_ui_types::CoreError::Terminal("local store has no command recovery".into()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandView {
    pub command_id: String,
    pub table: String,
    pub status: String,
    pub attempts: u32,
    pub reason: Option<String>,
}

/// One durable, idempotent local mutation awaiting replay to the server.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PendingWrite {
    /// Idempotency key — the server dedupes retries on this. Stable across replays.
    pub idempotency_key: String,
    /// Target table / entity type.
    pub table: String,
    /// The mutation as JSON (op + payload); opaque to the queue, meaningful to the sink.
    pub change_json: String,
    /// How many replay attempts have already failed.
    pub attempts: u32,
}

/// The result of attempting to replay one write through the server.
#[derive(Debug, Clone, PartialEq)]
pub enum WriteOutcome {
    /// Server accepted (or idempotently deduped) the write — drop it from the queue.
    Applied,
    /// Authoritative row returned by Forge's transaction receipt (including replay).
    Committed { change: RowChange },
    /// Transient failure (network / 5xx / conflict-retryable) — keep and back off.
    Retry,
    /// Terminal rejection (4xx validation) — quarantine as poison, stop retrying.
    Poison { reason: String },
}

/// Write-path sink: the queue flushes pending writes here. C-006's forge client
/// implements this against the Quarry REST/GraphQL API under RLS. The impl MUST
/// forward `idempotency_key` so server-side dedup makes replay safe.
#[async_trait]
pub trait WriteSink: Send + Sync {
    async fn send(&self, write: &PendingWrite) -> WriteOutcome;
}
