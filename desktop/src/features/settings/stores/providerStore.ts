// TJ-ARCH-MOB-001 compliant — Zustand owns the Rust transport boundary.
import { isTauri } from '@tauri-apps/api/core'
import {
  cloudModelGet,
  cloudModelSave,
  providerCatalog,
  providerDelete,
  providerList,
  providerSave,
} from '@prometheus-ags/tauri-plugin-gen-ui'
import type {
  ConfiguredCloudModel,
  ConfiguredProvider,
  ProviderCatalogEntry,
  SaveProviderRequest,
} from '@prometheus-ags/tauri-plugin-gen-ui'
import { create } from 'zustand'

interface ProviderState {
  catalog: ProviderCatalogEntry[]
  providers: ConfiguredProvider[]
  cloudModel: ConfiguredCloudModel | null
  loading: boolean
  saving: boolean
  error: string | null
  desktopAvailable: boolean
  load: () => Promise<void>
  saveProvider: (request: SaveProviderRequest) => Promise<boolean>
  removeProvider: (id: string) => Promise<void>
  saveCloudModel: (providerId: string, modelId: string) => Promise<boolean>
}

const WEB_BYOK_KEY = 'knowme-web-byok'

interface WebByokState {
  provider: SaveProviderRequest
  modelId: string
}

function readWebByok(): WebByokState | null {
  try {
    const value = window.sessionStorage.getItem(WEB_BYOK_KEY)
    return value ? JSON.parse(value) as WebByokState : null
  } catch {
    return null
  }
}

export function getSessionByok(): { provider: string; model: string; apiKey: string; baseUrl: string | null } | null {
  const value = readWebByok()
  if (!value?.provider.apiKey || !value.modelId) return null
  return {
    provider: value.provider.kind,
    model: value.modelId,
    apiKey: value.provider.apiKey,
    baseUrl: value.provider.baseUrl,
  }
}

function message(error: unknown): string {
  return error instanceof Error ? error.message : String(error)
}

export const useProviderStore = create<ProviderState>((set, get) => ({
  catalog: [],
  providers: [],
  cloudModel: null,
  loading: false,
  saving: false,
  error: null,
  desktopAvailable: isTauri(),

  load: async () => {
    set({ loading: true, error: null })
    try {
      if (!isTauri()) {
        const response = await fetch('/api/v1/providers/catalog')
        if (!response.ok) throw new Error('Hosted provider catalog is unavailable')
        const catalog = await response.json() as ProviderCatalogEntry[]
        const saved = readWebByok()
        set({
          catalog,
          providers: saved ? [{
            id: saved.provider.id,
            kind: saved.provider.kind,
            baseUrl: saved.provider.baseUrl,
            enabled: saved.provider.enabled,
            hasApiKey: Boolean(saved.provider.apiKey),
          }] : [],
          cloudModel: saved ? { providerId: saved.provider.id, modelId: saved.modelId } : null,
        })
        return
      }
      const [catalog, providers, cloudModel] = await Promise.all([
        providerCatalog(),
        providerList(),
        cloudModelGet(),
      ])
      set({ catalog, providers, cloudModel })
    } catch (error) {
      set({ error: message(error) })
    } finally {
      set({ loading: false })
    }
  },

  saveProvider: async (request) => {
    set({ saving: true, error: null })
    try {
      if (!isTauri()) {
        const current = readWebByok()
        window.sessionStorage.setItem(WEB_BYOK_KEY, JSON.stringify({
          provider: request,
          modelId: current?.modelId ?? '',
        } satisfies WebByokState))
        await get().load()
        return true
      }
      await providerSave(request)
      await get().load()
      return true
    } catch (error) {
      set({ error: message(error) })
      return false
    } finally {
      set({ saving: false })
    }
  },

  removeProvider: async (id) => {
    set({ saving: true, error: null })
    try {
      if (!isTauri()) {
        window.sessionStorage.removeItem(WEB_BYOK_KEY)
        await get().load()
        return
      }
      await providerDelete(id)
      await get().load()
    } catch (error) {
      set({ error: message(error) })
    } finally {
      set({ saving: false })
    }
  },

  saveCloudModel: async (providerId, modelId) => {
    set({ saving: true, error: null })
    try {
      if (!isTauri()) {
        const current = readWebByok()
        if (!current || current.provider.id !== providerId) throw new Error('Save a provider and key first')
        window.sessionStorage.setItem(WEB_BYOK_KEY, JSON.stringify({ ...current, modelId } satisfies WebByokState))
        await get().load()
        return true
      }
      await cloudModelSave(providerId, modelId)
      await get().load()
      return true
    } catch (error) {
      set({ error: message(error) })
      return false
    } finally {
      set({ saving: false })
    }
  },
}))
