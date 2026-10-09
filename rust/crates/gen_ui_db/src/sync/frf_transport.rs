// TJ-ARCH-MOB-001 compliant
//! Fabric-backed local-first transport. Production business reads use selected
//! server-catalog shapes through Gate's authorized Fabric facade. Each source
//! persists its opaque cursor with the projection transaction; Forge owns writes.
//! The legacy spine payload decoder remains available to existing callers.
use async_trait::async_trait;
use gen_ui_types::error::{CoreError, CoreResult};
use gen_ui_types::sync::{SyncStatus, SyncTransport};
use std::sync::Arc;

use super::seam::{LocalStore, RowChange, RowOp};
use super::status::SyncStatusHandle;
use super::write_queue::WriteQueue;

/// Where the FRF spine lives and which channel carries our row changes.
#[derive(Debug, Clone)]
pub struct FrfSyncConfig {
    /// Spine endpoint (gRPC), e.g. `http://localhost:29090`.
    pub endpoint: String,
    /// Gate-minted Bearer the SDK's `AuthInterceptor` attaches to every RPC.
    pub token: Option<String>,
    /// Tenant scope — FRF carries this on every message; we refuse cross-tenant rows.
    pub tenant_id: String,
    /// Spine channel `frf-postgres-cdc` publishes onto (its `CdcConfig::channel_path`).
    pub channel_path: String,
    /// Durable consumer identity. MUST be stable across restarts, or the spine replays
    /// the whole channel from the start on every boot.
    pub consumer_id: String,
    /// Max writes to flush per drain pass.
    pub write_batch: usize,
    /// Failed replays before a write is quarantined as poison.
    pub max_write_attempts: u32,
}

/// One decoded CDC row change as it appears in an `EventEnvelope.payload`.
///
/// A local mirror of `frf_domain::EntityChange`'s wire shape rather than a dependency on
/// the type: this crate must compile with the `frf` feature OFF (and for wasm32, where
/// the FRF SDK does not build at all), so the decode path cannot name FRF types. The
/// fields below are the frozen `proto-v1` contract — verified against
/// `frf-domain/src/entity.rs` at rev 9ba04ae.
#[derive(Debug, Clone, serde::Deserialize)]
struct EntityChangeWire {
    entity_id: String,
    tenant_id: String,
    /// Maps to our `RowChange::table` — FRF's entity type IS the table name for
    /// CDC-sourced changes (`frf-postgres-cdc` fills it from the pgoutput relation).
    entity_type: String,
    op: String,
    data: serde_json::Value,
}

/// Decode one spine payload into a [`RowChange`].
///
/// Returns `Ok(None)` for changes that are not ours to apply (foreign tenant) — those
/// are skipped, not errors. A malformed payload IS an error: silently dropping it would
/// lose a row with no trace.
///
/// Deliberately NOT gated on `feature = "frf"`: this is the wire contract, and it is the
/// part of the read lane that can be compiled and tested without the private SDK. Gating
/// it would mean the only untested code is the code nobody can build — exactly backwards.
pub fn row_change_from_payload(
    payload: &serde_json::Value,
    our_tenant: &str,
) -> CoreResult<Option<RowChange>> {
    let change: EntityChangeWire = serde_json::from_value(payload.clone())
        .map_err(|e| CoreError::Serde(format!("cdc payload: {e}")))?;

    // Defence in depth. FRF enforces tenant scoping server-side, but a client that
    // applies whatever arrives would turn any spine misconfiguration into cross-tenant
    // data landing in a local DB — the kind of bug that is invisible until it is a
    // breach. Cheap check; keep it.
    if change.tenant_id != our_tenant {
        tracing::warn!(
            expected = %our_tenant, got = %change.tenant_id,
            "dropping cross-tenant row change"
        );
        return Ok(None);
    }

    // `ChangeOp` is snake_case and #[non_exhaustive] upstream — match on the wire string
    // and treat anything unknown as terminal rather than guessing. `upsert` collapses
    // into Insert because LocalStore's insert path is already an upsert.
    let op = match change.op.as_str() {
        "insert" | "upsert" => RowOp::Insert,
        "update" => RowOp::Update,
        "delete" => RowOp::Delete,
        other => {
            return Err(CoreError::Terminal(format!("unknown change op: {other}")));
        }
    };

    Ok(Some(RowChange {
        table: change.entity_type,
        op,
        key: change.entity_id,
        value_json: change.data.to_string(),
    }))
}

/// [`SyncTransport`] over the FRF spine.
pub struct FrfSyncTransport {
    cfg: FrfSyncConfig,
    store: Arc<dyn LocalStore>,
    queue: Arc<WriteQueue>,
    status: SyncStatusHandle,
    read_sources: Vec<Arc<dyn gen_ui_types::sync::AuthorizedReadSource>>,
    stop: tokio::sync::watch::Sender<bool>,
    started: std::sync::atomic::AtomicBool,
}

impl FrfSyncTransport {
    pub fn new(
        cfg: FrfSyncConfig,
        store: Arc<dyn LocalStore>,
        sink: Arc<dyn super::seam::WriteSink>,
    ) -> Self {
        let status = SyncStatusHandle::new();
        // WriteQueue is Electric-agnostic — it only reads write_batch/max_write_attempts
        // off SyncConfig. Reuse it rather than fork a second queue (Rule 2/3).
        let queue_cfg = super::config::SyncConfig {
            electric_url: String::new(), // unused on the FRF write path
            shapes: Vec::new(),
            write_batch: cfg.write_batch,
            max_write_attempts: cfg.max_write_attempts,
        };
        let queue = Arc::new(WriteQueue::durable(&queue_cfg, sink, status.clone(), Arc::clone(&store)));
        Self {
            cfg,
            store,
            queue,
            status,
            read_sources: Vec::new(),
            stop: tokio::sync::watch::channel(false).0,
            started: std::sync::atomic::AtomicBool::new(false),
        }
    }

    pub fn with_read_sources(mut self, sources: Vec<Arc<dyn gen_ui_types::sync::AuthorizedReadSource>>) -> CoreResult<Self> {
        let mut tables = std::collections::HashSet::new();
        for source in &sources {
            if !tables.insert(source.table().to_owned()) {
                return Err(CoreError::Terminal("only one selected shape may own each local business table".into()));
            }
        }
        self.read_sources = sources;
        Ok(self)
    }

    pub fn stop(&self) { self.stop.send_replace(true); }

    pub async fn command_history(&self) -> CoreResult<Vec<super::CommandView>> { self.store.command_history().await }
    pub async fn retry_command(&self, id: &str) -> CoreResult<()> { self.store.retry_command(id).await }

    /// Subscribe to [`SyncStatus`] transitions for the UI sync chip.
    pub fn status_stream(&self) -> super::status::SyncStatusStream {
        self.status.subscribe()
    }

    /// Quarantined writes — for UI surfacing and manual retry tooling.
    pub async fn poison_writes(&self) -> Vec<super::seam::PendingWrite> {
        self.queue.poison_writes().await
    }

    /// Decode one spine payload and apply it to the local store.
    ///
    /// This is the whole read-lane body: the SDK subscribe loop is a thin driver that
    /// pumps `EventEnvelope.payload` through here. Keeping it separate from the SDK call
    /// is what makes the read path testable at all — the SDK itself is a private cross-org
    /// dep CI cannot fetch, so anything welded to it is untestable by construction.
    ///
    /// A foreign-tenant row is applied as a no-op (see `row_change_from_payload`).
    pub async fn apply_envelope_payload(&self, payload: &serde_json::Value) -> CoreResult<()> {
        let Some(change) = row_change_from_payload(payload, &self.cfg.tenant_id)? else {
            return Ok(());
        };
        // One row per batch: the spine delivers changes individually, and inventing a
        // batching window here would trade correctness (a partially-applied window on
        // crash) for a throughput win this PoC has no evidence it needs.
        self.store.apply_batch(&[change]).await
    }
}

#[async_trait]
impl SyncTransport for FrfSyncTransport {
    async fn start(&self) -> CoreResult<()> {
        if self.started.swap(true, std::sync::atomic::Ordering::SeqCst) {
            return Err(CoreError::Terminal("sync transport already started".into()));
        }
        self.queue.initialize().await?;
        self.status.set(SyncStatus::Syncing {
            pending_writes: self.status.pending(),
        });

        // Write-queue drain loop — identical to the Electric engine's, because the write
        // path did not change with the substrate.
        let queue = Arc::clone(&self.queue);
        let mut stop = self.stop.subscribe();
        gen_ui_runtime::spawn(async move {
            loop {
                if *stop.borrow() { return; }
                tokio::select! {
                    _ = stop.changed() => return,
                    _ = async {
                        if queue.drain().await == 0 {
                            tokio::time::sleep(std::time::Duration::from_millis(250)).await;
                        }
                    } => {}
                }
            }
        });

        self.start_read_lane()
    }

    /// Same contract as the Electric engine's: `change_json` carries the mutation and
    /// the idempotency key is caller-supplied or allocated in the durable local
    /// transaction, so a replayed write dedupes server-side.
    async fn enqueue_write(&self, change_json: &str) -> CoreResult<()> {
        let parsed: serde_json::Value =
            serde_json::from_str(change_json).map_err(|e| CoreError::Serde(e.to_string()))?;
        let table = parsed
            .get("table")
            .and_then(|v| v.as_str())
            .ok_or_else(|| CoreError::Terminal("write: missing \"table\"".into()))?
            .to_string();
        let idempotency_key = parsed
            .get("idempotency_key")
            .and_then(|v| v.as_str())
            .map(str::to_string)
            .unwrap_or_default();

        self.queue
            .enqueue(super::seam::PendingWrite {
                idempotency_key,
                table,
                change_json: change_json.to_string(),
                attempts: 0,
            })
            .await?;
        Ok(())
    }

    fn status(&self) -> SyncStatus {
        self.status.current()
    }
}

impl FrfSyncTransport {
    fn start_read_lane(&self) -> CoreResult<()> {
        if self.read_sources.is_empty() {
            self.status.set(SyncStatus::Offline);
            return Ok(());
        }
        for source in &self.read_sources {
            self.status.set_read(source.shape_id(), SyncStatus::Syncing { pending_writes: self.status.pending() });
        }
        for source in &self.read_sources {
            gen_ui_runtime::spawn(super::read_lane::run(
                Arc::clone(source), Arc::clone(&self.store), self.status.clone(), self.stop.subscribe(),
            ));
        }
        Ok(())
    }
}

impl Drop for FrfSyncTransport {
    fn drop(&mut self) { self.stop.send_replace(true); }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::sync::Mutex;

    /// Records what the read lane applied. A fake at the real IO boundary (the store),
    /// not a mock of internal code — per CLAUDE.md's testing rules.
    #[derive(Default)]
    struct SpyStore {
        applied: Mutex<Vec<RowChange>>,
    }

    #[async_trait]
    impl LocalStore for SpyStore {
        async fn apply_batch(&self, changes: &[RowChange]) -> CoreResult<()> {
            self.applied.lock().unwrap().extend_from_slice(changes);
            Ok(())
        }
        async fn truncate_shape(&self, _table: &str) -> CoreResult<()> {
            Ok(())
        }
    }

    /// Never-called sink: these tests exercise the READ lane only.
    struct NoopSink;

    #[async_trait]
    impl super::super::seam::WriteSink for NoopSink {
        async fn send(
            &self,
            _w: &super::super::seam::PendingWrite,
        ) -> super::super::seam::WriteOutcome {
            super::super::seam::WriteOutcome::Applied
        }
    }

    fn transport(tenant: &str) -> (FrfSyncTransport, Arc<SpyStore>) {
        let store = Arc::new(SpyStore::default());
        let cfg = FrfSyncConfig {
            endpoint: "http://localhost:29090".into(),
            token: None,
            tenant_id: tenant.into(),
            channel_path: "entity/changes".into(),
            consumer_id: "test".into(),
            write_batch: 8,
            max_write_attempts: 3,
        };
        let t = FrfSyncTransport::new(
            cfg,
            Arc::clone(&store) as Arc<dyn LocalStore>,
            Arc::new(NoopSink),
        );
        (t, store)
    }

    fn payload(op: &str, tenant: &str) -> serde_json::Value {
        json!({
            "entity_id": "11111111-1111-1111-1111-111111111111",
            "tenant_id": tenant,
            "entity_type": "notes",
            "op": op,
            "data": { "id": "11111111-1111-1111-1111-111111111111", "title": "hi" },
            "previous": null,
            "session_id": null,
            "timestamp": "2026-07-16T00:00:00Z",
            "version": 1
        })
    }

    #[test]
    fn decodes_insert_into_a_row_change() {
        let rc = row_change_from_payload(&payload("insert", "t1"), "t1")
            .unwrap()
            .expect("ours");
        assert_eq!(rc.table, "notes");
        assert_eq!(rc.op, RowOp::Insert);
        assert_eq!(rc.key, "11111111-1111-1111-1111-111111111111");
    }

    #[test]
    fn upsert_collapses_to_insert() {
        // LocalStore's insert path is already an upsert, so both must land the same way.
        let rc = row_change_from_payload(&payload("upsert", "t1"), "t1")
            .unwrap()
            .unwrap();
        assert_eq!(rc.op, RowOp::Insert);
    }

    #[test]
    fn drops_cross_tenant_rows_without_erroring() {
        // Skipped, not fatal: a foreign row is not a corrupt stream.
        let out = row_change_from_payload(&payload("insert", "other-tenant"), "t1").unwrap();
        assert!(out.is_none());
    }

    #[test]
    fn unknown_op_is_terminal_not_silently_skipped() {
        // ChangeOp is #[non_exhaustive] upstream — a new variant must fail loudly here
        // rather than be guessed at.
        let err = row_change_from_payload(&payload("truncate", "t1"), "t1").unwrap_err();
        assert!(matches!(err, CoreError::Terminal(_)), "got {err:?}");
    }

    #[test]
    fn malformed_payload_is_an_error_not_a_dropped_row() {
        let err = row_change_from_payload(&json!({ "nope": true }), "t1").unwrap_err();
        assert!(matches!(err, CoreError::Serde(_)), "got {err:?}");
    }

    // ── the read lane end-to-end (decode → store), sans SDK ──────────────────────

    #[tokio::test]
    async fn applies_an_own_tenant_row_to_the_store() {
        let (t, store) = transport("t1");
        t.apply_envelope_payload(&payload("insert", "t1"))
            .await
            .unwrap();

        let applied = store.applied.lock().unwrap();
        assert_eq!(applied.len(), 1);
        assert_eq!(applied[0].table, "notes");
        assert_eq!(applied[0].op, RowOp::Insert);
    }

    #[tokio::test]
    async fn cross_tenant_row_never_reaches_the_store() {
        // The check that matters: a foreign row must not be written locally, even
        // though decoding it succeeds.
        let (t, store) = transport("t1");
        t.apply_envelope_payload(&payload("insert", "attacker"))
            .await
            .unwrap();
        assert!(store.applied.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn malformed_payload_does_not_touch_the_store() {
        let (t, store) = transport("t1");
        assert!(t
            .apply_envelope_payload(&json!({ "nope": true }))
            .await
            .is_err());
        assert!(store.applied.lock().unwrap().is_empty());
    }
}
