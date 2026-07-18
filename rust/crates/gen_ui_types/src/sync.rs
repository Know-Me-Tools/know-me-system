// TJ-ARCH-MOB-001 compliant
//! SyncTransport — local-first sync seam. The DIY Electric-consumer + write-queue
//! (gen_ui_db::sync) implements this; a future prometheus-entity-sync (PES) client
//! can implement the same trait without touching callers.
use crate::error::CoreResult;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SyncStatus {
    Offline,
    Syncing { pending_writes: u32 },
    Live,
    Error { message: String },
}

#[async_trait]
pub trait SyncTransport: Send + Sync {
    /// Begin read-path sync for a shape/bucket, writing into the local store.
    async fn start(&self) -> CoreResult<()>;
    /// Enqueue a local write for replay through the server API.
    async fn enqueue_write(&self, change_json: &str) -> CoreResult<()>;
    /// Current status (drives the UI sync chip).
    fn status(&self) -> SyncStatus;
}
