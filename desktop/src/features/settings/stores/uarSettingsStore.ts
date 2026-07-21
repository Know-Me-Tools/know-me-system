// TJ-ARCH-MOB-001 compliant — Zustand owns the Rust transport boundary.
import { isTauri } from '@tauri-apps/api/core'
import {
  uarRuntimeStatus,
  uarServiceConfig,
  uarServiceProbe,
  uarServiceSave,
} from '@prometheus-ags/tauri-plugin-gen-ui'
import type {
  ConfiguredUarService,
  SaveUarServiceRequest,
  UarServiceProbeResult,
  UarRuntimeStatus,
} from '@prometheus-ags/tauri-plugin-gen-ui'
import { create } from 'zustand'

interface UarSettingsState {
  config: ConfiguredUarService | null
  status: UarRuntimeStatus | null
  probeResult: UarServiceProbeResult | null
  loading: boolean
  saving: boolean
  probing: boolean
  error: string | null
  desktopAvailable: boolean
  load: () => Promise<void>
  save: (request: SaveUarServiceRequest) => Promise<boolean>
  probe: () => Promise<boolean>
}

function message(error: unknown): string {
  return error instanceof Error ? error.message : String(error)
}

export const useUarSettingsStore = create<UarSettingsState>((set, get) => ({
  config: null,
  status: null,
  probeResult: null,
  loading: false,
  saving: false,
  probing: false,
  error: null,
  desktopAvailable: isTauri(),

  load: async () => {
    set({ loading: true, error: null })
    try {
      if (!isTauri()) {
        set({ config: null, status: null, desktopAvailable: false })
        return
      }
      const [config, status] = await Promise.all([
        uarServiceConfig(),
        uarRuntimeStatus(),
      ])
      set({ config, status, desktopAvailable: true })
    } catch (error) {
      set({ error: message(error) })
    } finally {
      set({ loading: false })
    }
  },

  save: async (request) => {
    set({ saving: true, error: null })
    try {
      if (!isTauri()) throw new Error('UAR service configuration is only available in the desktop app')
      await uarServiceSave(request)
      await get().load()
      return true
    } catch (error) {
      set({ error: message(error) })
      return false
    } finally {
      set({ saving: false })
    }
  },

  probe: async () => {
    set({ probing: true, error: null, probeResult: null })
    try {
      if (!isTauri()) throw new Error('UAR service probing is only available in the desktop app')
      const result = await uarServiceProbe()
      set({ probeResult: result })
      return result.ok
    } catch (error) {
      set({ error: message(error) })
      return false
    } finally {
      set({ probing: false })
    }
  },
}))
