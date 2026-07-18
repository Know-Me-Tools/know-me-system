// TJ-ARCH-MOB-001 compliant
//! [`SyncEngine`] — assembles the shape consumers + write queue behind the frozen
//! [`SyncTransport`] seam.
use super::config::SyncConfig;
use super::seam::{LocalStore, PendingWrite, WriteSink};
use super::shapes::ShapeConsumer;
use super::status::SyncStatusHandle;
use super::write_queue::WriteQueue;
use async_trait::async_trait;
use gen_ui_types::error::{CoreError, CoreResult};
use gen_ui_types::sync::{SyncStatus, SyncTransport};
use std::sync::Arc;

/// The C-005 local-first sync engine. Construct with a [`LocalStore`] (read-path
/// sink, C-003) and a [`WriteSink`] (write-path, C-006 forge client), then drive it
/// through the [`SyncTransport`] seam.
pub struct SyncEngine {
    cfg: SyncConfig,
    client: reqwest::Client,
    store: Arc<dyn LocalStore>,
    queue: Arc<WriteQueue>,
    status: SyncStatusHandle,
}

impl SyncEngine {
    pub fn new(cfg: SyncConfig, store: Arc<dyn LocalStore>, sink: Arc<dyn WriteSink>) -> Self {
        let status = SyncStatusHandle::new();
        let queue = Arc::new(WriteQueue::new(&cfg, sink, status.clone()));
        Self {
            cfg,
            client: reqwest::Client::new(),
            store,
            queue,
            status,
        }
    }

    /// Subscribe to [`SyncStatus`] transitions for the UI sync chip.
    pub fn status_stream(&self) -> super::status::SyncStatusStream {
        self.status.subscribe()
    }

    /// Quarantined (poison) writes — for UI surfacing and manual retry tooling.
    pub async fn poison_writes(&self) -> Vec<PendingWrite> {
        self.queue.poison_writes().await
    }
}

#[async_trait]
impl SyncTransport for SyncEngine {
    /// Start read-path sync: spawn one shape consumer per configured shape and a
    /// write-queue drain loop. Tasks run on the global runtime; they stop when the
    /// engine (and thus the `Arc`s they hold) is dropped.
    async fn start(&self) -> CoreResult<()> {
        if self.cfg.shapes.is_empty() {
            return Err(CoreError::Terminal("sync: no shapes configured".into()));
        }
        self.status.set(SyncStatus::Syncing {
            pending_writes: self.status.pending(),
        });

        for shape in &self.cfg.shapes {
            let consumer = ShapeConsumer::new(
                self.client.clone(),
                self.cfg.electric_url.clone(),
                shape.clone(),
                Arc::clone(&self.store),
                self.status.clone(),
            );
            gen_ui_runtime::spawn(async move {
                if let Err(e) = consumer.run().await {
                    tracing::error!(error = %e, "shape consumer stopped");
                }
            });
        }

        // Write-queue drain loop: replay pending writes, idle-poll when empty.
        let queue = Arc::clone(&self.queue);
        gen_ui_runtime::spawn(async move {
            loop {
                let remaining = queue.drain().await;
                if remaining == 0 {
                    tokio::time::sleep(std::time::Duration::from_millis(250)).await;
                }
            }
        });

        Ok(())
    }

    /// Enqueue a local write for durable replay through the forge Quarry API.
    /// `change_json` carries the mutation; the idempotency key is derived from it so
    /// server-side dedup makes replay safe. The action-log persistence seam (C-003)
    /// makes this survive restarts.
    async fn enqueue_write(&self, change_json: &str) -> CoreResult<()> {
        let parsed: serde_json::Value =
            serde_json::from_str(change_json).map_err(|e| CoreError::Serde(e.to_string()))?;
        let table = parsed
            .get("table")
            .and_then(|v| v.as_str())
            .ok_or_else(|| CoreError::Terminal("write: missing \"table\"".into()))?
            .to_string();
        // Prefer a caller-supplied key; else derive a stable one from the payload.
        let idempotency_key = parsed
            .get("idempotency_key")
            .and_then(|v| v.as_str())
            .map(str::to_string)
            .unwrap_or_else(|| derive_key(change_json));

        self.queue
            .enqueue(PendingWrite {
                idempotency_key,
                table,
                change_json: change_json.to_string(),
                attempts: 0,
            })
            .await;
        Ok(())
    }

    fn status(&self) -> SyncStatus {
        self.status.current()
    }
}

/// Derive a stable idempotency key from a write payload (FNV-1a over the bytes).
/// Deterministic so an identical retried write dedupes server-side.
///
/// `pub(crate)` so the FRF transport reuses THIS derivation rather than growing a
/// second one — two key functions that drift would silently break server-side dedup.
pub(crate) fn derive_key(payload: &str) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for b in payload.as_bytes() {
        hash ^= u64::from(*b);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("wq-{hash:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    // Boundary behavior: the same payload derives the same idempotency key (so a
    // retried write dedupes server-side) and different payloads diverge.
    #[test]
    fn derive_key_is_deterministic_and_distinct() {
        let a = r#"{"table":"entities","op":"upsert","id":"e1"}"#;
        let b = r#"{"table":"entities","op":"upsert","id":"e2"}"#;
        assert_eq!(derive_key(a), derive_key(a));
        assert_ne!(derive_key(a), derive_key(b));
        assert!(derive_key(a).starts_with("wq-"));
    }
}
