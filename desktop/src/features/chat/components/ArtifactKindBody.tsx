// TJ-ARCH-MOB-001 compliant
//
// The body of an artifact, dispatched on its kind.
//
// React handled three kinds — a2ui, svg, and "everything else" — so markdown,
// mermaid, video, link previews, code, documents, attachments and EVERY
// `agui_*` runtime event collapsed into a collapsed <details> labelled "View
// event payload" showing raw JSON. Including `agui_confirmation`, which on
// Flutter is an actionable choice UI: the user was shown a JSON dump where the
// other surface asks them a question.
//
// Kinds and their aliases mirror Flutter's `ContentBlockArtifact` switch
// (gen_ui_widgets/lib/src/content_block_view.dart) so the same artifact renders
// as the same thing on both surfaces.
import { useMemo, type FC } from 'react'
import DOMPurify from 'dompurify'
import { FileText } from 'lucide-react'
import ReactMarkdown from 'react-markdown'

import { MermaidBlock } from '@/components/assistant-ui/markdown-text'
import { localMediaSrc } from '../lib/localMedia'
import { useRemoteContent } from '../hooks/useRemoteContent'

/** What kind of body an artifact resolves to, after alias normalisation. */
export type ArtifactKind =
  | 'markdown'
  | 'mermaid'
  | 'svg'
  | 'video'
  | 'link'
  | 'code'
  | 'document'
  | 'a2ui'
  | 'agui'
  | 'unknown'

/**
 * Normalise an artifact kind to the body that renders it.
 *
 * Every alias is Flutter's. `agui_*` is checked FIRST because Flutter's prefix
 * test runs before its switch — an `agui_` kind never reaches the kind arms
 * even if it happens to collide with one.
 */
export function artifactKind(raw: string): ArtifactKind {
  const kind = raw.toLowerCase().trim()
  if (kind.startsWith('agui_')) return 'agui'
  switch (kind) {
    case 'markdown': case 'md': case 'document': return 'markdown'
    case 'mermaid': return 'mermaid'
    case 'svg': case 'image/svg+xml': return 'svg'
    case 'video': case 'mp4': case 'webm': return 'video'
    case 'opengraph': case 'open_graph': case 'link': return 'link'
    case 'code': return 'code'
    case 'pdf': case 'doc': case 'docx': case 'attachment': return 'document'
    case 'a2ui': case 'application/a2ui+json': return 'a2ui'
    default: return 'unknown'
  }
}

/** Unwrap a `{url: ...}` envelope, as Flutter's `_mediaUrl` does. */
export function mediaUrl(content: string): string {
  try {
    const parsed: unknown = JSON.parse(content)
    if (parsed && typeof parsed === 'object' && !Array.isArray(parsed)) {
      const record = parsed as Record<string, unknown>
      const url = record.url
      if (typeof url === 'string') return url
      // A user-attached video carries a durable filesystem `path` instead of a
      // url. Without this the payload fell through and the whole JSON string
      // was handed to <video src>, producing a permanently broken player.
      const path = record.path
      if (typeof path === 'string') return localMediaSrc(path)
    }
  } catch {
    // Not JSON — the content IS the url.
  }
  return content
}

/** Title for an `agui_*` event, mirroring Flutter's `_activityTitle`. */
export function aguiTitle(kind: string): string {
  switch (kind) {
    case 'agui_state_snapshot': return 'Agent state synchronized'
    case 'agui_state_delta': return 'Agent state updated'
    case 'agui_messages_snapshot': return 'Conversation synchronized'
    case 'agui_raw': return 'External agent event'
    default: break
  }
  if (kind.startsWith('agui_activity_')) {
    const name = kind.slice('agui_activity_'.length).replace(/_delta$/, '').replace(/_/g, ' ')
    return `${name.charAt(0).toUpperCase()}${name.slice(1)} activity`
  }
  return 'Agent event'
}

/** One-line summary of an agui payload, mirroring Flutter's `_mapSummary`. */
export function aguiSummary(content: string): string {
  let parsed: unknown
  try {
    parsed = JSON.parse(content)
  } catch {
    return 'Inspect event details'
  }
  if (Array.isArray(parsed)) return `${parsed.length} messages`
  if (parsed && typeof parsed === 'object') {
    const keys = Object.keys(parsed as Record<string, unknown>)
    if (keys.length === 0) return 'No visible fields'
    if (keys.length <= 3) return `Updated ${keys.join(', ')}`
    return `${keys.length} updates`
  }
  return 'State available'
}

function DocumentBody({ content }: { content: string }) {
  const meta = useMemo(() => {
    try {
      const parsed: unknown = JSON.parse(content)
      if (parsed && typeof parsed === 'object' && !Array.isArray(parsed)) {
        const record = parsed as Record<string, unknown>
        return {
          name: typeof record.name === 'string' ? record.name : null,
          size: typeof record.size === 'string' ? record.size : null,
          excerpt: typeof record.excerpt === 'string' ? record.excerpt : null,
        }
      }
    } catch {
      // Fall through to the raw-content presentation below.
    }
    return { name: null, size: null, excerpt: null }
  }, [content])

  if (!meta.name) {
    return <pre className="max-h-80 overflow-auto whitespace-pre-wrap px-4 py-3 font-mono text-xs">{content}</pre>
  }
  return (
    <div className="flex items-start gap-3 px-4 py-3">
      <FileText aria-hidden className="mt-0.5 size-5 shrink-0 text-[color:var(--color-fg-sub)]" />
      <div className="min-w-0">
        <p className="truncate text-sm font-medium text-[color:var(--color-fg)]">{meta.name}</p>
        {meta.size ? (
          <p className="text-xs text-[color:var(--color-fg-sub)]">{meta.size}</p>
        ) : null}
        {meta.excerpt ? (
          // Six lines, matching Flutter's preview cap — enough to recognise the
          // document, not so much that it competes with the answer.
          <p className="mt-1 line-clamp-6 text-xs text-[color:var(--color-fg-sub)]">{meta.excerpt}</p>
        ) : null}
      </div>
    </div>
  )
}

function LinkBody({ content }: { content: string }) {
  const meta = useMemo(() => {
    try {
      const parsed: unknown = JSON.parse(content)
      if (parsed && typeof parsed === 'object' && !Array.isArray(parsed)) {
        const record = parsed as Record<string, unknown>
        const read = (key: string) => (typeof record[key] === 'string' ? record[key] as string : null)
        return {
          url: read('url'),
          title: read('title'),
          description: read('description'),
          siteName: read('siteName') ?? read('site_name'),
          // Flutter has always rendered this (content_block_view.dart's link
          // preview delegates to ContentBlockImage); React dropped it, so the
          // same link showed a thumbnail on mobile and a bare title here.
          imageUrl: read('imageUrl') ?? read('image_url') ?? read('image'),
        }
      }
    } catch {
      // Bare URL rather than an envelope.
    }
    return { url: content, title: null, description: null, siteName: null, imageUrl: null }
  }, [content])

  if (!meta.url) {
    return <pre className="max-h-80 overflow-auto whitespace-pre-wrap px-4 py-3 font-mono text-xs">{content}</pre>
  }
  return (
    <a
      href={meta.url}
      target="_blank"
      rel="noreferrer noopener"
      className="block hover:bg-[color:var(--color-card-hov)]"
    >
      {meta.imageUrl ? <LinkThumbnail url={meta.imageUrl} /> : null}
      <div className="px-4 py-3">
      <p className="truncate text-sm font-medium text-[color:var(--color-fg)]">{meta.title ?? meta.url}</p>
      {meta.description ? (
        <p className="mt-1 line-clamp-2 text-xs text-[color:var(--color-fg-sub)]">{meta.description}</p>
      ) : null}
      {/* `--color-fg-sub`, not `fg-faint`: faint is 3.62:1 on this surface in
          dark mode, under the 4.5:1 floor for 12px text. This line is the only
          thing identifying where the link goes. */}
      <p className="mt-1 truncate text-xs text-[color:var(--color-fg-sub)]">{meta.siteName ?? meta.url}</p>
      </div>
    </a>
  )
}

/**
 * The OpenGraph image for a link preview.
 *
 * Resolved through `useRemoteContent` because an OG image is remote by
 * definition and is exactly the CORS/mixed-content case that hook exists for.
 *
 * A failed thumbnail renders NOTHING rather than a broken-image placeholder or
 * an error row: the preview's title, description and host are still useful on
 * their own, and the picture is decoration. It is marked aria-hidden for the
 * same reason — the surrounding link already carries the accessible name, so
 * announcing the image would repeat it.
 */
function LinkThumbnail({ url }: { url: string }) {
  const { src, failed, onError } = useRemoteContent(url, 'image')
  if (failed) return null
  return (
    <img
      src={src}
      onError={onError}
      alt=""
      aria-hidden="true"
      loading="lazy"
      className="max-h-48 w-full object-cover"
    />
  )
}

function VideoBody({ url }: { url: string }) {
  // Falls back to a native fetch when the webview cannot reach the origin —
  // CORS or mixed content would otherwise render a permanently broken player.
  const { src, failed, onError } = useRemoteContent(url, 'video')
  if (failed) {
    return (
      <p role="alert" className="px-4 py-3 text-sm text-[color:var(--color-fg-sub)]">
        This video could not be loaded.
      </p>
    )
  }
  return (
    // eslint-disable-next-line jsx-a11y/media-has-caption -- the model supplies no track
    <video
      src={src}
      onError={onError}
      controls
      className="max-h-[36rem] w-full"
      aria-label="Generated video artifact"
    />
  )
}

function AguiBody({ kind, content }: { kind: string; content: string }) {
  const title = aguiTitle(kind)
  const summary = aguiSummary(content)
  const pretty = useMemo(() => {
    try {
      return JSON.stringify(JSON.parse(content), null, 2)
    } catch {
      return content
    }
  }, [content])
  return (
    <details className="group">
      <summary className="cursor-pointer list-none px-4 py-3 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-[color:var(--color-ember)]">
        <span className="block text-sm font-medium text-[color:var(--color-fg)]">{title}</span>
        <span className="block text-xs text-[color:var(--color-fg-sub)]">{summary}</span>
      </summary>
      <pre className="max-h-80 overflow-auto whitespace-pre-wrap px-4 pb-4 font-mono text-xs text-[color:var(--color-fg-sub)]">{pretty}</pre>
    </details>
  )
}

interface ArtifactKindBodyProps {
  kind: ArtifactKind
  rawKind: string
  content: string
}

/** Render an artifact body. `a2ui` is handled by its own renderer upstream. */
export const ArtifactKindBody: FC<ArtifactKindBodyProps> = ({ kind, rawKind, content }) => {
  switch (kind) {
    case 'markdown':
      return (
        <div className="prose prose-sm dark:prose-invert max-w-none px-4 py-3">
          <ReactMarkdown>{content}</ReactMarkdown>
        </div>
      )
    case 'mermaid':
      return <MermaidBlock source={content} />
    case 'svg':
      return (
        <div
          className="max-h-[36rem] overflow-auto bg-[color:var(--color-muted)] p-4 [&_svg]:mx-auto [&_svg]:max-w-full"
          role="img"
          aria-label="Generated SVG artifact"
          // Sanitised to the SVG profile: an artifact is model-authored, so a
          // raw insert would be a script-injection path.
          dangerouslySetInnerHTML={{
            __html: DOMPurify.sanitize(content, { USE_PROFILES: { svg: true, svgFilters: true } }),
          }}
        />
      )
    case 'video':
      return <VideoBody url={mediaUrl(content)} />
    case 'link':
      return <LinkBody content={content} />
    case 'code':
      return (
        <pre className="max-h-80 overflow-auto whitespace-pre-wrap px-4 py-3 font-mono text-xs text-[color:var(--color-fg)]">
          {content}
        </pre>
      )
    case 'document':
      return <DocumentBody content={content} />
    case 'agui':
      return <AguiBody kind={rawKind.toLowerCase().trim()} content={content} />
    case 'a2ui':
    case 'unknown':
      return (
        <details className="group bg-[color:var(--color-bg-2)]">
          <summary className="cursor-pointer list-none px-4 py-3 text-sm font-medium text-[color:var(--color-fg-sub)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-[color:var(--color-ember)]">
            View event payload
          </summary>
          <pre className="max-h-80 overflow-auto whitespace-pre-wrap px-4 pb-4 font-mono text-xs text-[color:var(--color-fg-sub)]">{content}</pre>
        </details>
      )
  }
}
