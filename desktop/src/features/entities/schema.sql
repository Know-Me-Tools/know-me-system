-- TJ-ARCH-MOB-001 compliant
CREATE TABLE IF NOT EXISTS projects (
  id uuid PRIMARY KEY,
  tenant_id uuid NOT NULL,
  name text NOT NULL,
  description text,
  updated_at timestamptz NOT NULL DEFAULT now()
);
CREATE TABLE IF NOT EXISTS notes (
  id uuid PRIMARY KEY,
  tenant_id uuid NOT NULL,
  project_id uuid NOT NULL,
  title text NOT NULL,
  body text NOT NULL,
  updated_at timestamptz NOT NULL DEFAULT now()
);
CREATE TABLE IF NOT EXISTS chat_conversations (
  id uuid PRIMARY KEY,
  tenant_id text NOT NULL,
  title text NOT NULL,
  messages jsonb NOT NULL DEFAULT '[]'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);
CREATE TABLE IF NOT EXISTS chat_messages (
  id uuid PRIMARY KEY,
  tenant_id text NOT NULL,
  conversation_id uuid NOT NULL,
  parent_message_id uuid,
  branch_id uuid NOT NULL,
  sequence integer NOT NULL,
  role text NOT NULL,
  blocks jsonb NOT NULL DEFAULT '[]'::jsonb,
  run_id uuid,
  run_status text,
  run_phase text,
  status_message text,
  intent text,
  error jsonb,
  attachments jsonb NOT NULL DEFAULT '[]'::jsonb,
  generation_settings jsonb,
  diagnostics jsonb,
  context_plan_version text,
  context_telemetry jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS chat_messages_conversation_branch
  ON chat_messages (conversation_id, branch_id, sequence);

-- Migrations. Every CREATE TABLE above is IF NOT EXISTS, so a column added to
-- one of them never lands on a database that already exists — and the very next
-- INSERT then fails for EVERY message, not just the new field. Each column added
-- after the first release therefore needs a matching idempotent ALTER here.
--
-- These run through `db.exec(schemaSql)`, which executes the whole file. They are
-- deliberately NOT picked up by the `registerEntityFromSql` path, which filters to
-- statements containing "CREATE TABLE" — that path only needs the table shape.
ALTER TABLE chat_messages ADD COLUMN IF NOT EXISTS intent text;
-- `attachments` has been in the CREATE TABLE above since before release, but it
-- was only ever written as a hardcoded empty array, so a database created before
-- the column existed was never caught. Now that real values are persisted, that
-- latent gap becomes a failing INSERT on every message.
ALTER TABLE chat_messages ADD COLUMN IF NOT EXISTS attachments jsonb NOT NULL DEFAULT '[]'::jsonb;
