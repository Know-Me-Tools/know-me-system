-- TJ-ARCH-MOB-001 compliant
-- PoC-specific objects layered on FRF's Postgres (see README.md — FRF owns the stack;
-- this file owns only what KnowMe adds). Apply against FRF's `frf` database once the
-- fabric is up:
--
--   psql postgresql://frf:frf@localhost:15432/frf -f apps/knowme-poc/infra/knowme-sync.sql
--
-- Design notes (these survived the Electric→FRF pivot unchanged, because they are
-- driven by replay/idempotency — not by whichever CDC reader is in front):
--   * Client-generated UUID text PKs. An OFFLINE insert must already know its own id,
--     with no server round-trip to learn it — that is what lets the airplane-mode replay
--     converge instead of duplicating rows.
--   * Soft delete. A change feed cannot ship a row that no longer exists, so a delete
--     has to be a visible row change, not an absence.
--   * updated_at drives last-write-wins on replay. The PoC does NOT do CRDT merge; that
--     is honest for a demo and must stay labeled as such. (FRF's frf-crdt / Loro peer
--     lane exists if real merge is wanted later — out of scope for C-106.)

CREATE TABLE IF NOT EXISTS notes (
  id          TEXT PRIMARY KEY,
  title       TEXT NOT NULL DEFAULT '',
  body        TEXT NOT NULL DEFAULT '',
  updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted     BOOLEAN NOT NULL DEFAULT false
);

CREATE TABLE IF NOT EXISTS memories (
  id          TEXT PRIMARY KEY,
  text        TEXT NOT NULL,
  kind        TEXT NOT NULL DEFAULT 'note',
  entity      TEXT,
  updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted     BOOLEAN NOT NULL DEFAULT false
);

CREATE INDEX IF NOT EXISTS notes_updated_at_idx    ON notes (updated_at);
CREATE INDEX IF NOT EXISTS memories_updated_at_idx ON memories (updated_at);

-- REPLICA IDENTITY FULL: pgoutput otherwise emits only the PK for UPDATE/DELETE, so the
-- CDC consumer could not build a complete RowChange for the local store to upsert.
ALTER TABLE notes    REPLICA IDENTITY FULL;
ALTER TABLE memories REPLICA IDENTITY FULL;

-- The publication frf-postgres-cdc's slot subscribes to (CdcConfig::publication_name).
-- FRF's compose already runs Postgres with wal_level=logical / max_replication_slots=5.
DROP PUBLICATION IF EXISTS knowme_sync;
CREATE PUBLICATION knowme_sync FOR TABLE notes, memories;
