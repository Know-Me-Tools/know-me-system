// TJ-ARCH-MOB-001 compliant — component imports the hook only.
import type { ReactNode } from 'react'
import { useStartup } from '../hooks/useStartup'
import { Titlebar } from '../../../shared/components/Titlebar'

function StartupShell({ children }: { children: ReactNode }) {
  return (
    <div className="flex h-screen flex-col bg-[color:var(--color-bg)]">
      <Titlebar />
      <div className="flex flex-1 flex-col overflow-hidden">{children}</div>
    </div>
  )
}

export function StartupGate({ children }: { children: ReactNode }) {
  const { error, isReady, label, progress } = useStartup()

  if (error) {
    return (
      <StartupShell>
        <div role="alert" className="flex flex-1 flex-col items-center justify-center gap-2 px-8 text-center">
          <span className="eyebrow">Startup failed</span>
          <p className="max-w-md font-mono text-sm text-[color:var(--color-fg-sub)]">{error}</p>
        </div>
      </StartupShell>
    )
  }

  if (isReady) return <StartupShell>{children}</StartupShell>

  return (
    <StartupShell>
      <div aria-live="polite" aria-busy="true" className="flex flex-1 flex-col items-center justify-center gap-3 px-8">
        <progress value={progress} max={1} className="h-1 w-48 overflow-hidden rounded-full [&::-webkit-progress-bar]:bg-[color:var(--color-muted)] [&::-webkit-progress-value]:bg-[color:var(--color-ember)]" />
        <p className="text-sm text-[color:var(--color-fg-sub)]">{label}</p>
      </div>
    </StartupShell>
  )
}
