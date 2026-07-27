// TJ-ARCH-MOB-001 compliant — component consumes the hook, not the store.
//
// The honest cost of running locally: download progress while weights load, a
// way to cancel it, and throughput once a local run completes.
//
// These three readouts used to live on `LaneSwitcher`, alongside a UAR/on-device
// toggle that Flutter has no equivalent for and that desynced the lane from the
// conversation's mode. The toggle is gone; the readouts are not, because a
// multi-gigabyte download with no visible progress is the app appearing to hang.
// Flutter shows the same information in its model-operation banner.
import { Button } from '@/components/ui/button'
import { useLane } from '../hooks/useLane'

export function LaneStatus() {
  const { lane, loadProgress, throughput, error, cancelPreparation } = useLane()

  const isLoading = loadProgress !== null
  const showThroughput = !isLoading && lane === 'local' && throughput

  // Render nothing rather than an empty bar: a persistent strip of blank
  // background reads as a broken layout, and this is idle most of the time.
  if (!isLoading && !showThroughput && !error) return null

  return (
    <div className="flex min-h-12 flex-wrap items-center gap-3 bg-[color:var(--color-bg-2)] px-4 py-2 text-xs">
      {/* Announced politely — these update mid-run and must not interrupt a
          screen-reader user's current context. */}
      {isLoading ? (
        <span aria-live="polite" className="text-[color:var(--color-fg)] opacity-70">
          {loadProgress.text}
          {loadProgress.progress > 0 ? ` · ${Math.round(loadProgress.progress * 100)}%` : ''}
        </span>
      ) : null}

      {isLoading ? (
        <Button
          type="button"
          variant="secondary"
          className="min-h-11 min-w-11 rounded-lg px-3"
          onClick={() => void cancelPreparation()}
        >
          Cancel
        </Button>
      ) : null}

      {showThroughput ? (
        <span aria-live="polite" className="text-[color:var(--color-fg)] opacity-70">
          {/* Labelled an estimate: WebLLM reports no usage totals, so this counts
              streamed chunks (≈1 token each) rather than measuring exactly. */}
          ~{throughput.tokensPerSecond.toFixed(1)} tok/s
        </span>
      ) : null}

      {error ? (
        <span
          role="alert"
          className="max-w-[34rem] rounded-lg bg-[color:var(--color-ember-soft)] px-3 py-2 text-[color:var(--color-ember)]"
        >
          {error}
        </span>
      ) : null}
    </div>
  )
}
