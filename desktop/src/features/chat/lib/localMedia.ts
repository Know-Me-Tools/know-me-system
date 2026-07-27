// TJ-ARCH-MOB-001 compliant — pure helper, no invoke() and no React.
//
// A block that carries a filesystem path cannot be handed to an <img src>
// directly: the webview resolves it against the page origin and loads nothing.
// Tauri exposes a dedicated asset scheme for exactly this, and the app config
// scopes it to the attachments directory so a transcript cannot address
// arbitrary files on disk.
import { convertFileSrc, isTauri } from '@tauri-apps/api/core'

/**
 * A src the webview can load for a locally stored media file.
 *
 * On the web build there is no asset protocol and no filesystem, so a stored
 * path is meaningless. Returning an empty string makes the image fail its load
 * and hit the renderer's error branch — an honest broken state rather than a
 * request that hangs or silently resolves to the wrong origin.
 */
export function localMediaSrc(path: string): string {
  if (!path) return ''
  if (!isTauri()) return ''
  return convertFileSrc(path)
}

/** Whether a stored path can be rendered on this surface at all. */
export function canRenderLocalMedia(): boolean {
  return isTauri()
}
