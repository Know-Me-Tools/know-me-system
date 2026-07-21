## Why

The UAR chat surface (`frontend/src/components/assistant-ui/enhanced-thread.tsx`, 748 lines, plus the `features/chat/components/*` part renderers) predates the KnowMe UI standard: bordered composer with backdrop-blur and a shimmer streaming bar, avatar-led bubbles with elevation shadows (`bg-zinc-800` / `bg-muted/60` + `shadow-sm`), bespoke reasoning/tool cards with visible borders, and hardcoded zinc palette utilities that bypass the token system. With `uar-ui-token-convergence` landed, the chat surface is now the most visible violator of the standard — and the surface users actually look at.

## What Changes

- Re-skin the thread shell to KnowMe idioms (reference: `desktop/src/components/assistant-ui/thread.tsx`): thread max-width 44rem, sticky viewport footer, scroll-to-bottom pill, follow-up suggestion pills, centered welcome state.
- Rebuild the composer as a filled surface: `rounded-1.5rem` filled `--color-surface` shell, `focus-within` → `--color-card-hov`, no border/blur/shadow, ember caret, `--fg-faint` placeholder. Remove the shimmer streaming bar and border-ring focus (violates Flat 2.0 and KnowMe motion rules).
- Restyle user messages: remove avatar, trailing-aligned `bg-ember-soft rounded-2xl rounded-ee-md max-w-[80%]` bubble, hover action bar (copy/resubmit/edit) beside the bubble, branch picker below.
- Restyle assistant messages: remove avatar, grouped-parts layout with text on `bg-card rounded-2xl rounded-es-md`, footer with branch picker + action bar (copy/regenerate/delete/more), destructive error surface without borders.
- Rework reasoning and tool calls to the KnowMe collapsed-group idiom: reasoning = collapsed-by-default cyan-tinted group with streaming state; consecutive tool calls collapse into a ToolGroup; individual calls render via a ToolFallback-style card on the surface ladder.
- Re-style the bespoke UAR part renderers to the KnowMe ContentBlock presentation table: memory (cyan eyebrow on `--color-surface`), context-update (surface), skill activation (amber eyebrow), citation hover panel (ember eyebrow, expandable card), A2UI artifact (sanitized, on-card), attachment previews, `MetaChip` model/status chips (mono eyebrow on muted).
- Replace hardcoded `zinc` palette utilities with tokens; purge remaining `shadow-*`/border utilities in the chat feature per Flat 2.0.
- **No protocol, runtime, store, or wire-event changes.** Presentation only; `useExternalStoreRuntime` bridging and AG-UI/A2UI event handling are untouched.

## Capabilities

### New Capabilities

- `uar-chat-surface`: The KnowMe-idiom chat thread — composer, message anatomy, reasoning/tool grouping, and part presentation — for the UAR web UI.

### Modified Capabilities

- `uar-web-ui-design-tokens`: chat components consume the ported token set; remaining hardcoded palette utilities in the chat feature are removed.

## Impact

- `frontend/src/components/assistant-ui/enhanced-thread.tsx` (major rework), `enhanced-markdown-text.tsx` (minor), `frontend/src/features/chat/components/*` (tool-call-block, memory-chunk-block, context-update-block, skill-activation-block, a2ui-artifact-block, attachment-preview, ToolApprovalDialog, ContextUsageBar, AgentStatusIndicator), `frontend/src/components/citations/citation-hover-panel.tsx`.
- Reference sources (read-only): `desktop/src/components/assistant-ui/*`, `desktop/src/features/chat/components/RichDataRenderers.tsx`, `docs/knowme-ui-ux-standard.md` §7.
- Depends on `uar-ui-token-convergence` (landed: `bg-ember-soft`, `card-hov`, `fg-faint`, `cyan` utilities exist).
- Skill gates: `reference-ui-fidelity` (target = KnowMe desktop chat), `content-block-ui` (part-presentation parity), `a11y-gate`.
- Submodule discipline: feature branch in `rust/vendor/universal-agent-runtime`; parent repo gets a pointer bump after verification.
