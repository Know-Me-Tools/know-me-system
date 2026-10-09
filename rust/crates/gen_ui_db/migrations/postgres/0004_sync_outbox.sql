-- C11 local command authority; never replicated through personal CRDT lanes.
CREATE TABLE sync_outbox (
    sequence BIGSERIAL UNIQUE NOT NULL,
    scope TEXT NOT NULL,
    command_id TEXT NOT NULL,
    table_name TEXT NOT NULL CHECK (table_name IN ('notes', 'memories')),
    change_json TEXT NOT NULL,
    attempts BIGINT NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'committed', 'poison')),
    result_json JSONB,
    rejection TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    settled_at TIMESTAMPTZ,
    PRIMARY KEY (scope, command_id)
);
CREATE INDEX sync_outbox_replay ON sync_outbox(scope, status, sequence);
CREATE TABLE sync_projection_scope (
    singleton BOOLEAN PRIMARY KEY DEFAULT true CHECK (singleton),
    scope TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS notes (
    id TEXT PRIMARY KEY, title TEXT NOT NULL DEFAULT '', body TEXT NOT NULL DEFAULT '',
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(), deleted BOOLEAN NOT NULL DEFAULT false
);
CREATE TABLE IF NOT EXISTS memories (
    id TEXT PRIMARY KEY, text TEXT NOT NULL DEFAULT '', kind TEXT NOT NULL DEFAULT 'note', entity TEXT,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(), deleted BOOLEAN NOT NULL DEFAULT false
);
