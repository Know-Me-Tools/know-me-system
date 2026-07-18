// TJ-ARCH-MOB-001 compliant
//! PostgreSQL store for cloud Postgres and pglite-oxide's local wire server.

// `Path` is only used by the pglite embedded-server constructor below; gating the
// import keeps a plain `pg` build (no `pglite`) warning-free under the clippy gate.
#[cfg(feature = "pglite")]
use std::path::Path;
#[cfg(feature = "pglite")]
use std::sync::Arc;

use sqlx::{postgres::PgPoolOptions, PgPool};

#[cfg(feature = "pglite")]
use tokio::sync::OnceCell;

use super::{startup::StartupStore, RelationalResult};

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations/postgres");

#[derive(Clone)]
pub struct PostgresStore {
    pool: PgPool,
}

impl PostgresStore {
    pub async fn connect(url: &str) -> RelationalResult<Self> {
        let pool = PgPoolOptions::new().max_connections(5).connect(url).await?;
        Ok(Self { pool })
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

#[async_trait::async_trait]
impl StartupStore for PostgresStore {
    async fn migrate(&self) -> RelationalResult<()> {
        MIGRATOR.run(&self.pool).await.map_err(Into::into)
    }
    async fn execute_seed(&self, sql: &str) -> RelationalResult<()> {
        sqlx::raw_sql(sql).execute(&self.pool).await?;
        Ok(())
    }
}

#[cfg(feature = "pglite")]
#[derive(Clone)]
pub struct PgliteStore {
    store: PostgresStore,
    _server: Arc<pglite_oxide::PgliteServer>,
}

// One PgliteServer per process. pglite-oxide's `RootLock::acquire` enforces this
// itself via a process-global path registry plus an OS advisory file lock — a
// second in-process `start()` on the same directory fails immediately with
// "PGlite root is already in use". A check-then-act cell (`get()` → async start
// → `set()`) is NOT enough here: two concurrent `open()` calls (React StrictMode
// double-invokes the startup effect in dev, firing `run_migrations` twice) both
// see the empty cell and race a second server, and the loser fails startup.
// `tokio::sync::OnceCell::get_or_try_init` serializes initializers: the loser
// AWAITS the winner's in-flight init and shares its handle. On Err the cell is
// left unset, so a transiently failed init never poisons later retries.
#[cfg(feature = "pglite")]
static PGLITE_STORE: OnceCell<PgliteStore> = OnceCell::const_new();

#[cfg(feature = "pglite")]
impl PgliteStore {
    /// Open (or return/await the already-opening) singleton PGlite server for
    /// this process. `path` is only honoured by the call that performs the
    /// first successful initialization — later calls ignore it and hand back
    /// the existing handle, since a second path would imply a second server.
    pub async fn open(path: impl AsRef<Path>) -> RelationalResult<Self> {
        let path = path.as_ref().to_path_buf();
        PGLITE_STORE
            .get_or_try_init(|| async move {
                let server = pglite_oxide::PgliteServer::builder().path(&path).start()?;
                let url = server.database_url();
                // PGlite executes as a single-user Postgres: one session, with
                // wire connections multiplexed onto it. A pool of 1 matches the
                // engine's real concurrency; more connections only queue inside
                // the server and can deadlock interleaved transactions.
                let pool = PgPoolOptions::new()
                    .max_connections(1)
                    .connect(&url)
                    .await?;
                Ok::<_, super::RelationalError>(Self {
                    store: PostgresStore { pool },
                    _server: Arc::new(server),
                })
            })
            .await
            .cloned()
    }

    pub fn store(&self) -> &PostgresStore {
        &self.store
    }

    /// Apply the embedded Postgres migrations through the same pool returned
    /// by [`Self::store`]. Opening the server alone does not create tables.
    pub async fn migrate(&self) -> RelationalResult<()> {
        self.store.migrate().await
    }
}
