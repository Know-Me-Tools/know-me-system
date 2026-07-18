// TJ-ARCH-MOB-001 compliant — component imports the hook only.
import { useEffect } from 'react'
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import { useLane } from '../hooks/useLane'

/**
 * Cloud/local inference toggle, plus the honest cost of running locally:
 * download progress while weights load, tok/s once a local run completes.
 *
 * Renders nothing when no local lane exists (no Rust engine, no WebGPU) — an
 * absent capability is better hidden than shown broken.
 */
export function LaneSwitcher() {
  const { lane, localAvailable, loadProgress, throughput, error, switchLane, initLane } = useLane()

  useEffect(() => {
    void initLane()
  }, [initLane])

  if (!localAvailable) return null

  const isLoading = loadProgress !== null
  const isLocal = lane === 'local'

  return (
    <div className="flex min-h-12 items-center gap-3 bg-[color:var(--color-bg-2)] px-4 py-2 text-xs">
      <ToggleGroup
        aria-label="Inference lane"
        value={[lane]}
        disabled={isLoading}
        onValueChange={(values) => {
          const next = values.at(-1)
          if (next === 'cloud' || next === 'local') void switchLane(next)
        }}
        className="rounded-xl bg-[color:var(--color-surface)] p-1"
      >
        {(['cloud', 'local'] as const).map((option) => (
          <ToggleGroupItem
            key={option}
            value={option}
            aria-label={option === 'cloud' ? 'Cloud' : 'On-device'}
            className="h-7 rounded-lg px-3 text-[color:var(--color-fg-sub)] data-pressed:bg-[color:var(--color-ember)] data-pressed:text-white"
          >
            {option === 'cloud' ? 'Cloud' : 'On-device'}
          </ToggleGroupItem>
        ))}
      </ToggleGroup>

      {/* Progress and throughput are announced politely — they update mid-run
          and shouldn't interrupt a screen-reader user's current context. */}
      {isLoading && (
        <span aria-live="polite" className="text-[color:var(--color-fg)] opacity-70">
          Downloading model… {Math.round(loadProgress.progress * 100)}%
        </span>
      )}

      {!isLoading && isLocal && throughput && (
        <span aria-live="polite" className="text-[color:var(--color-fg)] opacity-70">
          {/* Labelled an estimate: WebLLM reports no usage totals, so this counts
              streamed chunks (≈1 token each) rather than measuring exactly. */}
          ~{throughput.tokensPerSecond.toFixed(1)} tok/s
        </span>
      )}

      {error && (
        <span role="alert" className="text-[color:var(--color-ember)]">
          {error}
        </span>
      )}
    </div>
  )
}
