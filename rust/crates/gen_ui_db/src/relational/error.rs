// TJ-ARCH-MOB-001 compliant
//! Typed errors for the relational library boundary.

#[derive(Debug, thiserror::Error)]
pub enum RelationalError {
    #[error("database operation failed: {0}")]
    Database(#[from] sqlx::Error),
    #[error("migration failed: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),
    #[error("seed bundle {name} could not be fetched: {source}")]
    SeedFetch {
        name: String,
        source: reqwest::Error,
    },
    #[error("seed bundle {name} has invalid UTF-8: {source}")]
    SeedEncoding {
        name: String,
        source: std::str::Utf8Error,
    },
    #[error("seed bundle {name} has an empty IPFS CID")]
    EmptyCid { name: String },
    #[error("sync attach failed: {0}")]
    Sync(String),
    #[cfg(feature = "pglite")]
    #[error("embedded PGlite server failed: {0}")]
    PgliteServer(#[from] anyhow::Error),
}

pub type RelationalResult<T> = Result<T, RelationalError>;
