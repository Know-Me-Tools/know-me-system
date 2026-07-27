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
  chatApproveToolConfirmation: vi.fn(),
  deleteConversationRecord: vi.fn(),
  deleteMessageRecords: vi.fn(),
  listConversationRecords: vi.fn(),
  listMessageRecords: vi.fn(),
  saveConversationRecord: vi.fn(),
  saveMessageRecords: vi.fn(),
  getSessionByok: vi.fn(),
  cancelHostedChat: vi.fn(),
  interruptWebLlm: vi.fn(),
  streamWebLlm: vi.fn(),
  getActiveLane: vi.fn(),
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
  chatApproveToolConfirmation: mocks.chatApproveToolConfirmation,
  // laneStore imports from this same module, and `runInferenceTurn` calls
  // `laneStore.init()`. Without these the send tests reach real IPC and throw.
  cancelModelOperation: vi.fn(),
  getActiveLane: mocks.getActiveLane,
  hasLocalEngine: vi.fn(async () => true),
  onLocalModelStatus: vi.fn(async () => () => {}),
  prepareLocalModel: vi.fn(),
  setActiveLane: vi.fn(),
  uarRuntimeStatus: vi.fn(async () => null),
  uarServiceConfig: vi.fn(async () => null),
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
  streamWebLlm: mocks.streamWebLlm,
  interruptWebLlm: mocks.interruptWebLlm,
}))

vi.mock('../api/hostedLane', () => ({
  streamHostedChat: vi.fn(),
  streamHostedA2uiAction: vi.fn(),
  cancelHostedChat: mocks.cancelHostedChat,
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
  mocks.cancelHostedChat.mockResolvedValue(undefined)
  mocks.interruptWebLlm.mockResolvedValue(undefined)
  mocks.streamAgentA2ui.mockResolvedValue('run-2')
  mocks.chatApproveToolConfirmation.mockResolvedValue('approval-run-1')
  // Local is the lane these tests certify; leaving it unset would silently
  // exercise a different one.
  mocks.getActiveLane.mockResolvedValue('local')
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

describe('chatStore stopped-turn intents', () => {
  it('sendSteer prefixes the course-correction and marks the turn', async () => {
    useChatStore.setState({ isStreaming: false, activeRunId: null })
    await useChatStore.getState().sendSteer('Focus on the cost side instead.')

    const sent = useChatStore.getState().messages.find((m) => m.intent === 'steer')
    expect(sent, 'the steered turn must be marked').toBeDefined()

    const text = sent!.content.map((b) => (b.type === 'text' ? b.text : '')).join('')
    // The correction survives verbatim AND is framed as a correction. The tag
    // rides in the content because that is the only carrier every lane forwards.
    expect(text).toContain('Focus on the cost side instead.')
    expect(text).toContain('stopped your previous response')

    // The PREFIXED text must be what reaches the lane — not the bare correction.
    expect(mocks.streamAgentA2ui).toHaveBeenCalled()
    const [sentText] = mocks.streamAgentA2ui.mock.calls.at(-1) as [string]
    expect(sentText).toContain('stopped your previous response')
    expect(sentText).toContain('Focus on the cost side instead.')
  })

  it('sendResume replays the cancelled partial turn', async () => {
    // This is the invariant Resume rests on, and it is held by the ABSENCE of a
    // runStatus filter in inferenceHistory — exactly what a later refactor
    // deletes without noticing.
    const user: Message = {
      id: 'user-1', role: 'user', content: [{ type: 'text', text: 'Write a long guide.' }],
      timestamp: '2026-07-19T12:00:00.000Z', branchId: 'conversation-1',
    }
    const stopped = assistantMessage({
      id: 'assistant-1', isStreaming: false, runStatus: 'cancelled',
      statusMessage: 'Cancelled by user',
      content: [{ type: 'text', text: 'Chapter one, partially wri' }],
    })
    useChatStore.setState({
      messages: [user, stopped], conversations: [conversation([user, stopped])],
      isStreaming: false, activeRunId: null,
    })

    await useChatStore.getState().sendResume()

    // streamAgentA2ui(text, history, conversationId, branchId)
    const [text, history] = mocks.streamAgentA2ui.mock.calls.at(-1) as [string, { role: string; content: string }[]]
    expect(text).toContain('Continue your previous response')
    expect(history.map((h) => `${h.role}:${h.content}`)).toEqual([
      'user:Write a long guide.',
      'assistant:Chapter one, partially wri',
    ])
    expect(useChatStore.getState().messages.some((m) => m.intent === 'resume')).toBe(true)
  })

  it('an ordinary send carries no intent', async () => {
    useChatStore.setState({ messages: [], conversations: [conversation([])], isStreaming: false, activeRunId: null })
    await useChatStore.getState().sendMessage('Just a question.')
    const user = useChatStore.getState().messages.find((m) => m.role === 'user')
    expect(user?.intent).toBeUndefined()
  })

  it('arming a steer survives until the send, and clears on conversation switch', () => {
    useChatStore.getState().armSteer()
    expect(useChatStore.getState().pendingSteer).toBe(true)
    useChatStore.getState().disarmSteer()
    expect(useChatStore.getState().pendingSteer).toBe(false)

    // An armed steer belongs to the turn it was armed from.
    useChatStore.setState({ isStreaming: false })
    useChatStore.getState().armSteer()
    useChatStore.getState().switchConversation('conversation-1')
    expect(useChatStore.getState().pendingSteer).toBe(false)
  })
})

describe('chatStore terminal cancellation', () => {
  it('settles the run as cancelled and KEEPS the partial content', () => {
    // Only the intermediate 'Cancelling' state was covered before. The terminal
    // state is the one the whole stopped-turn feature depends on.
    useChatStore.getState().streamBlock({ messageId: 'assistant-1', block: { type: 'text', text: 'Partial answer' } })
    useChatStore.getState().updateLifecycle({
      type: 'cancelled', messageId: 'assistant-1', runId: 'run-1',
      phase: 'generation', message: 'Cancelled by user',
    })

    const message = useChatStore.getState().messages.find((m) => m.id === 'assistant-1')
    expect(message?.runStatus).toBe('cancelled')
    expect(message?.isStreaming).toBe(false)
    expect(message?.statusMessage).toBe('Cancelled by user')
    expect(useChatStore.getState().isStreaming).toBe(false)
    expect(useChatStore.getState().activeRunId).toBeNull()
    // The load-bearing promise: a stop must not discard the work.
    expect(message?.content).toHaveLength(1)
    expect(message?.content[0]).toEqual({ type: 'text', text: 'Partial answer' })
  })
})

describe('chatStore cancel lane routing', () => {
  it('interrupts WebLLM in the browser-local lane instead of POSTing to the hosted service', async () => {
    mocks.isTauri.mockReturnValue(false)
    const { useLaneStore } = await import('../stores/laneStore')
    useLaneStore.setState({ lane: 'local' })

    await useChatStore.getState().cancelActiveRun()

    expect(mocks.interruptWebLlm).toHaveBeenCalledOnce()
    // The bug this replaces: a Stop here POSTed to a UAR session that had never
    // heard of the run, while tokens kept streaming.
    expect(mocks.cancelHostedChat).not.toHaveBeenCalled()
  })

  it('cancels through the hosted service on the web hosted lane', async () => {
    mocks.isTauri.mockReturnValue(false)
    const { useLaneStore } = await import('../stores/laneStore')
    useLaneStore.setState({ lane: 'uar' })

    await useChatStore.getState().cancelActiveRun()

    expect(mocks.cancelHostedChat).toHaveBeenCalledOnce()
    expect(mocks.interruptWebLlm).not.toHaveBeenCalled()
  })
})

describe('chatStore cancel failure handling', () => {
  it('surfaces an error when a refused cancel leaves the turn streaming', async () => {
    // Rust returns false when the run is no longer registered. If the turn is
    // somehow STILL streaming, the UI would otherwise sit on "Cancelling"
    // forever — the return value used to be discarded entirely.
    mocks.cancelAgentRun.mockResolvedValue(false)

    await useChatStore.getState().cancelActiveRun()

    const message = useChatStore.getState().messages.find((m) => m.id === 'assistant-1')
    expect(message?.error?.code).toBe('run_not_active')
    expect(message?.error?.retryable).toBe(false)
  })

  it('stays silent when a refused cancel followed a turn that already ended', async () => {
    // The normal outcome of a second stop tap. Reporting it would overwrite a
    // legitimate stopped turn with a failure bubble and hide the kept work.
    mocks.cancelAgentRun.mockResolvedValue(false)
    useChatStore.setState({
      messages: [assistantMessage({ isStreaming: false, runStatus: 'cancelled' })],
    })

    await useChatStore.getState().cancelActiveRun()

    // The assertion is on `error`, not `runStatus`: cancelActiveRun writes its
    // own optimistic phase before dispatching, so runStatus is transient here.
    // What must hold is that no failure bubble was minted.
    const message = useChatStore.getState().messages.find((m) => m.id === 'assistant-1')
    expect(message?.error).toBeUndefined()
  })

  it('reports a thrown cancel instead of hanging', async () => {
    mocks.cancelAgentRun.mockRejectedValue(new Error('bridge is gone'))

    await useChatStore.getState().cancelActiveRun()

    const message = useChatStore.getState().messages.find((m) => m.id === 'assistant-1')
    expect(message?.error?.code).toBe('run_cancel_failed')
    expect(message?.error?.message).toContain('bridge is gone')
  })
})

describe('attachment persistence round-trip', () => {
  it('writes the real paths instead of a hardcoded empty array', async () => {
    // `attachments: []` used to be hardcoded on every persist, so the column was
    // write-only: the chips rendered in the transcript and the files were gone
    // on reload, with nothing to resubmit.
    vi.useFakeTimers()
    try {
      // Sending is what triggers a persist; the store debounces it, so the
      // timers are advanced rather than waiting on wall-clock.
      await useChatStore.getState().sendMessage('look at these', {
        attachments: [
          { path: '/tmp/a.pdf', name: 'a.pdf', size: 2048, mime: 'application/pdf' },
          { path: '/tmp/b.png', name: 'b.png', size: 4096, mime: 'image/png' },
        ],
      })
      await vi.runAllTimersAsync()
    } finally {
      vi.useRealTimers()
    }

    const rows = mocks.saveMessageRecords.mock.calls.at(-1)?.[0] as
      { role: string; attachments: unknown; blocks: { type: string; kind?: string }[] }[] | undefined
    const userRow = rows?.find((row) => row.role === 'user')
    expect(userRow?.attachments).toEqual(['/tmp/a.pdf', '/tmp/b.png'])
    // The blocks are the transcript half of the same turn: they lead the
    // content so the files read as sent alongside the prompt, as on Flutter.
    expect(userRow?.blocks.filter((block) => block.kind === 'attachment')).toHaveLength(2)
    expect(userRow?.blocks.at(-1)?.type).toBe('text')
  })

  it('hydrates the paths back onto the message', async () => {
    mocks.listConversationRecords.mockResolvedValue([
      { id: 'conversation-1', tenant_id: 't', title: 'Chat', created_at: '2026-07-19T12:00:00.000Z', updated_at: '2026-07-19T12:00:00.000Z', messages: [] },
    ])
    mocks.listMessageRecords.mockResolvedValue([{
      id: 'user-1', tenant_id: 't', conversation_id: 'conversation-1',
      parent_message_id: null, branch_id: 'conversation-1', sequence: 0,
      role: 'user', blocks: [{ type: 'text', text: 'look at these' }],
      run_id: null, run_status: null, run_phase: null, status_message: null,
      intent: null, error: null,
      attachments: ['/tmp/a.pdf'],
      generation_settings: null, diagnostics: null,
      context_plan_version: null, context_telemetry: null,
      created_at: '2026-07-19T12:00:00.000Z', updated_at: '2026-07-19T12:00:00.000Z',
    }])
    useChatStore.setState({ isHydrated: false })

    await useChatStore.getState().hydrateConversations()

    const message = useChatStore.getState().messages.find((item) => item.id === 'user-1')
    expect(message?.attachmentPaths).toEqual(['/tmp/a.pdf'])
  })

  it('leaves attachmentPaths absent on a message that had none', async () => {
    // An empty array and "no attachments" must not be conflated — the absent
    // case is what every ordinary turn looks like.
    mocks.listConversationRecords.mockResolvedValue([
      { id: 'conversation-1', tenant_id: 't', title: 'Chat', created_at: '2026-07-19T12:00:00.000Z', updated_at: '2026-07-19T12:00:00.000Z', messages: [] },
    ])
    mocks.listMessageRecords.mockResolvedValue([{
      id: 'user-2', tenant_id: 't', conversation_id: 'conversation-1',
      parent_message_id: null, branch_id: 'conversation-1', sequence: 0,
      role: 'user', blocks: [{ type: 'text', text: 'plain question' }],
      run_id: null, run_status: null, run_phase: null, status_message: null,
      intent: null, error: null, attachments: [],
      generation_settings: null, diagnostics: null,
      context_plan_version: null, context_telemetry: null,
      created_at: '2026-07-19T12:00:00.000Z', updated_at: '2026-07-19T12:00:00.000Z',
    }])
    useChatStore.setState({ isHydrated: false })

    await useChatStore.getState().hydrateConversations()

    const message = useChatStore.getState().messages.find((item) => item.id === 'user-2')
    expect(message?.attachmentPaths).toBeUndefined()
  })
})

describe('approveToolConfirmation', () => {
  it('starts the continuation run and shows the tool-execution phase', async () => {
    const source = assistantMessage({ id: 'assistant-1', isStreaming: false, runStatus: 'complete' })
    useChatStore.setState({
      messages: [source],
      conversations: [conversation([source])],
      activeConversationId: 'conversation-1',
      activeRunId: null,
      isStreaming: false,
      isHydrated: true,
    })

    await useChatStore.getState().approveToolConfirmation('assistant-1', 'approval-1')

    expect(mocks.chatApproveToolConfirmation).toHaveBeenCalledWith('approval-1')
    const placeholder = useChatStore.getState().messages.at(-1)
    expect(placeholder?.parentMessageId).toBe('assistant-1')
    expect(placeholder?.runPhase).toBe('tool_execution')
    expect(placeholder?.statusMessage).toBe('Approving tool call')
  })

  it('refuses to approve against a turn that is still streaming', async () => {
    // Approving mid-stream would race the run that is still producing the card.
    await useChatStore.getState().approveToolConfirmation('assistant-1', 'approval-1')

    expect(mocks.chatApproveToolConfirmation).not.toHaveBeenCalled()
  })

  it('ignores an empty approval id rather than calling Rust with one', async () => {
    const source = assistantMessage({ isStreaming: false, runStatus: 'complete' })
    useChatStore.setState({ messages: [source], conversations: [conversation([source])] })

    await useChatStore.getState().approveToolConfirmation('assistant-1', '')

    expect(mocks.chatApproveToolConfirmation).not.toHaveBeenCalled()
  })

  it('surfaces a stale approval instead of leaving the bubble spinning', async () => {
    // Approvals are process-local: a card restored after a restart has no
    // pending registration behind it, and this is where the user finds out.
    mocks.chatApproveToolConfirmation.mockRejectedValue(new Error('unknown approval id'))
    const source = assistantMessage({ isStreaming: false, runStatus: 'complete' })
    useChatStore.setState({
      messages: [source], conversations: [conversation([source])], isStreaming: false,
    })

    await useChatStore.getState().approveToolConfirmation('assistant-1', 'approval-stale')

    const placeholder = useChatStore.getState().messages.at(-1)
    expect(placeholder?.runStatus).toBe('error')
    expect(placeholder?.isStreaming).toBe(false)
    expect(placeholder?.error?.code).toBe('tool_approval_start_failed')
  })
})
