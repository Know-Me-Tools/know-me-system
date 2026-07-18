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
  const [projectsDdl, notesDdl, conversationsDdl] = schemaSql.split(';').filter((sql) => sql.includes('CREATE TABLE'))
  if (!projectsDdl || !notesDdl || !conversationsDdl) throw new Error('shared entity DDL is incomplete')
  registerEntityFromSql({ entityType: 'Project', createTableSql: projectsDdl })
  registerEntityFromSql({ entityType: 'Note', createTableSql: notesDdl })
  registerEntityFromSql({ entityType: 'Conversation', createTableSql: conversationsDdl })

  if (isTauri()) {
    registerEntityTransport('Project', tauriTransport('projects', tenantId))
    registerEntityTransport('Note', tauriTransport('notes', tenantId))
    registerEntityTransport('Conversation', tauriTransport('conversations', tenantId))
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

export async function deleteConversationRecord(id: string): Promise<void> {
  useGraphStore.getState().removeEntity(CONVERSATION_TYPE, id)
  if (isTauri()) {
    await entityDelete('conversations', id)
    return
  }
  if (!browserDb) throw new Error('PGlite conversation store is not ready')
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
