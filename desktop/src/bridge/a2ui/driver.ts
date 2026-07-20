// TJ-ARCH-MOB-001 compliant
import type { ChatRunError, ContentBlock, MessageUsage, RunStatus } from './types'

export type RunLifecycleUpdate =
  | { type: 'started'; messageId: string; runId: string }
  | { type: 'phase'; messageId: string; runId: string; phase: string; status: RunStatus; message: string }
  | { type: 'error'; messageId: string; runId: string; error: ChatRunError }
  | { type: 'cancelled'; messageId: string; runId: string; phase: string; message: string }
  | { type: 'telemetry'; messageId: string; runId: string; payload: unknown }

export type CoreA2uiEvent =
  | { type: 'contentBlock'; messageId: string; blockIndex?: number; block: ContentBlock }
  | { type: 'messageComplete'; messageId: string; usage?: MessageUsage }
  | RunLifecycleUpdate

export function applyA2uiEvent(
  event: CoreA2uiEvent,
  streamBlock: (value: { messageId: string; blockIndex?: number; block: ContentBlock }) => void,
  finalize: (messageId: string, usage?: MessageUsage) => void,
  updateLifecycle: (update: RunLifecycleUpdate) => void = () => {},
): void {
  switch (event.type) {
    case 'contentBlock': streamBlock(event); break
    case 'messageComplete': finalize(event.messageId, event.usage); break
    case 'started':
    case 'phase':
    case 'error':
    case 'cancelled':
    case 'telemetry': updateLifecycle(event); break
  }
}

/**
 * The wire shape of `gen_ui_types::events::A2uiEvent`, which the Tauri plugin
 * emits verbatim (`#[serde(tag = "type", rename_all = "snake_case")]`).
 *
 * Deliberately NOT the same type as {@link CoreA2uiEvent}: the Rust event is
 * run-scoped (it carries a run_id and knows nothing about which UI message it
 * belongs to), while the store is message-scoped. The caller bridges the two by
 * binding the assistant message id when it starts a turn.
 */
export type A2uiWireEvent =
  | { type: 'run_started'; run_id: string }
  | { type: 'run_phase_changed'; run_id: string; phase: string; status: RunStatus; message: string }
  | { type: 'block'; block: ContentBlock }
  | { type: 'run_finished'; run_id: string }
  | { type: 'run_cancelled'; run_id: string; phase: string; message: string }
  | { type: 'run_error'; run_id: string; error: ChatRunError }
  | { type: 'context_telemetry'; run_id: string; payload: unknown }

/**
 * Adapt Rust wire events into the message-addressed events the store consumes.
 *
 * Rust streams one `block` per token delta for text and reasoning, each carrying
 * only that delta, so this accumulates each contiguous stream into one growing
 * block. The store treats `blockIndex` as replace-in-place, not append. Richer
 * semantic blocks remain atomic and are appended in order.
 *
 * Events arriving with no bound message (a stray run, or one racing ahead of the
 * caller) are dropped rather than guessed at.
 */
export function createA2uiWireAdapter(
  emit: (event: CoreA2uiEvent) => void,
): { bind: (messageId: string) => void; handle: (wire: A2uiWireEvent) => void } {
  let messageId: string | null = null
  let text = ''
  let thinking = ''
  let blockCount = 0
  let textBlockIndex: number | null = null
  let thinkingBlockIndex: number | null = null

  return {
    bind: (id: string) => {
      messageId = id
      text = ''
      thinking = ''
      blockCount = 0
      textBlockIndex = null
      thinkingBlockIndex = null
    },
    handle: (wire: A2uiWireEvent) => {
      if (messageId === null) return
      switch (wire.type) {
        case 'run_started':
          emit({ type: 'started', messageId, runId: wire.run_id })
          break
        case 'run_phase_changed':
          emit({ type: 'phase', messageId, runId: wire.run_id, phase: wire.phase, status: wire.status, message: wire.message })
          break
        case 'block':
          if (wire.block.type === 'text') {
            thinking = ''
            thinkingBlockIndex = null
            text += wire.block.text
            if (textBlockIndex === null) {
              textBlockIndex = blockCount
              blockCount += 1
              emit({ type: 'contentBlock', messageId, blockIndex: textBlockIndex, block: { type: 'text', text } })
            } else {
              emit({ type: 'contentBlock', messageId, blockIndex: textBlockIndex, block: { type: 'text', text } })
            }
          } else if (wire.block.type === 'thinking') {
            thinking += wire.block.text
            if (thinkingBlockIndex === null) {
              thinkingBlockIndex = blockCount
              blockCount += 1
              emit({ type: 'contentBlock', messageId, blockIndex: thinkingBlockIndex, block: { type: 'thinking', text: thinking } })
            } else {
              emit({ type: 'contentBlock', messageId, blockIndex: thinkingBlockIndex, block: { type: 'thinking', text: thinking } })
            }
          } else {
            thinking = ''
            thinkingBlockIndex = null
            emit({ type: 'contentBlock', messageId, block: wire.block })
            blockCount += 1
          }
          break
        case 'run_error':
          emit({ type: 'error', messageId, runId: wire.run_id, error: wire.error })
          messageId = null
          break
        case 'run_cancelled':
          emit({ type: 'cancelled', messageId, runId: wire.run_id, phase: wire.phase, message: wire.message })
          messageId = null
          break
        case 'context_telemetry':
          emit({ type: 'telemetry', messageId, runId: wire.run_id, payload: wire.payload })
          break
        case 'run_finished':
          emit({ type: 'messageComplete', messageId })
          messageId = null
          break
      }
    },
  }
}
