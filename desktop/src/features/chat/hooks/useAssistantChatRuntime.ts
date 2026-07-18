// TJ-ARCH-MOB-001 compliant — Assistant UI adapter; components never import the store.
import { useExternalStoreRuntime, type ThreadMessageLike } from '@assistant-ui/react'
import { useEffect } from 'react'
import type { ContentBlock, Message } from '@/bridge/a2ui/types'
import { useChatStore } from '../stores/chatStore'

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

function convertMessage(message: Message): ThreadMessageLike {
  return {
    id: message.id,
    role: message.role,
    createdAt: new Date(message.timestamp),
    content: message.content.map(toAssistantPart),
    ...(message.role === 'assistant' ? {
      status: message.isStreaming
        ? { type: 'running' as const }
        : { type: 'complete' as const, reason: 'stop' as const },
    } : {}),
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

  useEffect(() => {
    void hydrateConversations().catch((cause: unknown) => console.error('conversation hydration failed', cause))
  }, [hydrateConversations])

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
      if (text) await sendMessage(text)
    },
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
