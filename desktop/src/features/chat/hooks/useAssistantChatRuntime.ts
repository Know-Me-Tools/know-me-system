// TJ-ARCH-MOB-001 compliant — Assistant UI adapter; components never import the store.
import { useExternalStoreRuntime, type ThreadMessageLike } from '@assistant-ui/react'
import { useEffect } from 'react'
import type { ContentBlock, Message } from '@/bridge/a2ui/types'
import { createAttachmentAdapter, takeStagedAttachments } from '../adapters/attachmentAdapter'
import { localMediaSrc } from '../lib/localMedia'
import { useChatStore } from '../stores/chatStore'
import { useLaneStore } from '../stores/laneStore'

type AssistantPart = Exclude<ThreadMessageLike['content'], string>[number]

function parseObject(value: string): Record<string, unknown> {
  try {
    const parsed: unknown = JSON.parse(value)
    return parsed !== null && typeof parsed === 'object' && !Array.isArray(parsed)
      ? parsed as Record<string, unknown>
      : { value: parsed }
  } catch {
    return { value }
  }
}

function toAssistantPart(block: ContentBlock): AssistantPart {
  switch (block.type) {
    case 'text': return { type: 'text', text: block.text }
    case 'thinking': return { type: 'reasoning', text: block.text }
    case 'code': return { type: 'text', text: `\`\`\`${block.language}\n${block.code}\n\`\`\`` }
    case 'citation': return { type: 'data', name: 'citation', data: block }
    case 'memory': return { type: 'data', name: 'memory', data: block }
    case 'toolUse': return {
      type: 'tool-call', toolCallId: block.id, toolName: block.name,
      argsText: block.inputJson,
    }
    case 'toolResult': return {
      type: 'tool-call', toolCallId: block.toolUseId, toolName: 'tool result', args: {},
      result: parseObject(block.outputJson), isError: block.isError,
    }
    case 'skill': return { type: 'data', name: 'skill', data: block }
    case 'artifact': return { type: 'data', name: 'artifact', data: block }
    // Three carriers, checked in order of directness. `path` points into
    // app-owned storage and needs converting to something the webview can load
    // — a bare filesystem path in an <img src> resolves to nothing.
    //
    // `filename` is assistant-ui's only descriptive field on an image part
    // (there is no `alt`), so the block's alt text goes there and the mime is
    // the fallback, as before.
    case 'image': {
      const label = block.alt ?? block.mime
      if (block.url) return { type: 'image', image: block.url, filename: label }
      if (block.dataBase64) {
        return { type: 'image', image: `data:${block.mime};base64,${block.dataBase64}`, filename: label }
      }
      if (block.path) {
        // Empty on the web build: there is no asset protocol, so a stored path
        // cannot be loaded at all. Fall through rather than emitting an <img>
        // with src="" — a blank src renders a broken image with no hint why,
        // which is the same silent failure as the no-carrier case below.
        const src = localMediaSrc(block.path)
        if (src) return { type: 'image', image: src, filename: label }
      }
      // No usable carrier. Previously a missing one produced
      // `data:image/png;base64,` — an empty data URI, broken and unexplained.
      return { type: 'text', text: `[image unavailable: ${label}]` }
    }
    case 'divider': return { type: 'text', text: '\n\n' }
  }
}

/**
 * Merge adjacent text blocks, and adjacent thinking blocks, into one.
 *
 * A streamed turn arrives as many small deltas, and a transcript persisted by an
 * older client keeps them as separate blocks. Rendered one-per-block, that is N
 * chat bubbles where the user wrote one message — which reads as a layout bug
 * rather than what it is. Flutter has always done this
 * (`coalesceContentBlocks`, gen_ui_widgets/lib/src/content_block.dart:169);
 * React had no equivalent.
 *
 * Only text and thinking merge. Every other variant is a semantic boundary and
 * must survive intact.
 */
export function coalesceContentBlocks(blocks: ContentBlock[]): ContentBlock[] {
  const merged: ContentBlock[] = []
  for (const block of blocks) {
    const previous = merged.at(-1)
    if (block.type === 'text' && previous?.type === 'text') {
      merged[merged.length - 1] = { type: 'text', text: previous.text + block.text }
      continue
    }
    if (block.type === 'thinking' && previous?.type === 'thinking') {
      merged[merged.length - 1] = { type: 'thinking', text: previous.text + block.text }
      continue
    }
    merged.push(block)
  }
  return merged
}

/**
 * Whether a tool result is the local engine asking for approval rather than
 * reporting an outcome.
 *
 * Rust returns this shape when a proposed tool call is not provably read-only,
 * non-destructive and closed-world: the call is stored, NOT executed, and the
 * user decides. Mirrors Flutter's `_isToolConfirmationRequired`
 * (gen_ui_widgets/lib/src/content_block_view.dart) field for field — a looser
 * check here would make React offer approval for results Flutter treats as
 * plain failures.
 */
export function toolApprovalRequest(output: Record<string, unknown>): string | null {
  if (output.code !== 'local_tool_requires_confirmation') return null
  if (output.requiresConfirmation !== true) return null
  if (output.executionStatus !== 'not_executed') return null
  const approvalId = output.approvalId
  // Without an id there is nothing to approve — showing the card anyway would
  // be a button that cannot work.
  return typeof approvalId === 'string' && approvalId ? approvalId : null
}

/**
 * Assistant UI models a tool invocation and its eventual result as one
 * `tool-call` resource. The canonical transcript deliberately stores them as
 * two ordered ContentBlocks, so join them at this adapter boundary instead of
 * exposing two resources with the same toolCallId.
 */
function toAssistantParts(input: ContentBlock[]): AssistantPart[] {
  const blocks = coalesceContentBlocks(input)
  const toolUseIds = new Set(
    blocks
      .filter((block): block is Extract<ContentBlock, { type: 'toolUse' }> => block.type === 'toolUse')
      .map((block) => block.id),
  )
  const results = new Map(
    blocks
      .filter((block): block is Extract<ContentBlock, { type: 'toolResult' }> => block.type === 'toolResult')
      .map((block) => [block.toolUseId, block]),
  )

  return blocks.flatMap((block): AssistantPart[] => {
    if (block.type === 'toolResult' && toolUseIds.has(block.toolUseId)) return []
    if (block.type !== 'toolUse') return [toAssistantPart(block)]

    const result = results.get(block.id)
    const output = result ? parseObject(result.outputJson) : null
    const approvalId = output ? toolApprovalRequest(output) : null
    return [{
      type: 'tool-call',
      toolCallId: block.id,
      toolName: block.name,
      argsText: block.inputJson,
      ...(result ? { result: output, isError: result.isError } : {}),
      // React has had the whole approval card built since it shipped; it fires
      // on `requires-action`, which nothing ever emitted. Setting
      // `tool_approval: 'ask'` from React's OWN toolbar therefore produced an
      // unresolvable stall — the model waited on a decision the UI could not
      // offer.
      ...(approvalId ? { status: { type: 'requires-action' as const, reason: 'tool-approval' as const } } : {}),
    }]
  })
}

/** Map our run status onto assistant-ui's message status.
 *
 *  A cancelled turn must report `incomplete/cancelled`, not `complete/stop`.
 *  Reporting a stop as a normal completion is how the stopped-turn state became
 *  invisible to assistant-ui: `incomplete` + `reason: 'cancelled'` is first-class
 *  in @assistant-ui/core, and the stopped-turn action bar keys off it.
 *
 *  `error` deliberately still reports `complete/stop`. Giving it
 *  `incomplete/error` would make `MessagePrimitive.Error` render *in addition to*
 *  the existing `runError` data part above — duplicate error text on every failed
 *  run. Collapsing those two is a separate change.
 */
export function assistantStatus(message: Message): ThreadMessageLike['status'] {
  if (message.isStreaming) return { type: 'running' }
  if (message.runStatus === 'cancelled') return { type: 'incomplete', reason: 'cancelled' }
  return { type: 'complete', reason: 'stop' }
}

export function convertMessage(message: Message): ThreadMessageLike {
  const content = toAssistantParts(message.content)
  if (message.role === 'assistant') {
    if (message.runStatus === 'error' && message.error) {
      content.push({ type: 'data', name: 'runError', data: message.error } as AssistantPart)
    } else if (message.runStatus === 'cancelled') {
      content.push({ type: 'data', name: 'runCancelled', data: {
        phase: message.runPhase ?? 'generation',
        // Matches the phrase every Rust lane emits (gen_ui_agent/src/chat.rs).
        // One state must not carry two wordings; this fallback said "Run
        // cancelled", which only ever showed if Rust sent nothing.
        message: message.statusMessage ?? 'Cancelled by user',
        // How much survived the stop. The bar uses this to say "kept N blocks,
        // still in context" — and to stay silent at zero rather than claim work
        // that does not exist.
        keptBlocks: message.content.length,
      } } as AssistantPart)
    } else if (message.isStreaming && message.statusMessage) {
      content.push({ type: 'data', name: 'runLifecycle', data: {
        phase: message.runPhase ?? 'preparing', message: message.statusMessage,
      } } as AssistantPart)
    }
  }
  return {
    id: message.id,
    role: message.role,
    createdAt: new Date(message.timestamp),
    content,
    ...(message.role === 'assistant' ? { status: assistantStatus(message) } : {}),
  }
}

function textFromComposer(content: readonly { type: string; text?: string }[]): string {
  return content.filter((part) => part.type === 'text').map((part) => part.text ?? '').join('\n').trim()
}

/** The literal the approval card sends on Allow (tool-fallback.tsx). */
const APPROVED_RESULT = 'Approved by user'

/** Find the approval id behind a tool call, so a decision can be routed to Rust.
 *
 *  The id lives in the toolResult payload rather than on the part, because the
 *  part is assistant-ui's shape and the payload is Rust's. */
function approvalIdFor(messages: Message[], messageId: string, toolCallId: string): string | null {
  const message = messages.find((item) => item.id === messageId)
  if (!message) return null
  for (const block of message.content) {
    if (block.type !== 'toolResult' || block.toolUseId !== toolCallId) continue
    return toolApprovalRequest(parseObject(block.outputJson))
  }
  return null
}

export function useAssistantChatRuntime() {
  const messages = useChatStore((state) => state.messages)
  const isStreaming = useChatStore((state) => state.isStreaming)
  const sendMessage = useChatStore((state) => state.sendMessage)
  const conversations = useChatStore((state) => state.conversations)
  const activeConversationId = useChatStore((state) => state.activeConversationId)
  const hydrateConversations = useChatStore((state) => state.hydrateConversations)
  const createConversation = useChatStore((state) => state.createConversation)
  const switchConversation = useChatStore((state) => state.switchConversation)
  const renameConversation = useChatStore((state) => state.renameConversation)
  const deleteConversation = useChatStore((state) => state.deleteConversation)
  const cancelActiveRun = useChatStore((state) => state.cancelActiveRun)
  const sendSteer = useChatStore((state) => state.sendSteer)
  const pendingSteer = useChatStore((state) => state.pendingSteer)
  const disarmSteer = useChatStore((state) => state.disarmSteer)
  const approveToolConfirmation = useChatStore((state) => state.approveToolConfirmation)
  const initLane = useLaneStore((state) => state.init)

  useEffect(() => {
    void hydrateConversations().catch((cause: unknown) => console.error('conversation hydration failed', cause))
  }, [hydrateConversations])

  useEffect(() => {
    void initLane().catch((cause: unknown) => console.error('lane initialization failed', cause))
  }, [initLane])

  return useExternalStoreRuntime({
    messages,
    isRunning: isStreaming,
    suggestions: [
      { prompt: 'Help me reflect on what has been taking most of my energy lately.' },
      { prompt: 'What patterns do you notice across my recent memories?' },
      { prompt: 'Make a Mermaid map of the goals and people I have mentioned.' },
    ],
    convertMessage,
    onNew: async (message) => {
      const text = textFromComposer(message.content)
      // Resolve the staged paths for whatever the composer is sending. Doing it
      // here rather than in the store keeps the adapter's bookkeeping out of the
      // transport layer.
      const attachments = takeStagedAttachments((message.attachments ?? []).map((item) => item.id))
      // An attachment-only turn is legitimate — Flutter substitutes the same
      // prompt so the model is told what to do with the files.
      const body = text || (attachments.length > 0 ? 'Please review the attached files.' : '')
      if (!body) return
      // Steer arms the composer rather than sending, so the correction the user
      // then types is what gets marked — routing happens here, at submit time.
      if (pendingSteer) {
        disarmSteer()
        await sendSteer(body)
        return
      }
      await sendMessage(body, attachments.length > 0 ? { attachments } : undefined)
    },
    onCancel: cancelActiveRun,
    // The approval card's Allow/Deny lands here. React has always HAD the card;
    // with nothing emitting `requires-action` and nothing receiving the answer,
    // setting `tool_approval: 'ask'` from React's own toolbar was a stall the
    // user could not resolve.
    onAddToolResult: async ({ messageId, toolCallId, result }) => {
      const approvalId = approvalIdFor(useChatStore.getState().messages, messageId, toolCallId)
      if (!approvalId) return
      // Denial needs no round trip: the call was never executed, so there is
      // nothing to undo and no continuation to start. Rust keeps the pending
      // registration until the process ends, which is the correct outcome for
      // a call the user rejected.
      if (result !== APPROVED_RESULT) return
      await approveToolConfirmation(messageId, approvalId)
    },
    adapters: {
      // Registered at last: the Plus button, the dropzone and the chip row have
      // always rendered, but with no adapter behind them picking a file did
      // nothing at all.
      // Reads the id at pick time rather than closing over it: the adapter
      // outlives any single conversation, so a captured value would file a
      // late attachment under whichever thread was open when it was built.
      attachments: createAttachmentAdapter(() => useChatStore.getState().activeConversationId),
      threadList: {
        threadId: activeConversationId,
        threads: conversations.map((conversation) => ({
          id: conversation.id,
          status: 'regular' as const,
          title: conversation.title,
          custom: { updatedAt: conversation.updatedAt },
        })),
        onSwitchToNewThread: createConversation,
        onSwitchToThread: (id) => switchConversation(id),
        onRename: (id, title) => renameConversation(id, title),
        onDelete: (id) => deleteConversation(id),
      },
    },
  })
}
