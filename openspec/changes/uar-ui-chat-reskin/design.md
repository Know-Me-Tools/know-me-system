## Context

`uar-ui-token-convergence` established the KnowMe token set and Flat 2.0 enforcement in the UAR frontend. The chat surface still renders its pre-standard design: bordered glassy composer with a shimmer streaming bar, avatar-led message bubbles with `shadow-sm`, zinc-palette hardcodes, and bordered cards for reasoning/tools/errors. The binding target is `docs/knowme-ui-ux-standard.md` §7 (chat anatomy) implemented in `desktop/src/components/assistant-ui/thread.tsx` (488 lines) and `desktop/src/features/chat/components/RichDataRenderers.tsx`.

Both surfaces use assistant-ui primitives (`ThreadPrimitive`, `ComposerPrimitive`, `MessagePrimitive`, `ActionBarPrimitive`, `BranchPickerPrimitive`), so the reskin is a className/structure port, not a library change. UAR extras that KnowMe lacks (agent status indicator, context usage bar, tool approval dialog, A2UI input/display blocks) are kept and re-tokened, not removed.

## Goals / Non-Goals

**Goals:**

- Chat reads as the KnowMe standard: assistant prose as authored content on canvas, user messages as ember-tinted trailing bubbles, composer as an anchored filled surface, no visible borders/shadows anywhere in the thread.
- Reasoning collapsed by default on a cyan-tinted surface; tool calls grouped and collapsed; part renderers follow the KnowMe ContentBlock presentation table (§7.5).
- Motion limited to short opacity/background transitions; no shimmer bar, no decorative animation; `prefers-reduced-motion` honored.
- All color via tokens — zero `zinc-*`/hardcoded palette classes in the chat feature.

**Non-Goals:**

- No changes to `use-chat-runtime.ts`, stores, services, or AG-UI/A2UI event handling.
- No mapping of UAR message parts onto the closed 11-variant ContentBlock union (protocol decision deferred — see assessment).
- No shell/navigation work (`uar-ui-shell-navigation`), no a2ui-uar package re-token (`uar-ui-a2ui-retoken` beyond what the chat-embedded artifact block needs).
- No full verification loop (screenshot matrix, e2e) — that is `uar-ui-verify-gates`; this change does run its own build/typecheck/lint and a manual two-theme smoke pass.

## Decisions

### Thread shell mirrors the KnowMe layout contract

Adopt the KnowMe `ThreadRoot` structure: `ThreadPrimitive.Root` with `--thread-max-width: 44rem` (replaces the current 48rem `--aui-thread-max-width` in `index.css`), viewport with `turnAnchor="top"`, message group `gap-y-6`, sticky `ViewportFooter` holding scroll-to-bottom + suggestions + composer. Keep UAR's `ThreadAgentStatus` indicator but move it inside the max-width column and re-style it as a mono status line (no chip borders).

### Composer is a filled surface, not an outlined input

Replace the current `border border-input bg-background/80 backdrop-blur` dropzone with the KnowMe shell: `rounded-[1.5rem] bg-[hsl(var(--surface))] p-[10px] shadow-none transition-colors focus-within:bg-[hsl(var(--card-hov))]`. Input keeps `caret-primary`, placeholder `text-fg-faint`. Delete the shimmer streaming bar; streaming state is already conveyed by the message indicator and the cancel button. Send/cancel stay round ember buttons (`size-7/8 rounded-full`), attachment and dictation actions keep their positions per KnowMe `ComposerAction`.

### User messages: trailing ember-soft, no avatar

Adopt KnowMe `UserMessage` anatomy: grid with `grid-cols-[minmax(72px,1fr)_auto]`, bubble `max-w-[80%] bg-ember-soft rounded-2xl rounded-ee-md px-4 py-3`, action bar absolutely placed beside the bubble (`copy / resubmit / edit`), branch picker on the row below. Remove `UserAvatar` and the `bg-zinc-800 ... shadow-sm` bubble.

### Assistant messages: authored prose on canvas, grouped parts

Adopt KnowMe `AssistantMessage`: no avatar; `MessagePrimitive.Parts` (UAR's part-component map is preserved) wrapped so **text parts** render as `w-fit max-w-[80%] rounded-2xl rounded-es-md bg-card px-4 py-3`; consecutive reasoning parts collapse into one cyan-tinted `ReasoningRoot` group (collapsed by default, streaming state shows animated trigger); consecutive tool calls collapse into a `ToolGroupRoot` with a count trigger. Footer row holds branch picker + action bar (`copy / regenerate / delete / more→export-markdown`). `MessageError` becomes `bg-destructive/10 text-destructive rounded-md p-3` — no border, no Card.

UAR's runtime does not use `MessagePrimitive.GroupedParts` today (it maps parts directly); implement the grouping with `groupPartByType` exactly as the KnowMe reference does, keeping UAR's existing custom part components registered for their part types.

### Bespoke part renderers follow the ContentBlock presentation table

Per KnowMe §7.5 and `RichDataRenderers.tsx`:

| UAR component | Restyle |
|---|---|
| `memory-chunk-block` (recall/mutation/update) | cyan `eyebrow` label on `bg-surface` rounded-xl p-3 |
| `context-update-block` | `bg-surface` rounded-xl, mono meta, no border |
| `skill-activation-block` | amber eyebrow + sparkles on `bg-surface` |
| `citation-hover-panel` | ember eyebrow, expandable card on `bg-card`, inline marker |
| `a2ui-artifact-block` (display/input) | on-card surface, SVG sanitized via DOMPurify (verify existing sanitization; add if absent) |
| `tool-call-block` | superseded by ToolFallback/ToolGroup idiom inside the thread; keep the standalone component (admin reuses it) but re-token |
| `attachment-preview` | filled muted chips, no outline |
| `ContextUsageBar` | muted track, ember/cyan fill, mono percentage label |
| `ToolApprovalDialog` | dialog on `--color-card` with scrim only (no shadow/border), ember primary action |

### Hardcoded palette utilities are banned in the chat feature

Replace every `zinc-*`, `stone-*`, raw hex, and `shadow-*` class in `frontend/src/components/assistant-ui/` and `frontend/src/features/chat/` with token utilities. Add a grep check to the change's verification (`zinc-|stone-|shadow-(sm|md|lg|xl|2xl)` must return zero hits in those paths, excluding test fixtures).

### Motion budget

Only `fade-in`/`slide-in-from-bottom` entrances (already present, ≤200ms), opacity pulses for streaming indicators, and background-color transitions. Remove the composer shimmer keyframe usage; keep the keyframe definition only if admin uses it.

## Risks / Trade-offs

- **GroupedParts behavior change**: grouping reasoning/tool parts changes message DOM structure; UAR's Playwright selectors and any tests targeting part order may need updates. Mitigation: run the existing e2e in this change and fix selectors as part of the work.
- **assistant-ui version skew**: KnowMe desktop and UAR both pin `@assistant-ui/react` 0.14.x, but verify `groupPartByType`, `ToolGroupRoot`-style components exist in UAR's exact version before committing to the grouping approach; if absent, port grouping manually (filter consecutive same-type parts) — same visual result.
- **Removing avatars** drops the agent-identity cue; mitigated by the model/agent name already present in `MetaChip` and the agent status line.
- **Scope containment**: `enhanced-thread.tsx` is 748 lines with many condition helpers; rework incrementally (shell → composer → user → assistant → parts) with a build after each step rather than one rewrite.
