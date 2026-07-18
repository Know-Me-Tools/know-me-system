// TJ-ARCH-MOB-001 compliant — hook composes the store; no invoke() here.
// The hook is the layer boundary: it reads store state and derives everything
// the component needs (label, progress) so components import ONLY this hook.
import { useEffect } from 'react'
import { useStartupStore, phaseProgress } from '../stores/startupStore'

const PHASE_LABEL: Record<string, string> = {
  migrations: 'Applying migrations',
  seeds: 'Loading seed data',
  shapes: 'Attaching sync shapes',
  ready: 'Ready',
}

export function useStartup() {
  const phase = useStartupStore((s) => s.phase)
  const error = useStartupStore((s) => s.error)
  const run = useStartupStore((s) => s.run)
  useEffect(() => { void run() }, [run])
  return {
    phase,
    error,
    isReady: phase === 'ready',
    label: PHASE_LABEL[phase],
    progress: phaseProgress(phase),
  }
}
