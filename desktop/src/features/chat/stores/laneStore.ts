// TJ-ARCH-MOB-001 compliant — store layer
//
// Which inference lane chat runs on, and the throughput of the last local run.
//
// Three surfaces, one toggle:
//   - Tauri desktop → UAR/orchestrator or the pinned Rust local lane.
//   - Plain web     → WebLLM or UAR over the host REST bridge.
//   - Neither       → cloud only; the toggle is hidden rather than shown broken.
import { create } from 'zustand'
import { immer } from 'zustand/middleware/immer'
import { isTauri } from '@tauri-apps/api/core'
import {
  cancelModelOperation, getActiveLane, hasLocalEngine, onLocalModelStatus,
  prepareLocalModel, setActiveLane, uarRuntimeStatus, uarServiceConfig,
} from '@prometheus-ags/tauri-plugin-gen-ui'
import type {
  ChatLane,
  ConfiguredUarService,
  ModelOperationState,
  UarRuntimeStatus,
} from '@prometheus-ags/tauri-plugin-gen-ui'
import { isWebGpuAvailable, loadWebLlm, unloadWebLlm } from '../api/webllmLane'

/** Tokens/sec for a completed local run — the honest cost of on-device inference. */
export interface Throughput {
  tokens: number
  seconds: number
  tokensPerSecond: number
}

/** Model-download/init progress, 0..1, while the local lane warms up. */
export interface LaneLoadProgress {
  progress: number
  text: string
}

interface LaneState {
  lane: ChatLane
  /** False when neither a Rust engine nor WebGPU is available. */
  localAvailable: boolean
  /** True on desktop builds that can route chat through a configured UAR service. */
  uarSupported: boolean
  /** True when stored/env UAR config is sufficient to send a run. */
  uarConfigured: boolean
  /** Current UAR integration facts surfaced by the Rust core. */
  uarStatus: UarRuntimeStatus | null
  /** Stored UAR service config metadata; secrets/artifacts are not echoed back. */
  uarConfig: ConfiguredUarService | null
  /** Non-null only while the web lane is fetching/initialising weights. */
  loadProgress: LaneLoadProgress | null
  /** Last completed local run's throughput. Null on cloud or before a first run. */
  throughput: Throughput | null
  /** Why the last lane switch failed, for surfacing instead of silently reverting. */
  error: string | null
}

interface LaneActions {
  init: () => Promise<void>
  /** Returns whether the lane actually changed — a refused switch reports
   *  false so callers can leave their own selection where it was rather than
   *  claiming a mode the runtime declined. */
  switchLane: (lane: ChatLane) => Promise<boolean>
  prepareLocal: () => Promise<boolean>
  refreshUar: () => Promise<boolean>
  recordThroughput: (tokens: number, seconds: number) => void
  clearThroughput: () => void
  cancelPreparation: () => Promise<void>
}

function isRunnableUar(status: UarRuntimeStatus | null, config: ConfiguredUarService | null): boolean {
  return status?.currentState === 'service_runtime'
    || Boolean(config?.serviceUrl)
}

/**
 * Why the UAR lane is unavailable, in terms the current host can act on.
 *
 * macOS hosts the runtime IN-PROCESS, so it renders no service-URL field at
 * all — telling those users to "configure UAR in Settings" sends them hunting
 * for a control that does not exist. Windows and Linux are genuine service
 * clients, where that instruction is correct.
 *
 * Exported and shared because this message is produced from three places
 * (lane init, switchLane, and a queued run in chatStore); three copies of the
 * string is how the embedded case came to be wrong in all of them.
 */
export function uarUnavailableMessage(status: UarRuntimeStatus | null): string {
  return status?.targetMode === 'embedded_library'
    ? 'The embedded runtime is not ready yet. Check Settings → Model providers for the on-device model.'
    : 'Configure Universal Agent Runtime in Settings before using the UAR lane.'
}

/**
 * Whether a local-model operation is in flight.
 *
 * Mirrors Flutter's `_modelOperationBusy`: downloading, verifying, loading and
 * cancelling all count. The composer is gated on this, because a send during a
 * multi-gigabyte download reaches a lane whose weights are not resident — the
 * turn fails for a reason the user was given no way to see.
 */
export function isModelOperationBusy(progress: LaneLoadProgress | null): boolean {
  return progress !== null
}

function operationProgress(status: ModelOperationState): LaneLoadProgress | null {
  switch (status.state) {
    case 'downloading': return {
      progress: status.total_bytes > 0 ? status.bytes_received / status.total_bytes : 0,
      text: `Downloading model · ${(status.bytes_received / 1_048_576).toFixed(0)} of ${(status.total_bytes / 1_048_576).toFixed(0)} MB`,
    }
    case 'verifying': return { progress: 0.98, text: 'Verifying model' }
    case 'loading': return { progress: 0.99, text: 'Loading model into memory' }
    case 'cancelling': return { progress: 0, text: 'Cancelling model preparation' }
    default: return null
  }
}

export const useLaneStore = create<LaneState & LaneActions>()(
  immer((set, get) => ({
    lane: 'local',
    localAvailable: false,
    uarSupported: false,
    uarConfigured: false,
    uarStatus: null,
    uarConfig: null,
    loadProgress: null,
    throughput: null,
    error: null,

    init: async () => {
      if (isTauri()) {
        // Store is the only layer that calls invoke().
        const [available, lane, uarStatusValue, uarConfigValue] = await Promise.all([
          hasLocalEngine().catch(() => false),
          getActiveLane().catch((): ChatLane => 'cloud'),
          uarRuntimeStatus().catch((): UarRuntimeStatus | null => null),
          uarServiceConfig().catch((): ConfiguredUarService | null => null),
        ])
        const configured = isRunnableUar(uarStatusValue, uarConfigValue)
        const desktopLane: ChatLane = lane === 'cloud'
          ? (configured ? 'uar' : (available ? 'local' : 'uar'))
          : (lane === 'uar' && !configured && available ? 'local' : lane)
        set((s) => {
          s.localAvailable = available
          s.uarSupported = Boolean(uarStatusValue?.serviceRuntimeAvailable)
          s.uarConfigured = configured
          s.uarStatus = uarStatusValue
          s.uarConfig = uarConfigValue
          s.lane = desktopLane
          if (desktopLane === 'uar' && !configured) {
            s.error = uarUnavailableMessage(uarStatusValue)
          } else if (lane === 'cloud') {
            s.error = null
          }
        })
        if (desktopLane !== lane && (desktopLane === 'local' || (desktopLane === 'uar' && configured))) {
          void setActiveLane(desktopLane).catch((error: unknown) => set((s) => {
            s.error = error instanceof Error ? error.message : 'failed to select desktop inference lane'
          }))
        }
        if (available) {
          void onLocalModelStatus((status) => set((s) => {
            s.loadProgress = operationProgress(status)
            if (status.state === 'failed') s.error = status.error.message
          }))
        }
        return
      }
      // Plain web: use local WebLLM when available, otherwise the UAR HTTP bridge.
      const available = isWebGpuAvailable()
      set((s) => {
        s.localAvailable = available
        s.uarSupported = true
        s.uarConfigured = true
        s.uarStatus = null
        s.uarConfig = null
        s.lane = available ? 'local' : 'uar'
      })
    },

    refreshUar: async () => {
      if (!isTauri()) return true
      const [status, config] = await Promise.all([
        uarRuntimeStatus().catch((): UarRuntimeStatus | null => null),
        uarServiceConfig().catch((): ConfiguredUarService | null => null),
      ])
      const configured = isRunnableUar(status, config)
      set((s) => {
        s.uarSupported = Boolean(status?.serviceRuntimeAvailable)
        s.uarConfigured = configured
        s.uarStatus = status
        s.uarConfig = config
      })
      return configured
    },

    prepareLocal: async () => {
      set((s) => { s.error = null })
      try {
        if (isTauri()) {
          if (!await hasLocalEngine()) return false
          await prepareLocalModel()
          return true
        }
        await loadWebLlm((p) => set((s) => { s.loadProgress = p }))
        set((s) => {
          s.loadProgress = null
        })
        return true
      } catch (e: unknown) {
        set((s) => {
          s.error = e instanceof Error ? e.message : 'failed to prepare local inference'
          s.loadProgress = null
        })
        return false
      }
    },

    switchLane: async (lane) => {
      set((s) => { s.error = null })
      if (lane === 'local') {
        const ready = await get().prepareLocal()
        if (!ready) return false
      } else if (lane === 'uar') {
        const ready = await get().refreshUar()
        if (!ready) {
          set((s) => { s.error = uarUnavailableMessage(s.uarStatus) })
          return false
        }
      } else if (isTauri()) {
        set((s) => { s.error = 'Desktop chat uses UAR or Local. Configure Liter-LLM providers under UAR settings.' })
        return false
      } else if (lane === 'cloud') {
        await unloadWebLlm()
      }
      try {
        if (isTauri()) await setActiveLane(lane)
        set((s) => {
          s.lane = lane
          if (lane === 'cloud') s.throughput = null
        })
        return true
      } catch (e: unknown) {
        // Stay on the current lane and say why. Never silently fall back to
        // cloud — a user who asked for on-device inference must not be quietly
        // switched to a network provider.
        set((s) => { s.error = e instanceof Error ? e.message : 'failed to switch lane' })
        return false
      }
    },

    recordThroughput: (tokens, seconds) =>
      set((s) => {
        // Guard against a zero/absurd elapsed time producing an Infinity readout.
        s.throughput = seconds > 0
          ? { tokens, seconds, tokensPerSecond: tokens / seconds }
          : null
      }),

    clearThroughput: () => set((s) => { s.throughput = null }),

    cancelPreparation: async () => {
      if (isTauri()) await cancelModelOperation()
      set((s) => { s.loadProgress = { progress: 0, text: 'Cancelling model preparation' } })
    },
  })),
)
