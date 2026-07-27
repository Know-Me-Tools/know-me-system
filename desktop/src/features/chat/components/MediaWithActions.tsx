// TJ-ARCH-MOB-001 compliant — presentation; calls lib helpers, not invoke().
//
// An image in the transcript, with Save and (where the platform supports it)
// Share. Mirrors Flutter's block-header actions so the same conversation
// offers the same affordances on both surfaces.
import { useCallback, useEffect, useState, type FC } from 'react'
import { Download, Share2 } from 'lucide-react'

import {
  canShareMedia,
  saveMedia,
  shareMedia,
  type MediaActionOutcome,
} from '../lib/mediaActions'

interface MediaWithActionsProps {
  src: string
  fileName: string
  alt: string
}

export const MediaWithActions: FC<MediaWithActionsProps> = ({ src, fileName, alt }) => {
  const [status, setStatus] = useState<string | null>(null)
  // Resolved once on mount rather than per render: navigator.canShare is a
  // capability probe, not state.
  const [shareable] = useState(() => canShareMedia())

  // Clear the announcement after a moment so it does not sit under the image
  // forever, but leave it long enough to actually be read.
  useEffect(() => {
    if (!status) return
    const timer = setTimeout(() => setStatus(null), 4000)
    return () => clearTimeout(timer)
  }, [status])

  const announce = useCallback((outcome: MediaActionOutcome) => {
    setStatus(
      outcome.status === 'saved' ? 'Saved.'
        : outcome.status === 'shared' ? 'Shared.'
        // Dismissing the sheet is a normal choice, not an error.
        : outcome.status === 'cancelled' ? 'Sharing cancelled.'
        : outcome.message,
    )
  }, [])

  return (
    <figure className="my-2">
      <img
        src={src}
        alt={alt}
        className="max-h-[420px] min-h-[120px] w-auto max-w-full rounded-2xl object-contain"
      />
      <figcaption className="mt-1 flex items-center gap-1">
        <button
          type="button"
          // Names the FILE, so a screen reader moving through a transcript of
          // several images can tell the buttons apart.
          aria-label={`Save ${fileName}`}
          title={`Save ${fileName}`}
          onClick={() => void saveMedia(src, fileName).then(announce)}
          className="inline-flex h-11 w-11 items-center justify-center rounded-lg text-[color:var(--color-fg-sub)] hover:bg-[color:var(--color-card-hov)] hover:text-[color:var(--color-fg)] focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2"
        >
          <Download className="size-4" aria-hidden="true" />
        </button>
        {/* Absent, not disabled, where the webview has no file share: a control
            that silently does nothing is worse than no control. */}
        {shareable ? (
          <button
            type="button"
            aria-label={`Share ${fileName}`}
            title={`Share ${fileName}`}
            onClick={() => void shareMedia(src, fileName, 'image/*').then(announce)}
            className="inline-flex h-11 w-11 items-center justify-center rounded-lg text-[color:var(--color-fg-sub)] hover:bg-[color:var(--color-card-hov)] hover:text-[color:var(--color-fg)] focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2"
          >
            <Share2 className="size-4" aria-hidden="true" />
          </button>
        ) : null}
        {/* Announced politely and in TEXT — never status by colour alone. */}
        <span role="status" aria-live="polite" className="text-xs text-[color:var(--color-fg-sub)]">
          {status}
        </span>
      </figcaption>
    </figure>
  )
}
