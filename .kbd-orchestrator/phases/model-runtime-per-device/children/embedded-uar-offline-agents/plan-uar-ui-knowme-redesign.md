# PLAN: model-runtime-per-device › embedded-uar-offline-agents — UAR UI → KnowMe convergence (scoped)

Project: KnowMe
Date: 2026-07-21
OpenSpec available: YES (`openspec/` at project root)
Source assessment: `assessment-uar-ui-knowme-redesign.md` (kimi, 2026-07-21)
Operator decisions (2026-07-21): **port token values, keep Tailwind 3.4** · **scoped TanStack Query exemption** · **admin console keeps its scoped CRT theme**
Sequencing: runs as its own change set alongside the in-flight `embed-uar-mobile-offline` — that change touches
`crates/` (Rust kernel), this set touches only `frontend/` + `src-tauri/` window config, so there is no file-level
collision inside the submodule. Do not interleave commits in the same branch; use a separate worktree/branch.
Changes to implement: 5

## Scope cuts & trade-offs (sycophancy check S-03/S-07)

- **No Tailwind 4 migration** — deferred. KnowMe desktop is TW4-`@theme`-native; UAR stays on TW3 with ported values.
  Trade-off: the two apps' token *mechanisms* diverge (JS config vs CSS-first), so future token updates are a manual
  two-way sync until a shared token package is built. Accepted by operator for risk reduction.
- **TanStack Query stays** — a scoped exemption to blocking constraint `preserve-entity-management` is recorded for
  the vendored submodule (rationale: separate product surface; PEM already handles entity sync there). This exemption
  MUST be written into the change proposal so `/opsx:verify` does not fail the change on the blocking rule.
- **Admin console out of visual scope** — only token-plumbing fixes if the port breaks it; CRT theme untouched.
- **No message-protocol change** — UAR keeps its AG-UI/A2UI wire events and bespoke part types; this change aligns
  *presentation* to KnowMe's ContentBlock idioms, not the protocol. Mapping UAR parts onto the closed 11-variant
  union is a separate future decision (needs Rust-side `gen_ui_types` discussion).

## CHANGE LIST (ordered)

1. `uar-ui-token-convergence`: Port the KnowMe token system into the UAR frontend and purge dead configs
   - Scope: ui (styling infra)
   - Depends on: NONE
   - Recommended agent: Codex (focused, mechanical) or Kimi
   - Est. complexity: M
   - Complexity score: Medium
   - Model class: medium
   - Customer value: HIGH (unlocks every visual change)
   - Details: Rewrite `rust/vendor/universal-agent-runtime/frontend/src/index.css` to the KnowMe ladder
     (`#0B0F14/#111620/#161D29/#1C2535`, hover `#202B40`, muted `#253044`, ember `#FF6A3D`/`#E04E28`, ember-soft
     `#2B1A18`/`#FBE8E1`, cyan `#00C2DC`/`#0891B2`, status ramp), light theme via `body.light` override pattern,
     radius `0.75rem` + KnowMe scale, font stack Geist Variable body / Space Grotesk display / JetBrains Mono meta,
     `.eyebrow` utility. Add Flat 2.0 enforcement CSS (`--border: transparent`, global `border-transparent`,
     `box-shadow: none` on `.aui-root`, markdown border neutralization). Update `frontend/tailwind.config.ts` token
     bindings to match. Delete stale repo-root `tailwind.config.ts` + `postcss.config.cjs`. Swap wordmark/logo to the
     KnowMe monogram on product surfaces (keep UAR identity inside `/admin`). Gate: **hybrid-design-tokens** skill.

2. `uar-ui-chat-reskin`: Re-skin the chat surface to KnowMe thread/composer/message idioms
   - Scope: ui (chat feature)
   - Depends on: `uar-ui-token-convergence`
   - Recommended agent: Antigravity or Claude Code (complex multi-file UI)
   - Est. complexity: L
   - Complexity score: High
   - Model class: frontier
   - Customer value: HIGH (the surface users see)
   - Details: Rework `frontend/src/components/assistant-ui/enhanced-thread.tsx` (748 lines) to KnowMe idioms from
     `desktop/src/components/assistant-ui/thread.tsx`: thread max-width ~44rem, composer as filled
     `var(--color-surface)` surface radius 1.5rem with focus-within → `--color-card-hov`, user bubble
     `bg-ember-soft rounded-2xl rounded-ee-md` trailing-aligned, assistant prose on `--color-card` (no bubble stack),
     collapsed-by-default cyan-tinted reasoning groups, branch picker/edit composer/action bars/scroll-to-bottom per
     KnowMe thread. Re-style bespoke part renderers (`tool-call-block`, `memory-chunk-block`, `skill-activation-block`,
     `context-update-block`, `a2ui-artifact-block`, `attachment-preview`) to the KnowMe ContentBlock presentation
     table (cyan eyebrow memory, ember eyebrow citation, amber skill, DOMPurify-sanitized artifacts). Wire
     `--aui-thread-max-width`. Gates: **reference-ui-fidelity** (target: `desktop/` chat), **content-block-ui**.

3. `uar-ui-shell-navigation`: Adopt the KnowMe shell — single nav inventory, width-based rail/bottom nav, branded Tauri titlebar, status lane
   - Scope: ui (layout) + src-tauri window config
   - Depends on: `uar-ui-token-convergence`
   - Recommended agent: Antigravity or Claude Code
   - Est. complexity: M
   - Complexity score: Medium
   - Model class: medium
   - Customer value: MEDIUM
   - Details: Replace `components/layout/top-nav.tsx` + `left-sidebar.tsx` chrome with a KnowMe-style shell: single
     `DESTINATIONS` inventory (Threads / About / Admin) shared by a left rail on wide and a bottom bar under the
     compact width boundary (CSS-width based, no OS sniffing); active destination = ember-tinted background. Add a
     persistent model/lane/readiness status lane in the header (On-device / My server / Cloud distinct, never
     color-alone). Add the branded custom Tauri titlebar in `src-tauri` (window config + draggable region +
     min/max/close) per the **tauri-custom-titlebar** skill — note UAR's Tauri shell loads the in-process Axum URL,
     so the titlebar must live in the web UI chrome, not native decorations. Keep thread-list sidebar behavior
     (search/rename/delete) but re-skin to chrome/surface ladder. Gate: **mobile-navigation** skill rules.

4. `uar-ui-a2ui-retoken`: Re-token the `a2ui-uar` generative-UI renderer and data-part surfaces to KnowMe
   - Scope: ui (packages)
   - Depends on: `uar-ui-token-convergence`
   - Recommended agent: Codex or Kimi
   - Est. complexity: M
   - Complexity score: Medium
   - Model class: medium
   - Customer value: MEDIUM (agent-generated surfaces inherit the new system)
   - Details: Rebind the shadcn visual baseline in `frontend/packages/a2ui-uar/` (36 files) to the ported token set so
     agent-generated `uar.a2ui/1` surfaces render on the KnowMe ladder with Flat 2.0 enforcement. Align
     `features/a2ui/a2ui-surface-renderer.tsx` and `components/citations/citation-hover-panel.tsx` to the KnowMe
     citation/artifact presentation (expandable cards, ember eyebrow, sanitized SVG). Verify the A2UI testing admin
     page still renders fixtures correctly (visual check only; admin theme stays CRT).

5. `uar-ui-verify-gates`: Run the full KnowMe UI gate suite and refresh UAR's visual-test baselines
   - Scope: ui (verification)
   - Depends on: `uar-ui-chat-reskin`, `uar-ui-shell-navigation`, `uar-ui-a2ui-retoken`
   - Recommended agent: Kimi (verification loop) + Manual sign-off
   - Est. complexity: M
   - Complexity score: Medium
   - Model class: medium
   - Customer value: HIGH (certifies the redesign)
   - Details: Run **tauri-ui-review** screenshot loop at 320/768/1024/1440 in both themes against the quality bar;
     **a11y-gate** WCAG 2.2 AA checklist (4.5:1 body / 3:1 large text, focus rings ember, ≥44px targets,
     `prefers-reduced-motion`, 200% text scaling); regenerate Storybook 10 visual baselines and Playwright e2e
     snapshots; verify browser + Docker + Tauri webview all serve the same reskinned bundle from `static/`;
     update UAR docs (`frontend/README` or docs page) recording the TW3-port decision, the TanStack Query exemption,
     and the admin-theme exception.

## EXECUTION ROUND ORDER

Round 1: `uar-ui-token-convergence`
Round 2 (parallel, separate worktrees): `uar-ui-chat-reskin`, `uar-ui-shell-navigation`, `uar-ui-a2ui-retoken`
Round 3: `uar-ui-verify-gates`

## COMMANDS TO RUN

```
/opsx:new uar-ui-token-convergence
/opsx:new uar-ui-chat-reskin
/opsx:new uar-ui-shell-navigation
/opsx:new uar-ui-a2ui-retoken
/opsx:new uar-ui-verify-gates
```

Each proposal must record: the TanStack Query scoped exemption, the TW3-port decision, the admin CRT-theme
exception, and the no-protocol-change scope cut (see "Scope cuts" above).

PLAN COMPLETE
