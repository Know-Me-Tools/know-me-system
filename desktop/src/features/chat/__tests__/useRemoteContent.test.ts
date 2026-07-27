// TJ-ARCH-MOB-001 compliant
//
// Remote content the webview cannot fetch itself. Flutter has resolved these
// through Rust since it shipped; React handed the url straight to the browser,
// so the same conversation rendered on mobile and showed broken media on
// desktop.
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { act, renderHook, waitFor } from '@testing-library/react'

const mocks = vi.hoisted(() => ({
  isTauri: vi.fn(() => true),
  resolveRemoteContent: vi.fn(),
}))

vi.mock('@tauri-apps/api/core', () => ({ isTauri: mocks.isTauri }))
vi.mock('@prometheus-ags/tauri-plugin-gen-ui', () => ({
  resolveRemoteContent: mocks.resolveRemoteContent,
}))

import { useRemoteContent } from '../hooks/useRemoteContent'

describe('useRemoteContent', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    mocks.isTauri.mockReturnValue(true)
  })

  it('uses the direct url and does not call Rust unless it fails', () => {
    // Round-tripping every image through IPC would make the common case slower
    // to serve the uncommon one.
    const { result } = renderHook(() => useRemoteContent('https://example.test/a.png', 'image'))

    expect(result.current.src).toBe('https://example.test/a.png')
    expect(mocks.resolveRemoteContent).not.toHaveBeenCalled()
  })

  it('resolves natively once the browser reports a failure', async () => {
    mocks.resolveRemoteContent.mockResolvedValue({
      url: 'https://example.test/a.png',
      mime: 'image/png',
      dataBase64: 'AAAA',
    })
    const { result } = renderHook(() => useRemoteContent('https://example.test/a.png', 'image'))

    act(() => result.current.onError())

    await waitFor(() => {
      // Inline bytes are preferred: a data URI needs no second fetch, and the
      // second fetch is the thing that just failed.
      expect(result.current.src).toBe('data:image/png;base64,AAAA')
    })
  })

  it('uses the resolved url when Rust returned no inline bytes', async () => {
    mocks.resolveRemoteContent.mockResolvedValue({
      url: 'https://cdn.example.test/resolved.png',
      mime: 'image/png',
    })
    const { result } = renderHook(() => useRemoteContent('https://example.test/a.png', 'image'))

    act(() => result.current.onError())

    await waitFor(() => {
      expect(result.current.src).toBe('https://cdn.example.test/resolved.png')
    })
  })

  it('reports failure rather than retrying when Rust cannot resolve it either', async () => {
    mocks.resolveRemoteContent.mockRejectedValue(new Error('host unreachable'))
    const { result } = renderHook(() => useRemoteContent('https://example.test/a.png', 'image'))

    act(() => result.current.onError())

    await waitFor(() => expect(result.current.failed).toBe(true))
  })

  it('fails honestly on the web build, where there is no Rust to fall back to', async () => {
    mocks.isTauri.mockReturnValue(false)
    const { result } = renderHook(() => useRemoteContent('https://example.test/a.png', 'image'))

    act(() => result.current.onError())

    await waitFor(() => expect(result.current.failed).toBe(true))
    expect(mocks.resolveRemoteContent).not.toHaveBeenCalled()
  })

  it('drops a previous resolution when the url changes', async () => {
    // A new url is a new subject; keeping the old data URI would show the
    // previous image under the new one's alt text.
    mocks.resolveRemoteContent.mockResolvedValue({
      url: 'https://example.test/a.png', mime: 'image/png', dataBase64: 'AAAA',
    })
    const { result, rerender } = renderHook(
      ({ url }) => useRemoteContent(url, 'image'),
      { initialProps: { url: 'https://example.test/a.png' } },
    )
    act(() => result.current.onError())
    await waitFor(() => expect(result.current.src).toBe('data:image/png;base64,AAAA'))

    rerender({ url: 'https://example.test/b.png' })

    expect(result.current.src).toBe('https://example.test/b.png')
    expect(result.current.failed).toBe(false)
  })
})
