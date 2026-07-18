// TJ-ARCH-MOB-001 compliant — components consume this hook, never the store.
import { useEffect } from 'react'
import { useProviderStore } from '../stores/providerStore'

export function useProviderSettings() {
  const state = useProviderStore()
  useEffect(() => {
    void state.load()
    // The store action is stable for the lifetime of the Zustand store.
  }, [state.load])
  return state
}
