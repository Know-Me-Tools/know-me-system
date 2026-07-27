// TJ-ARCH-MOB-001 compliant — hook composes the store-owned transport.
//
// Remote media the webview cannot fetch itself.
//
// The model references images, video and links from origins the embedded
// browser may be blocked from — CORS, mixed content, or an origin it holds no
// credentials for. Flutter has resolved these through Rust since it shipped;
// React handed `block.url` straight to an <img src>, so the same conversation
// rendered on mobile and showed broken media on desktop.
//
// Resolution is LAZY: the direct URL is used first, and Rust is only asked
// after the browser has actually failed. Round-tripping every image through IPC
// would make the common case slower to serve the uncommon one.
import { useCallback, useEffect, useState } from 'react'
import { isTauri } from '@tauri-apps/api/core'
import { resolveRemoteContent, type RemoteContentKind } from '@prometheus-ags/tauri-plugin-gen-ui'

/** A src the browser can render, and whether we are still working it out. */
export interface RemoteContentState {
  /** The URL to render right now. Starts as the original. */
  src: string
  /** True once the direct URL failed and Rust could not resolve it either. */
  failed: boolean
  /** Call when the browser fails to load `src`; triggers native resolution. */
  onError: () => void
}

/**
 * Resolve a remote URL, falling back to Rust when the webview cannot fetch it.
 *
 * On the web build there is no Rust to fall back to, so a failure stays a
 * failure — reporting it honestly rather than retrying a fetch that cannot
 * succeed.
 */
export function useRemoteContent(url: string, kind: RemoteContentKind): RemoteContentState {
  const [src, setSrc] = useState(url)
  const [failed, setFailed] = useState(false)
  const [shouldResolve, setShouldResolve] = useState(false)

  // A new url is a new subject: drop whatever the previous one resolved to.
  useEffect(() => {
    setSrc(url)
    setFailed(false)
    setShouldResolve(false)
  }, [url])

  useEffect(() => {
    if (!shouldResolve) return
    if (!isTauri()) {
      setFailed(true)
      return
    }
    let active = true
    void resolveRemoteContent(url, kind)
      .then((resolved) => {
        if (!active) return
        // Prefer inline bytes: a data URI needs no second fetch, and the second
        // fetch is the thing that just failed.
        setSrc(resolved.dataBase64
          ? `data:${resolved.mime};base64,${resolved.dataBase64}`
          : resolved.url)
      })
      .catch(() => {
        if (active) setFailed(true)
      })
    return () => { active = false }
  }, [shouldResolve, url, kind])

  const onError = useCallback(() => setShouldResolve(true), [])

  return { src, failed, onError }
}
