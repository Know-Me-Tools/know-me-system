// TJ-ARCH-MOB-001 compliant — component imports the hook only.
import type { ReactNode } from 'react'
import { useEntityRuntime } from '../hooks/useEntityRuntime'

export function EntityRuntimeBoundary({ children }: { children: ReactNode }) {
  const { isReady, error } = useEntityRuntime('knowme-poc-local')
  if (error) return <div role="alert">Entity runtime failed: {error}</div>
  if (!isReady) return <div aria-live="polite">Starting entity runtime…</div>
  return <>{children}</>
}
