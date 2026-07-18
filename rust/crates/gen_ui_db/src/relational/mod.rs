// TJ-ARCH-MOB-001 compliant
//! Feature-gated relational storage and ordered application startup.
//! Postgres (`pg`; add `pglite` for embedded desktop PG via pglite-oxide) backs
//! desktop/hosted data. SQLite backs mobile entity envelopes while SurrealDB
//! remains the mobile memory/graph-RAG store.

pub mod config;
mod error;
mod seed;
#[cfg(feature = "sqlite")]
mod sqlite_entity;
mod startup;

#[cfg(feature = "pg")]
mod postgres;

pub use config::{AppSetting, ConfigStore, ModelPref, Provider};
pub use error::{RelationalError, RelationalResult};
#[cfg(feature = "pglite")]
pub use postgres::PgliteStore;
#[cfg(feature = "pg")]
pub use postgres::PostgresStore;
pub use seed::{SeedBundle, SeedSource};
#[cfg(feature = "sqlite")]
pub use sqlite_entity::SqliteEntityStore;
pub use startup::{Migrated, Ready, Startup, Uninitialized};
