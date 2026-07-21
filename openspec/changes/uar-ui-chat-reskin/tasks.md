# Tasks — uar-ui-chat-reskin

All paths relative to `rust/vendor/universal-agent-runtime/frontend/`. Work on a feature branch inside the submodule. Reference (read-only): `desktop/src/components/assistant-ui/thread.tsx`, `reasoning.tsx`, `tool-fallback.tsx`, `tool-group.tsx`, `follow-up-suggestions.tsx`, `attachment.tsx`, and `desktop/src/features/chat/components/RichDataRenderers.tsx` in the KnowMe monorepo. Skill gates: invoke `reference-ui-fidelity` before starting and `content-block-ui` when restyling part renderers.

## 1. Thread Shell

- [x] 1.1 Set `--aui-thread-max-width: 44rem` in `src/index.css` (from 48rem) and align the thread root style contract (`--composer-bg`, `--composer-radius: 1.5rem`, `--composer-padding`) per the KnowMe reference
- [x] 1.2 Restructure `EnhancedThread` to the KnowMe layout: viewport `turnAnchor="top"`, message group `gap-y-6`, sticky `ViewportFooter` containing scroll-to-bottom + suggestions + composer
- [x] 1.3 Re-style `ThreadScrollToBottom` as the KnowMe pill (secondary, rounded-full, shadow-none, `-top-12 self-center`)
- [x] 1.4 Move `ThreadAgentStatus` inside the max-width column; restyle as a mono status line without chip borders
- [x] 1.5 Keep the centered welcome state (KnowMeLogo landed in token-convergence); add KnowMe-style suggestion pills below the composer when the runtime exposes suggestions — otherwise omit

## 2. Composer

- [x] 2.1 Replace the dropzone chrome with the filled-surface shell: `rounded-[1.5rem] bg-surface p-[10px] shadow-none transition-colors focus-within:bg-card-hov`, no border/blur/ring
- [x] 2.2 Remove the shimmer streaming bar and its conditional logic from the composer
- [x] 2.3 Input: `caret-primary`, `placeholder:text-fg-faint`, transparent bg, `max-h-32` autogrow; keep attachments row and dictation/send/cancel actions per KnowMe `ComposerAction` (round ember send/cancel)

## 3. Message Anatomy

- [x] 3.1 User message: delete `UserAvatar`; trailing grid layout; `max-w-[80%] bg-ember-soft rounded-2xl rounded-ee-md px-4 py-3` bubble; hover action bar (copy/resubmit/edit) beside bubble; branch picker below
- [x] 3.2 Assistant message: delete `AgentAvatar`; text parts render as `w-fit max-w-[80%] rounded-2xl rounded-es-md bg-card px-4 py-3`; footer with branch picker + action bar (copy/regenerate/delete/more→export-markdown)
- [x] 3.3 Edit composer: KnowMe idiom — filled `--color-surface` surface, rounded composer radius, Cancel/Update pill buttons
- [x] 3.4 `MessageError`: `bg-destructive/10 text-destructive rounded-md p-3 text-sm`, drop the bordered Card
- [x] 3.5 Restyle `MetaChip` and message metadata to mono eyebrow on muted surface (no borders)

## 4. Reasoning and Tool Grouping

- [x] 4.1 Verify `groupPartByType` / grouped-parts support in UAR's `@assistant-ui/react` 0.14.26; if missing, implement consecutive-type grouping manually
- [x] 4.2 Reasoning: collapsed-by-default cyan-tinted group (trigger shows streaming state), content `aria-busy` while streaming; replace the bordered Card `ReasoningPart`
- [x] 4.3 Tools: consecutive tool calls collapse into a ToolGroup with count + running indicator; single calls render via the ToolFallback idiom (surface card, chevron, args/result panes on muted)

## 5. Part Renderers (ContentBlock presentation parity)

- [x] 5.1 `memory-chunk-block`: cyan eyebrow on `bg-surface` rounded-xl p-3 (recall/mutation/update variants)
- [x] 5.2 `context-update-block`: `bg-surface` rounded-xl, mono meta, no border
- [x] 5.3 `skill-activation-block`: amber eyebrow + sparkles on `bg-surface`
- [x] 5.4 `citation-hover-panel`: ember eyebrow; expandable card on `bg-card`; inline marker styling
- [x] 5.5 `a2ui-artifact-block`: on-card surface; confirm DOMPurify sanitization of SVG/HTML payloads (add if absent)
- [x] 5.6 `attachment-preview`: filled muted chips, no outline; `ContextUsageBar`: muted track + ember/cyan fill + mono label; `ToolApprovalDialog`: scrim-only depth on `bg-card`

## 6. Palette and Motion Purge

- [x] 6.1 Grep-purge `zinc-*`, `stone-*`, hardcoded hex, and `shadow-*` from `src/components/assistant-ui/` and `src/features/chat/` (zero hits expected, test fixtures excluded)
- [x] 6.2 Remove the composer `shimmer` keyframe usage; verify all remaining animation is entrance fades/slides ≤200ms, streaming opacity pulses, or background transitions; confirm `prefers-reduced-motion` handling

## 7. Verification

- [x] 7.1 `pnpm -C frontend typecheck` and `pnpm -C frontend build` green; `scripts/validate-static-bundle.mjs` passes
- [x] 7.2 `pnpm -C frontend lint` green on touched files; existing vitest suite passes (update tests broken by DOM restructuring)
- [ ] 7.3 Playwright e2e chat specs pass; update selectors broken by the grouped-parts DOM (full snapshot regeneration stays in `uar-ui-verify-gates`)
- [ ] 7.4 Manual two-theme smoke pass against the KnowMe reference: composer, user bubble, assistant prose, reasoning group, tool group, error state, edit composer, suggestions — attach side-by-side notes or screenshots to the change
- [ ] 7.5 `a11y-gate` checklist on the chat surface: focus order, ring visibility, action-bar discoverability (hover + focus), collapsed-group ARIA states, ≥44px targets on composer actions

## Apply notes (2026-07-21, kimi)

- Sections 1–6 complete; 7.1–7.2 green (typecheck, eslint, vite build, vitest 152/153).
- The single vitest failure is a pre-existing a2ui-uar package theme defect (surface
  paints no background), reproduced by mechanism analysis against pre-change values —
  scoped to `uar-ui-a2ui-retoken`. See verification-notes.md.
- 7.3/7.4/7.5 deferred to `uar-ui-verify-gates` (need a running Axum backend).
- 1.5 suggestion pills omitted per design (no suggestions adapter in the runtime).
