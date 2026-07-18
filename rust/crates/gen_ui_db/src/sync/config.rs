// TJ-ARCH-MOB-001 compliant
//! **LEGACY (Electric lane)** — `SyncConfig`/`ShapeSpec` configure the superseded
//! `SyncEngine`. The FRF path uses `FrfSyncConfig` (frf_transport.rs). `SyncConfig` is
//! still read by `WriteQueue` for its batch/attempt limits, so it is not dead yet.
//! Sync engine configuration. Pure data; no IO.

/// One Electric shape to consume. Kept minimal — the shape `where`/`columns`
/// filters that enforce tenant RLS at the shape factory are added when C-006 wires
/// the authenticated Electric URL. Keep this list identical to the web app's
/// `pglite-sync` shape list so both surfaces converge on the same rows.
#[derive(Debug, Clone)]
pub struct ShapeSpec {
    /// Local table the shape rows are written into (must exist before attach).
    pub table: String,
    /// Optional Postgres `where` filter forwarded to the shape factory.
    pub where_clause: Option<String>,
}

impl ShapeSpec {
    pub fn new(table: impl Into<String>) -> Self {
        Self {
            table: table.into(),
            where_clause: None,
        }
    }

    #[must_use]
    pub fn with_where(mut self, clause: impl Into<String>) -> Self {
        self.where_clause = Some(clause.into());
        self
    }
}

/// Full sync configuration.
#[derive(Debug, Clone)]
pub struct SyncConfig {
    /// Base URL of the Electric HTTP API (e.g. `https://gate.example/electric`).
    /// Through flint-gate this is the tenant-scoped, authenticated shape endpoint.
    pub electric_url: String,
    /// Shapes to consume on the read path.
    pub shapes: Vec<ShapeSpec>,
    /// Max local writes to flush per drain pass before yielding.
    pub write_batch: usize,
    /// After this many failed replays a write is quarantined (poison handler).
    pub max_write_attempts: u32,
}

impl SyncConfig {
    pub fn new(electric_url: impl Into<String>) -> Self {
        Self {
            electric_url: electric_url.into(),
            shapes: Vec::new(),
            write_batch: 64,
            max_write_attempts: 8,
        }
    }

    #[must_use]
    pub fn with_shape(mut self, shape: ShapeSpec) -> Self {
        self.shapes.push(shape);
        self
    }
}
