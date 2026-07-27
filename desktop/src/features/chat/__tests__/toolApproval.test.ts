// TJ-ARCH-MOB-001 compliant
//
// Tool approval. React already had the whole approval card built — it fires on
// a tool-call part with `status.type === 'requires-action'`, which nothing ever
// emitted. So setting `tool_approval: 'ask'` from React's OWN toolbar produced
// an unresolvable stall: the model waited on a decision the UI could not offer.
//
// The detection must match Flutter's `_isToolConfirmationRequired` field for
// field. A looser check would offer approval for results Flutter treats as
// plain failures, including the one Rust emits for an untrusted server, which
// is deliberately NOT approvable.
import { describe, expect, it, vi } from 'vitest'

vi.mock('@tauri-apps/api/core', () => ({ isTauri: () => false }))
vi.mock('@prometheus-ags/tauri-plugin-gen-ui', () => ({}))

import { convertMessage, toolApprovalRequest } from '../hooks/useAssistantChatRuntime'
import type { ContentBlock, Message } from '@/bridge/a2ui/types'

/** The payload Rust emits when a proposed tool call needs a human decision. */
const CONFIRMATION_PAYLOAD = {
  ok: false,
  code: 'local_tool_requires_confirmation',
  message: 'KnowMe did not execute it.',
  tool: 'write_file',
  approvalId: 'approval-1',
  requiresConfirmation: true,
  executionStatus: 'not_executed',
  reasons: ['tool_may_be_destructive'],
}

function assistant(content: ContentBlock[]): Message {
  return { id: 'a-1', role: 'assistant', content, timestamp: '2026-07-19T12:00:00.000Z' }
}

describe('toolApprovalRequest', () => {
  it('recognises the confirmation payload and returns its id', () => {
    expect(toolApprovalRequest(CONFIRMATION_PAYLOAD)).toBe('approval-1')
  })

  it('ignores an ordinary tool failure', () => {
    expect(toolApprovalRequest({ ok: false, code: 'tool_failed', message: 'nope' })).toBeNull()
  })

  it('ignores the untrusted-server refusal, which is not approvable', () => {
    // Rust emits `local_tool_server_not_trusted` from a sibling branch. It has
    // no pending registration behind it, so offering Approve would be a button
    // that cannot work.
    expect(toolApprovalRequest({
      ok: false,
      code: 'local_tool_server_not_trusted',
      requiresConfirmation: true,
      executionStatus: 'not_executed',
      approvalId: 'approval-1',
    })).toBeNull()
  })

  it('refuses a payload that claims confirmation but was already executed', () => {
    expect(toolApprovalRequest({ ...CONFIRMATION_PAYLOAD, executionStatus: 'executed' })).toBeNull()
  })

  it('refuses a payload with requiresConfirmation false', () => {
    expect(toolApprovalRequest({ ...CONFIRMATION_PAYLOAD, requiresConfirmation: false })).toBeNull()
  })

  it('refuses a payload with no approval id — there would be nothing to approve', () => {
    expect(toolApprovalRequest({ ...CONFIRMATION_PAYLOAD, approvalId: '' })).toBeNull()
    const { approvalId: _omitted, ...withoutId } = CONFIRMATION_PAYLOAD
    expect(toolApprovalRequest(withoutId)).toBeNull()
  })
})

describe('the adapter emits requires-action for an approval request', () => {
  it('marks the tool call as requiring action', () => {
    const converted = convertMessage(assistant([
      { type: 'toolUse', id: 'call-1', name: 'write_file', inputJson: '{"path":"/tmp/x"}' },
      { type: 'toolResult', toolUseId: 'call-1', outputJson: JSON.stringify(CONFIRMATION_PAYLOAD), isError: true },
    ]))
    const part = (converted.content as unknown as { type: string; status?: { type: string } }[])
      .find((item) => item.type === 'tool-call')
    expect(part?.status?.type).toBe('requires-action')
  })

  it('leaves an ordinary completed tool call alone', () => {
    // Every normal tool call must stay untouched — a spurious requires-action
    // would freeze a finished turn behind an approval prompt.
    const converted = convertMessage(assistant([
      { type: 'toolUse', id: 'call-1', name: 'read_file', inputJson: '{}' },
      { type: 'toolResult', toolUseId: 'call-1', outputJson: '{"ok":true}', isError: false },
    ]))
    const part = (converted.content as unknown as { type: string; status?: { type: string } }[])
      .find((item) => item.type === 'tool-call')
    expect(part?.status).toBeUndefined()
  })

  it('still carries the result payload alongside the approval status', () => {
    // The card renders the reasons from that payload; dropping it would leave
    // the user approving a call with no stated justification.
    const converted = convertMessage(assistant([
      { type: 'toolUse', id: 'call-1', name: 'write_file', inputJson: '{}' },
      { type: 'toolResult', toolUseId: 'call-1', outputJson: JSON.stringify(CONFIRMATION_PAYLOAD), isError: true },
    ]))
    const part = (converted.content as unknown as { type: string; result?: Record<string, unknown> }[])
      .find((item) => item.type === 'tool-call')
    expect(part?.result?.reasons).toEqual(['tool_may_be_destructive'])
  })
})
