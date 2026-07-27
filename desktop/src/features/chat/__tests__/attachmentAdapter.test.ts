// TJ-ARCH-MOB-001 compliant
//
// The attachment contract. React's composer rendered a Plus button, a dropzone
// and a chip row with no adapter behind them, so picking a file did nothing —
// a false affordance. These pin the limits and the copy to Flutter's, so a file
// accepted on one surface is accepted on the other and a rejection reads the
// same way.
import { describe, expect, it, vi } from 'vitest'

vi.mock('@tauri-apps/api/core', () => ({ isTauri: () => false }))
vi.mock('@prometheus-ags/tauri-plugin-gen-ui', () => ({ stageAttachment: vi.fn() }))

import {
  ACCEPT,
  ALLOWED_EXTENSIONS,
  ATTACHMENT_ERRORS,
  MAX_ATTACHMENTS,
  MAX_ATTACHMENT_BYTES,
  mimeFor,
  rejectionFor,
} from '../adapters/attachmentAdapter'

describe('attachment limits match Flutter', () => {
  it('accepts the same 13 extensions', () => {
    expect(ALLOWED_EXTENSIONS).toEqual([
      'png', 'jpg', 'jpeg', 'gif', 'webp', 'svg',
      'pdf', 'txt', 'md', 'doc', 'docx',
      'mp4', 'webm',
    ])
  })

  it('caps a file at 25 MB and a turn at 6 files', () => {
    // Rust re-enforces both; a client-side check is a courtesy, not a control.
    expect(MAX_ATTACHMENT_BYTES).toBe(25 * 1024 * 1024)
    expect(MAX_ATTACHMENTS).toBe(6)
  })

  it('builds the accept string from the same list', () => {
    expect(ACCEPT).toBe('.png,.jpg,.jpeg,.gif,.webp,.svg,.pdf,.txt,.md,.doc,.docx,.mp4,.webm')
  })
})

describe('rejectionFor', () => {
  it('accepts a file within the limits', () => {
    expect(rejectionFor({ name: 'notes.md', size: 1024 })).toBeNull()
  })

  it('rejects an oversized file with Flutter\'s wording', () => {
    expect(rejectionFor({ name: 'clip.mp4', size: MAX_ATTACHMENT_BYTES + 1 }))
      .toBe(ATTACHMENT_ERRORS.tooLarge)
  })

  it('accepts a file exactly at the limit', () => {
    // The Flutter check is `size > maxBytes`, so the boundary itself passes.
    expect(rejectionFor({ name: 'clip.mp4', size: MAX_ATTACHMENT_BYTES })).toBeNull()
  })

  it('rejects an unsupported extension', () => {
    expect(rejectionFor({ name: 'archive.zip', size: 10 })).toBe(ATTACHMENT_ERRORS.unavailable)
  })

  it('rejects a file with no extension at all', () => {
    expect(rejectionFor({ name: 'README', size: 10 })).toBe(ATTACHMENT_ERRORS.unavailable)
  })

  it('is case-insensitive about the extension', () => {
    // A file picked from a Windows share is commonly upper-cased; rejecting it
    // would be a difference the user cannot see or explain.
    expect(rejectionFor({ name: 'SCAN.PDF', size: 10 })).toBeNull()
  })
})

describe('mimeFor', () => {
  it('maps the image and document types Flutter maps', () => {
    expect(mimeFor('a.png')).toBe('image/png')
    expect(mimeFor('a.jpg')).toBe('image/jpeg')
    expect(mimeFor('a.jpeg')).toBe('image/jpeg')
    expect(mimeFor('a.svg')).toBe('image/svg+xml')
    expect(mimeFor('a.pdf')).toBe('application/pdf')
    expect(mimeFor('a.md')).toBe('text/markdown')
    expect(mimeFor('a.mp4')).toBe('video/mp4')
  })

  it('falls back for doc and docx, exactly as Flutter does', () => {
    // Not an oversight being copied: Flutter's switch has no arm for these, and
    // the two surfaces must agree on what is written into the transcript.
    expect(mimeFor('a.doc')).toBe('application/octet-stream')
    expect(mimeFor('a.docx')).toBe('application/octet-stream')
  })
})
