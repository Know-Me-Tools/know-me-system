// TJ-ARCH-MOB-001 compliant
//! gen_ui_db (L2) — relational (pg/sqlite) + sync + startup orchestrator.
//! Graph RAG lives in the sibling gen_ui_db_graph crate (C-004).
//! Trait seams (EntityTransport / SyncTransport) are defined in gen_ui_types.
//!
//! Module ownership (parallel worktree lanes, see plan.md):
//!   relational  → C-003 (sqlx pg/sqlite + migrations + typestate startup)
//!   sync        → C-005 (Electric shape consumer + DIY write queue)
//!
//! The sync engine writes read-path rows into a `sync::LocalStore` and flushes the
//! local write queue through a `sync::WriteSink` (forge Quarry API). Both are trait
//! seams so C-003's concrete store and C-006's forge client wire in without sync
//! depending on their crates — and a future prometheus-entity-sync (PES / PSyncV1)
//! client can replace the whole engine behind the same `SyncTransport` seam.

pub mod relational;
pub mod sync;
