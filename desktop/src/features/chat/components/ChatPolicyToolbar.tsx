// TJ-ARCH-MOB-001 compliant — conversation controls call hooks; UAR owns policy.
import { Bot, BrainCircuit, Check, Database, Layers3, Plug, ShieldCheck, Sparkles, Wrench } from 'lucide-react'
import type { ResourceSelection, RunPolicy } from '@prometheus-ags/tauri-plugin-gen-ui'
import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import { Popover, PopoverContent, PopoverHeader, PopoverTitle, PopoverTrigger } from '@/components/ui/popover'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { useChat } from '../hooks/useChat'
import { useChatControl } from '../hooks/useChatControl'
import { useLane } from '../hooks/useLane'
import type { ResourceOption } from '../stores/chatControlStore'

function ResourceControl({ label, icon: Icon, options, value, onChange }: { label: string; icon: typeof Sparkles; options: ResourceOption[]; value: ResourceSelection; onChange: (selection: ResourceSelection) => void }) {
  const summary = value.mode === 'selected' ? `${value.ids.length} selected` : value.mode
  function choose(id: string, checked: boolean) {
    const ids = checked ? [...new Set([...value.ids, id])].sort() : value.ids.filter((item) => item !== id)
    onChange({ mode: 'selected', ids, denied_ids: value.denied_ids })
  }
  return (
    <Popover>
      <PopoverTrigger render={<Button variant="ghost" className="min-h-11 rounded-xl px-3 text-xs text-[color:var(--color-fg-sub)]" />}>
        <Icon aria-hidden className="size-4" /><span>{label}</span><span className="rounded-full bg-[color:var(--color-muted)] px-2 py-0.5 text-[10px]">{summary}</span>
      </PopoverTrigger>
      <PopoverContent align="start" className="max-h-[26rem] w-80 overflow-y-auto border-0 bg-[color:var(--color-card)] shadow-none ring-0">
        <PopoverHeader><PopoverTitle>{label}</PopoverTitle><p className="text-xs text-[color:var(--color-fg-sub)]">Conversation scope · UAR resolves this against global and agent limits.</p></PopoverHeader>
        <div className="grid grid-cols-4 gap-1 rounded-xl bg-[color:var(--color-surface)] p-1">
          {(['auto', 'all', 'none', 'selected'] as const).map((mode) => <button key={mode} type="button" onClick={() => onChange({ ...value, mode, ids: mode === 'selected' ? value.ids : [] })} className={value.mode === mode ? 'min-h-9 rounded-lg bg-[color:var(--color-ember)] px-2 text-[11px] font-bold text-white' : 'min-h-9 rounded-lg px-2 text-[11px] text-[color:var(--color-fg-sub)] hover:bg-[color:var(--color-card-hov)]'}>{mode}</button>)}
        </div>
        {options.length ? <div className="mt-1 space-y-1">{options.map((option) => <label key={option.id} className="flex min-h-11 cursor-pointer items-center gap-3 rounded-lg px-2 hover:bg-[color:var(--color-card-hov)]"><Checkbox className="border-0 bg-[color:var(--color-muted)]" checked={value.mode === 'selected' && value.ids.includes(option.id)} onCheckedChange={(checked) => choose(option.id, checked === true)} /><span className="min-w-0"><span className="block truncate text-sm font-medium">{option.label}</span>{option.description ? <span className="block truncate text-xs text-[color:var(--color-fg-faint)]">{option.description}</span> : null}</span></label>)}</div> : <p className="p-3 text-xs text-[color:var(--color-fg-faint)]">No enabled resources are advertised by UAR.</p>}
      </PopoverContent>
    </Popover>
  )
}

export function ChatPolicyToolbar() {
  const { activeConversationId } = useChat()
  const control = useChatControl(activeConversationId)
  const { switchLane } = useLane()
  const mode = control.policy.chat_mode ?? 'uar'
  const contextStrategy = control.policy.context_strategy && typeof control.policy.context_strategy === 'object'
    ? String((control.policy.context_strategy as Record<string, unknown>).type ?? 'inherit')
    : 'inherit'

  const contextValue = (type: string): unknown | null => {
    switch (type) {
      case 'none': return { type: 'none' }
      case 'sliding_window': return { type, max_messages: 20 }
      case 'summarize': return { type, threshold: 12, summary_max_tokens: 500, model: null }
      case 'truncate_middle': return { type, keep_first: 4, keep_last: 16 }
      case 'hierarchical': return { type, short_term_turns: 5, mid_term_summary_tokens: 2_000, long_term_facts_tokens: 500 }
      default: return null
    }
  }

  async function setMode(next: 'local' | 'uar' | 'agent') {
    if (next === 'local') await switchLane('local')
    else await switchLane('uar')
    await control.patchPolicy({
      chat_mode: next,
      agent_id: next === 'agent' ? (control.policy.agent_id && !['default-agent', 'orchestrator-agent'].includes(control.policy.agent_id) ? control.policy.agent_id : control.agents.find((agent) => !agent.protected)?.id ?? null) : next === 'uar' ? 'orchestrator-agent' : null,
    })
  }

  return (
    <div className="flex min-h-12 shrink-0 items-center gap-2 overflow-x-auto bg-[color:var(--color-bg-2)] px-4 py-2" aria-label="Conversation controls">
      <div className="flex rounded-xl bg-[color:var(--color-surface)] p-1" role="group" aria-label="Chat mode">
        {([['uar', 'Orchestrator', BrainCircuit], ['agent', 'Agent', Bot], ['local', 'Local', Database]] as const).map(([id, label, Icon]) => <button key={id} type="button" onClick={() => void setMode(id)} aria-pressed={mode === id} className={mode === id ? 'flex min-h-9 items-center gap-2 rounded-lg bg-[color:var(--color-ember)] px-3 text-xs font-bold text-white' : 'flex min-h-9 items-center gap-2 rounded-lg px-3 text-xs text-[color:var(--color-fg-sub)] hover:bg-[color:var(--color-card-hov)]'}><Icon aria-hidden className="size-4" />{label}</button>)}
      </div>
      {mode === 'agent' ? <Select value={control.policy.agent_id ?? ''} onValueChange={(agentId) => void control.patchPolicy({ agent_id: agentId ?? null })}><SelectTrigger className="h-11 w-52 border-0 bg-[color:var(--color-surface)] shadow-none"><SelectValue placeholder="Choose agent" /></SelectTrigger><SelectContent className="border-0 shadow-none ring-0">{control.agents.map((agent) => <SelectItem key={agent.id} value={agent.id}>{agent.label}{agent.protected ? ' · built-in' : ''}</SelectItem>)}</SelectContent></Select> : null}
      <ResourceControl label="Skills" icon={Sparkles} options={control.skills} value={control.policy.skills} onChange={(value) => void control.setSelection('skills', value)} />
      <ResourceControl label="Tools" icon={Wrench} options={control.tools} value={control.policy.tools} onChange={(value) => void control.setSelection('tools', value)} />
      <ResourceControl label="MCP" icon={Plug} options={control.mcpServers} value={control.policy.mcp_servers} onChange={(value) => void control.setSelection('mcp_servers', value)} />
      <ResourceControl label="Knowledge" icon={Database} options={control.knowledgeBases} value={control.policy.knowledge_bases} onChange={(value) => void control.setSelection('knowledge_bases', value)} />
      <Button variant="ghost" className="min-h-11 rounded-xl px-3 text-xs text-[color:var(--color-fg-sub)]" onClick={() => void control.patchPolicy({ memory_enabled: control.policy.memory_enabled === false ? true : false })}><BrainCircuit aria-hidden className="size-4" />Memory {control.policy.memory_enabled === false ? 'off' : 'on'}{control.policy.memory_enabled !== null ? <Check aria-hidden className="size-3" /> : null}</Button>
      <Select value={contextStrategy} onValueChange={(value) => void control.patchPolicy({ context_strategy: contextValue(value ?? 'inherit') })}>
        <SelectTrigger aria-label="Conversation context strategy" className="h-11 w-44 border-0 bg-[color:var(--color-surface)] text-xs shadow-none"><Layers3 aria-hidden className="size-4" /><SelectValue /></SelectTrigger>
        <SelectContent className="border-0 shadow-none ring-0">
          <SelectItem value="inherit">Context · inherit</SelectItem><SelectItem value="none">Context · full history</SelectItem><SelectItem value="sliding_window">Context · sliding</SelectItem><SelectItem value="summarize">Context · summarize</SelectItem><SelectItem value="truncate_middle">Context · first + latest</SelectItem><SelectItem value="hierarchical">Context · hierarchical</SelectItem>
        </SelectContent>
      </Select>
      <Select value={control.policy.tool_approval} onValueChange={(value) => void control.patchPolicy({ tool_approval: (value ?? 'inherit') as RunPolicy['tool_approval'] })}>
        <SelectTrigger aria-label="Conversation tool approval" className="h-11 w-40 border-0 bg-[color:var(--color-surface)] text-xs shadow-none"><ShieldCheck aria-hidden className="size-4" /><SelectValue /></SelectTrigger>
        <SelectContent className="border-0 shadow-none ring-0"><SelectItem value="inherit">Approval · inherit</SelectItem><SelectItem value="auto">Approval · auto</SelectItem><SelectItem value="ask">Approval · ask</SelectItem><SelectItem value="deny">Tools · deny</SelectItem></SelectContent>
      </Select>
      {control.saving ? <span role="status" className="whitespace-nowrap text-xs text-[color:var(--color-fg-faint)]">Saving policy…</span> : null}
      {control.error ? <span role="alert" className="whitespace-nowrap rounded-lg bg-destructive/15 px-3 py-2 text-xs text-destructive">{control.error}</span> : null}
    </div>
  )
}
