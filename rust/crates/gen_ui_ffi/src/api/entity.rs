// TJ-ARCH-MOB-001 compliant
//! Entity CRUD intent surface. Mirrors the `EntityTransport` seam 1:1 so the Dart
//! `prometheus_entity_management` package drives a persistent SQLite store without
//! owning SQL or database lifecycle in Dart.
// `pub use` (not `use`) — frb_generated.rs re-exports this module's types via
// `use crate::api::entity::*`, which only sees items visible through a public path.
//
// frb's Result<T,E> detection only matches a literal `Result<...>` return
// type — it does NOT resolve through a generic type alias (see chat.rs's
// note). Every frb-exposed fn below spells out `Result<T, CoreError>`.
use gen_ui_types::transport::EntityTransport;
pub use gen_ui_types::transport::{EntityRecord, ListResult};
pub use gen_ui_types::view::ViewDescriptor;
pub use gen_ui_types::CoreError;
use once_cell::sync::OnceCell;
use std::path::PathBuf;

static STORE: OnceCell<gen_ui_db::relational::SqliteEntityStore> = OnceCell::new();

pub(crate) async fn init_store(path: PathBuf) -> Result<(), CoreError> {
    if STORE.get().is_some() {
        return Ok(());
    }
    let store = gen_ui_db::relational::SqliteEntityStore::open(path).await?;
    let _ = STORE.set(store);
    Ok(())
}

fn store() -> Result<&'static gen_ui_db::relational::SqliteEntityStore, CoreError> {
    STORE
        .get()
        .ok_or_else(|| CoreError::Terminal("mobile entity store is not initialized".into()))
}

/// List entities matching a view (filters/sorts/pagination compiled to SQL in Rust).
pub async fn entity_list(view: ViewDescriptor) -> Result<ListResult, CoreError> {
    store()?.list(&view).await
}

/// Fetch one entity by type + id.
pub async fn entity_get(
    entity_type: String,
    id: String,
) -> Result<Option<EntityRecord>, CoreError> {
    store()?.get(&entity_type, &id).await
}

/// Create an entity. Emits a ChangeEvent::Upsert on the entity_changes stream.
pub async fn entity_create(record: EntityRecord) -> Result<EntityRecord, CoreError> {
    store()?.create(&record).await
}

/// Update an entity. Emits a ChangeEvent::Upsert.
pub async fn entity_update(record: EntityRecord) -> Result<EntityRecord, CoreError> {
    store()?.update(&record).await
}

/// Delete an entity. Emits a ChangeEvent::Delete.
pub async fn entity_delete(entity_type: String, id: String) -> Result<(), CoreError> {
    store()?.delete(&entity_type, &id).await
}
