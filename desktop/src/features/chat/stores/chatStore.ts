// TJ-ARCH-MOB-001 compliant — store layer
import { create } from 'zustand'
import { immer } from 'zustand/middleware/immer'
import { subscribeWithSelector } from 'zustand/middleware'
import { isTauri } from '@tauri-apps/api/core'
import {
  cancelAgentRun,
  chatApproveToolConfirmation,
  memoryDeleteConversationContext,
  memoryDeleteMessageContext,
  onChatEvent,
  sendAgentInteractionA2ui,
  streamAgentA2ui,
} from '@prometheus-ags/tauri-plugin-gen-ui'
import type { InferenceHistoryMessage } from '@prometheus-ags/tauri-plugin-gen-ui'
import type { ContentBlock, Message, MessageIntent, MessageUsage } from '@/bridge/a2ui/types'
import { applyA2uiEvent, createA2uiWireAdapter, type A2uiWireEvent, type RunLifecycleUpdate } from '@/bridge/a2ui/driver'
import { interruptWebLlm, streamWebLlm } from '../api/webllmLane'
import { cancelHostedChat, streamHostedA2uiAction, streamHostedChat } from '../api/hostedLane'
import { uarUnavailableMessage, useLaneStore } from './laneStore'
import {
  deleteConversationRecord,
  deleteMessageRecords,
  listConversationRecords,
  listMessageRecords,
  saveConversationRecord,
  saveMessageRecords,
  type ConversationRecord,
  type MessageRecord,
} from '@/features/entities/stores/entityRuntime'

const CHAT_TENANT_ID = 'knowme-poc-local'

/** Prefixed to a steer so the model treats the stopped turn as work to build on.
 *
 *  Worded for the model, but it stays visible in the transcript, so it also has
 *  to read sensibly to the user who sent it.
 *
 *  Kept textually identical to mobile's `_steerPreamble`
 *  (mobile/lib/features/chat/presentation/providers/chat_notifier.dart). Both
 *  copies are deliberate: a shared constant would have to travel through the Rust
 *  bridge, and a duplicated string with a comment naming its twin is easier to
 *  keep honest than that. If you change one, change the other. */
const STEER_PREAMBLE =
  'I stopped your previous response part-way through. Keep the work you had '
  + 'already done in mind, then follow this correction instead of continuing '
  + 'the direction you were heading:'

/** Sent verbatim when the user resumes a stopped turn. Mirrors mobile's
 *  `_resumeInstruction`. */
const RESUME_INSTRUCTION =
  'Continue your previous response from where you stopped. Do not start over '
  + 'or repeat what you already wrote.'

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
    const messageRows: MessageRecord[] = conversation.messages.map((message, sequence) => ({
      id: message.id,
      tenant_id: CHAT_TENANT_ID,
      conversation_id: conversation.id,
      parent_message_id: message.parentMessageId ?? null,
      branch_id: message.branchId ?? conversation.id,
      sequence,
      role: message.role,
      blocks: message.content,
      run_id: message.runId ?? null,
      run_status: message.runStatus ?? null,
      run_phase: message.runPhase ?? null,
      status_message: message.statusMessage ?? null,
      intent: message.intent ?? null,
      error: message.error ?? null,
      // Was hardcoded `[]`, so the column was write-only and every attachment
      // vanished on reload while the transcript still showed its chips.
      attachments: message.attachmentPaths ?? [],
      // Rendered on every assistant bubble but never stored, so it disappeared
      // on reload while Flutter kept showing it on historical turns.
      provenance: message.provenance ?? null,
      generation_settings: message.generationSettings ?? null,
      diagnostics: message.diagnostics ?? null,
      context_plan_version: message.contextPlanVersion ?? null,
      context_telemetry: message.contextTelemetry ?? null,
      created_at: message.timestamp,
      updated_at: message.updatedAt ?? message.timestamp,
    }))
    void Promise.all([saveConversationRecord(row), saveMessageRecords(messageRows)])
      .catch((cause: unknown) => console.error('conversation persistence failed', cause))
  }, 250)
}

/**
 * Bridges Rust's run-scoped A2uiEvents to the store's message-scoped ones.
 * Module-level because `initListeners` (which consumes events) and
 * `sendMessage` (which knows the message id) are separate entry points.
 */
const wireAdapter = createA2uiWireAdapter((event) => {
  const store = useChatStore.getState()
  applyA2uiEvent(event, store.streamBlock, store.finalizeMessage, store.updateLifecycle)
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
async function streamLocalWeb(text: string, assistantId: string, history: InferenceHistoryMessage[]): Promise<void> {
  const turns = history
    .filter((message): message is InferenceHistoryMessage & { role: 'user' | 'assistant' } =>
      message.role === 'user' || message.role === 'assistant')
    .map((message) => ({ role: message.role, text: message.content }))

  const store = useChatStore.getState()
  const startedAt = performance.now()
  let tokens = 0
  // This lane has no Rust-issued run id, so the assistant message id doubles as
  // one. Without it `cancelActiveRun` returns early and a Stop tap does nothing.
  useChatStore.setState({ activeRunId: assistantId })
  try {
    for await (const event of streamWebLlm(text, assistantId, turns)) {
      if (event.type === 'contentBlock') tokens += 1
      // `updateLifecycle` was omitted here, so the lane could never report a
      // cancel — the event was accepted and dropped on the floor.
      applyA2uiEvent(event, store.streamBlock, store.finalizeMessage, store.updateLifecycle)
    }
    useLaneStore.getState().recordThroughput(tokens, (performance.now() - startedAt) / 1000)
  } catch (e: unknown) {
    // Surface the failure in the transcript rather than leaving the message
    // spinning forever.
    store.updateLifecycle({
      type: 'error', messageId: assistantId, runId: assistantId,
      error: { code: 'webllm_inference_failed', phase: 'generation', message: e instanceof Error ? e.message : String(e), retryable: true },
    })
  }
}

function inferenceHistory(messages: Message[], branchId: string): InferenceHistoryMessage[] {
  const history: InferenceHistoryMessage[] = []
  for (const message of messages) {
    if ((message.branchId ?? branchId) !== branchId) continue
    const answer = message.content
      .filter((block): block is Extract<ContentBlock, { type: 'text' }> => block.type === 'text')
      .map((block) => block.text)
      .join('')
    if (answer.trim()) history.push({ role: message.role, content: answer })
    for (const block of message.content) {
      if (block.type !== 'toolResult') continue
      // Older persisted Rust wire blocks used snake_case field names. Keep
      // those conversations resumable while the canonical contract uses
      // camelCase on every client.
      const legacy = block as unknown as Record<string, unknown>
      const output = typeof block.outputJson === 'string'
        ? block.outputJson
        : typeof legacy.output_json === 'string' ? legacy.output_json : ''
      const toolCallId = typeof block.toolUseId === 'string'
        ? block.toolUseId
        : typeof legacy.tool_use_id === 'string' ? legacy.tool_use_id : ''
      if (!output.trim()) continue
      history.push({
        role: 'tool',
        content: output,
        ...(toolCallId ? { tool_call_id: toolCallId } : {}),
      })
    }
  }
  return history
}

/** One file the user attached, as the composer adapter resolved it. */
export interface AttachmentInput {
  /** Durable path in app-owned storage. Rust reads the file from here, and the
   *  transcript can keep it — unlike a picker path, which is purgeable. */
  path: string
  name: string
  /** Size in bytes; formatted for display when the block is rendered. */
  size: number
  mime: string
  /** Classification decided by Rust, not re-derived from the mime string here.
   *  Optional so the web build, which has no Rust, can omit them. */
  isImage?: boolean
  isVideo?: boolean
  /** Intrinsic pixel dimensions, so the renderer reserves space before decode. */
  width?: number
  height?: number
}

/** Human-readable size, matching Flutter's `_formatBytes` exactly. */
export function formatBytes(bytes: number): string {
  if (bytes >= 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
  if (bytes >= 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${bytes} B`
}

/** The transcript record of an attachment — same shape Flutter emits, so a
 *  conversation started on either surface renders on both.
 *
 *  Images become a real `image` block carrying the durable path. This used to
 *  hardcode `kind: 'attachment'` for every file regardless of mime, which is
 *  why a picked PNG rendered as a grey document chip: `ContentBlock.Image` was
 *  defined in Rust and decoded by both UIs, but nothing ever constructed one.
 *
 *  The classification comes from Rust (`isImage`/`isVideo`) and falls back to
 *  the mime prefix only on the web build, where there is no Rust to ask. */
export function attachmentBlock(attachment: AttachmentInput): ContentBlock {
  const isImage = attachment.isImage ?? attachment.mime.startsWith('image/')
  const isVideo = attachment.isVideo ?? attachment.mime.startsWith('video/')

  if (isImage) {
    return {
      type: 'image',
      url: null,
      dataBase64: null,
      mime: attachment.mime,
      path: attachment.path,
      // Mandatory: a missing accessible name fails the a11y gate, and the file
      // name is the only description available at attach time.
      alt: attachment.name,
      width: attachment.width,
      height: attachment.height,
    }
  }

  return {
    type: 'artifact',
    id: attachment.path,
    kind: isVideo ? 'video' : 'attachment',
    content: JSON.stringify({
      name: attachment.name,
      size: formatBytes(attachment.size),
      mime: attachment.mime,
      path: attachment.path,
    }),
  }
}

async function runInferenceTurn(
  text: string,
  assistantId: string,
  branchId: string,
  history: InferenceHistoryMessage[],
  conversationId: string,
  /** Files to send with this turn, and the user message they belong to. Rust
   *  reads the paths and correlates the extracted content to that message. */
  attachments?: { paths: string[]; userMessageId: string },
): Promise<void> {
  const store = useChatStore.getState()
  let lane = useLaneStore.getState().lane
  if (!isTauri() && lane === 'local') {
    const ready = await useLaneStore.getState().prepareLocal()
    if (!ready) {
      const cause = useLaneStore.getState().error ?? 'Local model could not be prepared'
      store.updateLifecycle({ type: 'error', messageId: assistantId, runId: assistantId,
        error: { code: 'model_prepare_failed', phase: 'model_download', message: cause, retryable: true } })
      return
    }
    await streamLocalWeb(text, assistantId, history)
    persistActiveConversation()
    return
  }

  if (!isTauri()) {
    wireAdapter.bind(assistantId)
    const hostedHistory = history
      .filter((entry): entry is InferenceHistoryMessage & { role: 'user' | 'assistant' } =>
        entry.role === 'user' || entry.role === 'assistant')
      .map((entry) => ({ role: entry.role, content: entry.content }))
    try {
      for await (const event of streamHostedChat(text, hostedHistory, null, conversationId, branchId)) {
        wireAdapter.handle(event)
      }
    } catch (error) {
      wireAdapter.handle({ type: 'run_error', run_id: assistantId, error: {
        code: 'hosted_inference_failed', phase: 'generation',
        message: error instanceof Error ? error.message : String(error), retryable: true,
      } })
    }
    persistActiveConversation()
    return
  }

  if (isTauri()) {
    await useLaneStore.getState().init()
    lane = useLaneStore.getState().lane
    const laneState = useLaneStore.getState()
    if (lane === 'cloud') {
      store.updateLifecycle({ type: 'error', messageId: assistantId, runId: assistantId,
        error: { code: 'desktop_cloud_lane_removed', phase: 'queued', message: 'Desktop chat now uses UAR or Local. Configure Liter-LLM providers under UAR settings, then select UAR.', retryable: false } })
      return
    }
    if (lane === 'uar' && !laneState.uarConfigured) {
      store.updateLifecycle({ type: 'error', messageId: assistantId, runId: assistantId,
        error: { code: 'uar_not_configured', phase: 'queued', message: uarUnavailableMessage(laneState.uarStatus), retryable: true } })
      return
    }
    if (lane === 'local' && !laneState.localAvailable) {
      store.updateLifecycle({ type: 'error', messageId: assistantId, runId: assistantId,
        error: { code: 'local_runtime_unavailable', phase: 'queued', message: 'Local desktop inference is not available in this build. Select UAR after configuring it in Settings.', retryable: false } })
      return
    }
  }

  wireAdapter.bind(assistantId)
  await streamAgentA2ui(
    text,
    history,
    conversationId,
    branchId,
    attachments?.paths,
    attachments?.userMessageId,
  ).catch((error: unknown) => {
    console.error(error)
    store.updateLifecycle({ type: 'error', messageId: assistantId, runId: assistantId,
      error: { code: 'run_start_failed', phase: 'queued', message: error instanceof Error ? error.message : 'Chat failed to start', retryable: true } })
  })
  persistActiveConversation()
}

interface ChatState {
  messages: Message[]
  conversations: Conversation[]
  activeConversationId: string
  isStreaming: boolean
  activeRunId: string | null
  isHydrated: boolean
  /** Set when the user taps Steer, cleared once that send goes out.
   *
   *  Steer only ARMS the composer — it does not send. The intent has to survive
   *  until the user finishes typing the correction, otherwise it goes out as an
   *  anonymous new question and the model has no reason to treat the stopped
   *  turn as work to build on. A Steer that fired immediately would just be a
   *  Resume with extra words. */
  pendingSteer: boolean
}

interface ChatActions {
  sendMessage: (
    text: string,
    options?: { intent?: MessageIntent; attachments?: AttachmentInput[] },
  ) => Promise<void>
  /** Course-correct after a stop, keeping the partial work in play. */
  sendSteer: (text: string) => Promise<void>
  /** Carry on from where a stop landed. */
  sendResume: () => Promise<void>
  armSteer: () => void
  disarmSteer: () => void
  submitInteraction: (interaction: unknown) => Promise<void>
  /** Run a tool call the local model proposed and KnowMe refused to auto-run. */
  approveToolConfirmation: (sourceMessageId: string, approvalId: string) => Promise<void>
  resubmitMessage: (userMessageId: string) => Promise<void>
  deleteResponse: (assistantMessageId: string) => Promise<void>
  regenerateResponse: (assistantMessageId: string) => Promise<void>
  streamBlock: (params: { messageId: string; blockIndex?: number; block: ContentBlock }) => void
  finalizeMessage: (messageId: string, usage?: MessageUsage) => void
  updateLifecycle: (update: RunLifecycleUpdate) => void
  initListeners: () => () => void
  hydrateConversations: () => Promise<void>
  createConversation: () => void
  switchConversation: (id: string) => void
  renameConversation: (id: string, title: string) => void
  deleteConversation: (id: string) => Promise<void>
  cancelActiveRun: () => Promise<void>
}

/**
 * Decide whether a refused cancel is worth reporting.
 *
 * Rust returns `false` when the run is no longer registered — the NORMAL outcome
 * of a second stop tap, or of the run finishing in the gap before the call
 * landed. Treating that as an error would overwrite a legitimate stopped turn
 * with a failure bubble and hide the work the user just chose to keep. So only
 * surface an error when the turn is somehow STILL streaming; otherwise it has
 * already reached a terminal state. Mirrors chat_notifier.dart:544-564.
 */
function settleCancel(
  get: () => ChatState & ChatActions,
  messageId: string | undefined,
  accepted: boolean,
): void {
  if (accepted || !messageId) return
  const current = get().messages.find((item) => item.id === messageId)
  if (!current?.isStreaming) return
  get().updateLifecycle({
    type: 'error', messageId, runId: current.runId ?? messageId,
    error: {
      code: 'run_not_active', phase: 'generation', retryable: false,
      message: 'This run already ended and could not be cancelled.',
    },
  })
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
      pendingSteer: false,

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
          if (msg) {
            msg.isStreaming = false
            msg.runStatus = 'complete'
            msg.updatedAt = new Date().toISOString()
            if (usage) msg.usage = usage
          }
          s.isStreaming = false
          s.activeRunId = null
          const conversation = s.conversations.find((item) => item.id === s.activeConversationId)
          if (conversation) { conversation.messages = s.messages; conversation.updatedAt = new Date().toISOString() }
        })
        persistActiveConversation()
      },

      updateLifecycle: (update) => {
        set((s) => {
          const msg = s.messages.find((item) => item.id === update.messageId)
          if (!msg) return
          msg.runId = update.runId
          msg.updatedAt = new Date().toISOString()
          if (update.type === 'started') {
            msg.runStatus = 'queued'
            s.activeRunId = update.runId
          } else if (update.type === 'phase') {
            msg.runPhase = update.phase
            msg.runStatus = update.status
            msg.statusMessage = update.message
          } else if (update.type === 'telemetry') {
            const payload = update.payload
            const asObject = typeof payload === 'object' && payload !== null
              ? (payload as Record<string, unknown>) : undefined
            // Provenance rides the telemetry channel under a `provenance` key.
            // Capture it; a provenance-only event must not clobber real telemetry.
            const provenance = asObject?.provenance
            if (provenance && typeof provenance === 'object') {
              const p = provenance as Record<string, unknown>
              msg.provenance = {
                agentId: typeof p.agentId === 'string' ? p.agentId : undefined,
                providerId: typeof p.providerId === 'string' ? p.providerId : undefined,
                modelId: typeof p.modelId === 'string' ? p.modelId : undefined,
              }
            }
            if (asObject?.kind !== 'provenance') {
              msg.contextTelemetry = payload
              msg.contextPlanVersion = asObject && 'planVersion' in asObject
                ? String(asObject.planVersion ?? '') : msg.contextPlanVersion
            }
          } else if (update.type === 'error') {
            msg.error = update.error
            msg.runPhase = update.error.phase
            msg.runStatus = 'error'
            msg.isStreaming = false
            s.isStreaming = false
            s.activeRunId = null
          } else {
            msg.runPhase = update.phase
            msg.statusMessage = update.message
            msg.runStatus = 'cancelled'
            msg.isStreaming = false
            s.isStreaming = false
            s.activeRunId = null
          }
          const conversation = s.conversations.find((item) => item.id === s.activeConversationId)
          if (conversation) { conversation.messages = s.messages; conversation.updatedAt = new Date().toISOString() }
        })
        persistActiveConversation()
      },

      sendMessage: async (
        text: string,
        options?: { intent?: MessageIntent; attachments?: AttachmentInput[] },
      ) => {
        const branchId = get().messages.at(-1)?.branchId ?? get().activeConversationId
        const attachments = options?.attachments ?? []
        // Attachment blocks lead the content, as on Flutter: they are the
        // transcript record of what was sent, while the paths below are the
        // inference payload Rust reads.
        const userMsg: Message = {
          id: crypto.randomUUID(),
          role: 'user',
          content: [...attachments.map(attachmentBlock), { type: 'text' as const, text }],
          timestamp: new Date().toISOString(), branchId,
          ...(attachments.length > 0
            ? { attachmentPaths: attachments.map((attachment) => attachment.path) }
            : {}),
          ...(options?.intent ? { intent: options.intent } : {}),
        }
        const assistantId = crypto.randomUUID()
        const assistantMsg: Message = {
          id: assistantId, role: 'assistant', content: [], timestamp: new Date().toISOString(),
          isStreaming: true, parentMessageId: userMsg.id, branchId, runStatus: 'queued',
          // Seeded, not left blank: `statusMessage` is what the pending bubble
          // renders, so without it React showed a bare dot where Flutter says
          // "Queued" and then "Preparing response".
          runPhase: 'queued', statusMessage: 'Queued',
        }
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
        const history = inferenceHistory(get().messages.slice(0, -2), branchId)

        await runInferenceTurn(
          text, assistantId, branchId, history, get().activeConversationId,
          attachments.length > 0
            ? { paths: attachments.map((attachment) => attachment.path), userMessageId: userMsg.id }
            : undefined,
        )
      },

      /** Course-correct after a stop, keeping the partial work in play.
       *
       *  The user stopped *because* they were steering — they read the partial
       *  answer or a tool call and want a different direction. The preamble says
       *  that plainly, so the stopped turn reads to the model as work-in-progress
       *  to build on rather than a failed attempt to ignore.
       *
       *  The tag travels in the CONTENT because that is the only carrier every
       *  lane forwards verbatim — see `Message.intent` for why a typed field and
       *  a new role string both fail silently. */
      sendSteer: async (text: string) => {
        await get().sendMessage(`${STEER_PREAMBLE}\n\n${text}`, { intent: 'steer' })
      },

      /** Carry on from where a stop landed.
       *
       *  For the user who stopped and changed their mind. No backend change is
       *  needed: `inferenceHistory` filters only on `branchId` and non-empty text
       *  — there is no `runStatus` filter — and the cancelled branch of
       *  `updateLifecycle` never clears `content`, so the partial assistant turn
       *  is already replayed and the model sees
       *  `[… assistant(partial), user(continue)]`.
       *
       *  The instruction stays visible in the transcript rather than hidden, so
       *  the conversation remains honest about what was actually sent. */
      sendResume: async () => {
        await get().sendMessage(RESUME_INSTRUCTION, { intent: 'resume' })
      },

      armSteer: () => set((s) => { s.pendingSteer = true }),
      disarmSteer: () => set((s) => { s.pendingSteer = false }),

      approveToolConfirmation: async (sourceMessageId: string, approvalId: string) => {
        if (!approvalId) return
        const source = get().messages.find((item) => item.id === sourceMessageId)
        // Only a settled assistant turn can be approved: approving mid-stream
        // would race the run that is still producing the card.
        if (!source || source.role !== 'assistant' || source.isStreaming) return

        const branchId = source.branchId ?? get().activeConversationId
        const assistantId = crypto.randomUUID()
        const assistantMsg: Message = {
          id: assistantId,
          role: 'assistant',
          content: [],
          timestamp: new Date().toISOString(),
          isStreaming: true,
          parentMessageId: source.id,
          branchId,
          runStatus: 'queued',
          runPhase: 'tool_execution',
          statusMessage: 'Approving tool call',
        }
        set((s) => {
          s.messages.push(assistantMsg)
          s.isStreaming = true
          const conversation = s.conversations.find((item) => item.id === s.activeConversationId)
          if (conversation) { conversation.messages = s.messages; conversation.updatedAt = new Date().toISOString() }
        })
        persistActiveConversation()

        if (!isTauri()) {
          // Approvals live in the Rust process registry; the browser lane has
          // no engine proposing local tool calls in the first place.
          get().updateLifecycle({ type: 'error', messageId: assistantId, runId: assistantId, error: {
            code: 'tool_approval_unavailable', phase: 'tool_execution',
            message: 'Tool approval requires the desktop app.', retryable: false,
          } })
          return
        }

        wireAdapter.bind(assistantId)
        try {
          await chatApproveToolConfirmation(approvalId)
        } catch (error) {
          get().updateLifecycle({ type: 'error', messageId: assistantId, runId: assistantId, error: {
            code: 'tool_approval_start_failed', phase: 'tool_execution',
            // Approvals are process-local: a card restored after a restart has
            // no pending registration behind it, and this is where the user
            // finds that out rather than watching a bubble spin.
            message: error instanceof Error ? error.message : 'The tool approval could not be started.',
            retryable: false,
          } })
        }
        persistActiveConversation()
      },

      submitInteraction: async (interaction: unknown) => {
        const parent = get().messages.at(-1)
        const sourceRunId = typeof interaction === 'object' && interaction !== null && !Array.isArray(interaction)
          && typeof (interaction as Record<string, unknown>).sourceRunId === 'string'
          ? (interaction as Record<string, unknown>).sourceRunId as string
          : parent?.runId
        const branchId = parent?.branchId ?? get().activeConversationId
        // Record what the user actually did. Without this the transcript jumped
        // straight from the artifact to a spinner — a reply to nothing — and
        // the interaction never entered history, so the next turn's model could
        // not see the choice that produced it. Flutter has always recorded it
        // (chat_notifier.dart submitInteraction).
        const userMsg: Message = {
          id: crypto.randomUUID(),
          role: 'user',
          content: [{
            type: 'artifact',
            id: crypto.randomUUID(),
            kind: 'client_interaction',
            content: JSON.stringify(interaction),
          }],
          timestamp: new Date().toISOString(),
          branchId,
        }
        const assistantId = crypto.randomUUID()
        const assistantMsg: Message = {
          id: assistantId,
          role: 'assistant',
          content: [],
          timestamp: new Date().toISOString(),
          isStreaming: true,
          parentMessageId: userMsg.id,
          branchId,
          runStatus: 'queued',
          runPhase: 'queued',
          statusMessage: 'Queued',
        }
        // History is taken AFTER the user turn exists, so the interaction is in it.
        const history = inferenceHistory([...get().messages, userMsg], branchId)
        set((s) => {
          s.messages.push(userMsg, assistantMsg)
          s.isStreaming = true
          const conversation = s.conversations.find((item) => item.id === s.activeConversationId)
          if (conversation) { conversation.messages = s.messages; conversation.updatedAt = new Date().toISOString() }
        })
        persistActiveConversation()

        let interactionJson: string
        try {
          if (typeof interaction !== 'object' || interaction === null || Array.isArray(interaction)) {
            throw new Error('Interaction payload must be a JSON object')
          }
          const encoded = JSON.stringify(interaction)
          if (!encoded) {
            throw new Error('Interaction payload must be a JSON object')
          }
          interactionJson = encoded
        } catch (error) {
          get().updateLifecycle({
            type: 'error',
            messageId: assistantId,
            runId: assistantId,
            error: {
              code: 'interaction_encode_failed',
              phase: 'queued',
              message: error instanceof Error ? error.message : 'Interaction payload could not be encoded',
              retryable: false,
            },
          })
          return
        }

        if (!isTauri()) {
          if (!sourceRunId) {
            get().updateLifecycle({ type: 'error', messageId: assistantId, runId: assistantId, error: {
              code: 'interaction_source_run_missing', phase: 'queued', message: 'The source UAR run is unavailable for this interaction.', retryable: false,
            } })
            return
          }
          wireAdapter.bind(assistantId)
          try {
            for await (const event of streamHostedA2uiAction(interaction, sourceRunId)) wireAdapter.handle(event)
          } catch (error) {
            wireAdapter.handle({ type: 'run_error', run_id: assistantId, error: {
              code: 'interaction_start_failed', phase: 'tool_execution', message: error instanceof Error ? error.message : String(error), retryable: true,
            } })
          }
          persistActiveConversation()
          return
        }

        wireAdapter.bind(assistantId)
        await sendAgentInteractionA2ui(interactionJson, history, get().activeConversationId, branchId)
          .then((runId) => {
            get().updateLifecycle({ type: 'started', messageId: assistantId, runId })
          })
          .catch((error: unknown) => {
            console.error(error)
            get().updateLifecycle({
              type: 'error',
              messageId: assistantId,
              runId: assistantId,
              error: {
                code: 'interaction_start_failed',
                phase: 'queued',
                message: error instanceof Error ? error.message : String(error),
                retryable: true,
              },
            })
          })
        persistActiveConversation()
      },

      resubmitMessage: async (userMessageId: string) => {
        if (get().isStreaming) return
        const sourceIndex = get().messages.findIndex((message) => message.id === userMessageId && message.role === 'user')
        if (sourceIndex < 0) return
        const source = get().messages[sourceIndex]!
        const text = source.content
          .filter((block): block is Extract<ContentBlock, { type: 'text' }> => block.type === 'text')
          .map((block) => block.text)
          .join('')
        if (!text.trim()) return
        const branchId = crypto.randomUUID()
        const clonedUser: Message = {
          ...source,
          id: crypto.randomUUID(),
          branchId,
          timestamp: new Date().toISOString(),
          updatedAt: undefined,
        }
        const assistantId = crypto.randomUUID()
        const assistant: Message = {
          id: assistantId,
          role: 'assistant',
          content: [],
          timestamp: new Date().toISOString(),
          isStreaming: true,
          parentMessageId: clonedUser.id,
          branchId,
          runStatus: 'queued',
        }
        const history = inferenceHistory(
          get().messages.slice(0, sourceIndex),
          source.branchId ?? get().activeConversationId,
        )
        set((state) => {
          state.messages.push(clonedUser, assistant)
          state.isStreaming = true
          const conversation = state.conversations.find((item) => item.id === state.activeConversationId)
          if (conversation) { conversation.messages = state.messages; conversation.updatedAt = new Date().toISOString() }
        })
        persistActiveConversation()
        await runInferenceTurn(text, assistantId, branchId, history, get().activeConversationId)
      },

      deleteResponse: async (assistantMessageId: string) => {
        if (get().isStreaming) return
        const messages = get().messages
        const index = messages.findIndex((message) => message.id === assistantMessageId && message.role === 'assistant')
        if (index < 0) return
        const branchId = messages[index]!.branchId
        const removed = messages
          .slice(index)
          .filter((message) => message.branchId === branchId)
          .map((message) => message.id)
        if (isTauri()) await memoryDeleteMessageContext(removed)
        await deleteMessageRecords(removed)
        const removedSet = new Set(removed)
        set((state) => {
          state.messages = state.messages.filter((message) => !removedSet.has(message.id))
          const conversation = state.conversations.find((item) => item.id === state.activeConversationId)
          if (conversation) { conversation.messages = state.messages; conversation.updatedAt = new Date().toISOString() }
        })
        persistActiveConversation()
      },

      regenerateResponse: async (assistantMessageId: string) => {
        if (get().isStreaming) return
        const assistantIndex = get().messages.findIndex((message) => message.id === assistantMessageId && message.role === 'assistant')
        if (assistantIndex < 0) return
        const assistant = get().messages[assistantIndex]!
        const userIndex = assistant.parentMessageId
          ? get().messages.findIndex((message) => message.id === assistant.parentMessageId)
          : assistantIndex - 1
        const user = get().messages[userIndex]
        if (!user || user.role !== 'user') return
        const text = user.content
          .filter((block): block is Extract<ContentBlock, { type: 'text' }> => block.type === 'text')
          .map((block) => block.text)
          .join('')
        const branchId = assistant.branchId ?? user.branchId ?? get().activeConversationId
        const history = inferenceHistory(get().messages.slice(0, userIndex), branchId)
        await get().deleteResponse(assistantMessageId)
        const replacementId = crypto.randomUUID()
        const replacement: Message = {
          id: replacementId,
          role: 'assistant',
          content: [],
          timestamp: new Date().toISOString(),
          isStreaming: true,
          parentMessageId: user.id,
          branchId,
          runStatus: 'queued',
        }
        set((state) => {
          state.messages.push(replacement)
          state.isStreaming = true
          const conversation = state.conversations.find((item) => item.id === state.activeConversationId)
          if (conversation) { conversation.messages = state.messages; conversation.updatedAt = new Date().toISOString() }
        })
        persistActiveConversation()
        await runInferenceTurn(text, replacementId, branchId, history, get().activeConversationId)
      },

      hydrateConversations: async () => {
        if (get().isHydrated) return
        const rows = await listConversationRecords(CHAT_TENANT_ID)
        const hydrated = await Promise.all(rows.map(async (row) => {
          const messageRows = await listMessageRecords(CHAT_TENANT_ID, row.id)
          const messages = messageRows.length > 0 ? messageRows.map((message): Message => ({
            id: message.id,
            role: message.role as Message['role'],
            content: message.blocks as Message['content'],
            timestamp: message.created_at,
            updatedAt: message.updated_at,
            isStreaming: false,
            runId: message.run_id ?? undefined,
            parentMessageId: message.parent_message_id ?? undefined,
            branchId: message.branch_id,
            runStatus: message.run_status as Message['runStatus'],
            runPhase: message.run_phase ?? undefined,
            statusMessage: message.status_message ?? undefined,
            intent: (message.intent ?? undefined) as Message['intent'],
            error: message.error as Message['error'],
            generationSettings: message.generation_settings as Message['generationSettings'],
            diagnostics: message.diagnostics as Message['diagnostics'],
            contextPlanVersion: message.context_plan_version ?? undefined,
            contextTelemetry: message.context_telemetry,
            // Read back so a resubmit re-sends the same files rather than
            // silently dropping them.
            ...(Array.isArray(message.attachments) && message.attachments.length > 0
              ? { attachmentPaths: message.attachments as string[] }
              : {}),
            ...(message.provenance
              ? { provenance: message.provenance as Message['provenance'] }
              : {}),
          })) : row.messages as Message[]
          return {
            id: row.id, title: row.title, messages,
            createdAt: row.created_at, updatedAt: row.updated_at,
          }
        }))
        set((s) => {
          if (hydrated.length > 0) {
            s.conversations = hydrated
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
        // An armed steer belongs to the turn it was armed from. Carrying it into
        // another conversation would silently prefix the next message there with
        // a correction preamble about a stop that never happened.
        s.pendingSteer = false
      }),

      renameConversation: (id, title) => {
        set((s) => {
          const conversation = s.conversations.find((item) => item.id === id)
          if (conversation) { conversation.title = title; conversation.updatedAt = new Date().toISOString() }
        })
        if (id === get().activeConversationId) persistActiveConversation()
      },

      deleteConversation: async (id) => {
        if (isTauri()) await memoryDeleteConversationContext(id)
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

      cancelActiveRun: async () => {
        const runId = get().activeRunId
        if (!runId) return
        const message = get().messages.find((item) => item.runId === runId)
        if (message) get().updateLifecycle({
          type: 'phase', messageId: message.id, runId,
          phase: message.runPhase ?? 'generation', status: 'preparing', message: 'Cancelling',
        })
        // Mirror `runInferenceTurn`'s THREE-way lane split. This branched only on
        // isTauri(), so a Stop during a browser-local WebLLM run POSTed to the
        // remote UAR service for a run it had never heard of — while tokens kept
        // streaming and the UI showed "Cancelling", so it looked like it worked.
        // Every branch's outcome is checked. Previously the return value and any
        // rejection were both discarded, so a refused or failed cancel left the
        // turn showing "Cancelling" forever with no way out.
        try {
          if (isTauri()) {
            const accepted = await cancelAgentRun(runId)
            settleCancel(get, message?.id, accepted)
            return
          }
          if (useLaneStore.getState().lane === 'local') {
            await interruptWebLlm()
            return
          }
          await cancelHostedChat(get().activeConversationId)
        } catch (error: unknown) {
          if (!message) return
          get().updateLifecycle({
            type: 'error', messageId: message.id, runId,
            error: {
              code: 'run_cancel_failed', phase: 'generation', retryable: true,
              message: error instanceof Error ? error.message : 'Cancelling this run failed.',
            },
          })
        }
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
            } else if ((wire.type === 'run_finished' || wire.type === 'run_error' || wire.type === 'run_cancelled') && startedAt !== null) {
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
