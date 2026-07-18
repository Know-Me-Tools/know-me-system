-- TJ-ARCH-MOB-001 compliant
CREATE TABLE IF NOT EXISTS app_seed_versions (
    name TEXT PRIMARY KEY,
    version BIGINT NOT NULL CHECK (version >= 0),
    applied_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
