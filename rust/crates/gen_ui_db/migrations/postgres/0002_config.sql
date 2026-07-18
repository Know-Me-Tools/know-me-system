-- TJ-ARCH-MOB-001 compliant
-- Config DB v1: provider/model administration for the liter-llm gateway.
-- api_key_ref is a reference into the platform keychain / secure storage —
-- never a plaintext secret (Rule: no plaintext keys in the DB).
CREATE TABLE IF NOT EXISTS providers (
    id           TEXT PRIMARY KEY,
    kind         TEXT NOT NULL,
    base_url     TEXT,
    api_key_ref  TEXT,
    enabled      BOOLEAN NOT NULL DEFAULT true,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- One row per (surface, lane): surface is e.g. 'chat', lane is 'cloud' | 'local'.
CREATE TABLE IF NOT EXISTS model_prefs (
    surface      TEXT NOT NULL,
    lane         TEXT NOT NULL,
    provider_id  TEXT REFERENCES providers(id) ON DELETE SET NULL,
    model_id     TEXT NOT NULL,
    params       JSONB NOT NULL DEFAULT '{}'::jsonb,
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (surface, lane)
);

CREATE TABLE IF NOT EXISTS app_settings (
    key          TEXT PRIMARY KEY,
    value        JSONB NOT NULL,
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
