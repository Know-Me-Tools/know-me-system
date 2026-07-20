// TJ-ARCH-MOB-001 compliant — Zustand owns the Rust/HTTP transport boundary.
import { isTauri } from '@tauri-apps/api/core'
import {
  cloudModelGet,
  cloudModelSave,
  providerCatalog,
  providerDelete,
  providerList,
  providerSave,
  providerTest,
} from '@prometheus-ags/tauri-plugin-gen-ui'
import type {
  ConfiguredCloudModel,
  ConfiguredProvider,
  ProviderCatalogEntry,
  ProviderTestResult,
  SaveProviderRequest,
} from '@prometheus-ags/tauri-plugin-gen-ui'
import { create } from 'zustand'

interface ProviderState {
  catalog: ProviderCatalogEntry[]
  providers: ConfiguredProvider[]
  cloudModel: ConfiguredCloudModel | null
  testResult: ProviderTestResult | null
  loading: boolean
  saving: boolean
  testing: boolean
  error: string | null
  desktopAvailable: boolean
  load: () => Promise<void>
  saveProvider: (request: SaveProviderRequest) => Promise<boolean>
  removeProvider: (id: string) => Promise<void>
  testProvider: (id: string, model?: string | null) => Promise<boolean>
  saveCloudModel: (providerId: string, modelId: string) => Promise<boolean>
}

async function jsonRequest<T>(path: string, init?: RequestInit): Promise<T> {
  const response = await fetch(path, {
    ...init,
    headers: { 'content-type': 'application/json', ...init?.headers },
  })
  if (!response.ok) {
    const payload = await response.json().catch(() => null) as { message?: string; error?: string } | null
    throw new Error(payload?.message ?? payload?.error ?? `UAR request failed (${response.status})`)
  }
  if (response.status === 204) return undefined as T
  return response.json() as Promise<T>
}

function message(error: unknown): string {
  return error instanceof Error ? error.message : String(error)
}

type JsonObject = Record<string, unknown>
function object(value: unknown): JsonObject { return typeof value === 'object' && value !== null && !Array.isArray(value) ? value as JsonObject : {} }
function array(value: unknown): JsonObject[] { return Array.isArray(value) ? value.map(object) : [] }
function string(value: unknown): string { return typeof value === 'string' ? value : '' }
function bool(value: unknown, fallback = false): boolean { return typeof value === 'boolean' ? value : fallback }
function numberOrNull(value: unknown): number | null { return typeof value === 'number' ? value : null }

function webModel(id: string, raw: unknown): ProviderCatalogEntry['models'][number] {
  const value = object(raw); const limit = object(value.limit); const modalities = object(value.modalities)
  const inputs = Array.isArray(modalities.input) ? modalities.input : []
  return {
    id,
    displayName: string(value.name) || null,
    contextWindow: numberOrNull(limit.context),
    maxOutputTokens: numberOrNull(limit.output),
    supportsVision: inputs.includes('image'),
    supportsTools: bool(value.tool_call),
    supportsReasoning: bool(value.reasoning),
    supportsStructuredOutput: bool(value.structured_output),
    supportsStreaming: true,
    enabled: true,
  }
}

async function loadWebProviders(): Promise<[ProviderCatalogEntry[], ConfiguredProvider[], ConfiguredCloudModel | null]> {
  const [catalogRaw, configuredRaw] = await Promise.all([
    jsonRequest<JsonObject>('/api/models'),
    jsonRequest<JsonObject>('/api/uar/providers'),
  ])
  const configured = array(configuredRaw.providers).map((provider): ConfiguredProvider => ({
    id: string(provider.id), displayName: string(provider.display_name) || string(provider.id),
    baseUrl: string(provider.base_url) || null, enabled: bool(provider.enabled),
    hasApiKey: bool(provider.credential_configured), protocol: provider.protocol,
    defaultModel: string(provider.default_model) || null,
    models: array(provider.models).map((model) => ({
      id: string(model.id), displayName: string(model.display_name) || null,
      contextWindow: numberOrNull(model.context_window), maxOutputTokens: numberOrNull(model.max_output_tokens),
      supportsVision: bool(model.supports_vision), supportsTools: bool(model.supports_tools),
      supportsReasoning: false, supportsStructuredOutput: false, supportsStreaming: true, enabled: bool(model.enabled, true),
    })),
  }))
  const byId = new Map(configured.map((provider) => [provider.id, provider]))
  const catalog = Object.entries(catalogRaw).map(([id, raw]): ProviderCatalogEntry => {
    const value = object(raw); const models = object(value.models); const saved = byId.get(id)
    return { id, displayName: string(value.display_name) || id, defaultBaseUrl: string(value.base_url) || null,
      requiresApiKey: true, configured: Boolean(saved), enabled: saved?.enabled ?? false,
      models: Object.entries(models).map(([modelId, model]) => webModel(modelId, model)) }
  }).sort((left, right) => Number(right.configured) - Number(left.configured) || left.displayName.localeCompare(right.displayName))
  const defaultId = string(configuredRaw.default_id)
  const defaultProvider = configured.find((provider) => provider.id === defaultId)
  return [catalog, configured, defaultProvider?.defaultModel ? { providerId: defaultId, modelId: defaultProvider.defaultModel } : null]
}

export const useProviderStore = create<ProviderState>((set, get) => ({
  catalog: [],
  providers: [],
  cloudModel: null,
  testResult: null,
  loading: false,
  saving: false,
  testing: false,
  error: null,
  desktopAvailable: isTauri(),

  load: async () => {
    set({ loading: true, error: null })
    try {
      const [catalog, providers, cloudModel] = isTauri()
        ? await Promise.all([providerCatalog(), providerList(), cloudModelGet()])
        : await loadWebProviders()
      set({ catalog, providers, cloudModel })
    } catch (error) {
      set({ error: message(error) })
    } finally {
      set({ loading: false })
    }
  },

  saveProvider: async (request) => {
    set({ saving: true, error: null, testResult: null })
    try {
      if (isTauri()) await providerSave(request)
      else await jsonRequest<void>(`/api/uar/providers/${encodeURIComponent(request.id)}`, { method: 'PUT', body: JSON.stringify({
        id: request.id, display_name: request.displayName ?? request.id, base_url: request.baseUrl ?? '',
        api_key: request.apiKey ?? undefined, enabled: request.enabled, protocol: request.protocol ?? 'auto',
        default_model: request.defaultModel ?? null,
        models: (request.models ?? []).map((model) => ({ id: model.id, display_name: model.displayName,
          context_window: model.contextWindow, max_output_tokens: model.maxOutputTokens,
          supports_vision: model.supportsVision, supports_tools: model.supportsTools, enabled: model.enabled })),
      }) })
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
    set({ saving: true, error: null, testResult: null })
    try {
      if (isTauri()) await providerDelete(id)
      else await jsonRequest<void>(`/api/uar/providers/${encodeURIComponent(id)}`, { method: 'DELETE' })
      await get().load()
    } catch (error) {
      set({ error: message(error) })
    } finally {
      set({ saving: false })
    }
  },

  testProvider: async (id, model) => {
    set({ testing: true, error: null, testResult: null })
    try {
      const result = isTauri()
        ? await providerTest(id, model)
        : await jsonRequest<JsonObject>(`/api/uar/providers/${encodeURIComponent(id)}/test`, { method: 'POST', body: JSON.stringify({ model: model ?? null }) }).then((value) => ({
          ok: bool(value.ok), providerId: string(value.provider_id), modelId: string(value.model_id),
          latencyMs: Number(value.latency_ms ?? 0), receivedText: bool(value.received_text),
        }))
      set({ testResult: result })
      return result.ok
    } catch (error) {
      set({ error: message(error) })
      return false
    } finally {
      set({ testing: false })
    }
  },

  saveCloudModel: async (providerId, modelId) => {
    set({ saving: true, error: null })
    try {
      if (isTauri()) await cloudModelSave(providerId, modelId)
      else {
        const provider = get().providers.find((entry) => entry.id === providerId)
        if (!provider) throw new Error(`Provider '${providerId}' is not configured`)
        await jsonRequest(`/api/uar/providers/${encodeURIComponent(providerId)}`, { method: 'PUT', body: JSON.stringify({
          id: provider.id, display_name: provider.displayName, base_url: provider.baseUrl ?? '', protocol: provider.protocol,
          default_model: modelId, models: provider.models.map((model) => ({ id: model.id, display_name: model.displayName,
            context_window: model.contextWindow, max_output_tokens: model.maxOutputTokens,
            supports_vision: model.supportsVision, supports_tools: model.supportsTools, enabled: model.enabled })), enabled: provider.enabled,
        }) })
        await jsonRequest(`/api/uar/providers/${encodeURIComponent(providerId)}/default`, { method: 'POST' })
      }
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
