// TJ-ARCH-MOB-001 compliant — Assistant UI adapter; components never import the store.
import { useExternalStoreRuntime, type ThreadMessageLike } from '@assistant-ui/react'
import { useEffect } from 'react'
import type { ContentBlock, Message } from '@/bridge/a2ui/types'
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
    case 'image': return block.url
      ? { type: 'image', image: block.url }
      : { type: 'image', image: `data:${block.mime};base64,${block.dataBase64 ?? ''}` }
    case 'divider': return { type: 'text', text: '\n\n' }
  }
}

/**
 * Assistant UI models a tool invocation and its eventual result as one
 * `tool-call` resource. The canonical transcript deliberately stores them as
 * two ordered ContentBlocks, so join them at this adapter boundary instead of
 * exposing two resources with the same toolCallId.
 */
function toAssistantParts(blocks: ContentBlock[]): AssistantPart[] {
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
    return [{
      type: 'tool-call',
      toolCallId: block.id,
      toolName: block.name,
      argsText: block.inputJson,
      ...(result ? { result: parseObject(result.outputJson), isError: result.isError } : {}),
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
      if (!text) return
      // Steer arms the composer rather than sending, so the correction the user
      // then types is what gets marked — routing happens here, at submit time.
      if (pendingSteer) {
        disarmSteer()
        await sendSteer(text)
        return
      }
      await sendMessage(text)
    },
    onCancel: cancelActiveRun,
    adapters: {
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
