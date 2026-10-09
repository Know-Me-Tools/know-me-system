-- Authorized read progress is committed with its projection, never ahead of it.
CREATE TABLE sync_read_cursors (
    scope TEXT NOT NULL,
    shape_id TEXT NOT NULL,
    table_name TEXT NOT NULL CHECK (table_name IN ('notes', 'memories')),
    handle TEXT,
    offset_value TEXT,
    generation BIGINT NOT NULL DEFAULT 0,
    status TEXT NOT NULL CHECK (status IN ('syncing', 'live', 'resnapshot', 'revoked')),
    reason TEXT,
    PRIMARY KEY (scope, shape_id),
    UNIQUE (scope, table_name),
    CHECK ((handle IS NULL) = (offset_value IS NULL))
);
