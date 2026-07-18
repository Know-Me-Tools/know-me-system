// TJ-ARCH-MOB-001 compliant
//! Local-first sync engine (C-005 seams; C-106 FRF substrate).
//!
//! **Substrate: flint-realtime-fabric (FRF). ElectricSQL was dropped 2026-07-16** — FRF
//! reads `pgoutput` off its own replication slot, i.e. the same Postgres mechanism
//! Electric consumes, so the two are alternatives, not layers. See `sync/README.md` for
//! the full picture and the per-platform status.
//!
//! Read path (desktop): CDC → Iggy spine channel → [`FrfSyncTransport`] → [`LocalStore`].
//! Write path (all native): local mutation → **write queue** (idempotent keys,
//! exponential backoff, poison handler) → [`WriteSink`] → forge/Quarry → Postgres, which
//! the WAL picks up and fans back out. Writes deliberately do NOT go to the spine: it
//! persists nothing, so they would fan out once and then vanish.
//!
//! A [`SyncStatus`] broadcast drives the UI sync chip.
//!
//! [`SyncEngine`] is the **legacy Electric lane**, superseded by [`FrfSyncTransport`].
//!
//! Mobile's read lane is NOT wired: it has no Postgres (embedded SurrealDB is both config
//! and memory backend there), so [`PgLocalStore`] cannot serve it — see README/C-106 T5.
//!
//! Web path: this module compiles to a documented no-op on `wasm32` (tonic/HTTP-2 does
//! not build for the browser) so the workspace still builds for the browser leaf.
//!
//! Everything sits behind the frozen [`gen_ui_types::sync::SyncTransport`] seam, so
//! PES (PSyncV1) can replace the engine later without touching callers.

mod status;

#[cfg(not(target_arch = "wasm32"))]
mod config;
#[cfg(not(target_arch = "wasm32"))]
mod engine;
#[cfg(not(target_arch = "wasm32"))]
mod frf_transport;
#[cfg(not(target_arch = "wasm32"))]
mod seam;
#[cfg(not(target_arch = "wasm32"))]
mod shapes;
#[cfg(not(target_arch = "wasm32"))]
mod write_queue;
// Needs sqlx::PgPool → `pg`. Desktop/web only; mobile's local store is SurrealDB
// (gen_ui_db_graph), matching how `relational` is gated.
#[cfg(all(not(target_arch = "wasm32"), feature = "pg"))]
mod local_store;

pub use status::{SyncStatusHandle, SyncStatusStream};
// Re-export the frozen seam types so callers use one import path.
pub use gen_ui_types::sync::{SyncStatus, SyncTransport};

#[cfg(not(target_arch = "wasm32"))]
pub use config::{ShapeSpec, SyncConfig};
#[cfg(not(target_arch = "wasm32"))]
pub use engine::SyncEngine;
// C-106: the FRF-backed transport (the substrate the PoC actually runs) + the concrete
// read-path store. `SyncEngine` above stays for the Electric lane until it is removed.
#[cfg(not(target_arch = "wasm32"))]
pub use frf_transport::{row_change_from_payload, FrfSyncConfig, FrfSyncTransport};
#[cfg(all(not(target_arch = "wasm32"), feature = "pg"))]
pub use local_store::PgLocalStore;
#[cfg(not(target_arch = "wasm32"))]
pub use seam::{LocalStore, PendingWrite, RowChange, RowOp, WriteOutcome, WriteSink};

/// wasm32 stub. No Rust engine runs in-browser (tonic/HTTP-2 does not build for
/// wasm32); FRF's browser story is frf-wasm / Connect-web from the JS side. Kept so
/// `cargo check --target wasm32-unknown-unknown -p gen_ui_wasm` stays green and callers
/// get a clear compile error if they try to build the native engine for the web.
#[cfg(target_arch = "wasm32")]
pub mod web_note {
    //! Browser sync is reached JS-side (frf-wasm / Connect-web), not from Rust.
    //! See `crates/gen_ui_db/src/sync/README.md`.
}
