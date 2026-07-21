// TJ-ARCH-MOB-001 compliant — components consume this hook, never the store.
import { useEffect } from 'react'
import { useUarSettingsStore } from '../stores/uarSettingsStore'

export function useUarSettings() {
  const state = useUarSettingsStore()
  useEffect(() => {
    void state.load()
    // The store action is stable for the lifetime of the Zustand store.
  }, [state.load])
  return state
}
