// TJ-ARCH-MOB-001 compliant
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type {
  ConfiguredUarService,
  SaveUarServiceRequest,
  UarRuntimeStatus,
} from '@prometheus-ags/tauri-plugin-gen-ui'

const mocks = vi.hoisted(() => ({
  isTauri: vi.fn(() => true),
  uarRuntimeStatus: vi.fn(),
  uarServiceConfig: vi.fn(),
  uarServiceProbe: vi.fn(),
  uarServiceSave: vi.fn(),
}))

vi.mock('@tauri-apps/api/core', () => ({
  isTauri: mocks.isTauri,
}))

vi.mock('@prometheus-ags/tauri-plugin-gen-ui', () => ({
  uarRuntimeStatus: mocks.uarRuntimeStatus,
  uarServiceConfig: mocks.uarServiceConfig,
  uarServiceProbe: mocks.uarServiceProbe,
  uarServiceSave: mocks.uarServiceSave,
}))

import { useUarSettingsStore } from '../stores/uarSettingsStore'

const initial = useUarSettingsStore.getState()

const loadedConfig: ConfiguredUarService = {
  serviceUrl: 'http://127.0.0.1:8787',
  hasApiKey: true,
  hasAgentArtifact: true,
  sessionId: 'knowme-session',
}

const loadedStatus: UarRuntimeStatus = {
  authority: 'universal-agent-runtime',
  targetMode: 'http_service',
  currentState: 'service_runtime',
  localProviderRole: 'knowme_local_provider_backend',
  mobileSidecarAllowed: false,
  localProviderBridgeAvailable: true,
  serviceRuntimeAvailable: true,
  serviceArtifactResponseAvailable: true,
  embeddedKernelAvailable: false,
  embeddedKernelBlocker: null,
  submodulePath: 'rust/vendor/universal-agent-runtime',
  submoduleCommit: null,
}

beforeEach(() => {
  useUarSettingsStore.setState(initial, true)
  vi.clearAllMocks()
  mocks.isTauri.mockReturnValue(true)
  mocks.uarServiceConfig.mockResolvedValue(loadedConfig)
  mocks.uarRuntimeStatus.mockResolvedValue(loadedStatus)
  mocks.uarServiceProbe.mockResolvedValue({
    serviceUrl: 'http://127.0.0.1:8787',
    ok: true,
    httpStatus: 200,
    runtimeStatus: 'ok',
    message: 'UAR service reachable · status ok',
  })
  mocks.uarServiceSave.mockResolvedValue(undefined)
})

describe('uarSettingsStore', () => {
  it('loads service config metadata and runtime status through the Tauri plugin boundary', async () => {
    await useUarSettingsStore.getState().load()

    const state = useUarSettingsStore.getState()
    expect(state.desktopAvailable).toBe(true)
    expect(state.config).toEqual(loadedConfig)
    expect(state.status?.currentState).toBe('service_runtime')
    expect(state.loading).toBe(false)
    expect(state.error).toBeNull()
  })

  it('saves UAR service settings then refreshes redacted metadata', async () => {
    const request: SaveUarServiceRequest = {
      serviceUrl: 'http://127.0.0.1:8787',
      apiKey: 'secret',
      agentArtifactJson: '{ "agent": "knowme" }',
      sessionId: 'knowme-session',
    }

    const saved = await useUarSettingsStore.getState().save(request)

    expect(saved).toBe(true)
    expect(mocks.uarServiceSave).toHaveBeenCalledWith(request)
    expect(mocks.uarServiceConfig).toHaveBeenCalled()
    expect(mocks.uarRuntimeStatus).toHaveBeenCalled()
    expect(useUarSettingsStore.getState().saving).toBe(false)
  })

  it('probes the saved UAR service without changing configuration', async () => {
    const ok = await useUarSettingsStore.getState().probe()

    const state = useUarSettingsStore.getState()
    expect(ok).toBe(true)
    expect(mocks.uarServiceProbe).toHaveBeenCalledTimes(1)
    expect(state.probeResult?.message).toContain('reachable')
    expect(state.probing).toBe(false)
    expect(state.error).toBeNull()
  })
})
