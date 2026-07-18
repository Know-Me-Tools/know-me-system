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
      { type: 'contentBlock', messageId: 'msg-1', blockIndex: 0, block: { type: 'text', text: 'Hello' } },
      { type: 'contentBlock', messageId: 'msg-1', blockIndex: 0, block: { type: 'text', text: 'Hello world' } },
      { type: 'messageComplete', messageId: 'msg-1' },
    ])
  })

  it('closes the message on run_error so the UI stops spinning', () => {
    const { events, adapter } = collect()
    adapter.bind('msg-2')

    adapter.handle({ type: 'block', block: { type: 'text', text: 'partial' } })
    adapter.handle({ type: 'run_error', message: 'model load failed' })

    // The partial answer is preserved, with the failure appended.
    expect(events.at(-2)).toEqual({
      type: 'contentBlock',
      messageId: 'msg-2',
      blockIndex: 0,
      block: { type: 'text', text: 'partial\n\nmodel load failed' },
    })
    expect(events.at(-1)).toEqual({ type: 'messageComplete', messageId: 'msg-2' })
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
