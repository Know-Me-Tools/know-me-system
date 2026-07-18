// TJ-ARCH-MOB-001 compliant
//! EntityTransport — the entity data-access seam. Implemented per entity type in
//! gen_ui_db / gen_ui_client; exposed to Dart via gen_ui_ffi. UI never implements it.
use crate::error::CoreResult;
use crate::view::ViewDescriptor;
use async_trait::async_trait;
use flutter_rust_bridge_macros::frb;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EntityRecord {
    pub id: String,
    pub entity_type: String,
    pub data_json: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ListResult {
    pub items: Vec<EntityRecord>,
    pub next_cursor: Option<String>,
}

// frb only keeps foreign types reachable through a `pub fn` in the codegen
// root's own crate; `entity_changes`'s `Stream<ChangeEvent>` return in
// gen_ui_ffi::api::streams doesn't connect back far enough through frb's
// generic-stream analysis, so ChangeEvent was silently dropped ("ignored
// because ... neither used by any pub functions") without this explicit
// unignore marker. (flutter_rust_bridge_macros, not the full flutter_rust_bridge
// runtime crate — proc-macro-only, no wasm/runtime surface, see Cargo.toml.)
//
// KNOWN GAP: even unignored, this enum still generates as an opaque
// RustOpaqueInterface (no Dart-visible variants/fields) rather than a
// transparent sealed class — struct-like enum variants appear to need
// different handling than the plain structs (EntityRecord, MemoryHit, ...)
// that DO generate transparently. Not investigated further since
// entity_changes/ChangeEvent has no real producer yet (C-003's stub, per the
// doc comment on entity_changes in gen_ui_ffi::api::streams) — nothing
// currently depends on Dart being able to read a ChangeEvent's fields. Revisit
// when C-003 actually wires a producer.
#[frb(unignore)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum ChangeEvent {
    Upsert {
        record: EntityRecord,
    },
    Delete {
        entity_type: String,
        id: String,
    },
    Invalidate {
        entity_type: String,
        list_key: Option<String>,
    },
}

#[async_trait]
pub trait EntityTransport: Send + Sync {
    async fn list(&self, view: &ViewDescriptor) -> CoreResult<ListResult>;
    async fn get(&self, entity_type: &str, id: &str) -> CoreResult<Option<EntityRecord>>;
    async fn create(&self, record: &EntityRecord) -> CoreResult<EntityRecord>;
    async fn update(&self, record: &EntityRecord) -> CoreResult<EntityRecord>;
    async fn delete(&self, entity_type: &str, id: &str) -> CoreResult<()>;
}
