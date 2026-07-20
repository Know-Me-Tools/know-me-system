// TJ-ARCH-MOB-001 compliant — this store is the only React transport boundary
// for UAR-owned settings and administrative resources.
import { isTauri } from '@tauri-apps/api/core'
import {
  uarAdminSnapshot,
  uarAgentDelete,
  uarAgentSave,
  uarMcpServerDelete,
  uarMcpServerSave,
  uarMemoryDelete,
  uarKnowledgeBaseCreate,
  uarKnowledgeBaseDelete,
  uarSettingSave,
} from '@prometheus-ags/tauri-plugin-gen-ui'
import type { UarAdminSnapshot } from '@prometheus-ags/tauri-plugin-gen-ui'
import { create } from 'zustand'

const apiRoot = (import.meta.env.VITE_UAR_URL as string | undefined)?.replace(/\/$/, '') ?? ''

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const response = await fetch(`${apiRoot}${path}`, {
    ...init,
    headers: { 'content-type': 'application/json', ...init?.headers },
  })
  if (!response.ok) {
    const body = await response.text()
    throw new Error(body || `UAR request failed (${response.status})`)
  }
  if (response.status === 204) return undefined as T
  return response.json() as Promise<T>
}

async function webSnapshot(): Promise<UarAdminSnapshot> {
  const safe = async (path: string): Promise<unknown> => {
    try { return await request(path) }
    catch (error) { return { items: [], error: message(error) } }
  }
  const [settings, settingTypes, agents, skills, tools, knowledgeBases, mcpServers, memories] = await Promise.all([
    safe('/api/uar/settings'),
    safe('/api/uar/settings/types'),
    safe('/api/uar/discovery/agents'),
    safe('/api/uar/discovery/skills'),
    safe('/api/uar/discovery/tools'),
    safe('/api/uar/knowledge-bases'),
    safe('/api/uar/mcp/servers'),
    safe('/api/admin/memories'),
  ])
  return { settings, settingTypes, agents, skills, tools, knowledgeBases, mcpServers, memories }
}

function message(error: unknown): string {
  return error instanceof Error ? error.message : String(error)
}

interface UarAdminState {
  snapshot: UarAdminSnapshot | null
  loading: boolean
  saving: boolean
  error: string | null
  load: () => Promise<void>
  saveSetting: (key: string, value: unknown) => Promise<boolean>
  saveAgent: (agent: unknown) => Promise<boolean>
  deleteAgent: (id: string) => Promise<boolean>
  saveMcpServer: (name: string, server: unknown) => Promise<boolean>
  deleteMcpServer: (name: string) => Promise<boolean>
  deleteMemory: (id: string) => Promise<boolean>
  createKnowledgeBase: (request: unknown) => Promise<boolean>
  deleteKnowledgeBase: (id: string) => Promise<boolean>
}

export const useUarAdminStore = create<UarAdminState>((set, get) => {
  async function mutate(action: () => Promise<unknown>): Promise<boolean> {
    set({ saving: true, error: null })
    try {
      await action()
      await get().load()
      return true
    } catch (error) {
      set({ error: message(error) })
      return false
    } finally {
      set({ saving: false })
    }
  }

  return {
    snapshot: null,
    loading: false,
    saving: false,
    error: null,
    load: async () => {
      set({ loading: true, error: null })
      try {
        set({ snapshot: isTauri() ? await uarAdminSnapshot() : await webSnapshot() })
      } catch (error) {
        set({ error: message(error) })
      } finally {
        set({ loading: false })
      }
    },
    saveSetting: (key, value) => mutate(() => isTauri()
      ? uarSettingSave(key, value)
      : request(`/api/uar/settings/${encodeURIComponent(key)}`, { method: 'PUT', body: JSON.stringify({ value }) })),
    saveAgent: (agent) => mutate(() => {
      const id = typeof agent === 'object' && agent && 'id' in agent ? String((agent as { id?: unknown }).id ?? '') : ''
      return isTauri()
        ? uarAgentSave(agent)
        : request(id ? `/api/agents/${encodeURIComponent(id)}` : '/api/agents', {
          method: id ? 'PUT' : 'POST',
          body: JSON.stringify(agent),
        })
    }),
    deleteAgent: (id) => mutate(() => isTauri()
      ? uarAgentDelete(id)
      : request(`/api/agents/${encodeURIComponent(id)}`, { method: 'DELETE' })),
    saveMcpServer: (name, server) => mutate(() => isTauri()
      ? uarMcpServerSave(name, server)
      : request(`/api/uar/mcp/servers/${encodeURIComponent(name)}`, { method: 'PUT', body: JSON.stringify(server) })),
    deleteMcpServer: (name) => mutate(() => isTauri()
      ? uarMcpServerDelete(name)
      : request(`/api/uar/mcp/servers/${encodeURIComponent(name)}`, { method: 'DELETE' })),
    deleteMemory: (id) => mutate(() => isTauri()
      ? uarMemoryDelete(id)
      : request(`/api/admin/memories/${encodeURIComponent(id)}`, { method: 'DELETE' })),
    createKnowledgeBase: (body) => mutate(() => isTauri()
      ? uarKnowledgeBaseCreate(body)
      : request('/api/uar/knowledge-bases/', { method: 'POST', body: JSON.stringify(body) })),
    deleteKnowledgeBase: (id) => mutate(() => isTauri()
      ? uarKnowledgeBaseDelete(id)
      : request(`/api/uar/knowledge-bases/${encodeURIComponent(id)}`, { method: 'DELETE' })),
  }
})
