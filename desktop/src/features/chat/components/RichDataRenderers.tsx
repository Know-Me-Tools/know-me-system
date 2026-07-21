// TJ-ARCH-MOB-001 compliant — Assistant UI data-part registration.
import { useAssistantDataUI, type DataMessagePartComponent } from '@assistant-ui/react'
import DOMPurify from 'dompurify'
import { useMemo, useState } from 'react'
import { AlertTriangle, Ban, BookOpenText, Brain, LoaderCircle, PackageOpen, Send, Sparkles } from 'lucide-react'
import type { ContentBlock } from '@/bridge/a2ui/types'
import { Button } from '@/components/ui/button'
import { useChat } from '../hooks/useChat'
import { A2uiArtifactRenderer } from './A2uiArtifactRenderer'

type BlockOf<T extends ContentBlock['type']> = Extract<ContentBlock, { type: T }>

const MemoryData: DataMessagePartComponent<BlockOf<'memory'>> = ({ data }) => (
  <section className="my-3 rounded-xl bg-[color:var(--color-surface)] p-4" aria-label="Memory event">
    <div className="flex items-center gap-2 text-[color:var(--color-cyan)]"><Brain className="size-4" /><span className="eyebrow !text-[color:var(--color-cyan)]">Memory · {data.operation}</span></div>
    <p className="mt-2 font-semibold text-[color:var(--color-fg)]">{data.key}</p>
    {data.value ? <p className="mt-1 text-sm leading-relaxed text-[color:var(--color-fg-sub)]">{data.value}</p> : null}
  </section>
)

const CitationData: DataMessagePartComponent<BlockOf<'citation'>> = ({ data }) => (
  <figure className="my-3 rounded-xl bg-[color:var(--color-card)] p-4">
    <figcaption className="flex items-center gap-2 text-[color:var(--color-ember)]"><BookOpenText className="size-4" /><span className="eyebrow">Citation · {data.source}</span></figcaption>
    <blockquote className="mt-2 text-sm leading-relaxed text-[color:var(--color-fg-sub)]">{data.quote}</blockquote>
  </figure>
)

const SkillData: DataMessagePartComponent<BlockOf<'skill'>> = ({ data }) => (
  <div className="my-3 flex items-center gap-3 rounded-xl bg-[color:var(--color-surface)] p-4">
    <Sparkles className="size-4 text-[color:var(--color-amber)]" />
    <div><p className="text-sm font-semibold text-[color:var(--color-fg)]">{data.name}</p><p className="text-xs text-[color:var(--color-fg-sub)]">{data.status}</p></div>
  </div>
)

function parseJsonObject(value: string): Record<string, unknown> | null {
  try {
    const parsed: unknown = JSON.parse(value)
    return parsed !== null && typeof parsed === 'object' && !Array.isArray(parsed)
      ? parsed as Record<string, unknown>
      : null
  } catch {
    return null
  }
}

function artifactIdFrom(payload: Record<string, unknown> | null, fallback: string): string {
  const response = payload?.a2uiResponse
  if (response && typeof response === 'object' && !Array.isArray(response)) {
    const candidate = (response as Record<string, unknown>).artifactId ?? (response as Record<string, unknown>).artifact_id
    if (typeof candidate === 'string' && candidate.trim()) return candidate
  }
  const candidate = payload?.artifactId ?? payload?.artifact_id ?? payload?.surfaceId
  return typeof candidate === 'string' && candidate.trim() ? candidate : fallback
}

function defaultArtifactResponse(payload: Record<string, unknown> | null): string {
  const response = payload?.a2uiResponse
  if (response && typeof response === 'object' && !Array.isArray(response)) {
    const value = (response as Record<string, unknown>).response
    if (value !== undefined) return JSON.stringify(value, null, 2)
  }
  return '{}'
}

function isInteractiveArtifact(kind: string, payload: Record<string, unknown> | null): boolean {
  if (payload?.a2uiResponse || payload?.action) return true
  return /a2ui|input|form|select|approval|confirm/i.test(kind)
}

const ArtifactData: DataMessagePartComponent<BlockOf<'artifact'>> = ({ data }) => {
  const { submitInteraction } = useChat()
  const isSvg = data.kind.toLowerCase() === 'svg'
  const isA2ui = /(^a2ui$|application\/a2ui\+json)/i.test(data.kind)
  const payload = useMemo(() => parseJsonObject(data.content), [data.content])
  const interactive = isInteractiveArtifact(data.kind, payload)
  const [responseJson, setResponseJson] = useState(() => defaultArtifactResponse(payload))
  const [submitting, setSubmitting] = useState(false)
  const [submitted, setSubmitted] = useState(false)
  const [submitError, setSubmitError] = useState<string | null>(null)
  const safeSvg = isSvg
    ? DOMPurify.sanitize(data.content, { USE_PROFILES: { svg: true, svgFilters: true } })
    : ''
  const submitResponse = async () => {
    const response = parseJsonObject(responseJson)
    if (!response) {
      setSubmitError('Response must be a JSON object.')
      return
    }
    setSubmitting(true)
    setSubmitError(null)
    try {
      await submitInteraction({
        ...(payload ?? {}),
        a2uiResponse: {
          artifactId: artifactIdFrom(payload, data.id),
          response,
        },
      })
      setSubmitted(true)
    } catch (error) {
      setSubmitError(error instanceof Error ? error.message : 'Failed to submit artifact response.')
    } finally {
      setSubmitting(false)
    }
  }

  return (
    <section className="my-3 overflow-hidden rounded-xl bg-[color:var(--color-surface)]">
      <div className="flex items-center gap-2 bg-[color:var(--color-card)] px-4 py-3"><PackageOpen className="size-4 text-[color:var(--color-ember)]" /><span className="eyebrow">Artifact · {data.kind}</span></div>
      {isA2ui ? (
        <A2uiArtifactRenderer source={data.content} title={`Artifact ${data.id}`} sourceRunId={data.id} onAction={submitInteraction} />
      ) : isSvg ? (
        <div
          className="max-h-[36rem] overflow-auto bg-[color:var(--color-muted)] p-4 [&_svg]:mx-auto [&_svg]:max-w-full"
          role="img"
          aria-label="Generated SVG artifact"
          dangerouslySetInnerHTML={{ __html: safeSvg }}
        />
      ) : (
        <details className="group bg-[color:var(--color-bg-2)]">
          <summary className="cursor-pointer list-none px-4 py-3 text-sm font-medium text-[color:var(--color-fg-sub)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-[color:var(--color-ember)]">
            View event payload
          </summary>
          <pre className="max-h-80 overflow-auto whitespace-pre-wrap px-4 pb-4 font-mono text-xs text-[color:var(--color-fg-sub)]">{data.content}</pre>
        </details>
      )}
      {interactive && !isA2ui ? (
        <div className="space-y-3 bg-[color:var(--color-bg-2)] p-4">
          <label className="block text-xs font-semibold uppercase tracking-[0.16em] text-[color:var(--color-fg-sub)]" htmlFor={`artifact-response-${data.id}`}>
            A2UI response JSON
          </label>
          <textarea
            id={`artifact-response-${data.id}`}
            className="min-h-24 w-full resize-y rounded-xl border-0 bg-[color:var(--color-card)] p-3 font-mono text-xs text-[color:var(--color-fg)] outline-none ring-0 placeholder:text-[color:var(--color-fg-sub)] focus-visible:ring-2 focus-visible:ring-[color:var(--color-ember)]"
            value={responseJson}
            onChange={(event) => setResponseJson(event.currentTarget.value)}
            disabled={submitting || submitted}
            spellCheck={false}
          />
          <div className="flex items-center justify-between gap-3">
            <p className="text-xs text-[color:var(--color-fg-sub)]" role={submitError ? 'alert' : 'status'}>
              {submitError ?? (submitted ? 'Response submitted.' : 'Submit sends this response back to the UAR artifact-response flow.')}
            </p>
            <Button
              type="button"
              variant="secondary"
              size="sm"
              onClick={() => void submitResponse()}
              disabled={submitting || submitted}
            >
              <Send className="size-3.5" aria-hidden />
              {submitting ? 'Submitting' : submitted ? 'Submitted' : 'Submit'}
            </Button>
          </div>
        </div>
      ) : null}
    </section>
  )
}

interface RunLifecycleData { phase: string; message: string }
interface RunErrorData extends RunLifecycleData { code: string; retryable: boolean; diagnostics?: string }

const RunLifecycle: DataMessagePartComponent<RunLifecycleData> = ({ data }) => (
  <section className="my-3 flex items-center gap-3 rounded-xl bg-[color:var(--color-surface)] p-4" role="status" aria-live="polite">
    <LoaderCircle className="size-4 animate-spin motion-reduce:animate-none" aria-hidden />
    <div><p className="text-sm font-semibold">{data.message}</p><p className="text-xs text-[color:var(--color-fg-sub)]">{data.phase.replaceAll('_', ' ')}</p></div>
  </section>
)

const RunError: DataMessagePartComponent<RunErrorData> = ({ data }) => (
  <section className="my-3 flex items-start gap-3 rounded-xl bg-destructive/10 p-4 text-destructive" role="alert">
    <AlertTriangle className="mt-0.5 size-4 shrink-0" aria-hidden />
    <div><p className="text-sm font-semibold">{data.message}</p><p className="mt-1 text-xs">{data.phase.replaceAll('_', ' ')} · {data.code}{data.retryable ? ' · retry available' : ''}</p></div>
  </section>
)

const RunCancelled: DataMessagePartComponent<RunLifecycleData> = ({ data }) => (
  <section className="my-3 flex items-center gap-3 rounded-xl bg-[color:var(--color-surface)] p-4" role="status">
    <Ban className="size-4" aria-hidden /><span className="text-sm">{data.message}</span>
  </section>
)

export function RichDataRenderers() {
  useAssistantDataUI({ name: 'memory', render: MemoryData })
  useAssistantDataUI({ name: 'citation', render: CitationData })
  useAssistantDataUI({ name: 'skill', render: SkillData })
  useAssistantDataUI({ name: 'artifact', render: ArtifactData })
  useAssistantDataUI({ name: 'runLifecycle', render: RunLifecycle })
  useAssistantDataUI({ name: 'runError', render: RunError })
  useAssistantDataUI({ name: 'runCancelled', render: RunCancelled })
  return null
}
