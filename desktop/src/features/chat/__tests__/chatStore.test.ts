// TJ-ARCH-MOB-001 compliant
import { beforeEach, describe, expect, it, vi } from 'vitest'

const mocks = vi.hoisted(() => ({
  isTauri: vi.fn(() => true),
  cancelAgentRun: vi.fn(),
  memoryDeleteConversationContext: vi.fn(),
  memoryDeleteMessageContext: vi.fn(),
  onChatEvent: vi.fn(),
  sendAgentInteractionA2ui: vi.fn(),
  streamAgentA2ui: vi.fn(),
  deleteConversationRecord: vi.fn(),
  deleteMessageRecords: vi.fn(),
  listConversationRecords: vi.fn(),
  listMessageRecords: vi.fn(),
  saveConversationRecord: vi.fn(),
  saveMessageRecords: vi.fn(),
  getSessionByok: vi.fn(),
}))

vi.mock('@tauri-apps/api/core', () => ({
  isTauri: mocks.isTauri,
}))

vi.mock('@prometheus-ags/tauri-plugin-gen-ui', () => ({
  cancelAgentRun: mocks.cancelAgentRun,
  memoryDeleteConversationContext: mocks.memoryDeleteConversationContext,
  memoryDeleteMessageContext: mocks.memoryDeleteMessageContext,
  onChatEvent: mocks.onChatEvent,
  sendAgentInteractionA2ui: mocks.sendAgentInteractionA2ui,
  streamAgentA2ui: mocks.streamAgentA2ui,
}))

vi.mock('@/features/entities/stores/entityRuntime', () => ({
  deleteConversationRecord: mocks.deleteConversationRecord,
  deleteMessageRecords: mocks.deleteMessageRecords,
  listConversationRecords: mocks.listConversationRecords,
  listMessageRecords: mocks.listMessageRecords,
  saveConversationRecord: mocks.saveConversationRecord,
  saveMessageRecords: mocks.saveMessageRecords,
}))

vi.mock('@/features/settings/stores/providerStore', () => ({
  getSessionByok: mocks.getSessionByok,
}))

vi.mock('../api/webllmLane', () => ({
  streamWebLlm: vi.fn(),
}))

vi.mock('../api/hostedLane', () => ({
  streamHostedChat: vi.fn(),
}))

import { useChatStore } from '../stores/chatStore'
import type { Conversation } from '../stores/chatStore'
import type { Message } from '@/bridge/a2ui/types'

function assistantMessage(overrides: Partial<Message> = {}): Message {
  return {
    id: 'assistant-1',
    role: 'assistant',
    content: [],
    timestamp: '2026-07-19T12:00:00.000Z',
    updatedAt: undefined,
    isStreaming: true,
    runId: 'run-1',
    runStatus: 'generating',
    runPhase: 'generation',
    statusMessage: 'Generating',
    branchId: 'conversation-1',
    ...overrides,
  }
}

function conversation(messages: Message[]): Conversation {
  return {
    id: 'conversation-1',
    title: 'Cancellation test',
    messages,
    createdAt: '2026-07-19T12:00:00.000Z',
    updatedAt: '2026-07-19T12:00:00.000Z',
  }
}

beforeEach(() => {
  vi.clearAllMocks()
  mocks.isTauri.mockReturnValue(true)
  mocks.cancelAgentRun.mockResolvedValue(true)
  mocks.listConversationRecords.mockResolvedValue([])
  mocks.listMessageRecords.mockResolvedValue([])
  mocks.saveConversationRecord.mockResolvedValue(undefined)
  mocks.saveMessageRecords.mockResolvedValue(undefined)
  mocks.sendAgentInteractionA2ui.mockResolvedValue('interaction-run-1')
  const message = assistantMessage()
  useChatStore.setState({
    messages: [message],
    conversations: [conversation([message])],
    activeConversationId: 'conversation-1',
    activeRunId: 'run-1',
    isStreaming: true,
    isHydrated: true,
  })
})

describe('chatStore A2UI interactions', () => {
  it('creates a run placeholder and submits structured interactions through Tauri while a UAR run is active', async () => {
    const user: Message = {
      id: 'user-1',
      role: 'user',
      content: [{ type: 'text', text: 'Show me the approval surface.' }],
      timestamp: '2026-07-19T12:00:00.000Z',
      branchId: 'conversation-1',
    }
    const artifact: Message = {
      id: 'assistant-artifact-1',
      role: 'assistant',
      content: [{ type: 'artifact', id: 'artifact-123', kind: 'a2ui', content: '{"surface":"approval"}' }],
      timestamp: '2026-07-19T12:00:01.000Z',
      branchId: 'conversation-1',
    }
    useChatStore.setState({
      messages: [user, artifact],
      conversations: [conversation([user, artifact])],
      activeConversationId: 'conversation-1',
      activeRunId: 'uar-run-paused-1',
      isStreaming: true,
      isHydrated: true,
    })

    const interaction = {
      a2uiResponse: {
        artifactId: 'artifact-123',
        response: { approved: true },
      },
    }
    await useChatStore.getState().submitInteraction(interaction)

    expect(mocks.sendAgentInteractionA2ui).toHaveBeenCalledWith(
      JSON.stringify(interaction),
      [{ role: 'user', content: 'Show me the approval surface.' }],
      'conversation-1',
      'conversation-1',
    )
    const state = useChatStore.getState()
    const assistant = state.messages.at(-1)
    expect(assistant?.role).toBe('assistant')
    expect(assistant?.parentMessageId).toBe('assistant-artifact-1')
    expect(assistant?.runId).toBe('interaction-run-1')
    expect(assistant?.runStatus).toBe('queued')
    expect(assistant?.isStreaming).toBe(true)
    expect(state.activeRunId).toBe('interaction-run-1')
  })

  it('persists a visible error bubble for invalid interaction payloads', async () => {
    const source = assistantMessage({ id: 'assistant-artifact-1', isStreaming: false, runStatus: 'complete' })
    useChatStore.setState({
      messages: [source],
      conversations: [conversation([source])],
      activeConversationId: 'conversation-1',
      activeRunId: null,
      isStreaming: false,
      isHydrated: true,
    })

    await useChatStore.getState().submitInteraction(['not', 'an', 'object'])

    expect(mocks.sendAgentInteractionA2ui).not.toHaveBeenCalled()
    const state = useChatStore.getState()
    const error = state.messages.at(-1)
    expect(error?.role).toBe('assistant')
    expect(error?.runStatus).toBe('error')
    expect(error?.isStreaming).toBe(false)
    expect(error?.error?.code).toBe('interaction_encode_failed')
    expect(state.isStreaming).toBe(false)
    expect(state.activeRunId).toBeNull()
  })
})

describe('chatStore cancellation', () => {
  it('marks the active run as cancelling and calls the Tauri cancel command', async () => {
    await useChatStore.getState().cancelActiveRun()

    const state = useChatStore.getState()
    const message = state.messages[0]
    expect(mocks.cancelAgentRun).toHaveBeenCalledWith('run-1')
    expect(message?.runStatus).toBe('preparing')
    expect(message?.runPhase).toBe('generation')
    expect(message?.statusMessage).toBe('Cancelling')
    expect(message?.isStreaming).toBe(true)
    expect(state.activeRunId).toBe('run-1')
  })

  it('does not call cancellation when there is no active run', async () => {
    useChatStore.setState({ activeRunId: null })

    await useChatStore.getState().cancelActiveRun()

    expect(mocks.cancelAgentRun).not.toHaveBeenCalled()
  })
})
