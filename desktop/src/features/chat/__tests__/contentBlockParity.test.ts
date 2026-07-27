// TJ-ARCH-MOB-001 compliant
//
// Regression guards for silent-failure bugs found auditing React against the
// Flutter reference. Each of these shipped as "working" because nothing errored.
import { describe, expect, it, vi } from 'vitest'

vi.mock('@tauri-apps/api/core', () => ({ isTauri: () => false }))
vi.mock('@prometheus-ags/tauri-plugin-gen-ui', () => ({}))

import { coalesceContentBlocks, convertMessage } from '../hooks/useAssistantChatRuntime'
import type { ContentBlock, Message } from '@/bridge/a2ui/types'

function assistant(content: ContentBlock[]): Message {
  return { id: 'a-1', role: 'assistant', content, timestamp: '2026-07-19T12:00:00.000Z' }
}

describe('coalesceContentBlocks', () => {
  it('merges adjacent text blocks into one', () => {
    // A streamed turn arrives as many deltas. Rendered one-per-block, that is N
    // chat bubbles where the user wrote one message — it reads as a layout bug.
    const merged = coalesceContentBlocks([
      { type: 'text', text: 'Hello ' },
      { type: 'text', text: 'world' },
    ])
    expect(merged).toEqual([{ type: 'text', text: 'Hello world' }])
  })

  it('merges adjacent thinking blocks', () => {
    const merged = coalesceContentBlocks([
      { type: 'thinking', text: 'reason ' },
      { type: 'thinking', text: 'continues' },
    ])
    expect(merged).toEqual([{ type: 'thinking', text: 'reason continues' }])
  })

  it('never merges across a semantic boundary', () => {
    // Every other variant is a boundary and must survive intact — merging text
    // across a divider or a tool call would rewrite what the model actually did.
    const blocks: ContentBlock[] = [
      { type: 'text', text: 'before' },
      { type: 'divider' },
      { type: 'text', text: 'after' },
    ]
    expect(coalesceContentBlocks(blocks)).toHaveLength(3)
  })

  it('leaves a single block untouched', () => {
    const blocks: ContentBlock[] = [{ type: 'text', text: 'only' }]
    expect(coalesceContentBlocks(blocks)).toEqual(blocks)
  })
})

describe('image blocks reach the renderer', () => {
  it('emits an image part carrying the mime as its descriptive field', () => {
    // The render switch had no `case "image"`, so images hit `default: return
    // null` and rendered NOTHING — silent data loss with no error surface.
    // assistant-ui's image part has no `alt`, so `filename` carries the mime,
    // which the renderer turns into the accessible name.
    const converted = convertMessage(assistant([
      { type: 'image', url: 'https://example.test/a.png', dataBase64: null, mime: 'image/png' },
    ]))
    const part = (converted.content as { type: string; image?: string; filename?: string }[])
      .find((p) => p.type === 'image')
    expect(part).toBeDefined()
    expect(part?.image).toBe('https://example.test/a.png')
    expect(part?.filename).toBe('image/png')
  })

  it('falls back to a data URI when there is no url', () => {
    const converted = convertMessage(assistant([
      { type: 'image', url: null, dataBase64: 'AAAA', mime: 'image/webp' },
    ]))
    const part = (converted.content as { type: string; image?: string }[])
      .find((p) => p.type === 'image')
    expect(part?.image).toBe('data:image/webp;base64,AAAA')
  })
})
