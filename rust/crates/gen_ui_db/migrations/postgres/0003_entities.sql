-- TJ-ARCH-MOB-001 compliant
-- Local-first entity envelope used by the Tauri guest. Product schemas remain
-- in the generated surface; pglite-oxide stores the same JSON records PGlite
-- stores in the browser and PEM normalizes in its client graph.
CREATE TABLE IF NOT EXISTS entity_records (
    entity_type TEXT NOT NULL,
    id TEXT NOT NULL,
    data_json JSONB NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (entity_type, id)
);

CREATE INDEX IF NOT EXISTS entity_records_updated
    ON entity_records (entity_type, updated_at DESC);
