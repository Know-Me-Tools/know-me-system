// TJ-ARCH-MOB-001 compliant — Zustand is the only UAR control-plane transport layer.
import { isTauri } from '@tauri-apps/api/core'
import {
  chatControlCatalog,
  conversationEffectiveConfig,
  conversationPolicyGet,
  conversationPolicySave,
  laneAvailability,
} from '@prometheus-ags/tauri-plugin-gen-ui'
import type { ChatControlPlaneCatalog, ChatLane, LaneAvailability, ResourceSelection, RunPolicy } from '@prometheus-ags/tauri-plugin-gen-ui'
import { create } from 'zustand'
import { useLaneStore } from './laneStore'

export interface ResourceOption { id: string; label: string; description?: string }
export interface AgentOption extends ResourceOption { protected: boolean }

interface NormalizedCatalog {
  agents: AgentOption[]
  skills: ResourceOption[]
  mcpServers: ResourceOption[]
  tools: ResourceOption[]
  knowledgeBases: ResourceOption[]
}

/** How this conversation runs.
 *
 *  Deliberately the plugin's own `chat_mode` union rather than a wider local
 *  copy: Rust also understands `cloud`, but no picker on any surface offers it,
 *  and a mode the UI can never produce has no business in this type. */
export type ChatMode = NonNullable<RunPolicy['chat_mode']>

interface ChatControlState extends NormalizedCatalog {
  conversationId: string | null
  policy: RunPolicy
  effective: unknown
  loading: boolean
  saving: boolean
  error: string | null
  /** Reason a lane cannot be selected, keyed by lane. Absent means usable. */
  laneAvailability: Partial<Record<ChatLane, string>>
  load: (conversationId: string) => Promise<void>
  patchPolicy: (patch: Partial<RunPolicy>) => Promise<void>
  setSelection: (key: 'skills' | 'tools' | 'mcp_servers' | 'knowledge_bases', selection: ResourceSelection) => Promise<void>
  setMode: (mode: ChatMode, agentId?: string) => Promise<void>
}

/**
 * Map the picker's user-facing mode onto an execution lane.
 *
 * Mirrors `lane_for_chat_mode` in chat.rs: `agent` and `uar` both run through
 * the embedded UAR and differ only in which agent artifact the run resolves.
 */
export function laneForMode(mode: ChatMode): ChatLane {
  // `agent` and `uar` both run through the embedded UAR — they differ only in
  // which agent artifact the run resolves.
  return mode === 'local' ? 'local' : 'uar'
}

const inheritSelection = (): ResourceSelection => ({ mode: 'inherit', ids: [], denied_ids: [] })
const defaultPolicy = (): RunPolicy => ({
  version: 1,
  chat_mode: 'uar',
  agent_id: 'orchestrator-agent',
  model: null,
  skills: inheritSelection(),
  tools: inheritSelection(),
  mcp_servers: inheritSelection(),
  knowledge_bases: inheritSelection(),
  memory_enabled: null,
  context_strategy: null,
  tool_approval: 'ask',
})

function objects(value: unknown): Record<string, unknown>[] {
  return Array.isArray(value) ? value.filter((item): item is Record<string, unknown> => Boolean(item && typeof item === 'object')) : []
}

function normalize(raw: ChatControlPlaneCatalog): NormalizedCatalog {
  const agentsRoot = raw.agents && typeof raw.agents === 'object' ? raw.agents as Record<string, unknown> : {}
  const toolsRoot = raw.tools && typeof raw.tools === 'object' ? raw.tools as Record<string, unknown> : {}
  return {
    agents: objects(agentsRoot.runtime_agents).map((agent) => {
      const id = String(agent.id ?? '')
      // An AgentArtifact carries its display name at `metadata.title`; the flat
      // `name`/`title` keys only appear on some service-shaped payloads.
      // Checking metadata FIRST is why embedded surfaces show real names
      // instead of raw ids — this chain used to start at `name` and every
      // embedded agent rendered as its id.
      const metadata = agent.metadata && typeof agent.metadata === 'object' ? agent.metadata as Record<string, unknown> : {}
      const label = String(metadata.title ?? agent.name ?? agent.title ?? id)
      return { id, label, description: typeof agent.description === 'string' ? agent.description : undefined, protected: id === 'default-agent' || id === 'orchestrator-agent' }
    }).filter((agent) => agent.id),
    skills: objects(raw.skills).map((skill) => ({ id: String(skill.skill_id ?? skill.id ?? ''), label: String(skill.title ?? skill.name ?? skill.skill_id ?? skill.id ?? ''), description: typeof skill.description === 'string' ? skill.description : undefined })).filter((skill) => skill.id),
    mcpServers: Array.isArray(toolsRoot.mcp_servers) ? toolsRoot.mcp_servers.map((id) => ({ id: String(id), label: String(id) })) : [],
    tools: objects(toolsRoot.tools).map((tool) => ({ id: String(tool.namespaced_name ?? tool.name ?? ''), label: String(tool.name ?? tool.namespaced_name ?? ''), description: typeof tool.description === 'string' ? tool.description : undefined })).filter((tool) => tool.id),
    knowledgeBases: objects(raw.knowledgeBases).map((kb) => ({ id: String(kb.id ?? ''), label: String(kb.name ?? kb.id ?? ''), description: typeof kb.description === 'string' ? kb.description : undefined })).filter((kb) => kb.id),
  }
}

async function api<T>(path: string, init?: RequestInit): Promise<T> {
  const response = await fetch(path, { ...init, headers: { 'content-type': 'application/json', ...init?.headers } })
  if (!response.ok) throw new Error(`UAR control-plane request failed (${response.status})`)
  return response.status === 204 ? undefined as T : response.json() as Promise<T>
}

function message(error: unknown): string { return error instanceof Error ? error.message : String(error) }

export const useChatControlStore = create<ChatControlState>((set, get) => ({
  agents: [], skills: [], mcpServers: [], tools: [], knowledgeBases: [],
  conversationId: null,
  policy: defaultPolicy(),
  effective: null,
  loading: false,
  saving: false,
  error: null,
  laneAvailability: {},

  load: async (conversationId) => {
    set({ loading: true, error: null, conversationId })
    try {
      const [raw, policy] = isTauri()
        ? await Promise.all([chatControlCatalog(), conversationPolicyGet(conversationId)])
        : await Promise.all([
            Promise.all([
              api<unknown>('/api/uar/discovery/agents'),
              api<unknown>('/api/uar/discovery/skills'),
              api<unknown>('/api/uar/discovery/tools'),
              api<unknown>('/api/uar/knowledge-bases'),
            ]).then(([agents, skills, tools, knowledgeBases]) => ({ agents, skills, tools, knowledgeBases })),
            api<RunPolicy | null>(`/api/uar/conversations/${encodeURIComponent(conversationId)}/policy`).catch((error: unknown) => {
              if (error instanceof Error && error.message.includes('(404)')) return null
              throw error
            }),
          ])
      set({ ...normalize(raw), policy: policy ?? defaultPolicy() })
    } catch (error) {
      set({ error: message(error) })
    } finally { set({ loading: false }) }

    // Availability is loaded separately and never fails the catalog: an
    // unavailable probe must not blank the agent list. Absent data simply means
    // no row is marked blocked, which degrades to the pre-existing behaviour.
    if (!isTauri()) return
    try {
      const lanes: LaneAvailability[] = await laneAvailability()
      set({
        laneAvailability: Object.fromEntries(
          lanes
            .filter((entry) => !entry.available)
            .map((entry) => [entry.lane, entry.reason ?? 'This mode is unavailable right now.'] as const),
        ),
      })
    } catch {
      // A failed probe is not worth an error banner — the picker just shows
      // every row as selectable, and a genuinely blocked lane still reports its
      // reason through `setMode`.
      set({ laneAvailability: {} })
    }
  },

  patchPolicy: async (patch) => {
    const conversationId = get().conversationId
    if (!conversationId) return
    const policy = { ...get().policy, ...patch }
    set({ saving: true, error: null, policy })
    try {
      if (isTauri()) {
        await conversationPolicySave(conversationId, policy)
        const effective = await conversationEffectiveConfig(conversationId).catch(() => null)
        set({ effective })
      } else {
        await api(`/api/uar/conversations/${encodeURIComponent(conversationId)}/policy`, { method: 'PUT', body: JSON.stringify(policy) })
      }
    } catch (error) { set({ error: message(error) }) }
    finally { set({ saving: false }) }
  },

  setSelection: async (key, selection) => get().patchPolicy({ [key]: selection }),

  /**
   * Switch this conversation's mode.
   *
   * Rust rejects a lane whose runtime is unavailable, and that rejection has to
   * reach the user: switch the lane FIRST, and on failure record the reason and
   * leave the displayed selection alone. Moving the selection optimistically
   * would make the UI claim a mode the runtime refused — worse than an
   * unresponsive control, because the next send would silently route elsewhere.
   * Mirrors chat_control_notifier.dart:92-134, ordering included.
   */
  setMode: async (mode, agentId) => {
    if (get().saving) return
    // `uar` means the orchestrator specifically; `agent` carries the chosen id;
    // local and cloud have no agent at all.
    const agentIdForMode = mode === 'agent' ? agentId ?? null : mode === 'uar' ? 'orchestrator-agent' : null
    set({ saving: true, error: null })
    const switched = await useLaneStore.getState().switchLane(laneForMode(mode))
    if (!switched) {
      set({ error: useLaneStore.getState().error ?? 'That mode is unavailable right now.', saving: false })
      return
    }
    // The lane switch succeeded, so the selection is real. Reflect it now — a
    // brand-new conversation has no id yet, so `patchPolicy` cannot persist and
    // this is the only place the choice is recorded until the thread exists.
    set({ policy: { ...get().policy, chat_mode: mode, agent_id: agentIdForMode }, saving: false })
    await get().patchPolicy({ chat_mode: mode, agent_id: agentIdForMode })
  },
}))
