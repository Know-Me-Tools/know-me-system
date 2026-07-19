// TJ-ARCH-MOB-001 compliant — all Tauri IPC lives in this store module.
import { isTauri } from '@tauri-apps/api/core'
import {
  entityGet,
  entityList,
  entityCreate,
  entityUpdate,
  entityDelete,
  entityRuntimeStart,
  entityRuntimeStop,
  type EntityRecord as PluginEntityRecord,
} from '@prometheus-ags/tauri-plugin-gen-ui'
import { PGlite } from '@electric-sql/pglite'
import {
  createPGlitePersistenceAdapter,
  registerEntityFromSql,
  registerEntityTransport,
  startLocalFirstGraph,
  useGraphStore,
  type EntityTransport,
} from '@prometheus-ags/prometheus-entity-management'
import schemaSql from '../schema.sql?raw'

export interface EntityRow { id: string; tenant_id: string; [key: string]: unknown }

interface EntityRuntimeSession {
  tenantId: string
  dispose: () => void
}

let session: EntityRuntimeSession | null = null
let sessionInflight: Promise<EntityRuntimeSession> | null = null
let consumers = 0
let browserDb: PGlite | null = null

// The plugin's wire shape ({id, entityType, dataJson}) is schema-agnostic —
// dataJson is the entity payload as a JSON string (see gen_ui_types::transport
// and the PEM Dart mirror's identical dataJson: String pattern). Adapt it into
// the row shape PEM's transport expects; tenant scoping is the caller's job.
function toEntityRow(record: PluginEntityRecord, tenantId: string): EntityRow {
  const data = JSON.parse(record.dataJson) as Record<string, unknown>
  return { ...data, id: record.id, tenant_id: tenantId }
}

function tauriTransport(entityType: string, tenantId: string): EntityTransport<EntityRow> {
  return {
    identify: (row) => row.id,
    authoritative: true,
    // The plugin's ViewDescriptor filters/sorts/pagination land with the real
    // entity-view UI (C-104+) — list() ignores the PEM ListQuery for now and
    // fetches everything, matching this transport's current pre-C-104 scope.
    list: async () => {
      const result = await entityList({ entityType, filters: [], sorts: [], limit: null, cursor: null })
      const rows = result.items.map((r) => toEntityRow(r, tenantId))
      return { rows, total: rows.length, nextCursor: result.nextCursor }
    },
    get: async (id) => {
      const record = await entityGet(entityType, id)
      return record ? toEntityRow(record, tenantId) : null
    },
  }
}

function pgliteTransport(db: PGlite, table: string, tenantId: string): EntityTransport<EntityRow> {
  return {
    identify: (row) => row.id,
    authoritative: true,
    list: async ({ limit = 100 }) => {
      const result = await db.query<EntityRow>(
        `SELECT * FROM ${table} WHERE tenant_id = $1 ORDER BY updated_at DESC LIMIT $2`,
        [tenantId, limit],
      )
      return { rows: result.rows, total: result.rows.length, nextCursor: null }
    },
    get: async (id) => {
      const result = await db.query<EntityRow>(
        `SELECT * FROM ${table} WHERE tenant_id = $1 AND id = $2 LIMIT 1`, [tenantId, id],
      )
      return result.rows[0] ?? null
    },
  }
}

async function createEntityRuntime(tenantId: string): Promise<EntityRuntimeSession> {
  const tableDdls = schemaSql.split(';').filter((sql) => sql.includes('CREATE TABLE'))
  const projectsDdl = tableDdls.find((sql) => sql.includes('projects'))
  const notesDdl = tableDdls.find((sql) => sql.includes('notes'))
  const conversationsDdl = tableDdls.find((sql) => sql.includes('chat_conversations'))
  const messagesDdl = tableDdls.find((sql) => sql.includes('chat_messages'))
  if (!projectsDdl || !notesDdl || !conversationsDdl || !messagesDdl) throw new Error('shared entity DDL is incomplete')
  registerEntityFromSql({ entityType: 'Project', createTableSql: projectsDdl })
  registerEntityFromSql({ entityType: 'Note', createTableSql: notesDdl })
  registerEntityFromSql({ entityType: 'Conversation', createTableSql: conversationsDdl })
  registerEntityFromSql({ entityType: 'Message', createTableSql: messagesDdl })

  if (isTauri()) {
    registerEntityTransport('Project', tauriTransport('projects', tenantId))
    registerEntityTransport('Note', tauriTransport('notes', tenantId))
    registerEntityTransport('Conversation', tauriTransport('conversations', tenantId))
    registerEntityTransport('Message', tauriTransport('messages', tenantId))
    await entityRuntimeStart(tenantId)
    return { tenantId, dispose: () => { void entityRuntimeStop() } }
  }

  // Normal browsers persist PGlite in IndexedDB. Some embedded/restricted web
  // shells omit IndexedDB entirely; selecting the memory filesystem up front
  // keeps the app usable there without letting Emscripten fall through a noisy
  // filesystem error. The full browser deployment remains durable by default.
  const dataDir = typeof indexedDB === 'undefined' ? 'memory://' : 'idb://gen-ui'
  const db = await PGlite.create(dataDir, { relaxedDurability: true })
  browserDb = db
  await db.exec(schemaSql)
  registerEntityTransport('Project', pgliteTransport(db, 'projects', tenantId))
  registerEntityTransport('Note', pgliteTransport(db, 'notes', tenantId))
  registerEntityTransport('Conversation', pgliteTransport(db, 'chat_conversations', tenantId))
  registerEntityTransport('Message', pgliteTransport(db, 'chat_messages', tenantId))
  const graph = startLocalFirstGraph({
    storage: await createPGlitePersistenceAdapter(db),
    key: `tenant:${tenantId}`,
    replayPendingActions: true,
  })
  return {
    tenantId,
    dispose: () => {
      graph.dispose()
      browserDb = null
      void db.close()
    },
  }
}

export interface ConversationRecord extends Record<string, unknown> {
  id: string
  tenant_id: string
  title: string
  messages: unknown[]
  created_at: string
  updated_at: string
}

export interface MessageRecord extends Record<string, unknown> {
  id: string
  tenant_id: string
  conversation_id: string
  parent_message_id: string | null
  branch_id: string
  sequence: number
  role: string
  blocks: unknown[]
  run_id: string | null
  run_status: string | null
  run_phase: string | null
  status_message: string | null
  error: unknown
  attachments: unknown[]
  generation_settings: unknown
  diagnostics: unknown
  context_plan_version: string | null
  context_telemetry: unknown
  created_at: string
  updated_at: string
}

const CONVERSATION_TYPE = 'Conversation'

function graphConversation(row: ConversationRecord): void {
  useGraphStore.getState().upsertEntity(CONVERSATION_TYPE, row.id, row)
}

export async function listConversationRecords(tenantId: string): Promise<ConversationRecord[]> {
  let rows: ConversationRecord[]
  if (isTauri()) {
    const result = await entityList({ entityType: 'conversations', filters: [], sorts: [], limit: null, cursor: null })
    rows = result.items.map((record) => toEntityRow(record, tenantId) as unknown as ConversationRecord)
  } else {
    if (!browserDb) throw new Error('PGlite conversation store is not ready')
    const result = await browserDb.query<ConversationRecord>(
      'SELECT * FROM chat_conversations WHERE tenant_id = $1 ORDER BY updated_at DESC', [tenantId],
    )
    rows = result.rows
  }
  rows.forEach(graphConversation)
  return rows
}

export async function saveConversationRecord(row: ConversationRecord): Promise<void> {
  graphConversation(row)
  if (isTauri()) {
    const record = { id: row.id, entityType: 'conversations', dataJson: JSON.stringify(row) }
    const existing = await entityGet('conversations', row.id)
    await (existing ? entityUpdate(record) : entityCreate(record))
    return
  }
  if (!browserDb) throw new Error('PGlite conversation store is not ready')
  await browserDb.query(
    `INSERT INTO chat_conversations (id, tenant_id, title, messages, created_at, updated_at)
     VALUES ($1, $2, $3, $4::jsonb, $5, $6)
     ON CONFLICT (id) DO UPDATE SET title = EXCLUDED.title, messages = EXCLUDED.messages,
       updated_at = EXCLUDED.updated_at`,
    [row.id, row.tenant_id, row.title, JSON.stringify(row.messages), row.created_at, row.updated_at],
  )
}

export async function listMessageRecords(tenantId: string, conversationId: string): Promise<MessageRecord[]> {
  if (isTauri()) {
    const result = await entityList({ entityType: 'messages', filters: [], sorts: [], limit: null, cursor: null })
    return result.items
      .map((record) => toEntityRow(record, tenantId) as unknown as MessageRecord)
      .filter((row) => row.conversation_id === conversationId)
      .sort((a, b) => a.sequence - b.sequence)
  }
  if (!browserDb) throw new Error('PGlite message store is not ready')
  const result = await browserDb.query<MessageRecord>(
    'SELECT * FROM chat_messages WHERE tenant_id = $1 AND conversation_id = $2 ORDER BY sequence ASC',
    [tenantId, conversationId],
  )
  return result.rows
}

export async function saveMessageRecords(rows: MessageRecord[]): Promise<void> {
  for (const row of rows) {
    useGraphStore.getState().upsertEntity('Message', row.id, row)
    if (isTauri()) {
      const record = { id: row.id, entityType: 'messages', dataJson: JSON.stringify(row) }
      const existing = await entityGet('messages', row.id)
      await (existing ? entityUpdate(record) : entityCreate(record))
      continue
    }
    if (!browserDb) throw new Error('PGlite message store is not ready')
    await browserDb.query(
      `INSERT INTO chat_messages
       (id, tenant_id, conversation_id, parent_message_id, branch_id, sequence, role, blocks,
        run_id, run_status, run_phase, status_message, error, attachments, generation_settings,
        diagnostics, context_plan_version, context_telemetry, created_at, updated_at)
       VALUES ($1,$2,$3,$4,$5,$6,$7,$8::jsonb,$9,$10,$11,$12,$13::jsonb,$14::jsonb,$15::jsonb,$16::jsonb,$17,$18::jsonb,$19,$20)
       ON CONFLICT (id) DO UPDATE SET blocks=EXCLUDED.blocks, run_id=EXCLUDED.run_id,
       run_status=EXCLUDED.run_status, run_phase=EXCLUDED.run_phase, status_message=EXCLUDED.status_message,
       error=EXCLUDED.error, generation_settings=EXCLUDED.generation_settings, diagnostics=EXCLUDED.diagnostics,
       context_plan_version=EXCLUDED.context_plan_version, context_telemetry=EXCLUDED.context_telemetry,
       updated_at=EXCLUDED.updated_at`,
      [row.id, row.tenant_id, row.conversation_id, row.parent_message_id, row.branch_id, row.sequence,
       row.role, JSON.stringify(row.blocks), row.run_id, row.run_status, row.run_phase, row.status_message,
       JSON.stringify(row.error), JSON.stringify(row.attachments), JSON.stringify(row.generation_settings),
       JSON.stringify(row.diagnostics), row.context_plan_version, JSON.stringify(row.context_telemetry),
       row.created_at, row.updated_at],
    )
  }
}

export async function deleteMessageRecords(ids: string[]): Promise<void> {
  if (ids.length === 0) return
  ids.forEach((id) => useGraphStore.getState().removeEntity('Message', id))
  if (isTauri()) {
    await Promise.all(ids.map((id) => entityDelete('messages', id)))
    return
  }
  if (!browserDb) throw new Error('PGlite message store is not ready')
  await browserDb.query('DELETE FROM chat_messages WHERE id = ANY($1::uuid[])', [ids])
}

export async function deleteConversationRecord(id: string): Promise<void> {
  useGraphStore.getState().removeEntity(CONVERSATION_TYPE, id)
  if (isTauri()) {
    const messages = await entityList({ entityType: 'messages', filters: [], sorts: [], limit: null, cursor: null })
    await Promise.all(messages.items.filter((record) => {
      const row = toEntityRow(record, '') as Record<string, unknown>
      return row.conversation_id === id
    }).map((record) => entityDelete('messages', record.id)))
    await entityDelete('conversations', id)
    return
  }
  if (!browserDb) throw new Error('PGlite conversation store is not ready')
  await browserDb.query('DELETE FROM chat_messages WHERE conversation_id = $1', [id])
  await browserDb.query('DELETE FROM chat_conversations WHERE id = $1', [id])
}

// React StrictMode mounts, releases, then remounts effects in development. Share
// one in-flight runtime across those consumers so two PGlite instances never
// race the same IndexedDB database, and close it only after the final release.
export async function startEntityRuntime(tenantId: string): Promise<() => void> {
  if (session && session.tenantId !== tenantId) {
    throw new Error(`entity runtime already active for tenant ${session.tenantId}`)
  }
  consumers += 1

  try {
    if (!session) {
      sessionInflight ??= createEntityRuntime(tenantId)
      const pending = sessionInflight
      session = await pending
      if (sessionInflight === pending) sessionInflight = null
    }
  } catch (cause) {
    consumers -= 1
    sessionInflight = null
    throw cause
  }

  const acquired = session
  let released = false
  return () => {
    if (released) return
    released = true
    consumers -= 1
    if (consumers === 0 && session === acquired) {
      acquired.dispose()
      session = null
    }
  }
}
