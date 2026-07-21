import { useMemo, useState } from 'react'
import { Check, ChevronDown, Code2, Copy, Library, LoaderCircle } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { useA2uiArtifactStore } from '../stores/a2uiArtifactStore'

type JsonObject = Record<string, unknown>

interface A2uiComponent extends JsonObject {
  id: string
  component: string
}

interface Surface {
  id: string
  components: Map<string, A2uiComponent>
  data: JsonObject
}

interface Props {
  source: string
  title: string
  sourceRunId: string
  onAction: (action: JsonObject) => Promise<void>
}

function object(value: unknown): JsonObject {
  return value !== null && typeof value === 'object' && !Array.isArray(value) ? value as JsonObject : {}
}

function parseSource(source: string): Surface[] {
  const surfaces = new Map<string, Surface>()
  for (const line of source.split('\n').map((value) => value.trim()).filter(Boolean)) {
    const message = object(JSON.parse(line))
    const create = object(message.createSurface)
    const update = object(message.updateComponents)
    const data = object(message.updateDataModel)
    const deleted = object(message.deleteSurface)
    const surfaceId = String(create.surfaceId ?? update.surfaceId ?? data.surfaceId ?? deleted.surfaceId ?? '')
    if (!surfaceId) throw new Error('A2UI message is missing surfaceId')
    if (Object.keys(deleted).length) {
      surfaces.delete(surfaceId)
      continue
    }
    const surface = surfaces.get(surfaceId) ?? { id: surfaceId, components: new Map(), data: {} }
    if (Array.isArray(update.components)) {
      for (const value of update.components) {
        const component = object(value) as A2uiComponent
        if (!component.id || !component.component) throw new Error('A2UI component is missing id or component')
        surface.components.set(component.id, component)
      }
    }
    if (Object.keys(data).length) setPath(surface.data, String(data.path ?? '/'), data.value)
    surfaces.set(surfaceId, surface)
  }
  return [...surfaces.values()]
}

function pathSegments(path: string): string[] {
  return path.split('/').filter(Boolean).map((value) => value.replaceAll('~1', '/').replaceAll('~0', '~'))
}

function readPath(root: JsonObject, path: string): unknown {
  let value: unknown = root
  for (const segment of pathSegments(path)) value = object(value)[segment]
  return value
}

function setPath(root: JsonObject, path: string, value: unknown): void {
  const segments = pathSegments(path)
  if (!segments.length && value && typeof value === 'object' && !Array.isArray(value)) {
    Object.assign(root, value)
    return
  }
  let cursor = root
  segments.forEach((segment, index) => {
    if (index === segments.length - 1) cursor[segment] = value
    else cursor = object(cursor[segment] ??= {})
  })
}

function dynamic(value: unknown, data: JsonObject): unknown {
  const binding = object(value)
  return typeof binding.path === 'string' ? readPath(data, binding.path) : value
}

function referencedIds(component: A2uiComponent): string[] {
  const values = [component.child, ...(Array.isArray(component.children) ? component.children : [])]
  return values.filter((value): value is string => typeof value === 'string')
}

function rootIds(surface: Surface): string[] {
  const referenced = new Set([...surface.components.values()].flatMap(referencedIds))
  return [...surface.components.keys()].filter((id) => !referenced.has(id))
}

function SurfaceView({ surface, sourceRunId, onAction }: { surface: Surface; sourceRunId: string; onAction: Props['onAction'] }) {
  const [, refresh] = useState(0)
  const render = (id: string): React.ReactNode => {
    const component = surface.components.get(id)
    if (!component) return null
    switch (component.component) {
      case 'Text': {
        const text = String(dynamic(component.text, surface.data) ?? '')
        const variant = String(component.variant ?? 'body')
        return variant.includes('title') || /^#{1,3}\s/.test(text)
          ? <h3 key={id} className="text-lg font-semibold text-[color:var(--color-fg)]">{text.replace(/^#{1,3}\s*/, '')}</h3>
          : <p key={id} className="whitespace-pre-wrap text-sm leading-relaxed text-[color:var(--color-fg)]">{text}</p>
      }
      case 'Button': {
        const event = object(object(component.action).event)
        return <Button key={id} type="button" onClick={() => void onAction({
          sourceRunId,
          version: 'v0.9',
          action: {
            surfaceId: surface.id,
            name: String(event.name ?? 'action'),
            sourceComponentId: id,
            timestamp: new Date().toISOString(),
            context: object(event.context),
          },
          a2uiClientDataModel: surface.data,
          artifactId: surface.id,
        })}>{render(String(component.child ?? ''))}</Button>
      }
      case 'TextField': {
        const binding = object(component.value)
        const value = String(dynamic(component.value, surface.data) ?? '')
        return <label key={id} className="grid gap-2 text-sm font-medium text-[color:var(--color-fg)]">
          {String(component.label ?? '')}
          <input className="min-h-11 rounded-xl border-0 bg-[color:var(--color-bg-2)] px-3 text-sm outline-none focus-visible:ring-2 focus-visible:ring-[color:var(--color-ember)]" value={value} placeholder={String(component.placeholder ?? '')} onChange={(event) => {
            if (typeof binding.path === 'string') setPath(surface.data, binding.path, event.currentTarget.value)
            refresh((value) => value + 1)
          }} />
        </label>
      }
      case 'CheckBox': {
        const binding = object(component.value)
        const checked = Boolean(dynamic(component.value, surface.data))
        return <label key={id} className="flex min-h-11 items-center gap-3 text-sm text-[color:var(--color-fg)]">
          <input type="checkbox" checked={checked} onChange={(event) => {
            if (typeof binding.path === 'string') setPath(surface.data, binding.path, event.currentTarget.checked)
            refresh((value) => value + 1)
          }} />
          {String(component.label ?? '')}
        </label>
      }
      case 'ChoicePicker': {
        const binding = object(component.value)
        const selected = dynamic(component.value, surface.data)
        const options = Array.isArray(component.options) ? component.options.map(object) : []
        return <label key={id} className="grid gap-2 text-sm font-medium text-[color:var(--color-fg)]">
          {String(component.label ?? '')}
          <select className="min-h-11 rounded-xl border-0 bg-[color:var(--color-bg-2)] px-3 outline-none focus-visible:ring-2 focus-visible:ring-[color:var(--color-ember)]" value={Array.isArray(selected) ? String(selected[0] ?? '') : String(selected ?? '')} onChange={(event) => {
            if (typeof binding.path === 'string') setPath(surface.data, binding.path, component.variant === 'multiple' ? [event.currentTarget.value] : event.currentTarget.value)
            refresh((value) => value + 1)
          }}>
            {options.map((option) => <option key={String(option.value)} value={String(option.value)}>{String(option.label)}</option>)}
          </select>
        </label>
      }
      case 'Row': return <div key={id} className="flex flex-wrap items-center gap-3">{referencedIds(component).map(render)}</div>
      case 'Column': return <div key={id} className="grid gap-3">{referencedIds(component).map(render)}</div>
      case 'Card': return <section key={id} className="rounded-2xl bg-[color:var(--color-card)] p-4">{render(String(component.child ?? ''))}</section>
      case 'Divider': return <div key={id} className="h-2" aria-hidden />
      default: return <p key={id} role="alert" className="rounded-xl bg-destructive/10 p-3 text-sm text-destructive">Unsupported A2UI component: {component.component}</p>
    }
  }
  return <div className="grid gap-3">{rootIds(surface).map(render)}</div>
}

export function A2uiArtifactRenderer({ source, title, sourceRunId, onAction }: Props) {
  const parsed = useMemo(() => {
    try { return { surfaces: parseSource(source), error: null as string | null } }
    catch (error) { return { surfaces: [], error: error instanceof Error ? error.message : 'Invalid A2UI source' } }
  }, [source])
  const [sourceOpen, setSourceOpen] = useState(false)
  const [promoted, setPromoted] = useState(false)
  const promoting = useA2uiArtifactStore((state) => state.promoting)
  const error = useA2uiArtifactStore((state) => state.error)
  const promoteArtifact = useA2uiArtifactStore((state) => state.promote)
  const promote = async () => {
    if (await promoteArtifact(title, source)) setPromoted(true)
  }
  if (parsed.error) return <p role="alert" className="bg-destructive/10 p-4 text-sm text-destructive">A2UI protocol error: {parsed.error}</p>
  return <div className="bg-[color:var(--color-bg-2)] p-4">
    <div className="grid gap-4">{parsed.surfaces.map((surface) => <SurfaceView key={surface.id} surface={surface} sourceRunId={sourceRunId} onAction={onAction} />)}</div>
    <div className="mt-4 flex flex-wrap items-center gap-2">
      <Button type="button" variant="ghost" size="sm" onClick={() => setSourceOpen((value) => !value)}><Code2 className="size-4" />Source<ChevronDown className={`size-4 transition-transform ${sourceOpen ? 'rotate-180' : ''}`} /></Button>
      <Button type="button" variant="ghost" size="sm" onClick={() => void navigator.clipboard.writeText(source)}><Copy className="size-4" />Copy source</Button>
      <Button type="button" variant="ghost" size="sm" disabled={promoting || promoted} onClick={() => void promote()}>{promoting ? <LoaderCircle className="size-4 animate-spin" /> : promoted ? <Check className="size-4" /> : <Library className="size-4" />}{promoted ? 'Added to library' : 'Add to UAR library'}</Button>
    </div>
    {sourceOpen ? <pre className="mt-3 max-h-80 overflow-auto rounded-xl bg-[color:var(--color-surface)] p-4 font-mono text-xs text-[color:var(--color-fg-sub)]">{source}</pre> : null}
    {error ? <p role="alert" className="mt-3 text-sm text-destructive">{error}</p> : null}
  </div>
}
