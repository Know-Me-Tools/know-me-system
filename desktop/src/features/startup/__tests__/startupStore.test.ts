// TJ-ARCH-MOB-001 compliant
import { beforeEach, describe, expect, it, vi } from 'vitest'

const invoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invoke(...args),
  isTauri: () => true,
}))

import { useStartupStore, PHASE_ORDER, phaseProgress } from '../stores/startupStore'

const initial = useStartupStore.getState()
beforeEach(() => {
  useStartupStore.setState(initial, true)
  invoke.mockReset()
})

describe('startupStore', () => {
  it('runs the boot sequence in order: migrations → seeds → shapes → ready', async () => {
    invoke.mockResolvedValue(undefined)

    await useStartupStore.getState().run()

    expect(useStartupStore.getState().phase).toBe('ready')
    // The boot-order invariant: shapes come AFTER migrations and seeds.
    expect(invoke.mock.calls.map((c) => c[0])).toEqual([
      'plugin:gen-ui|run_migrations',
      'plugin:gen-ui|load_seeds',
      'plugin:gen-ui|attach_sync_shapes',
    ])
  })

  it('halts on a failed migration without attaching shapes', async () => {
    invoke.mockRejectedValueOnce('migration 003 failed')

    await useStartupStore.getState().run()

    const s = useStartupStore.getState()
    expect(s.error).toContain('migration 003 failed')
    expect(s.phase).not.toBe('ready')
    // Shapes must never attach when migrations fail (they'd hit unknown columns).
    expect(invoke.mock.calls.map((c) => c[0])).not.toContain('plugin:gen-ui|attach_sync_shapes')
  })

  it('exposes monotonic progress across the phase order', () => {
    const progresses = PHASE_ORDER.map(phaseProgress)
    for (let i = 1; i < progresses.length; i++) {
      expect(progresses[i]).toBeGreaterThan(progresses[i - 1])
    }
    expect(phaseProgress('ready')).toBe(1)
  })
})
