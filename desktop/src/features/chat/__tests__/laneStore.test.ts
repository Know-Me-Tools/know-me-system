// TJ-ARCH-MOB-001 compliant
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type {
  ConfiguredUarService,
  ModelOperationState,
  UarRuntimeStatus,
} from '@prometheus-ags/tauri-plugin-gen-ui'

const mocks = vi.hoisted(() => ({
  isTauri: vi.fn(() => true),
  cancelModelOperation: vi.fn(),
  getActiveLane: vi.fn(),
  hasLocalEngine: vi.fn(),
  onLocalModelStatus: vi.fn(),
  prepareLocalModel: vi.fn(),
  setActiveLane: vi.fn(),
  uarRuntimeStatus: vi.fn(),
  uarServiceConfig: vi.fn(),
  isWebGpuAvailable: vi.fn(() => false),
  loadWebLlm: vi.fn(),
  unloadWebLlm: vi.fn(),
}))

vi.mock('@tauri-apps/api/core', () => ({
  isTauri: mocks.isTauri,
}))

vi.mock('@prometheus-ags/tauri-plugin-gen-ui', () => ({
  cancelModelOperation: mocks.cancelModelOperation,
  getActiveLane: mocks.getActiveLane,
  hasLocalEngine: mocks.hasLocalEngine,
  onLocalModelStatus: mocks.onLocalModelStatus,
  prepareLocalModel: mocks.prepareLocalModel,
  setActiveLane: mocks.setActiveLane,
  uarRuntimeStatus: mocks.uarRuntimeStatus,
  uarServiceConfig: mocks.uarServiceConfig,
}))

vi.mock('../api/webllmLane', () => ({
  isWebGpuAvailable: mocks.isWebGpuAvailable,
  loadWebLlm: mocks.loadWebLlm,
  unloadWebLlm: mocks.unloadWebLlm,
}))

import { useLaneStore } from '../stores/laneStore'

const initial = useLaneStore.getState()

function runtime(overrides: Partial<UarRuntimeStatus> = {}): UarRuntimeStatus {
  return {
    authority: 'universal-agent-runtime',
    targetMode: 'http_service',
    currentState: 'projection_only',
    localProviderRole: 'knowme_local_provider_backend',
    mobileSidecarAllowed: false,
    localProviderBridgeAvailable: true,
    serviceRuntimeAvailable: true,
    serviceArtifactResponseAvailable: true,
    embeddedKernelAvailable: false,
    embeddedKernelBlocker: 'mobile embedded kernel not wired',
    submodulePath: 'rust/vendor/universal-agent-runtime',
    submoduleCommit: null,
    ...overrides,
  }
}

function config(overrides: Partial<ConfiguredUarService> = {}): ConfiguredUarService {
  return {
    serviceUrl: null,
    hasApiKey: false,
    hasAgentArtifact: false,
    sessionId: null,
    ...overrides,
  }
}

beforeEach(() => {
  useLaneStore.setState(initial, true)
  vi.clearAllMocks()
  mocks.isTauri.mockReturnValue(true)
  mocks.hasLocalEngine.mockResolvedValue(true)
  mocks.getActiveLane.mockResolvedValue('cloud')
  mocks.onLocalModelStatus.mockImplementation((_callback: (state: ModelOperationState) => void) => undefined)
  mocks.prepareLocalModel.mockResolvedValue(undefined)
  mocks.setActiveLane.mockResolvedValue(undefined)
  mocks.uarRuntimeStatus.mockResolvedValue(runtime())
  mocks.uarServiceConfig.mockResolvedValue(config())
})

describe('laneStore UAR lane', () => {
  it('activates the UAR lane only when service config is runnable', async () => {
    mocks.uarRuntimeStatus.mockResolvedValue(runtime({ currentState: 'service_runtime' }))
    mocks.uarServiceConfig.mockResolvedValue(config({
      serviceUrl: null,
      hasAgentArtifact: true,
    }))

    await useLaneStore.getState().init()
    await useLaneStore.getState().switchLane('uar')

    const state = useLaneStore.getState()
    expect(state.uarSupported).toBe(true)
    expect(state.uarConfigured).toBe(true)
    expect(state.lane).toBe('uar')
    expect(mocks.setActiveLane).toHaveBeenCalledWith('uar')
  })

  it('falls back to local when persisted desktop lane is cloud and UAR is not configured', async () => {
    mocks.uarRuntimeStatus.mockResolvedValue(runtime({ currentState: 'projection_only' }))
    mocks.uarServiceConfig.mockResolvedValue(config({
      serviceUrl: null,
      hasAgentArtifact: false,
    }))

    await useLaneStore.getState().init()

    const state = useLaneStore.getState()
    expect(state.lane).toBe('local')
    expect(state.uarSupported).toBe(true)
    expect(state.uarConfigured).toBe(false)
    expect(mocks.setActiveLane).toHaveBeenCalledWith('local')
  })

  it('surfaces a configuration error instead of selecting an unconfigured UAR lane', async () => {
    mocks.getActiveLane.mockResolvedValue('local')
    mocks.uarRuntimeStatus.mockResolvedValue(runtime({ currentState: 'projection_only' }))
    mocks.uarServiceConfig.mockResolvedValue(config({
      serviceUrl: null,
      hasAgentArtifact: false,
    }))

    await useLaneStore.getState().init()
    await useLaneStore.getState().switchLane('uar')

    const state = useLaneStore.getState()
    expect(state.lane).toBe('local')
    expect(state.uarSupported).toBe(true)
    expect(state.uarConfigured).toBe(false)
    expect(state.error).toContain('Configure Universal Agent Runtime')
    expect(mocks.setActiveLane).not.toHaveBeenCalledWith('uar')
  })

  it('rejects cloud selection on Tauri desktop', async () => {
    mocks.getActiveLane.mockResolvedValue('local')

    await useLaneStore.getState().init()
    await useLaneStore.getState().switchLane('cloud')

    const state = useLaneStore.getState()
    expect(state.lane).toBe('local')
    expect(state.error).toContain('Desktop chat uses UAR or Local')
    expect(mocks.setActiveLane).not.toHaveBeenCalledWith('cloud')
  })
})
