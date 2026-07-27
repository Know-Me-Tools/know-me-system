// TJ-ARCH-MOB-001 compliant
//
// The four explicit intents on a stopped turn: Steer, Resume, Retry, Discard.
//
// Users stop a run *because* they are steering — they read the partial answer or
// a tool call and want a different direction. Every path after a stop used to be
// the same undifferentiated "send another message", so the app could not tell
// those four things apart. Steer and Resume lead because they KEEP the user's
// work; Retry and Discard are subordinate so the destructive option stops
// competing for the tap.
//
// Ported from Flutter's `_StoppedTurnBar` (mobile chat_screen.dart) — same
// weighting, same copy, same "kept N blocks" reassurance.
import type { FC } from 'react'
import { PencilIcon, PlayIcon, RefreshCwIcon, Trash2Icon } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { useChatStore } from '../stores/chatStore'

interface StoppedTurnActionsProps {
  messageId: string
  /** How many content blocks survived the stop. Resume is pointless at zero. */
  keptBlocks: number
}

/** Light-theme contrast fix for a TEXT label on the ember primary.
 *
 *  `--primary-foreground` is #FFFFFF in light mode, which is 3.97:1 on ember —
 *  fine for the icon-only Send/Stop buttons that use this variant elsewhere
 *  (non-text UI needs 3:1) but short of the 4.5:1 that applies to these 14px
 *  labels. Dark text on the same ember is 4.84:1.
 *
 *  Scoped here rather than changed in the shared token, since every other
 *  consumer of the variant is icon-only and already compliant.
 */
const PRIMARY_TEXT_FIX = 'text-[#0B0F14] dark:text-[color:var(--primary-foreground)]'

/** Visible label, the accessible name, and the icon for one intent.
 *
 *  The accessible name carries the CONSEQUENCE, not just the verb — Retry is
 *  destructive in place, and a user who only hears "Retry" has not been told it
 *  discards the work this bar exists to preserve. Each visible label leads its
 *  aria-label so WCAG 2.5.3 Label in Name holds.
 */
const LABELS = {
  steer: 'Steer from here — write a new instruction, keeping the work so far',
  resume: 'Resume — continue from where it stopped',
  retry: 'Retry — replaces this response and discards the work so far',
  discard: 'Discard this stopped response',
} as const

export const StoppedTurnActions: FC<StoppedTurnActionsProps> = ({ messageId, keptBlocks }) => {
  const armSteer = useChatStore((s) => s.armSteer)
  const sendResume = useChatStore((s) => s.sendResume)
  const regenerateResponse = useChatStore((s) => s.regenerateResponse)
  const deleteResponse = useChatStore((s) => s.deleteResponse)

  return (
    // flex-wrap so four actions reflow instead of clipping at 200% zoom.
    // Flat 2.0: no border, no divider, no shadow — this row sits on the message
    // background and is distinguished by weight and position alone.
    <div className="flex flex-wrap items-center gap-2" data-testid="stopped-turn-actions">
      <Button
        size="default"
        variant="default"
        className={PRIMARY_TEXT_FIX}
        aria-label={LABELS.steer}
        // Steer ARMS the composer; it does not send. The correction the user
        // then types is what gets marked. A Steer that fired immediately would
        // just be a Resume with extra words.
        onClick={() => armSteer()}
      >
        <PencilIcon aria-hidden="true" />
        Steer
      </Button>

      {keptBlocks > 0 && (
        <Button
          size="default"
          variant="default"
          className={PRIMARY_TEXT_FIX}
          aria-label={LABELS.resume}
          onClick={() => void sendResume()}
        >
          <PlayIcon aria-hidden="true" />
          Resume
        </Button>
      )}

      <Button
        size="default"
        variant="ghost"
        aria-label={LABELS.retry}
        onClick={() => void regenerateResponse(messageId)}
      >
        <RefreshCwIcon aria-hidden="true" />
        Retry
      </Button>

      <Button
        size="default"
        variant="ghost"
        aria-label={LABELS.discard}
        onClick={() => {
          if (window.confirm('Discard this stopped response and the later messages on this branch?')) {
            void deleteResponse(messageId)
          }
        }}
      >
        <Trash2Icon aria-hidden="true" />
        Discard
      </Button>
    </div>
  )
}
