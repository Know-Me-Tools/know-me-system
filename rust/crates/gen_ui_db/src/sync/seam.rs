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
