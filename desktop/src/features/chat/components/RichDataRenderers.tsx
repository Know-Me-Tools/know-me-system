// TJ-ARCH-MOB-001 compliant — Assistant UI data-part registration.
import { useAssistantDataUI, type DataMessagePartComponent } from '@assistant-ui/react'
import DOMPurify from 'dompurify'
import { BookOpenText, Brain, PackageOpen, Sparkles } from 'lucide-react'
import type { ContentBlock } from '@/bridge/a2ui/types'

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

const ArtifactData: DataMessagePartComponent<BlockOf<'artifact'>> = ({ data }) => {
  const isSvg = data.kind.toLowerCase() === 'svg'
  const safeSvg = isSvg
    ? DOMPurify.sanitize(data.content, { USE_PROFILES: { svg: true, svgFilters: true } })
    : ''
  return (
    <section className="my-3 overflow-hidden rounded-xl bg-[color:var(--color-surface)]">
      <div className="flex items-center gap-2 bg-[color:var(--color-card)] px-4 py-3"><PackageOpen className="size-4 text-[color:var(--color-ember)]" /><span className="eyebrow">Artifact · {data.kind}</span></div>
      {isSvg ? (
        <div
          className="max-h-[36rem] overflow-auto bg-[color:var(--color-muted)] p-4 [&_svg]:mx-auto [&_svg]:max-w-full"
          role="img"
          aria-label="Generated SVG artifact"
          dangerouslySetInnerHTML={{ __html: safeSvg }}
        />
      ) : (
        <pre className="max-h-96 overflow-auto whitespace-pre-wrap p-4 font-mono text-xs text-[color:var(--color-fg-sub)]">{data.content}</pre>
      )}
    </section>
  )
}

export function RichDataRenderers() {
  useAssistantDataUI({ name: 'memory', render: MemoryData })
  useAssistantDataUI({ name: 'citation', render: CitationData })
  useAssistantDataUI({ name: 'skill', render: SkillData })
  useAssistantDataUI({ name: 'artifact', render: ArtifactData })
  return null
}
