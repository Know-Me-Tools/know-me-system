// TJ-ARCH-MOB-001 compliant — components consume hooks, never transports.
import { useEffect } from 'react'
import { useUarAdminStore } from '../stores/uarAdminStore'

export function useUarAdmin() {
  const state = useUarAdminStore()
  useEffect(() => {
    void state.load()
  }, [state.load])
  return state
}
