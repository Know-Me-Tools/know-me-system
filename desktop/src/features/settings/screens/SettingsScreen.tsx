// TJ-ARCH-MOB-001 compliant — responsive settings workspace; transport stays in hooks/stores.
import { useEffect, useMemo, useState } from 'react'
import {
  Bot,
  BrainCircuit,
  Check,
  ChevronRight,
  Database,
  Eye,
  KeyRound,
  Laptop,
  MemoryStick,
  Network,
  Package,
  Palette,
  Plug,
  Server,
  Settings2,
  ShieldCheck,
  Sparkles,
  Trash2,
  X,
} from 'lucide-react'
import type { ProviderModelEntry } from '@prometheus-ags/tauri-plugin-gen-ui'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Switch } from '@/components/ui/switch'
import { useProviderSettings } from '../hooks/useProviderSettings'
import { useSettings } from '../hooks/useSettings'
import { useUarSettings } from '../hooks/useUarSettings'
import { useModels } from '@/features/models/hooks/useModels'
import {
  AgentsPanel,
  ConnectionsPanel,
  GlobalRunPolicyPanel,
  KnowledgePanel,
  MemoryPanel,
  PluginsPanel,
  SettingsRegistryPanel,
  SkillsPanel,
} from '../components/UarAdminPanels'

type SectionId = 'providers' | 'runtime' | 'local-models' | 'agents' | 'skills' | 'knowledge' | 'memory' | 'connections' | 'privacy' | 'appearance' | 'plugins' | 'system'

const sections = [
  ['providers', 'Model providers', Server],
  ['runtime', 'Universal Agent Runtime', Network],
  ['local-models', 'Local models', MemoryStick],
  ['agents', 'Agents', Bot],
  ['skills', 'Skills', Sparkles],
  ['knowledge', 'Knowledge bases', Database],
  ['memory', 'Memories', BrainCircuit],
  ['connections', 'MCP & external access', Plug],
  ['privacy', 'Privacy & data', ShieldCheck],
  ['appearance', 'Appearance', Palette],
  ['plugins', 'Plugins', Package],
  ['system', 'System information', Laptop],
] as const satisfies ReadonlyArray<readonly [SectionId, string, typeof Server]>

const fieldClass = 'h-11 border-0 bg-[color:var(--color-surface)] shadow-none focus-visible:ring-2'

function capabilityLabels(model: ProviderModelEntry): string[] {
  return [
    model.supportsTools && 'Tools',
    model.supportsReasoning && 'Reasoning',
    model.supportsVision && 'Vision',
    model.supportsStructuredOutput && 'Structured output',
    model.supportsStreaming && 'Streaming',
  ].filter(Boolean) as string[]
}

function SettingsToggle({ label, caption, checked, onChange }: { label: string; caption: string; checked: boolean; onChange: () => void }) {
  return (
    <div className="flex min-h-20 items-center justify-between gap-6 rounded-xl bg-[color:var(--color-surface)] p-4">
      <div><p className="font-semibold">{label}</p><p className="mt-1 text-sm text-[color:var(--color-fg-sub)]">{caption}</p></div>
      <Switch checked={checked} onCheckedChange={onChange} aria-label={label} />
    </div>
  )
}

export function SettingsScreen({ overlay = false, onClose }: { overlay?: boolean; onClose?: () => void }) {
  const { settings, toggle, devices } = useSettings()
  const providers = useProviderSettings()
  const uar = useUarSettings()
  const localModels = useModels()
  const [section, setSection] = useState<SectionId>('providers')
  const [providerId, setProviderId] = useState('')
  const [baseUrl, setBaseUrl] = useState('')
  const [apiKey, setApiKey] = useState('')
  const [enabled, setEnabled] = useState(true)
  const [defaultModel, setDefaultModel] = useState('')
  const [modelDrafts, setModelDrafts] = useState<ProviderModelEntry[]>([])
  const [uarServiceUrl, setUarServiceUrl] = useState('')
  const [uarApiKey, setUarApiKey] = useState('')

  const orderedProviders = useMemo(() => [...providers.catalog].sort((left, right) => {
    if (left.configured !== right.configured) return left.configured ? -1 : 1
    return left.displayName.localeCompare(right.displayName)
  }), [providers.catalog])
  const selectedCatalog = orderedProviders.find((entry) => entry.id === providerId) ?? orderedProviders[0]
  const configured = providers.providers.find((entry) => entry.id === selectedCatalog?.id)

  useEffect(() => {
    if (!providerId && orderedProviders[0]) setProviderId(orderedProviders[0].id)
  }, [orderedProviders, providerId])

  useEffect(() => {
    if (!selectedCatalog) return
    setBaseUrl(configured?.baseUrl ?? selectedCatalog.defaultBaseUrl ?? '')
    setEnabled(configured?.enabled ?? true)
    setDefaultModel(configured?.defaultModel ?? (providers.cloudModel?.providerId === selectedCatalog.id ? providers.cloudModel.modelId : ''))
    setModelDrafts((configured?.models.length ? configured.models : selectedCatalog.models).map((model) => ({ ...model })))
    setApiKey('')
  }, [selectedCatalog?.id, configured?.id, providers.cloudModel?.providerId, providers.cloudModel?.modelId])

  useEffect(() => setUarServiceUrl(uar.config?.serviceUrl ?? ''), [uar.config?.serviceUrl])

  async function saveProvider() {
    if (!selectedCatalog) return
    const selectedDefault = defaultModel || modelDrafts.find((model) => model.enabled)?.id || null
    await providers.saveProvider({
      id: selectedCatalog.id,
      kind: selectedCatalog.id,
      displayName: selectedCatalog.displayName,
      baseUrl: baseUrl.trim() || selectedCatalog.defaultBaseUrl,
      apiKey: apiKey.trim() || null,
      enabled,
      protocol: configured?.protocol,
      defaultModel: selectedDefault,
      models: modelDrafts,
    })
    if (selectedDefault) await providers.saveCloudModel(selectedCatalog.id, selectedDefault)
    setApiKey('')
  }

  async function saveUar() {
    await uar.save({
      serviceUrl: uarServiceUrl.trim(),
      apiKey: uarApiKey.trim() || null,
      agentArtifactJson: '',
      sessionId: uar.config?.sessionId ?? null,
    })
    setUarApiKey('')
  }

  const shell = overlay
    ? 'fixed inset-x-0 bottom-0 top-9 z-50 bg-[color:var(--color-bg)]'
    : 'h-full bg-[color:var(--color-bg)]'

  return (
    <div className={`${shell} flex min-h-0 flex-col`} role={overlay ? 'dialog' : undefined} aria-modal={overlay || undefined} aria-label="Settings">
      <header className="flex min-h-16 shrink-0 items-center justify-between bg-[color:var(--color-bg-2)] px-4 sm:px-6">
        <div className="flex items-center gap-3"><Settings2 aria-hidden className="size-5 text-[color:var(--color-ember)]" /><div><h1 className="font-display text-xl font-bold">Settings</h1><p className="text-xs text-[color:var(--color-fg-sub)]">KnowMe and Universal Agent Runtime</p></div></div>
        {overlay ? <Button variant="ghost" size="icon" className="size-11" onClick={onClose} aria-label="Close settings"><X aria-hidden /></Button> : null}
      </header>

      <div className="flex min-h-0 flex-1 flex-col lg:grid lg:grid-cols-[15rem_minmax(16rem,20rem)_minmax(0,1fr)]">
        <nav aria-label="Settings categories" className="flex shrink-0 gap-1 overflow-x-auto bg-[color:var(--color-bg-2)] p-3 lg:flex-col lg:overflow-y-auto">
          {sections.map(([id, label, Icon]) => (
            <button key={id} type="button" onClick={() => setSection(id)} aria-current={section === id ? 'page' : undefined}
              className={section === id
                ? 'flex min-h-11 shrink-0 items-center gap-3 rounded-xl bg-[color:var(--color-card)] px-3 text-left text-sm font-semibold text-[color:var(--color-fg)]'
                : 'flex min-h-11 shrink-0 items-center gap-3 rounded-xl px-3 text-left text-sm text-[color:var(--color-fg-sub)] hover:bg-[color:var(--color-card-hov)] hover:text-[color:var(--color-fg)]'}>
              <Icon aria-hidden className="size-4" /><span>{label}</span>
            </button>
          ))}
        </nav>

        {section === 'providers' ? (
          <>
            <aside className="min-h-0 overflow-y-auto bg-[color:var(--color-bg)] p-3" aria-label="Providers">
              <Input aria-label="Search providers" placeholder="Search providers…" className={fieldClass} onChange={(event) => {
                const value = event.target.value.toLowerCase()
                const match = orderedProviders.find((provider) => provider.displayName.toLowerCase().includes(value) || provider.id.toLowerCase().includes(value))
                if (match) setProviderId(match.id)
              }} />
              <div className="mt-3 space-y-1">
                {orderedProviders.map((provider) => (
                  <button key={provider.id} type="button" onClick={() => setProviderId(provider.id)}
                    className={provider.id === selectedCatalog?.id ? 'flex min-h-12 w-full items-center gap-3 rounded-xl bg-[color:var(--color-card)] px-3 text-left' : 'flex min-h-12 w-full items-center gap-3 rounded-xl px-3 text-left hover:bg-[color:var(--color-card-hov)]'}>
                    <span className="grid size-8 shrink-0 place-items-center rounded-full bg-[color:var(--color-muted)] text-xs font-bold">{provider.displayName.slice(0, 2).toUpperCase()}</span>
                    <span className="min-w-0 flex-1 truncate text-sm font-medium">{provider.displayName}</span>
                    {provider.configured ? <span className="rounded-full bg-emerald-500/15 px-2 py-1 text-[10px] font-bold text-emerald-500">ON</span> : <ChevronRight aria-hidden className="size-4 text-[color:var(--color-fg-faint)]" />}
                  </button>
                ))}
              </div>
            </aside>

            <main className="min-h-0 overflow-y-auto bg-[color:var(--color-bg-2)] p-5 sm:p-7">
              {selectedCatalog ? <div className="mx-auto max-w-4xl">
                <div className="flex flex-wrap items-start justify-between gap-4">
                  <div><p className="eyebrow">UAR · Liter-LLM</p><h2 className="mt-1 font-display text-2xl font-bold">{selectedCatalog.displayName}</h2><p className="mt-1 text-sm text-[color:var(--color-fg-sub)]">{selectedCatalog.models.length} catalog models · credentials remain in UAR storage</p></div>
                  <Switch checked={enabled} onCheckedChange={setEnabled} aria-label={`Enable ${selectedCatalog.displayName}`} />
                </div>

                <div className="mt-6 grid gap-4 sm:grid-cols-2">
                  <div className="space-y-2"><Label htmlFor="provider-key"><KeyRound aria-hidden className="size-4" /> API key</Label><Input id="provider-key" type="password" autoComplete="off" value={apiKey} onChange={(event) => setApiKey(event.target.value)} placeholder={configured?.hasApiKey ? 'Configured in UAR · enter to replace' : 'Enter provider credential'} className={fieldClass} /></div>
                  <div className="space-y-2"><Label htmlFor="provider-url">API host</Label><Input id="provider-url" value={baseUrl} onChange={(event) => setBaseUrl(event.target.value)} placeholder="Provider default" className={fieldClass} /></div>
                </div>

                <section className="mt-7" aria-labelledby="provider-models-title">
                  <div className="flex items-end justify-between gap-4"><div><h3 id="provider-models-title" className="font-bold">Models</h3><p className="text-sm text-[color:var(--color-fg-sub)]">Only enabled models are exposed to UAR agents and chat.</p></div><span className="text-xs text-[color:var(--color-fg-faint)]">{modelDrafts.filter((model) => model.enabled).length} enabled</span></div>
                  <div className="mt-3 space-y-2">
                    {modelDrafts.map((model, index) => (
                      <div key={model.id} className="rounded-xl bg-[color:var(--color-surface)] p-4">
                        <div className="flex items-center gap-3"><Switch checked={model.enabled} onCheckedChange={(checked) => setModelDrafts((current) => current.map((item, itemIndex) => itemIndex === index ? { ...item, enabled: checked } : item))} aria-label={`Enable ${model.id}`} /><button type="button" className="min-w-0 flex-1 text-left" onClick={() => setDefaultModel(model.id)}><p className="truncate text-sm font-semibold">{model.displayName || model.id}</p><p className="truncate font-mono text-xs text-[color:var(--color-fg-faint)]">{model.id}</p></button>{defaultModel === model.id ? <span className="flex items-center gap-1 text-xs font-bold text-[color:var(--color-ember)]"><Check aria-hidden className="size-3" /> Default</span> : null}</div>
                        <div className="mt-3 flex flex-wrap gap-1.5">{model.contextWindow ? <span className="rounded-full bg-[color:var(--color-muted)] px-2 py-1 text-[10px]">{model.contextWindow.toLocaleString()} context</span> : null}{capabilityLabels(model).map((label) => <span key={label} className="rounded-full bg-[color:var(--color-muted)] px-2 py-1 text-[10px]">{label}</span>)}</div>
                      </div>
                    ))}
                  </div>
                </section>

                <div className="sticky bottom-0 mt-6 flex flex-wrap items-center justify-between gap-3 rounded-xl bg-[color:var(--color-card)] p-3">
                  <div className="flex gap-2">{configured ? <Button variant="ghost" size="icon" aria-label={`Delete ${selectedCatalog.displayName}`} onClick={() => void providers.removeProvider(selectedCatalog.id)}><Trash2 aria-hidden /></Button> : null}<Button variant="secondary" disabled={!configured || providers.testing} onClick={() => void providers.testProvider(selectedCatalog.id, defaultModel || null)}><Eye aria-hidden />{providers.testing ? 'Testing…' : 'Test connection'}</Button></div>
                  <Button disabled={providers.saving} onClick={() => void saveProvider()}>{providers.saving ? 'Saving…' : 'Save provider'}</Button>
                </div>
                {providers.testResult ? <p role="status" className="mt-3 rounded-xl bg-emerald-500/15 p-3 text-sm text-emerald-500">Connection succeeded with {providers.testResult.modelId} in {providers.testResult.latencyMs} ms.</p> : null}
                {providers.error ? <p role="alert" className="mt-3 rounded-xl bg-destructive/15 p-3 text-sm text-destructive">{providers.error}</p> : null}
              </div> : <p className="text-[color:var(--color-fg-sub)]">Loading the UAR provider catalog…</p>}
            </main>
          </>
        ) : (
          <main className="min-h-0 overflow-y-auto bg-[color:var(--color-bg)] p-5 sm:p-8 lg:col-span-2">
            <div className="mx-auto max-w-4xl">
              <p className="eyebrow">{sections.find(([id]) => id === section)?.[1]}</p>
              {section === 'runtime' ? <>
                <h2 className="mt-2 font-display text-3xl font-bold">Universal Agent Runtime</h2><p className="mt-2 text-[color:var(--color-fg-sub)]">UAR owns cloud providers, agents, skills, tools, knowledge and execution policy. Desktop may use an embedded runtime or service; mobile and web use the service contract.</p>
                <div className="mt-6 rounded-xl bg-[color:var(--color-bg-2)] p-5"><div className="grid gap-4 sm:grid-cols-2"><div className="space-y-2"><Label htmlFor="uar-url">Service URL</Label><Input id="uar-url" value={uarServiceUrl} onChange={(event) => setUarServiceUrl(event.target.value)} placeholder="http://127.0.0.1:8787" className={fieldClass} /></div><div className="space-y-2"><Label htmlFor="uar-key">Service API key · optional</Label><Input id="uar-key" type="password" value={uarApiKey} onChange={(event) => setUarApiKey(event.target.value)} placeholder={uar.config?.hasApiKey ? 'Configured · enter to replace' : 'UAR service credential'} className={fieldClass} /></div></div><div className="mt-4 flex flex-wrap gap-2"><Button disabled={!uarServiceUrl.trim() || uar.saving} onClick={() => void saveUar()}>Save runtime</Button><Button variant="secondary" disabled={!uar.config?.serviceUrl || uar.probing} onClick={() => void uar.probe()}>{uar.probing ? 'Testing…' : 'Test runtime'}</Button></div>{uar.probeResult ? <p role="status" className="mt-3 text-sm text-[color:var(--color-fg-sub)]">{uar.probeResult.message}</p> : null}{uar.error ? <p role="alert" className="mt-3 text-sm text-destructive">{uar.error}</p> : null}</div><GlobalRunPolicyPanel />
              </> : null}
              {section === 'appearance' ? <><h2 className="mt-2 font-display text-3xl font-bold">Appearance</h2><div className="mt-6"><SettingsToggle label="Dark theme" caption="Use KnowMe’s accessible dark palette across every surface." checked={settings.dark} onChange={() => toggle('dark')} /></div></> : null}
              {section === 'privacy' ? <><h2 className="mt-2 font-display text-3xl font-bold">Privacy &amp; data</h2><div className="mt-6 space-y-3"><SettingsToggle label="Peer-to-peer conversation sync" caption="Synchronize only between devices authorized by your user-owned discovery keys." checked={settings.sync} onChange={() => toggle('sync')} /><SettingsToggle label="Anonymous diagnostics" caption="Share crash and performance signals; never conversation content." checked={settings.diagnostics} onChange={() => toggle('diagnostics')} />{devices.map(([name, role, state]) => <div key={name} className="flex min-h-20 items-center justify-between rounded-xl bg-[color:var(--color-surface)] p-4"><div><p className="font-semibold">{name}</p><p className="text-sm text-[color:var(--color-fg-sub)]">{role}</p></div><span className="text-xs text-[color:var(--color-fg-faint)]">{state}</span></div>)}</div></> : null}
              {section === 'local-models' ? <><h2 className="mt-2 font-display text-3xl font-bold">Local models</h2><p className="mt-2 text-[color:var(--color-fg-sub)]">Models supplied by the KnowMe container can run directly or participate in UAR-controlled inference through the local provider trait.</p><div className="mt-6 space-y-3">{localModels.models.filter((model) => model.id === 'local').map((model) => <div key={model.id} className="flex min-h-24 items-center justify-between gap-4 rounded-xl bg-[color:var(--color-surface)] p-4"><div><p className="font-semibold">{model.name}</p><div className="mt-2 flex flex-wrap gap-2">{model.tags.map((tag) => <span key={tag} className="rounded-full bg-[color:var(--color-muted)] px-2 py-1 text-xs">{tag}</span>)}</div></div><Button onClick={() => void localModels.switchLane('local')}>{model.action}</Button></div>)}</div></> : null}
              {section === 'agents' ? <AgentsPanel /> : null}
              {section === 'skills' ? <SkillsPanel /> : null}
              {section === 'knowledge' ? <KnowledgePanel /> : null}
              {section === 'memory' ? <MemoryPanel /> : null}
              {section === 'connections' ? <ConnectionsPanel /> : null}
              {section === 'plugins' ? <PluginsPanel /> : null}
              {section === 'system' ? <SettingsRegistryPanel title="System & inference settings" caption="Every registered UAR setting is searchable here, including protocol ports, context strategy, skill selection, persistence, governance and model defaults." /> : null}
            </div>
          </main>
        )}
      </div>
    </div>
  )
}
