// TJ-ARCH-MOB-001 compliant
//
// Saving and sharing media from a chat bubble.
//
// No tauri-plugin-dialog / tauri-plugin-fs behind this: the asset scheme is
// already fetchable, so one anchor-download path covers web and Tauri without
// the five-place plugin registration that broke the release build in 12134b6.
import { describe, expect, it, vi, beforeEach } from 'vitest'

vi.mock('@tauri-apps/api/core', () => ({
  isTauri: () => false,
  convertFileSrc: (path: string) => `asset://localhost/${encodeURIComponent(path)}`,
}))

import { mediaSourceFor, saveMedia, shareMedia, canShareMedia } from '../lib/mediaActions'

/** A response whose body is `blob`, as fetch would return it. */
function respondWith(blob: Blob, ok = true, status = 200) {
  return vi.fn().mockResolvedValue({ ok, status, blob: async () => blob })
}

beforeEach(() => {
  vi.restoreAllMocks()
  // jsdom implements neither; the anchor click is a no-op there, which is
  // exactly what we want — we assert the outcome, not the browser's download.
  globalThis.URL.createObjectURL = vi.fn(() => 'blob:probe')
  globalThis.URL.revokeObjectURL = vi.fn()
})

describe('mediaSourceFor', () => {
  it('prefers a url, then inline bytes, then a stored path', () => {
    expect(mediaSourceFor({ url: 'https://ex.test/a.png' })).toBe('https://ex.test/a.png')
    expect(mediaSourceFor({ dataBase64: 'AAAA', mime: 'image/png' }))
      .toBe('data:image/png;base64,AAAA')
  })

  it('returns null for a stored path where it cannot be resolved', () => {
    // The web build has no asset protocol, so a filesystem path is meaningless.
    // Null lets the caller hide the control rather than offer a dead one.
    expect(mediaSourceFor({ path: '/data/a.png' })).toBeNull()
    expect(mediaSourceFor({})).toBeNull()
  })
})

describe('saveMedia', () => {
  it('saves a real media file', async () => {
    vi.stubGlobal('fetch', respondWith(new Blob(['x'], { type: 'image/png' })))
    await expect(saveMedia('asset://x.png', 'photo.png')).resolves.toEqual({ status: 'saved' })
  })

  it('rejects an HTML fallback page instead of saving it as an image', async () => {
    // CAUGHT IN THE BROWSER, not by an earlier test: a dev server answers an
    // unknown path with 200 + an HTML fallback, so `response.ok` was true and
    // a missing image "saved" successfully as an HTML document named
    // photo.png. The content type has to agree that this is media.
    vi.stubGlobal('fetch', respondWith(new Blob(['<!doctype html>'], { type: 'text/html' })))

    const result = await saveMedia('asset://gone.png', 'photo.png')

    expect(result.status).toBe('failed')
    expect(result).toHaveProperty('message', expect.stringContaining('no longer available'))
  })

  it('reports a transport failure rather than throwing', async () => {
    vi.stubGlobal('fetch', respondWith(new Blob([]), false, 404))
    const result = await saveMedia('asset://gone.png', 'photo.png')
    expect(result.status).toBe('failed')
  })
})

describe('shareMedia', () => {
  it('reports unsupported rather than throwing where there is no share sheet', async () => {
    vi.stubGlobal('fetch', respondWith(new Blob(['x'], { type: 'image/png' })))
    // jsdom has no navigator.share, which is also the real state of most
    // desktop webviews — the UI hides the control there.
    expect(canShareMedia()).toBe(false)

    const result = await shareMedia('asset://x.png', 'photo.png', 'image/png')
    expect(result.status).toBe('failed')
  })

  it('treats a dismissed share sheet as cancelled, not failed', async () => {
    vi.stubGlobal('fetch', respondWith(new Blob(['x'], { type: 'image/png' })))
    const abort = Object.assign(new Error('dismissed'), { name: 'AbortError' })
    vi.stubGlobal('navigator', {
      share: vi.fn().mockRejectedValue(abort),
      canShare: vi.fn(() => true),
    })

    const result = await shareMedia('asset://x.png', 'photo.png', 'image/png')

    // Choosing not to share is a normal outcome; calling it an error is a lie.
    expect(result.status).toBe('cancelled')
  })
})
