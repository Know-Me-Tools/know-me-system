// TJ-ARCH-MOB-001 compliant — Zustand is the only UAR control-plane transport layer.
import { isTauri } from '@tauri-apps/api/core'
import {
  chatControlCatalog,
  conversationEffectiveConfig,
  conversationPolicyGet,
  conversationPolicySave,
} from '@prometheus-ags/tauri-plugin-gen-ui'
import type { ChatControlPlaneCatalog, ResourceSelection, RunPolicy } from '@prometheus-ags/tauri-plugin-gen-ui'
import { create } from 'zustand'

export interface ResourceOption { id: string; label: string; description?: string }
export interface AgentOption extends ResourceOption { protected: boolean }

interface NormalizedCatalog {
  agents: AgentOption[]
  skills: ResourceOption[]
  mcpServers: ResourceOption[]
  tools: ResourceOption[]
  knowledgeBases: ResourceOption[]
}

interface ChatControlState extends NormalizedCatalog {
  conversationId: string | null
  policy: RunPolicy
  effective: unknown
  loading: boolean
  saving: boolean
  error: string | null
  load: (conversationId: string) => Promise<void>
  patchPolicy: (patch: Partial<RunPolicy>) => Promise<void>
  setSelection: (key: 'skills' | 'tools' | 'mcp_servers' | 'knowledge_bases', selection: ResourceSelection) => Promise<void>
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
      return { id, label: String(agent.name ?? agent.title ?? id), description: typeof agent.description === 'string' ? agent.description : undefined, protected: id === 'default-agent' || id === 'orchestrator-agent' }
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
}))
