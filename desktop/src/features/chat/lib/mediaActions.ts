// TJ-ARCH-MOB-001 compliant — pure helpers; no invoke(), no React.
//
// Saving and sharing media out of a chat bubble.
//
// Deliberately NO tauri-plugin-dialog / tauri-plugin-fs. Those would be five
// registration points EACH (Cargo.toml, package.json, lib.rs,
// capabilities/default.json, call site) — the same trap that broke the release
// build in 12134b6, where cargo check passed while the app could not build
// because ACL resolution happens in the consuming crate.
//
// They are not needed. The asset scheme enabled for rendering (C4) is
// fetchable from the webview, so an anchor with `download` saves the file on
// BOTH web and Tauri through one code path. Fewer moving parts, and nothing
// that can silently fail to register.
import { isTauri } from '@tauri-apps/api/core'

import { localMediaSrc } from './localMedia'

/** What a save or share attempt did, for the UI to announce. */
export type MediaActionOutcome =
  | { status: 'saved' }
  | { status: 'shared' }
  | { status: 'cancelled' }
  | { status: 'failed'; message: string }

/**
 * Whether this surface can offer a native share sheet.
 *
 * `navigator.share` with files is the only share primitive available to a
 * webview. It is absent on most desktop browsers and on Linux/Windows Tauri
 * webviews, so the control is hidden rather than shown and broken — a control
 * that silently does nothing is worse than no control.
 */
export function canShareMedia(file?: File): boolean {
  if (typeof navigator === 'undefined' || !navigator.share) return false
  if (!file) return typeof navigator.canShare === 'function'
  return navigator.canShare?.({ files: [file] }) ?? false
}

/**
 * Fetch a stored or remote media file as a Blob the browser can act on.
 *
 * `response.ok` is NOT sufficient. A dev server (and many SPA hosts) answer an
 * unknown path with a 200 and an HTML fallback page, so a missing image would
 * "save" successfully as an HTML document named photo.png. Caught in the
 * browser, where a missing file reported `saved`. The content type has to
 * agree that this is media.
 */
async function loadBlob(src: string): Promise<Blob> {
  const response = await fetch(src)
  if (!response.ok) {
    throw new Error(`could not read the file (${response.status})`)
  }
  const blob = await response.blob()
  if (blob.type.startsWith('text/html')) {
    throw new Error('that file is no longer available')
  }
  return blob
}

/**
 * Resolve a block's carrier to something fetchable.
 *
 * Returns null when there is nothing to act on — a stored path on the web
 * build, for instance, where there is no asset protocol to resolve it.
 */
export function mediaSourceFor(block: {
  url?: string | null
  dataBase64?: string | null
  mime?: string
  path?: string
}): string | null {
  if (block.url) return block.url
  if (block.dataBase64) return `data:${block.mime ?? 'application/octet-stream'};base64,${block.dataBase64}`
  if (block.path) return localMediaSrc(block.path) || null
  return null
}

/**
 * Save a media file to the user's disk.
 *
 * On Tauri this goes through the browser's own download path rather than a
 * native save dialog. The user still chooses the destination if their webview
 * is configured to ask; if not, it lands in Downloads — the same behaviour as
 * every other download they make.
 */
export async function saveMedia(src: string, fileName: string): Promise<MediaActionOutcome> {
  try {
    const blob = await loadBlob(src)
    const objectUrl = URL.createObjectURL(blob)
    const anchor = document.createElement('a')
    anchor.href = objectUrl
    anchor.download = fileName
    document.body.appendChild(anchor)
    anchor.click()
    anchor.remove()
    // Revoked on the next tick, not immediately: revoking synchronously can
    // cancel the download the click just started.
    setTimeout(() => URL.revokeObjectURL(objectUrl), 0)
    return { status: 'saved' }
  } catch (error) {
    return {
      status: 'failed',
      message: error instanceof Error ? error.message : 'could not save the file',
    }
  }
}

/**
 * Hand a media file to the platform share sheet, where one exists.
 *
 * Dismissing the sheet reports 'cancelled' rather than 'failed': choosing not
 * to share is a normal outcome, and calling it an error would be a lie.
 */
export async function shareMedia(
  src: string,
  fileName: string,
  mime: string,
): Promise<MediaActionOutcome> {
  try {
    const blob = await loadBlob(src)
    const file = new File([blob], fileName, { type: mime })
    if (!canShareMedia(file)) {
      return { status: 'failed', message: 'Sharing files is not supported here.' }
    }
    await navigator.share({ files: [file], title: fileName })
    return { status: 'shared' }
  } catch (error) {
    // AbortError is the user closing the sheet, which is not a failure.
    if (error instanceof Error && error.name === 'AbortError') {
      return { status: 'cancelled' }
    }
    return {
      status: 'failed',
      message: error instanceof Error ? error.message : 'could not share the file',
    }
  }
}

/** Whether media actions can do anything at all on this surface. */
export function canActOnMedia(): boolean {
  // Web can always download a blob; Tauri can too, via the asset scheme.
  return typeof document !== 'undefined' || isTauri()
}
