// TJ-ARCH-MOB-001 compliant — store layer (the ONLY place invoke() is called).
import { create } from 'zustand'
import { isTauri } from '@tauri-apps/api/core'
import { attachSyncShapes, loadSeeds, runMigrations } from '@prometheus-ags/tauri-plugin-gen-ui'

export type StartupPhase = 'migrations' | 'seeds' | 'shapes' | 'ready'

export const PHASE_ORDER: StartupPhase[] = ['migrations', 'seeds', 'shapes', 'ready']

const PHASE_PROGRESS: Record<StartupPhase, number> = {
  migrations: 0.25,
  seeds: 0.5,
  shapes: 0.85,
  ready: 1,
}
export const phaseProgress = (p: StartupPhase): number => PHASE_PROGRESS[p]

interface StartupState {
  phase: StartupPhase
  error: string | null
  run: () => Promise<void>
}

// Boot-order invariant: migrations → seeds → shapes. Sync shapes fail on unknown
// columns, so migrations+seeds MUST run first. On web the wasm core runs the same
// sequence; until wired, the steps resolve immediately so the gate opens.
//
// React StrictMode double-invokes the mounting effect in dev, so `run()` must be
// idempotent: a second call while a run is in flight joins the same promise
// instead of firing the whole boot sequence (and its Tauri commands) twice. The
// marker is cleared after either outcome so later retries are real new runs.
let inflight: Promise<void> | null = null

export const useStartupStore = create<StartupState>((set) => ({
  phase: 'migrations',
  error: null,
  run: () => {
    inflight ??= (async () => {
      try {
        set({ phase: 'migrations', error: null })
        if (isTauri()) await runMigrations()
        set({ phase: 'seeds' })
        if (isTauri()) await loadSeeds()
        set({ phase: 'shapes' })
        if (isTauri()) await attachSyncShapes()
        set({ phase: 'ready' })
      } catch (e) {
        set({ error: String(e) })
      } finally {
        inflight = null
      }
    })()
    return inflight
  },
}))
