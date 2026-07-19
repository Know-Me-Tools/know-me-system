// TJ-ARCH-MOB-001 compliant
//
// These assert against the REAL wire shape gen_ui_types::events::A2uiEvent
// serializes to (`#[serde(tag = "type", rename_all = "snake_case")]`), which the
// Tauri plugin emits verbatim. That contract previously went unverified, and the
// driver had drifted to a shape Rust never emitted — the desktop lane rendered
// nothing. Keep these in sync with the Rust enum.
import { describe, expect, it } from 'vitest'
import { createA2uiWireAdapter, type A2uiWireEvent, type CoreA2uiEvent } from '../driver'

function collect() {
  const events: CoreA2uiEvent[] = []
  const adapter = createA2uiWireAdapter((e) => events.push(e))
  return { events, adapter }
}

describe('createA2uiWireAdapter', () => {
  it('accumulates per-token deltas into one growing text block', () => {
    const { events, adapter } = collect()
    adapter.bind('msg-1')

    const wire: A2uiWireEvent[] = [
      { type: 'run_started', run_id: 'run-1' },
      { type: 'block', block: { type: 'text', text: 'Hello' } },
      { type: 'block', block: { type: 'text', text: ' world' } },
      { type: 'run_finished', run_id: 'run-1' },
    ]
    wire.forEach(adapter.handle)

    // Rust sends deltas; the store renders one block that grows.
    expect(events).toEqual([
      { type: 'started', messageId: 'msg-1', runId: 'run-1' },
      { type: 'contentBlock', messageId: 'msg-1', blockIndex: 0, block: { type: 'text', text: 'Hello' } },
      { type: 'contentBlock', messageId: 'msg-1', blockIndex: 0, block: { type: 'text', text: 'Hello world' } },
      { type: 'messageComplete', messageId: 'msg-1' },
    ])
  })

  it('preserves partial content and emits a typed run error', () => {
    const { events, adapter } = collect()
    adapter.bind('msg-2')

    adapter.handle({ type: 'block', block: { type: 'text', text: 'partial' } })
    adapter.handle({
      type: 'run_error', run_id: 'run-2',
      error: { code: 'model_load_failed', phase: 'model_loading', message: 'model load failed', retryable: true },
    })

    // The failure remains lifecycle state, never fabricated assistant text.
    expect(events.at(-2)).toEqual({
      type: 'contentBlock',
      messageId: 'msg-2',
      blockIndex: 0,
      block: { type: 'text', text: 'partial' },
    })
    expect(events.at(-1)).toEqual({
      type: 'error', messageId: 'msg-2', runId: 'run-2',
      error: { code: 'model_load_failed', phase: 'model_loading', message: 'model load failed', retryable: true },
    })
  })

  it('keeps thinking blocks when answer text starts', () => {
    const { events, adapter } = collect()
    adapter.bind('msg-reasoning')
    adapter.handle({ type: 'block', block: { type: 'thinking', text: 'reason' } })
    adapter.handle({ type: 'block', block: { type: 'text', text: 'answer' } })
    adapter.handle({ type: 'block', block: { type: 'text', text: ' done' } })
    expect(events).toEqual([
      { type: 'contentBlock', messageId: 'msg-reasoning', block: { type: 'thinking', text: 'reason' } },
      { type: 'contentBlock', messageId: 'msg-reasoning', blockIndex: 1, block: { type: 'text', text: 'answer' } },
      { type: 'contentBlock', messageId: 'msg-reasoning', blockIndex: 1, block: { type: 'text', text: 'answer done' } },
    ])
  })

  it('drops events that arrive with no bound message rather than guessing', () => {
    const { events, adapter } = collect()
    adapter.handle({ type: 'block', block: { type: 'text', text: 'stray' } })
    expect(events).toEqual([])
  })

  it('stops attributing events to a message once its run has finished', () => {
    const { events, adapter } = collect()
    adapter.bind('msg-3')
    adapter.handle({ type: 'run_finished', run_id: 'run-3' })
    events.length = 0

    // A late/duplicate event must not reopen a completed message.
    adapter.handle({ type: 'block', block: { type: 'text', text: 'late' } })
    expect(events).toEqual([])
  })

  it('resets accumulated text between turns', () => {
    const { events, adapter } = collect()
    adapter.bind('msg-4')
    adapter.handle({ type: 'block', block: { type: 'text', text: 'first' } })
    adapter.handle({ type: 'run_finished', run_id: 'run-4' })

    adapter.bind('msg-5')
    events.length = 0
    adapter.handle({ type: 'block', block: { type: 'text', text: 'second' } })

    // 'first' must not bleed into the next message.
    expect(events).toEqual([
      { type: 'contentBlock', messageId: 'msg-5', blockIndex: 0, block: { type: 'text', text: 'second' } },
    ])
  })
})
