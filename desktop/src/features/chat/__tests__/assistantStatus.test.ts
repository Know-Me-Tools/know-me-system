// TJ-ARCH-MOB-001 compliant
//
// A cancelled turn must be legible to assistant-ui as *incomplete*, not as a
// normal completion. Reporting `complete/stop` for a stop is what made the
// stopped-turn state invisible to the primitives — the action bar keys off this.
import { describe, expect, it, vi } from 'vitest'

// The hook module pulls in lane init and hydration on import; only the two pure
// functions under test are needed, so stub the IPC surface they never touch.
vi.mock('@tauri-apps/api/core', () => ({ isTauri: () => false }))
vi.mock('@prometheus-ags/tauri-plugin-gen-ui', () => ({}))

import { assistantStatus, convertMessage } from '../hooks/useAssistantChatRuntime'
import type { Message } from '@/bridge/a2ui/types'

function assistant(overrides: Partial<Message> = {}): Message {
  return {
    id: 'assistant-1',
    role: 'assistant',
    content: [],
    timestamp: '2026-07-19T12:00:00.000Z',
    ...overrides,
  }
}

describe('assistantStatus', () => {
  it('reports a cancelled turn as incomplete', () => {
    expect(assistantStatus(assistant({ runStatus: 'cancelled' })))
      .toEqual({ type: 'incomplete', reason: 'cancelled' })
  })

  it('reports a streaming turn as running', () => {
    expect(assistantStatus(assistant({ isStreaming: true })))
      .toEqual({ type: 'running' })
  })

  it('reports a finished turn as complete', () => {
    expect(assistantStatus(assistant({ runStatus: 'complete' })))
      .toEqual({ type: 'complete', reason: 'stop' })
  })

  it('leaves an errored turn reporting complete', () => {
    // Deliberate: `incomplete/error` would make MessagePrimitive.Error render in
    // addition to the existing runError data part — duplicate error text on every
    // failed run. Collapsing those two is a separate change.
    expect(assistantStatus(assistant({ runStatus: 'error' })))
      .toEqual({ type: 'complete', reason: 'stop' })
  })
})

describe('convertMessage cancelled turn', () => {
  it('carries the kept-block count and the shared wording', () => {
    const converted = convertMessage(assistant({
      runStatus: 'cancelled',
      statusMessage: 'Cancelled by user',
      content: [{ type: 'text', text: 'partial' }, { type: 'divider' }],
    }))

    const part = (converted.content as { type: string; name?: string; data?: unknown }[])
      .find((p) => p.type === 'data' && p.name === 'runCancelled')
    expect(part, 'a cancelled turn must emit the runCancelled part').toBeDefined()
    expect(part?.data).toMatchObject({ message: 'Cancelled by user', keptBlocks: 2 })
    expect(converted.status).toEqual({ type: 'incomplete', reason: 'cancelled' })
  })

  it('falls back to the phrase every Rust lane uses', () => {
    // Only reachable if Rust sent no message at all — but it must not introduce
    // a second wording for one state.
    const converted = convertMessage(assistant({ runStatus: 'cancelled' }))
    const part = (converted.content as { type: string; name?: string; data?: { message?: string } }[])
      .find((p) => p.type === 'data' && p.name === 'runCancelled')
    expect(part?.data?.message).toBe('Cancelled by user')
  })
})
