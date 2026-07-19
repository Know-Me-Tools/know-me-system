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
