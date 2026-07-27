// TJ-ARCH-MOB-001 compliant
//
// setMode's ordering contract. The order is the whole safety property: switch
// the lane FIRST, and only move the displayed selection once the runtime has
// accepted it. Moving it optimistically would make the UI claim a mode the
// runtime refused — worse than an unresponsive control, because the next send
// would silently route somewhere else.
import { beforeEach, describe, expect, it, vi } from 'vitest'

const mocks = vi.hoisted(() => ({
  isTauri: vi.fn(() => true),
  switchLane: vi.fn(),
  laneError: null as string | null,
  conversationPolicySave: vi.fn(),
  conversationEffectiveConfig: vi.fn(),
}))

vi.mock('@tauri-apps/api/core', () => ({ isTauri: mocks.isTauri }))

vi.mock('@prometheus-ags/tauri-plugin-gen-ui', () => ({
  chatControlCatalog: vi.fn(),
  conversationEffectiveConfig: mocks.conversationEffectiveConfig,
  conversationPolicyGet: vi.fn(),
  conversationPolicySave: mocks.conversationPolicySave,
  laneAvailability: vi.fn(),
}))

vi.mock('../stores/laneStore', () => ({
  useLaneStore: {
    getState: () => ({ switchLane: mocks.switchLane, error: mocks.laneError }),
  },
}))

import { laneForMode, useChatControlStore } from '../stores/chatControlStore'

describe('laneForMode', () => {
  it('maps local to the local lane', () => {
    expect(laneForMode('local')).toBe('local')
  })

  it('maps both agentic modes to the uar lane', () => {
    // `agent` and `uar` differ only in which agent artifact the run resolves;
    // both execute through the embedded UAR. Mirrors lane_for_chat_mode.
    expect(laneForMode('uar')).toBe('uar')
    expect(laneForMode('agent')).toBe('uar')
  })
})

describe('setMode', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    mocks.laneError = null
    mocks.conversationPolicySave.mockResolvedValue(undefined)
    mocks.conversationEffectiveConfig.mockResolvedValue(null)
    useChatControlStore.setState({
      conversationId: 'conversation-1',
      policy: { ...useChatControlStore.getState().policy, chat_mode: 'uar', agent_id: 'orchestrator-agent' },
      saving: false,
      error: null,
    })
  })

  it('leaves the selection alone when the runtime refuses the lane', async () => {
    mocks.switchLane.mockResolvedValue(false)
    mocks.laneError = 'No local inference engine in this build.'

    await useChatControlStore.getState().setMode('local')

    const state = useChatControlStore.getState()
    expect(state.policy.chat_mode).toBe('uar')
    expect(state.policy.agent_id).toBe('orchestrator-agent')
    expect(state.error).toBe('No local inference engine in this build.')
    expect(state.saving).toBe(false)
    // Nothing was persisted — a refused switch must not write a policy the
    // runtime cannot honour.
    expect(mocks.conversationPolicySave).not.toHaveBeenCalled()
  })

  it('always surfaces a reason for a refusal, even when the lane store gave none', async () => {
    mocks.switchLane.mockResolvedValue(false)
    mocks.laneError = null

    await useChatControlStore.getState().setMode('local')

    expect(useChatControlStore.getState().error).toBeTruthy()
  })

  it('moves the selection once the lane switch succeeds', async () => {
    mocks.switchLane.mockResolvedValue(true)

    await useChatControlStore.getState().setMode('local')

    const state = useChatControlStore.getState()
    expect(state.policy.chat_mode).toBe('local')
    expect(state.policy.agent_id).toBeNull()
    expect(state.error).toBeNull()
  })

  it('switches the lane before persisting the policy', async () => {
    const order: string[] = []
    mocks.switchLane.mockImplementation(async () => { order.push('switchLane'); return true })
    mocks.conversationPolicySave.mockImplementation(async () => { order.push('save') })

    await useChatControlStore.getState().setMode('agent', 'writer-agent')

    expect(order).toEqual(['switchLane', 'save'])
  })

  it('forces the orchestrator agent for uar mode', async () => {
    mocks.switchLane.mockResolvedValue(true)

    await useChatControlStore.getState().setMode('uar')

    expect(useChatControlStore.getState().policy.agent_id).toBe('orchestrator-agent')
  })

  it('carries the chosen agent id in agent mode', async () => {
    mocks.switchLane.mockResolvedValue(true)

    await useChatControlStore.getState().setMode('agent', 'writer-agent')

    expect(useChatControlStore.getState().policy.agent_id).toBe('writer-agent')
  })

  it('ignores a second call while one is already in flight', async () => {
    // Double-clicking the trigger must not fire two lane switches — the second
    // would race the first and could land the UI on the losing mode.
    mocks.switchLane.mockResolvedValue(true)
    useChatControlStore.setState({ saving: true })

    await useChatControlStore.getState().setMode('local')

    expect(mocks.switchLane).not.toHaveBeenCalled()
  })
})
