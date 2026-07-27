// TJ-ARCH-MOB-001 compliant — adapter layer; talks to the plugin, not to React.
//
// The composer's Plus button, dropzone and chip row have always rendered, but no
// AttachmentAdapter was registered behind them — so picking a file did nothing.
// A control that looks like it works is worse than no control, and this closes
// that gap.
//
// Limits and copy come from Flutter (`_pickAttachments`,
// mobile/.../chat_screen.dart) so the two surfaces accept the same files and
// explain a rejection the same way.
import { isTauri } from '@tauri-apps/api/core'
import { stageAttachment } from '@prometheus-ags/tauri-plugin-gen-ui'
import type { AttachmentAdapter, PendingAttachment, CompleteAttachment, Attachment } from '@assistant-ui/react'

import type { AttachmentInput } from '../stores/chatStore'

/** The 13 extensions Flutter accepts. Kept in this order to stay diffable. */
export const ALLOWED_EXTENSIONS = [
  'png', 'jpg', 'jpeg', 'gif', 'webp', 'svg',
  'pdf', 'txt', 'md', 'doc', 'docx',
  'mp4', 'webm',
] as const

/** 25 MB per file — Flutter's ceiling. Rust re-checks it; this is the courtesy. */
export const MAX_ATTACHMENT_BYTES = 25 * 1024 * 1024

/** At most 6 files per turn. Rust enforces the same cap when it reads them. */
export const MAX_ATTACHMENTS = 6

/** Flutter's error strings, verbatim — one wording per condition across both surfaces. */
export const ATTACHMENT_ERRORS = {
  tooLarge: 'Files larger than 25 MB were not attached.',
  unavailable: 'Some files were unavailable to the app.',
  both: 'Some files were too large or unavailable to the app.',
} as const

/** `accept` string for the file input, derived from the same list. */
export const ACCEPT = ALLOWED_EXTENSIONS.map((extension) => `.${extension}`).join(',')

/** Mime for an extension, mirroring Flutter's `_mimeFor`. */
export function mimeFor(fileName: string): string {
  const extension = fileName.split('.').pop()?.toLowerCase()
  switch (extension) {
    case 'png': return 'image/png'
    case 'jpg':
    case 'jpeg': return 'image/jpeg'
    case 'gif': return 'image/gif'
    case 'webp': return 'image/webp'
    case 'svg': return 'image/svg+xml'
    case 'pdf': return 'application/pdf'
    case 'md': return 'text/markdown'
    case 'txt': return 'text/plain'
    case 'mp4': return 'video/mp4'
    case 'webm': return 'video/webm'
    // doc/docx fall through, matching Flutter — the extension is accepted but
    // carries no specific mime there either.
    default: return 'application/octet-stream'
  }
}

/** Rejection reason for a file, or null when it is acceptable. */
export function rejectionFor(file: { name: string; size: number }): string | null {
  const extension = file.name.split('.').pop()?.toLowerCase() ?? ''
  if (!(ALLOWED_EXTENSIONS as readonly string[]).includes(extension)) {
    return ATTACHMENT_ERRORS.unavailable
  }
  if (file.size > MAX_ATTACHMENT_BYTES) return ATTACHMENT_ERRORS.tooLarge
  return null
}

/** Resolved attachments, keyed by the assistant-ui attachment id.
 *
 *  assistant-ui's Attachment carries no filesystem path, so the staged path is
 *  held here and read back at send time. Entries are removed when the
 *  attachment is removed or the turn is sent, so this cannot grow unbounded. */
const stagedPaths = new Map<string, AttachmentInput>()

/** The staged input for an attachment id, if it resolved. */
export function stagedAttachment(id: string): AttachmentInput | undefined {
  return stagedPaths.get(id)
}

/** Drain everything staged for the turn being sent. */
export function takeStagedAttachments(ids: string[]): AttachmentInput[] {
  const resolved: AttachmentInput[] = []
  for (const id of ids) {
    const attachment = stagedPaths.get(id)
    if (attachment) {
      resolved.push(attachment)
      stagedPaths.delete(id)
    }
  }
  return resolved
}

/**
 * assistant-ui's attachment type for a mime.
 *
 * NOT cosmetic: `useAttachmentSrc` (components/assistant-ui/attachment.tsx)
 * gates the composer thumbnail and the preview dialog on `type === 'image'`.
 * This adapter used to hardcode `'document'` for everything, so a picked PNG
 * showed a grey chip with no preview — on top of rendering as a document in
 * the transcript.
 */
export function attachmentTypeFor(mime: string): 'image' | 'document' | 'file' {
  if (mime.startsWith('image/')) return 'image'
  if (mime.startsWith('video/')) return 'file'
  return 'document'
}

/**
 * Stage a picked file and remember where it landed.
 *
 * On Tauri the bytes are copied into app-owned storage by Rust, which also
 * decides the mime, whether it is an image or video, and its pixel dimensions.
 * Those answers are used verbatim — the previous code inferred the type here
 * and always said `document`, which is exactly the duplicated decision that
 * broke image rendering on every surface.
 *
 * On the web there is no Rust: the file is accepted into the transcript and
 * previewed from an object URL, but carries no durable path, so the turn sends
 * without file content. That degradation is honest rather than silent — the
 * composer still shows the thumbnail it can produce locally.
 */
export function createAttachmentAdapter(
  /** Reads the conversation the file is being attached to, at pick time.
   *  A getter rather than a value: the adapter is constructed once per runtime
   *  but the active conversation changes underneath it. */
  conversationId: () => string,
): AttachmentAdapter {
  return {
    accept: ACCEPT,

    async add({ file }): Promise<PendingAttachment> {
      const rejection = rejectionFor(file)
      if (rejection) throw new Error(rejection)

      const id = crypto.randomUUID()
      // Fallback for the web build, where Rust cannot classify the bytes.
      let mime = mimeFor(file.name)

      if (isTauri()) {
        const bytes = new Uint8Array(await file.arrayBuffer())
        const stored = await stageAttachment(conversationId(), file.name, bytes)
        mime = stored.mime
        stagedPaths.set(id, {
          path: stored.path,
          name: stored.fileName,
          size: stored.sizeBytes,
          mime: stored.mime,
          isImage: stored.isImage,
          isVideo: stored.isVideo,
          width: stored.width,
          height: stored.height,
        })
      }

      return {
        id,
        type: attachmentTypeFor(mime),
        name: file.name,
        contentType: mime,
        file,
        status: { type: 'requires-action', reason: 'composer-send' },
      }
    },

    async remove(attachment: Attachment): Promise<void> {
      stagedPaths.delete(attachment.id)
    },

    async send(attachment: PendingAttachment): Promise<CompleteAttachment> {
      return {
        id: attachment.id,
        type: attachment.type,
        name: attachment.name,
        contentType: attachment.contentType,
        content: [],
        status: { type: 'complete' },
      }
    },
  }
}
