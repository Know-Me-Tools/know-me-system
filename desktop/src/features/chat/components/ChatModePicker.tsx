// TJ-ARCH-MOB-001 compliant — component consumes hooks, never the store directly.
//
// One control that answers "what is this conversation running on?" without
// being opened, and opens to the full list of choices: every agent defined in
// the embedded UAR, plus Local, which talks to the on-device model directly.
//
// Ported from Flutter's `_chooseMode` sheet + `_ModeChoiceTile`
// (mobile/lib/features/chat/presentation/widgets/chat_policy_toolbar.dart) —
// same copy, same blocked-with-reason contract, same "never colour alone"
// selection semantics. The Flutter version uses a bottom sheet on a phone; on
// desktop that reads as a modal interruption for a routine choice, so this uses
// a dropdown at pointer widths and falls back to a bottom drawer on narrow
// viewports where a dropdown would be a cramped popover.
import { useEffect, useRef, useState } from 'react'
import { Cpu, Network } from 'lucide-react'

import { Button } from '@/components/ui/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import {
  Drawer,
  DrawerContent,
  DrawerDescription,
  DrawerHeader,
  DrawerTitle,
  DrawerTrigger,
} from '@/components/ui/drawer'
import { useChat } from '../hooks/useChat'
import { useChatControl } from '../hooks/useChatControl'
import type { AgentOption, ChatMode } from '../stores/chatControlStore'

/** Verbatim from Flutter — one wording per concept across both surfaces. */
const HEADING = 'Conversation mode'
const SUBTITLE =
  'Agents run through the Universal Agent Runtime. Local talks to the on-device model directly.'
const LOCAL_LABEL = 'Local'
const LOCAL_SUBTITLE = 'On-device model, no agent runtime'

/** The `sm` breakpoint. Below it a dropdown becomes a cramped popover. */
const DRAWER_MAX_WIDTH = 600

/** One selectable mode. */
interface ModeChoice {
  key: string
  label: string
  subtitle?: string
  mode: ChatMode
  agentId?: string
  selected: boolean
  /** Present when the runtime cannot serve this mode; also the reason shown. */
  blockedReason?: string
}

/**
 * Label for the trigger: the resolved agent's NAME, or "Local".
 *
 * Showing the agent by name is the point of a single trigger — it answers what
 * the conversation runs on without being opened. Mirrors `_modeLabel`.
 */
export function modeLabel(mode: ChatMode, agentId: string | null, agents: AgentOption[]): string {
  if (mode === 'local') return LOCAL_LABEL
  const match = agents.find((agent) => agent.id === agentId)
  if (match) return match.label
  return agentId ? agentId : 'Orchestrator'
}

/**
 * Build the full choice list: every catalog agent, then Local.
 *
 * Deliberately no cloud entry — Flutter offers none, and the two surfaces must
 * present the same set or a comparison between them means nothing.
 */
export function modeChoices(
  agents: AgentOption[],
  mode: ChatMode,
  agentId: string | null,
  laneAvailability: Partial<Record<string, string>>,
): ModeChoice[] {
  const uarBlocked = laneAvailability.uar
  const localBlocked = laneAvailability.local
  return [
    ...agents.map((agent) => ({
      key: agent.id,
      label: agent.label,
      // The Orchestrator row is mode `uar`; every other agent is mode `agent`
      // carrying its own id. Both resolve to the UAR lane.
      mode: (agent.id === 'orchestrator-agent' ? 'uar' : 'agent') as ChatMode,
      agentId: agent.id,
      // Selected when this SPECIFIC agent is the conversation's agent — not
      // merely when the mode happens to be agentic.
      selected: mode !== 'local' && agentId === agent.id,
      ...(uarBlocked ? { blockedReason: uarBlocked } : {}),
    })),
    {
      key: 'local',
      label: LOCAL_LABEL,
      subtitle: LOCAL_SUBTITLE,
      mode: 'local' as ChatMode,
      selected: mode === 'local',
      ...(localBlocked ? { blockedReason: localBlocked } : {}),
    },
  ]
}

/** True while the viewport is narrow enough that a drawer beats a dropdown. */
function useIsNarrow(): boolean {
  const [narrow, setNarrow] = useState(
    () => typeof window !== 'undefined' && window.innerWidth < DRAWER_MAX_WIDTH,
  )
  useEffect(() => {
    const query = window.matchMedia(`(max-width: ${DRAWER_MAX_WIDTH - 1}px)`)
    const sync = () => setNarrow(query.matches)
    sync()
    query.addEventListener('change', sync)
    return () => query.removeEventListener('change', sync)
  }, [])
  return narrow
}

interface ModeRowProps {
  choice: ModeChoice
  onSelect: () => void
}

/**
 * One row in the picker.
 *
 * Selection is conveyed by `aria-checked` and a filled radio mark — never by
 * colour alone. A blocked row is disabled and shows the runtime's reason IN
 * PLACE OF its subtitle, so the user learns why instead of clicking a dead
 * control. 44px minimum height meets WCAG 2.5.8; Flutter's floor is 48.
 */
function ModeRow({ choice, onSelect }: ModeRowProps) {
  const blocked = Boolean(choice.blockedReason)
  const detail = choice.blockedReason ?? choice.subtitle
  return (
    <button
      type="button"
      role="menuitemradio"
      aria-checked={choice.selected}
      aria-disabled={blocked}
      disabled={blocked}
      onClick={onSelect}
      className={
        blocked
          // Dimmed, but still legible — the reason is the point, and a user who
          // cannot read why a mode is unavailable has learned nothing. Matches
          // Flutter, which dims blocked rows to mutedForeground rather than
          // fading them out.
          ? 'flex min-h-11 w-full items-center gap-3 rounded-lg px-3 py-2 text-left text-[color:var(--color-fg-sub)]'
          : 'flex min-h-11 w-full items-center gap-3 rounded-lg px-3 py-2 text-left text-[color:var(--color-fg)] hover:bg-[color:var(--color-card-hov)]'
      }
    >
      {/* Radio mark drawn from currentColor so it survives forced-colors mode,
          where a background-only indicator would vanish. */}
      <span
        aria-hidden
        className="flex size-5 shrink-0 items-center justify-center rounded-full border-2 border-current"
      >
        {choice.selected ? <span className="size-2.5 rounded-full bg-current" /> : null}
      </span>
      <span className="min-w-0">
        <span className="block truncate text-sm font-medium">{choice.label}</span>
        {detail ? (
          // `--color-fg-sub`, not `--color-fg-faint`: faint is 3.18:1 on the
          // card in dark mode, under the 4.5:1 floor for 12px text. This line
          // carries the blocked REASON — the row's whole purpose — so it has to
          // be readable, not decorative. fg-sub is 7.02:1 dark / 7.56:1 light.
          <span className="block truncate text-xs text-[color:var(--color-fg-sub)]">{detail}</span>
        ) : null}
      </span>
    </button>
  )
}

function PickerBody({ choices, onSelect }: { choices: ModeChoice[]; onSelect: (choice: ModeChoice) => void }) {
  return (
    <div role="group" aria-label={HEADING} className="space-y-0.5">
      {choices.map((choice) => (
        <ModeRow key={choice.key} choice={choice} onSelect={() => onSelect(choice)} />
      ))}
    </div>
  )
}

export function ChatModePicker() {
  const { activeConversationId } = useChat()
  const control = useChatControl(activeConversationId)
  const narrow = useIsNarrow()
  // Deliberately UNCONTROLLED. A controlled `open` here fought Base UI's own
  // menu state machine and swallowed keyboard activation entirely: Enter on the
  // trigger left aria-expanded false, so the picker was pointer-only — a WCAG
  // 2.1.1 failure. Base UI already closes the menu when an item is chosen.

  const mode: ChatMode = control.policy.chat_mode ?? 'uar'
  const agentId = control.policy.agent_id ?? null
  const label = modeLabel(mode, agentId, control.agents)
  const choices = modeChoices(control.agents, mode, agentId, control.laneAvailability)
  const Icon = mode === 'local' ? Cpu : Network
  // The radio group is keyed by choice, so the selected row is the selected key.
  const selectedKey = choices.find((choice) => choice.selected)?.key ?? ''

  // Set by the keyboard handler so the click it also generates is ignored.
  const keyboardActivated = useRef(false)

  function choose(choice: ModeChoice) {
    void control.setMode(choice.mode, choice.agentId)
  }

  const trigger = (
    <Button
      variant="ghost"
      disabled={control.saving}
      // The accessible name carries both the control's purpose and its current
      // value — "Orchestrator" alone tells a screen-reader user nothing about
      // what the button does.
      aria-label={`${HEADING}: ${label}`}
      className="min-h-11 shrink-0 rounded-xl bg-[color:var(--color-surface)] px-3 text-xs text-[color:var(--color-fg)]"
    >
      <Icon aria-hidden className="size-4" />
      <span className="max-w-[12rem] truncate">{label}</span>
    </Button>
  )

  if (narrow) {
    return (
      <Drawer>
        <DrawerTrigger render={trigger} />
        <DrawerContent className="border-0 bg-[color:var(--color-card)] shadow-none">
          <DrawerHeader>
            <DrawerTitle>{HEADING}</DrawerTitle>
            <DrawerDescription>{SUBTITLE}</DrawerDescription>
          </DrawerHeader>
          <div className="max-h-[60vh] overflow-y-auto px-3 pb-6">
            <PickerBody choices={choices} onSelect={choose} />
          </div>
        </DrawerContent>
      </Drawer>
    )
  }

  return (
    <DropdownMenu>
      <DropdownMenuTrigger render={trigger} />
      <DropdownMenuContent
        align="start"
        className="max-h-[26rem] w-80 overflow-y-auto border-0 bg-[color:var(--color-card)] p-2 shadow-none ring-0"
      >
        <div className="px-3 pb-2">
          <p className="text-sm font-semibold text-[color:var(--color-fg)]">{HEADING}</p>
          <p className="mt-1 text-xs text-[color:var(--color-fg-sub)]">{SUBTITLE}</p>
        </div>
        {/* The library's own radio group, not hand-rolled `role="menuitemradio"`
            buttons. Base UI drives roving focus through ITS item components, so
            raw buttons rendered a menu the arrow keys could not move within —
            open but keyboard-inert. The primitives supply the role, aria-checked
            and focus management; the visual row stays ours. */}
        <DropdownMenuRadioGroup value={selectedKey}>
          {choices.map((choice) => (
            <DropdownMenuRadioItem
              key={choice.key}
              value={choice.key}
              disabled={Boolean(choice.blockedReason)}
              // Radio items default to closeOnClick:false so one menu can hold
              // several groups. This menu is a single either/or choice, so
              // leaving it open after a pick reads as the click not registering.
              closeOnClick
              // Keyboard activation, explicitly.
              //
              // Base UI's button layer routes Enter on a non-native element to
              // the item's EXTERNAL onClick, but MenuRadioItem installs its own
              // onClick earlier in the merge chain and that one wins — so Enter
              // reached nothing, and a row was arrow-reachable yet impossible
              // to activate (WCAG 2.1.1). onKeyDown is passed through ahead of
              // the library's handling, so it is the hook that reliably fires.
              //
              // Both handlers are required and they overlap: Enter also
              // produces a click here, so the two together fired `choose` TWICE
              // — two lane switches and two policy writes for one pick. The
              // keyboard handler claims the activation and the click handler
              // stands down for it, rather than dropping either one (keyboard
              // alone breaks the mouse; click alone is the original bug).
              onKeyDown={(event) => {
                if (event.key !== 'Enter' && event.key !== ' ') return
                event.preventDefault()
                keyboardActivated.current = true
                choose(choice)
              }}
              onClick={() => {
                if (keyboardActivated.current) {
                  keyboardActivated.current = false
                  return
                }
                choose(choice)
              }}
              className={
                choice.blockedReason
                  ? 'min-h-11 gap-3 px-3 py-2 text-[color:var(--color-fg-sub)]'
                  : 'min-h-11 gap-3 px-3 py-2 text-[color:var(--color-fg)]'
              }
            >
              <span className="min-w-0">
                <span className="block truncate text-sm font-medium">{choice.label}</span>
                {choice.blockedReason ?? choice.subtitle ? (
                  <span className="block truncate text-xs text-[color:var(--color-fg-sub)]">
                    {choice.blockedReason ?? choice.subtitle}
                  </span>
                ) : null}
              </span>
            </DropdownMenuRadioItem>
          ))}
        </DropdownMenuRadioGroup>
      </DropdownMenuContent>
    </DropdownMenu>
  )
}
