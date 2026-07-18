// TJ-ARCH-MOB-001 compliant
import type { ContentBlock, MessageUsage } from './types'

export type CoreA2uiEvent =
  | { type: 'contentBlock'; messageId: string; blockIndex?: number; block: ContentBlock }
  | { type: 'messageComplete'; messageId: string; usage?: MessageUsage }

export function applyA2uiEvent(
  event: CoreA2uiEvent,
  streamBlock: (value: { messageId: string; blockIndex?: number; block: ContentBlock }) => void,
  finalize: (messageId: string, usage?: MessageUsage) => void,
): void {
  switch (event.type) {
    case 'contentBlock': streamBlock(event); break
    case 'messageComplete': finalize(event.messageId, event.usage); break
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
  | { type: 'block'; block: ContentBlock }
  | { type: 'run_finished'; run_id: string }
  | { type: 'run_error'; message: string }

/**
 * Adapt Rust wire events into the message-addressed events the store consumes.
 *
 * Rust streams one `block` per token delta, each carrying only that delta, so
 * this accumulates them into the single growing text block the store renders
 * (blockIndex 0 = replace, not append) — which is exactly what the WebLLM lane
 * already emits. Both lanes therefore look identical from the store upward.
 *
 * Events arriving with no bound message (a stray run, or one racing ahead of the
 * caller) are dropped rather than guessed at.
 */
export function createA2uiWireAdapter(
  emit: (event: CoreA2uiEvent) => void,
): { bind: (messageId: string) => void; handle: (wire: A2uiWireEvent) => void } {
  let messageId: string | null = null
  let text = ''

  return {
    bind: (id: string) => {
      messageId = id
      text = ''
    },
    handle: (wire: A2uiWireEvent) => {
      if (messageId === null) return
      switch (wire.type) {
        case 'run_started':
          break
        case 'block':
          if (wire.block.type === 'text') {
            text += wire.block.text
            emit({ type: 'contentBlock', messageId, blockIndex: 0, block: { type: 'text', text } })
          } else {
            // Richer blocks (tool use, citations) pass straight through once the
            // core starts emitting them.
            emit({ type: 'contentBlock', messageId, block: wire.block })
          }
          break
        case 'run_error':
          // Surface the failure in the transcript rather than leaving the
          // message spinning forever.
          emit({
            type: 'contentBlock',
            messageId,
            blockIndex: 0,
            block: { type: 'text', text: text ? `${text}\n\n${wire.message}` : wire.message },
          })
          emit({ type: 'messageComplete', messageId })
          messageId = null
          break
        case 'run_finished':
          emit({ type: 'messageComplete', messageId })
          messageId = null
          break
      }
    },
  }
}
