// TJ-ARCH-MOB-001 compliant — view-only components over the UAR admin hook.
import { useEffect, useMemo, useState } from 'react'
import { Bot, Database, Plug, Plus, Save, SlidersHorizontal, Sparkles, Trash2 } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Switch } from '@/components/ui/switch'
import { Textarea } from '@/components/ui/textarea'
import { useUarAdmin } from '../hooks/useUarAdmin'
import { ModelRoutePicker, type ModelRoute } from './ModelRoutePicker'

type JsonRecord = Record<string, unknown>

/** Read a `{ provider_id, model_id }` route from a policy draft, or null. */
function modelRoute(value: unknown): ModelRoute | null {
  const model = record(value)
  const providerId = text(model.provider_id)
  const modelId = text(model.model_id)
  return providerId && modelId ? { provider_id: providerId, model_id: modelId } : null
}

const fieldClass = 'min-h-11 border-0 bg-[color:var(--color-surface)] shadow-none focus-visible:ring-2'

function record(value: unknown): JsonRecord {
  return typeof value === 'object' && value !== null && !Array.isArray(value) ? value as JsonRecord : {}
}

function array(value: unknown, key?: string): JsonRecord[] {
  const candidate = key ? record(value)[key] : value
  return Array.isArray(candidate) ? candidate.map(record) : []
}

function text(value: unknown): string {
  return typeof value === 'string' ? value : ''
}

function resourceError(value: unknown): string | null {
  return text(record(value).error) || null
}

function PanelHeader({ icon: Icon, title, caption }: { icon: typeof Bot; title: string; caption: string }) {
  return <div className="flex items-start gap-3"><span className="grid size-10 shrink-0 place-items-center rounded-xl bg-[color:var(--color-card)]"><Icon aria-hidden className="size-5 text-[color:var(--color-ember)]" /></span><div><h2 className="font-display text-2xl font-bold">{title}</h2><p className="mt-1 text-sm leading-6 text-[color:var(--color-fg-sub)]">{caption}</p></div></div>
}

function ErrorLine({ value }: { value: string | null }) {
  return value ? <p role="alert" className="mt-4 rounded-xl bg-destructive/15 p-3 text-sm text-destructive">{value}</p> : null
}

export function AgentsPanel() {
  const admin = useUarAdmin()
  const agents = array(admin.snapshot?.agents, 'runtime_agents')
  const [selectedId, setSelectedId] = useState('')
  const selected = agents.find((agent) => text(agent.id) === selectedId) ?? agents[0]
  const [newMode, setNewMode] = useState(false)
  const [title, setTitle] = useState('')
  const [description, setDescription] = useState('')
  const [systemPrompt, setSystemPrompt] = useState('')
  const [agentPolicy, setAgentPolicy] = useState<JsonRecord>(defaultGlobalPolicy())

  function policyFor(agent: JsonRecord): JsonRecord {
    const extension = record(record(agent.extensions)['uar.run_policy'])
    if (Object.keys(extension).length) return { ...defaultGlobalPolicy(), ...extension }
    const policy = record(agent.policy); const tools = record(policy.tools); const skills = record(policy.skills); const memory = record(agent.memory); const kb = record(memory.kb); const provider = record(record(policy.provider).default)
    const allow = ids(tools.allow); const preferred = ids(skills.prefer); const knowledge = ids(kb.knowledge_bases)
    return {
      ...defaultGlobalPolicy(),
      model: text(provider.provider) && text(provider.model) ? { provider_id: text(provider.provider), model_id: text(provider.model) } : null,
      skills: preferred.length ? { mode: 'selected', ids: preferred, denied_ids: [] } : { mode: 'auto', ids: [], denied_ids: [] },
      tools: allow.includes('*') ? { mode: 'all', ids: [], denied_ids: ids(tools.deny) } : allow.length ? { mode: 'selected', ids: allow, denied_ids: ids(tools.deny) } : { mode: 'auto', ids: [], denied_ids: ids(tools.deny) },
      knowledge_bases: kb.enabled === false ? { mode: 'none', ids: [], denied_ids: [] } : knowledge.length ? { mode: 'selected', ids: knowledge, denied_ids: [] } : { mode: 'auto', ids: [], denied_ids: [] },
      memory_enabled: record(memory.conversation).enabled !== false,
    }
  }

  function select(agent: JsonRecord) {
    setNewMode(false)
    setSelectedId(text(agent.id))
    setTitle(text(record(agent.metadata).title))
    setDescription(text(record(agent.metadata).description))
    setSystemPrompt(text(record(agent.prompt).system))
    setAgentPolicy(policyFor(agent))
  }

  function startNew() {
    setNewMode(true); setSelectedId(''); setTitle(''); setDescription(''); setSystemPrompt(''); setAgentPolicy(defaultGlobalPolicy())
  }

  useEffect(() => {
    if (selected && !newMode) setAgentPolicy(policyFor(selected))
  }, [selected?.id, newMode])

  async function save() {
    const id = newMode ? '' : text(selected?.id)
    const savedTitle = newMode ? title : (title || text(record(selected?.metadata).title))
    const savedDescription = newMode ? description : (description || text(record(selected?.metadata).description))
    const savedPrompt = newMode ? systemPrompt : (systemPrompt || text(record(selected?.prompt).system))
    const base = newMode ? {
      version: '1.0.0', kind: 'agent', id: '',
      metadata: { title: savedTitle, description: savedDescription, tags: [], author: null, icon: null },
      runtime: { entry: 'default', protocols: {} },
      policy: { provider: { default: { provider: '', model: '' }, fallbacks: [] }, tools: { allow: ['*'], deny: [], max_concurrent: 1, execution_mode: 'direct' }, skills: { prefer: [], max_active: 3 } },
      schemas: { inputs: null, outputs: null, state: null }, prompt: { system: savedPrompt, instructions: [] },
      memory: { conversation: { enabled: true }, kb: { enabled: true, knowledge_bases: [], citation_required: false } },
      tools: { bundles: [] }, ui: { forms: { enabled: false }, artifacts: { enabled: true, preferred_types: [] } }, extensions: { 'uar.run_policy': agentPolicy },
    } : {
      ...selected,
      id,
      metadata: { ...record(selected?.metadata), title: savedTitle, description: savedDescription },
      prompt: { ...record(selected?.prompt), system: savedPrompt },
      extensions: { ...record(selected?.extensions), 'uar.run_policy': agentPolicy },
    }
    await admin.saveAgent(base)
    setNewMode(false)
  }

  const protectedAgent = ['default-agent', 'orchestrator-agent'].includes(text(selected?.id))
  const displayedTitle = newMode ? title : (title || text(record(selected?.metadata).title))
  const displayedDescription = newMode ? description : (description || text(record(selected?.metadata).description))
  const displayedPrompt = newMode ? systemPrompt : (systemPrompt || text(record(selected?.prompt).system))
  const toolsRoot = record(admin.snapshot?.tools)
  const choices: Record<string, ResourceChoice[]> = {
    skills: array(admin.snapshot?.skills).map((item) => ({ id: text(item.skill_id) || text(item.id), label: text(item.title) || text(item.name) || text(item.skill_id) })).filter((item) => item.id),
    tools: array(toolsRoot.tools).map((item) => ({ id: text(item.namespaced_name) || text(item.name), label: text(item.name) || text(item.namespaced_name) })).filter((item) => item.id),
    mcp_servers: ids(toolsRoot.mcp_servers).map((id) => ({ id, label: id })),
    knowledge_bases: array(admin.snapshot?.knowledgeBases).map((item) => ({ id: text(item.id), label: text(item.name) || text(item.id) })).filter((item) => item.id),
  }
  return <div>
    <PanelHeader icon={Bot} title="Agents" caption="Agent definitions and execution metadata are stored and enforced by UAR. The Orchestrator and Default agents are protected." />
    <div className="mt-6 grid gap-4 md:grid-cols-[16rem_minmax(0,1fr)]">
      <div className="space-y-2"><Button variant="secondary" className="w-full justify-start" onClick={startNew}><Plus aria-hidden /> Add agent</Button>{agents.map((agent) => <button key={text(agent.id)} type="button" onClick={() => select(agent)} className="min-h-12 w-full rounded-xl bg-[color:var(--color-surface)] px-3 text-left"><span className="block truncate font-semibold">{text(record(agent.metadata).title) || text(agent.id)}</span><span className="block truncate text-xs text-[color:var(--color-fg-faint)]">{text(agent.id)}</span></button>)}</div>
      <div className="space-y-4 rounded-xl bg-[color:var(--color-bg-2)] p-5">
        <div><Label htmlFor="agent-title">Name</Label><Input id="agent-title" className={fieldClass} value={displayedTitle} onChange={(event) => setTitle(event.target.value)} /></div>
        <div><Label htmlFor="agent-description">Description</Label><Input id="agent-description" className={fieldClass} value={displayedDescription} onChange={(event) => setDescription(event.target.value)} /></div>
        <div><Label htmlFor="agent-prompt">System prompt</Label><Textarea id="agent-prompt" className={`${fieldClass} min-h-40`} value={displayedPrompt} onChange={(event) => setSystemPrompt(event.target.value)} /></div>
        <div className="space-y-2"><p className="text-sm font-semibold">Model</p><p className="text-xs leading-5 text-[color:var(--color-fg-faint)]">Overrides the global default for this agent. Leave on global default to inherit.</p><ModelRoutePicker value={modelRoute(agentPolicy.model)} onChange={(route) => setAgentPolicy((current) => ({ ...current, model: route }))} inheritLabel="Global default" disabled={admin.saving} /></div>
        <div className="space-y-3"><p className="text-sm font-semibold">Agent execution policy</p>{(['skills', 'tools', 'mcp_servers', 'knowledge_bases'] as const).map((key) => <PolicyResourceEditor key={key} label={{ skills: 'Skills', tools: 'Tools', mcp_servers: 'MCP servers', knowledge_bases: 'Knowledge bases' }[key]} value={record(agentPolicy[key])} options={choices[key]} onChange={(value) => setAgentPolicy((current) => ({ ...current, [key]: value }))} />)}</div>
        <div className="grid gap-3 sm:grid-cols-2"><label className="rounded-xl bg-[color:var(--color-surface)] p-4 text-sm font-semibold">Memory<select className={`${fieldClass} mt-2 w-full rounded-xl px-3 font-normal`} value={agentPolicy.memory_enabled === false ? 'off' : 'on'} onChange={(event) => setAgentPolicy((current) => ({ ...current, memory_enabled: event.target.value === 'on' }))}><option value="on">Enabled</option><option value="off">Disabled</option></select></label><label className="rounded-xl bg-[color:var(--color-surface)] p-4 text-sm font-semibold">Tool approval<select className={`${fieldClass} mt-2 w-full rounded-xl px-3 font-normal`} value={text(agentPolicy.tool_approval) || 'inherit'} onChange={(event) => setAgentPolicy((current) => ({ ...current, tool_approval: event.target.value }))}><option value="inherit">Global default</option><option value="auto">UAR governance</option><option value="ask">Always ask</option><option value="deny">Deny tools</option></select></label></div>
        <div className="flex justify-between gap-2">{!newMode && !protectedAgent && selected ? <Button variant="ghost" onClick={() => void admin.deleteAgent(text(selected.id))}><Trash2 aria-hidden /> Delete</Button> : <span /> }<Button disabled={admin.saving || !displayedTitle.trim()} onClick={() => void save()}><Save aria-hidden /> Save agent</Button></div>
      </div>
    </div><ErrorLine value={admin.error || resourceError(admin.snapshot?.agents)} />
  </div>
}

export function SkillsPanel() {
  const admin = useUarAdmin()
  const skills = array(admin.snapshot?.skills)
  const settings = array(admin.snapshot?.settings)
  const configured = useMemo(() => new Map(settings.filter((setting) => text(setting.key).startsWith('skill_config.')).map((setting) => [text(setting.key), record(setting.data)])), [settings])
  return <div><PanelHeader icon={Sparkles} title="Skills" caption="UAR discovers, selects, activates and persists skills. Global switches here feed the same policy resolver used by agents and conversation controls." />
    <div className="mt-6 space-y-2">{skills.map((skill) => {
      const id = text(skill.skill_id); const shortId = id.split('::').at(-1) ?? id; const key = `skill_config.${shortId}`; const current = configured.get(key); const enabled = current ? current.enabled !== false : true
      return <div key={id} className="flex min-h-20 items-center justify-between gap-4 rounded-xl bg-[color:var(--color-surface)] p-4"><div className="min-w-0"><p className="font-semibold">{text(skill.title) || shortId}</p><p className="mt-1 line-clamp-2 text-sm text-[color:var(--color-fg-sub)]">{text(skill.description)}</p></div><Switch checked={enabled} onCheckedChange={(checked) => void admin.saveSetting(key, { ...(current ?? {}), enabled: checked, description: text(skill.description) })} aria-label={`Enable ${text(skill.title) || shortId}`} /></div>
    })}</div><ErrorLine value={admin.error || resourceError(admin.snapshot?.skills)} /></div>
}

const inheritSelection = () => ({ mode: 'inherit', ids: [] as string[], denied_ids: [] as string[] })

function defaultGlobalPolicy(): JsonRecord {
  return {
    version: 1, chat_mode: null, agent_id: null, model: null,
    skills: inheritSelection(), tools: inheritSelection(), mcp_servers: inheritSelection(), knowledge_bases: inheritSelection(),
    memory_enabled: null, context_strategy: null, tool_approval: 'inherit',
  }
}

function ids(value: unknown): string[] {
  return Array.isArray(value) ? value.map(String) : []
}

function PolicyResourceEditor({ label, value, options, onChange }: { label: string; value: JsonRecord; options: ResourceChoice[]; onChange: (next: JsonRecord) => void }) {
  const mode = text(value.mode) || 'inherit'
  const selected = ids(value.ids)
  return <div className="rounded-xl bg-[color:var(--color-surface)] p-4"><div className="flex items-center justify-between gap-3"><p className="font-semibold">{label}</p><select aria-label={`${label} global mode`} className={`${fieldClass} rounded-xl px-3 text-sm`} value={mode} onChange={(event) => onChange({ ...value, mode: event.target.value, ids: event.target.value === 'selected' ? selected : [] })}><option value="inherit">Inherit runtime defaults</option><option value="auto">UAR selects</option><option value="all">Allow all</option><option value="none">Disable all</option><option value="selected">Select resources</option></select></div>{mode === 'selected' ? <div className="mt-3 grid gap-2 sm:grid-cols-2">{options.map((option) => <label key={option.id} className="flex min-h-11 items-center gap-3 rounded-lg bg-[color:var(--color-card)] px-3 text-sm"><input type="checkbox" checked={selected.includes(option.id)} onChange={(event) => onChange({ ...value, mode: 'selected', ids: event.target.checked ? [...new Set([...selected, option.id])].sort() : selected.filter((id) => id !== option.id) })} /><span className="truncate">{option.label}</span></label>)}</div> : null}</div>
}

interface ResourceChoice { id: string; label: string }

export function GlobalRunPolicyPanel() {
  const admin = useUarAdmin()
  const settings = array(admin.snapshot?.settings)
  const setting = settings.find((item) => text(item.key) === 'run_policy.global')
  const stored = { ...defaultGlobalPolicy(), ...record(setting?.data) }
  const [draft, setDraft] = useState<JsonRecord>(stored)
  useEffect(() => {
    if (setting) setDraft({ ...defaultGlobalPolicy(), ...record(setting.data) })
  }, [setting?.key, JSON.stringify(setting?.data)])
  const toolsRoot = record(admin.snapshot?.tools)
  const choices: Record<string, ResourceChoice[]> = {
    skills: array(admin.snapshot?.skills).map((item) => ({ id: text(item.skill_id) || text(item.id), label: text(item.title) || text(item.name) || text(item.skill_id) })).filter((item) => item.id),
    tools: array(toolsRoot.tools).map((item) => ({ id: text(item.namespaced_name) || text(item.name), label: text(item.name) || text(item.namespaced_name) })).filter((item) => item.id),
    mcp_servers: ids(toolsRoot.mcp_servers).map((id) => ({ id, label: id })),
    knowledge_bases: array(admin.snapshot?.knowledgeBases).map((item) => ({ id: text(item.id), label: text(item.name) || text(item.id) })).filter((item) => item.id),
  }
  const updateResource = (key: string, value: JsonRecord) => setDraft((current) => ({ ...current, [key]: value }))
  const contextType = text(record(draft.context_strategy).type) || 'inherit'
  const contextValue = (type: string): unknown => type === 'inherit' ? null : type === 'none' ? { type } : type === 'sliding_window' ? { type, max_messages: 20 } : type === 'summarize' ? { type, threshold: 12, summary_max_tokens: 500, model: null } : type === 'truncate_middle' ? { type, keep_first: 4, keep_last: 16 } : { type, short_term_turns: 5, mid_term_summary_tokens: 2_000, long_term_facts_tokens: 500 }
  return <div className="mt-8"><PanelHeader icon={SlidersHorizontal} title="Global chat policy" caption="This policy is persisted by UAR and is the first scope applied to every agent and conversation. Lower scopes may narrow it but cannot re-enable globally denied resources." />
    <div className="mt-5 rounded-xl bg-[color:var(--color-surface)] p-4"><p className="text-sm font-semibold">Default model</p><p className="mt-1 text-sm leading-6 text-[color:var(--color-fg-sub)]">The model every conversation uses unless an agent or conversation overrides it. Leave on runtime default to use the on-device model.</p><div className="mt-3"><ModelRoutePicker value={modelRoute(draft.model)} onChange={(route) => setDraft((current) => ({ ...current, model: route }))} inheritLabel="Runtime default" disabled={admin.saving} /></div></div>
    <div className="mt-5 grid gap-3">{(['skills', 'tools', 'mcp_servers', 'knowledge_bases'] as const).map((key) => <PolicyResourceEditor key={key} label={{ skills: 'Skills', tools: 'Tools', mcp_servers: 'MCP servers', knowledge_bases: 'Knowledge bases' }[key]} value={record(draft[key])} options={choices[key]} onChange={(value) => updateResource(key, value)} />)}</div>
    <div className="mt-3 grid gap-3 sm:grid-cols-3">
      <label className="rounded-xl bg-[color:var(--color-surface)] p-4 text-sm font-semibold">Context strategy<select className={`${fieldClass} mt-2 w-full rounded-xl px-3 font-normal`} value={contextType} onChange={(event) => setDraft((current) => ({ ...current, context_strategy: contextValue(event.target.value) }))}><option value="inherit">Runtime default</option><option value="none">Full history</option><option value="sliding_window">Sliding window</option><option value="summarize">Summarize</option><option value="truncate_middle">First + latest</option><option value="hierarchical">Hierarchical</option></select></label>
      <label className="rounded-xl bg-[color:var(--color-surface)] p-4 text-sm font-semibold">Tool approval<select className={`${fieldClass} mt-2 w-full rounded-xl px-3 font-normal`} value={text(draft.tool_approval) || 'inherit'} onChange={(event) => setDraft((current) => ({ ...current, tool_approval: event.target.value }))}><option value="inherit">Runtime default</option><option value="auto">UAR governance</option><option value="ask">Always ask</option><option value="deny">Deny tools</option></select></label>
      <label className="rounded-xl bg-[color:var(--color-surface)] p-4 text-sm font-semibold">Memory<select className={`${fieldClass} mt-2 w-full rounded-xl px-3 font-normal`} value={draft.memory_enabled === true ? 'on' : draft.memory_enabled === false ? 'off' : 'inherit'} onChange={(event) => setDraft((current) => ({ ...current, memory_enabled: event.target.value === 'inherit' ? null : event.target.value === 'on' }))}><option value="inherit">Runtime default</option><option value="on">Enabled</option><option value="off">Disabled</option></select></label>
    </div>
    <div className="mt-4 flex justify-end"><Button disabled={admin.saving || !setting} onClick={() => void admin.saveSetting('run_policy.global', draft)}><Save aria-hidden />{admin.saving ? 'Saving…' : 'Save global policy'}</Button></div><ErrorLine value={admin.error || (!setting ? 'UAR has not registered the global run-policy setting yet. Restart the updated UAR service.' : null)} />
  </div>
}

export function KnowledgePanel() {
  const admin = useUarAdmin(); const bases = array(admin.snapshot?.knowledgeBases); const [name, setName] = useState(''); const [description, setDescription] = useState('')
  return <div><PanelHeader icon={Database} title="Knowledge bases" caption="UAR owns indexing, retrieval, citations and provenance. Conversation controls can inherit, select or deny any configured base." />
    <div className="mt-6 rounded-xl bg-[color:var(--color-bg-2)] p-4"><div className="grid gap-3 sm:grid-cols-2"><Input aria-label="Knowledge base name" placeholder="Knowledge base name" className={fieldClass} value={name} onChange={(event) => setName(event.target.value)} /><Input aria-label="Knowledge base description" placeholder="Description" className={fieldClass} value={description} onChange={(event) => setDescription(event.target.value)} /></div><Button className="mt-3" disabled={!name.trim() || admin.saving} onClick={() => void admin.createKnowledgeBase({ name: name.trim(), description: description.trim() || null }).then((ok) => { if (ok) { setName(''); setDescription('') } })}><Plus aria-hidden /> Create knowledge base</Button></div>
    <div className="mt-4 space-y-2">{bases.map((base) => <div key={text(base.id)} className="flex min-h-20 items-center justify-between rounded-xl bg-[color:var(--color-surface)] p-4"><div><p className="font-semibold">{text(base.name)}</p><p className="text-sm text-[color:var(--color-fg-sub)]">{text(base.description) || 'No description'} · {Number(base.document_count ?? 0)} documents</p></div><Button variant="ghost" size="icon" aria-label={`Delete ${text(base.name)}`} disabled={text(base.name) === 'default'} onClick={() => void admin.deleteKnowledgeBase(text(base.id))}><Trash2 aria-hidden /></Button></div>)}</div><ErrorLine value={admin.error || resourceError(admin.snapshot?.knowledgeBases)} /></div>
}

export function MemoryPanel() {
  const admin = useUarAdmin(); const memories = array(admin.snapshot?.memories, 'items'); const settings = array(admin.snapshot?.settings); const setting = settings.find((item) => item.key === 'memory.enabled'); const enabled = setting?.data !== false
  return <div><PanelHeader icon={Database} title="Memories" caption="Governed user and conversation memories remain in UAR storage. Reasoning is never promoted into future context by default." />
    <div className="mt-6 flex min-h-20 items-center justify-between rounded-xl bg-[color:var(--color-bg-2)] p-4"><div><p className="font-semibold">Memory retrieval</p><p className="text-sm text-[color:var(--color-fg-sub)]">Apply governed memories during context assembly.</p></div><Switch checked={enabled} onCheckedChange={(checked) => void admin.saveSetting('memory.enabled', checked)} aria-label="Enable memory retrieval" /></div>
    <div className="mt-4 space-y-2">{memories.map((memory) => <div key={text(memory.id)} className="flex items-start justify-between gap-4 rounded-xl bg-[color:var(--color-surface)] p-4"><div><p className="line-clamp-3 text-sm">{text(memory.content) || text(memory.text)}</p><p className="mt-1 text-xs text-[color:var(--color-fg-faint)]">{text(memory.scope)}</p></div><Button variant="ghost" size="icon" aria-label="Delete memory" onClick={() => void admin.deleteMemory(text(memory.id))}><Trash2 aria-hidden /></Button></div>)}</div><ErrorLine value={admin.error || resourceError(admin.snapshot?.memories)} /></div>
}

export function ConnectionsPanel() {
  const admin = useUarAdmin(); const servers = array(admin.snapshot?.mcpServers, 'servers'); const [name, setName] = useState(''); const [transport, setTransport] = useState<'remote_http' | 'stdio'>('remote_http'); const [endpoint, setEndpoint] = useState('')
  async function save() { const payload = transport === 'stdio' ? { name, enabled: true, transport, command: endpoint, args: [], env: {}, sandboxed: true } : { name, enabled: true, transport, url: endpoint, env: {} }; if (await admin.saveMcpServer(name, payload)) { setName(''); setEndpoint('') } }
  return <div><PanelHeader icon={Plug} title="MCP & external access" caption="UAR owns MCP connections, tool discovery and execution policy. Credentials are write-only and are never returned to clients." />
    <div className="mt-6 rounded-xl bg-[color:var(--color-bg-2)] p-4"><div className="grid gap-3 sm:grid-cols-[1fr_10rem_2fr]"><Input aria-label="MCP server name" placeholder="Server name" className={fieldClass} value={name} onChange={(event) => setName(event.target.value)} /><select aria-label="MCP transport" className={`${fieldClass} rounded-xl px-3`} value={transport} onChange={(event) => setTransport(event.target.value as typeof transport)}><option value="remote_http">Remote HTTP</option><option value="stdio">Local stdio</option></select><Input aria-label="MCP endpoint" placeholder={transport === 'stdio' ? 'Command' : 'https://…'} className={fieldClass} value={endpoint} onChange={(event) => setEndpoint(event.target.value)} /></div><Button className="mt-3" disabled={!name.trim() || !endpoint.trim() || admin.saving} onClick={() => void save()}><Plus aria-hidden /> Add MCP server</Button></div>
    <div className="mt-4 space-y-2">{servers.map((server) => <div key={text(server.name)} className="flex min-h-20 items-center justify-between gap-4 rounded-xl bg-[color:var(--color-surface)] p-4"><div><p className="font-semibold">{text(server.name)}</p><p className="text-sm text-[color:var(--color-fg-sub)]">{text(server.transport)} · {server.connected ? 'connected' : 'not connected'} · {array(server.tools).length} tools</p></div><div className="flex items-center gap-2"><span className={server.connected ? 'text-xs text-emerald-500' : 'text-xs text-[color:var(--color-fg-faint)]'}>{server.connected ? 'Live' : 'Offline'}</span><Button variant="ghost" size="icon" aria-label={`Delete ${text(server.name)}`} onClick={() => void admin.deleteMcpServer(text(server.name))}><Trash2 aria-hidden /></Button></div></div>)}</div><ErrorLine value={admin.error || resourceError(admin.snapshot?.mcpServers)} /></div>
}

export function SettingsRegistryPanel({ title, caption, prefixes }: { title: string; caption: string; prefixes?: string[] }) {
  const admin = useUarAdmin(); const all = array(admin.snapshot?.settings); const settings = prefixes ? all.filter((item) => prefixes.some((prefix) => text(item.key).startsWith(prefix))) : all; const [query, setQuery] = useState(''); const filtered = settings.filter((item) => `${text(item.name)} ${text(item.key)}`.toLowerCase().includes(query.toLowerCase()))
  return <div><PanelHeader icon={Save} title={title} caption={caption} /><Input aria-label="Search UAR settings" placeholder="Search settings…" className={`${fieldClass} mt-6`} value={query} onChange={(event) => setQuery(event.target.value)} />
    <div className="mt-4 space-y-2">{filtered.slice(0, 100).map((setting) => <SettingEditor key={text(setting.key)} setting={setting} save={(value) => admin.saveSetting(text(setting.key), value)} />)}</div><ErrorLine value={admin.error || resourceError(admin.snapshot?.settings)} /></div>
}

function SettingEditor({ setting, save }: { setting: JsonRecord; save: (value: unknown) => Promise<boolean> }) {
  const initial = setting.data; const [value, setValue] = useState(typeof initial === 'string' ? initial : JSON.stringify(initial)); const isBool = typeof initial === 'boolean'
  return <div className="rounded-xl bg-[color:var(--color-surface)] p-4"><div className="flex items-start justify-between gap-4"><div><p className="font-semibold">{text(setting.name) || text(setting.key)}</p><p className="font-mono text-xs text-[color:var(--color-fg-faint)]">{text(setting.key)}</p></div>{isBool ? <Switch checked={Boolean(initial)} onCheckedChange={(checked) => void save(checked)} aria-label={text(setting.name)} /> : null}</div>{!isBool ? <div className="mt-3 flex gap-2"><Input aria-label={text(setting.name)} className={fieldClass} value={value} onChange={(event) => setValue(event.target.value)} /><Button variant="secondary" onClick={() => { let parsed: unknown = value; try { parsed = JSON.parse(value) } catch { /* plain string */ } void save(parsed) }}>Save</Button></div> : null}</div>
}

export function PluginsPanel() {
  const admin = useUarAdmin(); const types = array(admin.snapshot?.settingTypes)
  return <div><PanelHeader icon={Plug} title="Plugins" caption="WASM components register metadata and JSON settings schemas in UAR. KnowMe renders those schemas here; operational plugin UI continues through A2UI." /><div className="mt-6 grid gap-3 sm:grid-cols-2">{types.map((type) => <div key={text(type.key)} className="rounded-xl bg-[color:var(--color-surface)] p-4"><p className="font-semibold">{text(type.name)}</p><p className="font-mono text-xs text-[color:var(--color-fg-faint)]">{text(type.key)}</p><p className="mt-2 text-xs text-[color:var(--color-fg-sub)]">Schema registered · {Object.keys(record(record(type.schema).properties)).length} fields</p></div>)}</div><ErrorLine value={admin.error || resourceError(admin.snapshot?.settingTypes)} /></div>
}
