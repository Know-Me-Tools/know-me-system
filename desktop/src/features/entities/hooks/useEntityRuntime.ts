// TJ-ARCH-MOB-001 compliant
import { useEffect, useState } from 'react'
import { startEntityRuntime } from '../stores/entityRuntime'

export function useEntityRuntime(tenantId: string | null) {
  const [isReady, setIsReady] = useState(false)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    setIsReady(false)
    setError(null)
    if (!tenantId) return undefined
    let active = true
    let dispose: (() => void) | undefined
    void startEntityRuntime(tenantId)
      .then((cleanup) => {
        if (!active) {
          cleanup()
          return
        }
        dispose = cleanup
        setIsReady(true)
      })
      .catch((cause: unknown) => {
        if (active) setError(String(cause))
      })
    return () => {
      active = false
      dispose?.()
    }
  }, [tenantId])

  return { isReady, error }
}
