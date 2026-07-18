// TJ-ARCH-MOB-001 compliant — store layer
import { create } from 'zustand'
import { immer } from 'zustand/middleware/immer'
import { subscribeWithSelector } from 'zustand/middleware'
import { isTauri } from '@tauri-apps/api/core'
import { onChatEvent, streamAgentA2ui } from '@prometheus-ags/tauri-plugin-gen-ui'
import type { ContentBlock, Message, MessageUsage } from '@/bridge/a2ui/types'
import { applyA2uiEvent, createA2uiWireAdapter, type A2uiWireEvent } from '@/bridge/a2ui/driver'
import { streamWebLlm } from '../api/webllmLane'
import { streamHostedChat } from '../api/hostedLane'
import { useLaneStore } from './laneStore'
import { getSessionByok } from '@/features/settings/stores/providerStore'
import {
  deleteConversationRecord,
  listConversationRecords,
  saveConversationRecord,
  type ConversationRecord,
} from '@/features/entities/stores/entityRuntime'

const CHAT_TENANT_ID = 'knowme-poc-local'

export interface Conversation {
  id: string
  title: string
  messages: Message[]
  createdAt: string
  updatedAt: string
}

function newConversation(): Conversation {
  const now = new Date().toISOString()
  return { id: crypto.randomUUID(), title: 'New conversation', messages: [], createdAt: now, updatedAt: now }
}

const initialConversation = newConversation()
let persistenceTimer: ReturnType<typeof setTimeout> | undefined

function persistActiveConversation(): void {
  clearTimeout(persistenceTimer)
  persistenceTimer = setTimeout(() => {
    const state = useChatStore.getState()
    const conversation = state.conversations.find((item) => item.id === state.activeConversationId)
    if (!conversation) return
    const row: ConversationRecord = {
      id: conversation.id,
      tenant_id: CHAT_TENANT_ID,
      title: conversation.title,
      messages: conversation.messages,
      created_at: conversation.createdAt,
      updated_at: conversation.updatedAt,
    }
    void saveConversationRecord(row).catch((cause: unknown) => console.error('conversation persistence failed', cause))
  }, 250)
}

/**
 * Bridges Rust's run-scoped A2uiEvents to the store's message-scoped ones.
 * Module-level because `initListeners` (which consumes events) and
 * `sendMessage` (which knows the message id) are separate entry points.
 */
const wireAdapter = createA2uiWireAdapter((event) => {
  const store = useChatStore.getState()
  applyA2uiEvent(event, store.streamBlock, store.finalizeMessage)
})

/**
 * Drive the in-browser WebLLM lane, feeding its CoreA2uiEvents through the same
 * applyA2uiEvent path the Tauri lane's events take, and timing the run so the UI
 * can report honest on-device tok/s.
 *
 * Token count is approximated by counting streamed chunks: WebLLM's streaming
 * response carries no usage totals, and one chunk is one token in practice. It
 * is labelled an estimate in the UI rather than presented as exact.
 */
async function streamLocalWeb(text: string, assistantId: string, history: string[]): Promise<void> {
  const turns = history.reduce<{ role: 'user' | 'assistant'; text: string }[]>((acc, _, i, arr) => {
    if (i % 2 === 0 && (arr[i] === 'user' || arr[i] === 'assistant')) {
      acc.push({ role: arr[i] as 'user' | 'assistant', text: arr[i + 1] ?? '' })
    }
    return acc
  }, [])

  const store = useChatStore.getState()
  const startedAt = performance.now()
  let tokens = 0
  try {
    for await (const event of streamWebLlm(text, assistantId, turns)) {
      if (event.type === 'contentBlock') tokens += 1
      applyA2uiEvent(event, store.streamBlock, store.finalizeMessage)
    }
    useLaneStore.getState().recordThroughput(tokens, (performance.now() - startedAt) / 1000)
  } catch (e: unknown) {
    // Surface the failure in the transcript rather than leaving the message
    // spinning forever.
    store.streamBlock({
      messageId: assistantId,
      block: { type: 'text', text: `Local inference failed: ${e instanceof Error ? e.message : String(e)}` },
    })
    store.finalizeMessage(assistantId)
  }
}

interface ChatState {
  messages: Message[]
  conversations: Conversation[]
  activeConversationId: string
  isStreaming: boolean
  activeRunId: string | null
  isHydrated: boolean
}

interface ChatActions {
  sendMessage: (text: string) => Promise<void>
  streamBlock: (params: { messageId: string; blockIndex?: number; block: ContentBlock }) => void
  finalizeMessage: (messageId: string, usage?: MessageUsage) => void
  initListeners: () => () => void
  hydrateConversations: () => Promise<void>
  createConversation: () => void
  switchConversation: (id: string) => void
  renameConversation: (id: string, title: string) => void
  deleteConversation: (id: string) => Promise<void>
}

export const useChatStore = create<ChatState & ChatActions>()(
  subscribeWithSelector(
    immer((set, get) => ({
      messages: initialConversation.messages,
      conversations: [initialConversation],
      activeConversationId: initialConversation.id,
      isStreaming: false,
      activeRunId: null,
      isHydrated: false,

      streamBlock: ({ messageId, blockIndex, block }) => {
        set((s) => {
          const msg = s.messages.find((m) => m.id === messageId)
          if (!msg) return
          if (blockIndex !== undefined && blockIndex < msg.content.length) {
            msg.content[blockIndex] = block
          } else {
            msg.content.push(block)
          }
          const conversation = s.conversations.find((item) => item.id === s.activeConversationId)
          if (conversation) { conversation.messages = s.messages; conversation.updatedAt = new Date().toISOString() }
        })
        persistActiveConversation()
      },

      finalizeMessage: (messageId, usage) => {
        set((s) => {
          const msg = s.messages.find((m) => m.id === messageId)
          if (msg) { msg.isStreaming = false; if (usage) msg.usage = usage }
          s.isStreaming = false
          const conversation = s.conversations.find((item) => item.id === s.activeConversationId)
          if (conversation) { conversation.messages = s.messages; conversation.updatedAt = new Date().toISOString() }
        })
        persistActiveConversation()
      },

      sendMessage: async (text: string) => {
        const userMsg: Message = { id: crypto.randomUUID(), role: 'user', content: [{ type: 'text', text }], timestamp: new Date().toISOString() }
        const assistantId = crypto.randomUUID()
        const assistantMsg: Message = { id: assistantId, role: 'assistant', content: [], timestamp: new Date().toISOString(), isStreaming: true }
        set((s) => {
          s.messages.push(userMsg, assistantMsg)
          s.isStreaming = true
          const conversation = s.conversations.find((item) => item.id === s.activeConversationId)
          if (conversation) {
            conversation.messages = s.messages
            conversation.updatedAt = new Date().toISOString()
            if (conversation.title === 'New conversation') conversation.title = text.slice(0, 52)
          }
        })
        persistActiveConversation()

        // Exclude both records just appended. `text` is passed separately;
        // including the new user row here would submit the prompt twice.
        const history = get().messages.slice(0, -2).flatMap(
          (m) => [m.role, m.content.find((b) => b.type === 'text')?.text ?? ''],
        )

        // Outside Tauri the Rust lane doesn't exist, so a `local` selection is
        // fulfilled in-browser by WebLLM. Inside Tauri, lane selection lives in
        // the Rust config DB and stream_agent_a2ui already routes accordingly —
        // hence no lane branch on that path.
        const lane = useLaneStore.getState().lane
        if (!isTauri() && lane === 'local') {
          const ready = await useLaneStore.getState().prepareLocal()
          if (!ready) {
            const cause = useLaneStore.getState().error ?? 'Local model could not be prepared'
            get().streamBlock({
              messageId: assistantId,
              blockIndex: 0,
              block: { type: 'text', text: `Local inference is unavailable: ${cause}` },
            })
            get().finalizeMessage(assistantId)
            return
          }
          await streamLocalWeb(text, assistantId, history)
          persistActiveConversation()
          return
        }

        if (!isTauri()) {
          const byok = getSessionByok()
          if (!byok) {
            get().streamBlock({
              messageId: assistantId,
              blockIndex: 0,
              block: { type: 'text', text: 'Choose Local for zero-configuration chat, or add a session-only cloud provider in Settings.' },
            })
            get().finalizeMessage(assistantId)
            return
          }
          wireAdapter.bind(assistantId)
          const hostedHistory = history.reduce<{ role: 'user' | 'assistant'; content: string }[]>((messages, value, index, values) => {
            if (index % 2 === 0 && (value === 'user' || value === 'assistant')) {
              messages.push({ role: value, content: values[index + 1] ?? '' })
            }
            return messages
          }, [])
          try {
            for await (const event of streamHostedChat(text, hostedHistory, byok)) wireAdapter.handle(event)
          } catch (error) {
            wireAdapter.handle({ type: 'run_error', message: error instanceof Error ? error.message : String(error) })
          }
          persistActiveConversation()
          return
        }

        // Rust's events are run-scoped and carry no message id, so tell the
        // adapter which message this turn's events belong to before starting it.
        wireAdapter.bind(assistantId)
        // Store calls invoke() (via the plugin's typed wrapper) — never
        // component or hook.
        await streamAgentA2ui(text, history).catch((e: unknown) => {
          console.error(e)
          // The run never started, so no run_error will arrive to close the
          // message — finalize it here or it spins forever.
          get().streamBlock({
            messageId: assistantId,
            blockIndex: 0,
            block: { type: 'text', text: e instanceof Error ? e.message : 'Chat failed to start' },
          })
          get().finalizeMessage(assistantId)
        })
        persistActiveConversation()
      },

      hydrateConversations: async () => {
        if (get().isHydrated) return
        const rows = await listConversationRecords(CHAT_TENANT_ID)
        set((s) => {
          if (rows.length > 0) {
            s.conversations = rows.map((row) => ({
              id: row.id,
              title: row.title,
              messages: row.messages as Message[],
              createdAt: row.created_at,
              updatedAt: row.updated_at,
            }))
            s.activeConversationId = s.conversations[0]!.id
            s.messages = s.conversations[0]!.messages
          }
          s.isHydrated = true
        })
      },

      createConversation: () => {
        const conversation = newConversation()
        set((s) => {
          s.conversations.unshift(conversation)
          s.activeConversationId = conversation.id
          s.messages = conversation.messages
          s.isStreaming = false
        })
        persistActiveConversation()
      },

      switchConversation: (id) => set((s) => {
        const conversation = s.conversations.find((item) => item.id === id)
        if (!conversation || s.isStreaming) return
        s.activeConversationId = id
        s.messages = conversation.messages
      }),

      renameConversation: (id, title) => {
        set((s) => {
          const conversation = s.conversations.find((item) => item.id === id)
          if (conversation) { conversation.title = title; conversation.updatedAt = new Date().toISOString() }
        })
        if (id === get().activeConversationId) persistActiveConversation()
      },

      deleteConversation: async (id) => {
        await deleteConversationRecord(id)
        set((s) => {
          s.conversations = s.conversations.filter((item) => item.id !== id)
          if (s.conversations.length === 0) s.conversations.push(newConversation())
          if (s.activeConversationId === id) {
            s.activeConversationId = s.conversations[0]!.id
            s.messages = s.conversations[0]!.messages
          }
        })
      },

      initListeners: () => {
        // Wire A2UI events from Rust to store — no-op outside a real Tauri
        // context (this bundle also serves as a plain web page with no
        // __TAURI_INTERNALS__ bridge, where listen() throws synchronously).
        if (!isTauri()) return () => {}
        // Time the Rust local lane the same way the web lane times itself, so
        // on-device tok/s is reported on both. Cloud runs aren't timed: tok/s
        // there measures the network and the provider's load, not this machine,
        // so showing it would invite a meaningless comparison.
        let startedAt: number | null = null
        let blocks = 0
        const unlistenPromise = onChatEvent<A2uiWireEvent>((wire) => {
          if (useLaneStore.getState().lane === 'local') {
            if (wire.type === 'block') {
              startedAt ??= performance.now()
              blocks += 1
            } else if ((wire.type === 'run_finished' || wire.type === 'run_error') && startedAt !== null) {
              useLaneStore.getState().recordThroughput(blocks, (performance.now() - startedAt) / 1000)
              startedAt = null
              blocks = 0
            }
          }
          wireAdapter.handle(wire)
        })
        return () => { unlistenPromise.then((fn) => fn()) }
      },
    }))
  )
)
