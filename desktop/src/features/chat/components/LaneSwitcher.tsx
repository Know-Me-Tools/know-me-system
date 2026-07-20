// TJ-ARCH-MOB-001 compliant — component imports the hook only.
import { useEffect } from 'react'
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import { Button } from '@/components/ui/button'
import { useLane } from '../hooks/useLane'

/**
 * Cloud/local inference toggle, plus the honest cost of running locally:
 * download progress while weights load, tok/s once a local run completes.
 *
 * Renders nothing when no local lane exists (no Rust engine, no WebGPU) — an
 * absent capability is better hidden than shown broken.
 */
export function LaneSwitcher() {
  const {
    lane,
    localAvailable,
    uarSupported,
    uarConfigured,
    loadProgress,
    throughput,
    error,
    switchLane,
    initLane,
    cancelPreparation,
  } = useLane()

  useEffect(() => {
    void initLane()
  }, [initLane])

  if (!localAvailable && !uarSupported) return null

  const isLoading = loadProgress !== null
  const isLocal = lane === 'local'
  const options = [
    ...(uarSupported ? [{ id: 'uar' as const, label: 'UAR', aria: 'Universal Agent Runtime' }] : []),
    ...(localAvailable ? [{ id: 'local' as const, label: 'On-device', aria: 'On-device' }] : []),
  ]

  return (
    <div className="flex min-h-12 flex-wrap items-center gap-3 bg-[color:var(--color-bg-2)] px-4 py-2 text-xs">
      <ToggleGroup
        aria-label="Inference lane"
        value={[lane]}
        disabled={isLoading}
        onValueChange={(values) => {
          const next = values.at(-1)
          if (next === 'cloud' || next === 'local' || next === 'uar') void switchLane(next)
        }}
        className="rounded-xl bg-[color:var(--color-surface)] p-1"
      >
        {options.map((option) => (
          <ToggleGroupItem
            key={option.id}
            value={option.id}
            aria-label={option.aria}
            title={option.id === 'uar' && !uarConfigured ? 'Configure UAR in Settings before use' : undefined}
            className="h-8 rounded-lg px-4 text-[color:var(--color-fg-sub)] data-pressed:bg-[color:var(--color-ember)] data-pressed:text-white"
          >
            {option.label}
          </ToggleGroupItem>
        ))}
      </ToggleGroup>

      {/* Progress and throughput are announced politely — they update mid-run
          and shouldn't interrupt a screen-reader user's current context. */}
      {isLoading && (
        <span aria-live="polite" className="text-[color:var(--color-fg)] opacity-70">
          {loadProgress.text}{loadProgress.progress > 0 ? ` · ${Math.round(loadProgress.progress * 100)}%` : ''}
        </span>
      )}
      {isLoading && (
        <Button type="button" variant="secondary" className="min-h-11 min-w-11 rounded-lg px-3" onClick={() => void cancelPreparation()}>
          Cancel
        </Button>
      )}

      {!isLoading && isLocal && throughput && (
        <span aria-live="polite" className="text-[color:var(--color-fg)] opacity-70">
          {/* Labelled an estimate: WebLLM reports no usage totals, so this counts
              streamed chunks (≈1 token each) rather than measuring exactly. */}
          ~{throughput.tokensPerSecond.toFixed(1)} tok/s
        </span>
      )}

      {error && (
        <span role="alert" className="max-w-[34rem] rounded-lg bg-[color:var(--color-ember-soft)] px-3 py-2 text-[color:var(--color-ember)]">
          {error}
        </span>
      )}
    </div>
  )
}
