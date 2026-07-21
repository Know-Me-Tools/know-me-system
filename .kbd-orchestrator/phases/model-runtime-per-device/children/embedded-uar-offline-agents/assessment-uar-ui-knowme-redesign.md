# ASSESSMENT: model-runtime-per-device › embedded-uar-offline-agents — UAR UI → KnowMe redesign (proposed change)

Project: KnowMe
Date: 2026-07-21
Assessor: kimi (frontier class per model_policy.phases.kbd-assess)
Scope note: This is a **new-scope assessment** requested by the operator — redesign the web UI of the
`rust/vendor/universal-agent-runtime` (UAR) submodule to adopt the KnowMe React 19 (desktop/Tauri) UI standards,
architecture, and style, while keeping UAR's Tailwind/CSS styling mechanism. It is written as a separate file so
the completed phase assessment (`assessment.md`, for `embed-uar-mobile-offline`) is not clobbered.

Codebase baseline: UAR ships a complete, production-grade React 19 web UI (`frontend/`, ~285 files / ~34.5k lines TS+TSX)
with its own ember-on-navy Flat 2.0 theme; KnowMe desktop (`desktop/`) is the binding target standard per
`docs/knowme-ui-ux-standard.md` and `AGENTS.md`.
Cross-tool progress: parent phase 4/5 changes done by codex; child change `embed-uar-mobile-offline` 0/1 (codex, implementation_ready).

## Key finding: the two design systems are siblings, not strangers

Both UAR and KnowMe already use **Flat 2.0 with the identical ember brand accent `#FF6A3D`**, shadcn/ui primitives,
assistant-ui chat, Zustand, and CSS-variable theming. The redesign is therefore **convergence, not a rewrite** —
but there are real structural gaps (below) that make it a major change, not a reskin.

## IMPLEMENTATION STATUS (redesign-relevant areas)

- UAR web UI itself: **DONE** — Vite 8 + React 19.2, react-router-dom 7 (3 routes), assistant-ui 0.14 chat, full admin
  surface (19 sections), A2UI renderer packages, PGlite persistence, Storybook/vitest/Playwright tooling.
  Served as static bundle by Axum; Tauri shell just points the webview at the in-process server (no `invoke()` layer).
- Design-token parity with KnowMe: **PARTIAL** —
  - UAR: Tailwind 3.4 + JS config, tokens in `frontend/src/index.css` (368 lines), shadcn style `base-vega`, radius
    `0.625rem`, dark bg `#0F1923`, fonts Inter/Space Grotesk/JetBrains Mono, themes dark/light/high-contrast/system
    via custom Zustand theme store (`uar-theme` localStorage key).
  - KnowMe: Tailwind 4 `@theme` CSS-first (no JS config), shadcn style `base-nova`, radius `0.75rem`, 4-step surface
    ladder `#0B0F14/#111620/#161D29/#1C2535`, **ember-soft selection surface `#2B1A18`**, cyan `#00C2DC` AI/status
    semantics, effective body font Geist Variable, `body.light` override mechanism (`knowme-theme` key),
    Flat 2.0 *enforced in CSS* (`--border: transparent`, global `border-transparent`, `box-shadow:none` on `.aui-root`).
  - Gap: different surface-ladder values, radius scale, font stack, selection/hover token model, theme mechanism,
    and KnowMe's hard no-border/no-shadow enforcement is absent in UAR (UAR's Flat 2.0 is by-convention).
- Component architecture: **PARTIAL — one blocking-constraint conflict** —
  - UAR frontend uses **TanStack Query 5** at the root, which violates the blocking KnowMe constraint
    `preserve-entity-management` ("do not introduce TanStack Query") *if the redesigned UI is held to KnowMe rules*.
    Mitigating fact: UAR already vendors `@prometheus-ags/prometheus-entity-management` (15-package monorepo) and uses
    it for entity sync via SSE — the TanStack Query layer could be retired onto PEM, but that is an architecture
    change beyond styling.
  - UAR: 45 Zustand stores (some hold server-ish state); KnowMe rule: Zustand = ephemeral UI state only.
  - UAR layering (pages → features/hooks → stores → services/fetch, enforced by `check-frontend-boundaries.mjs`) is
    compatible in spirit with KnowMe's Component → Hook → Store → transport; transport differs (HTTP/SSE vs Tauri
    invoke), which is fine — UAR is a server-served SPA.
  - Routing: react-router-dom 7 vs KnowMe's TanStack Router — cosmetic unless full architectural adoption is in scope.
- Chat surface: **PARTIAL** —
  - Both use `useExternalStoreRuntime`. KnowMe converts store messages into the **closed 11-variant ContentBlock union**
    (`packages/gen-ui-react/src/content-block.ts`, mirrored in Rust `gen_ui_types` and Flutter) with registered
    data-part renderers (Memory/Citation/Skill/Artifact), filled-surface composer (radius 1.5rem), ember-soft user
    bubble `rounded-2xl rounded-ee-md`, assistant text on `--color-card`.
  - UAR renders its own bespoke part components (`tool-call-block.tsx`, `memory-chunk-block.tsx`,
    `skill-activation-block.tsx`, `context-update-block.tsx`, `a2ui-artifact-block.tsx`, 748-line
    `enhanced-thread.tsx`). These overlap semantically with ContentBlock variants but are not the shared contract.
  - Gap: adopting KnowMe style means re-skinning UAR's thread to the KnowMe composer/bubble/reasoning idiom and
    deciding whether UAR's message parts map onto the 11-variant union (content-block-ui skill is the gate).
- Shell & navigation: **PARTIAL** —
  - UAR: left sidebar + top nav only; admin uses a separate icon-rail shell; Tauri window has no branded titlebar
    (webview `location.replace` to localhost).
  - KnowMe: single `DESTINATIONS` inventory, rail on wide / bottom nav below the M3 compact boundary, branded custom
    Tauri titlebar, persistent model/lane/readiness status. Gap: shell pattern, width-based nav, titlebar, status lane.
- Admin surface: **UAR-only** — 19 sections (~10.9k lines) with a scoped CRT "terminal" theme
  (`:root[data-admin-theme="terminal"]`). No KnowMe counterpart. Open decision: keep as a deliberately-scoped
  exception, or re-token to the KnowMe ladder. Recommend keeping it scoped (it is an operator console, not product UI).
- Generative UI: **PARTIAL** — UAR owns `a2ui-uar` (36 files) + vendored Google A2UI siblings; KnowMe has
  `gen-ui-react` ContentBlock + `A2uiArtifactRenderer`. Redesign must not break `uar.a2ui/1` rendering; visual
  baseline of `a2ui-uar` should be re-tokened to KnowMe.
- Dead/stale styling configs: **MISSING cleanup** — repo-root `uar/tailwind.config.ts` + `postcss.config.cjs` are a
  stale Material-3-purple system pointing at nonexistent paths; delete or reconcile during the redesign.
- Tests/tooling for UI: **DONE (UAR-side)** — Storybook 10, vitest, Playwright e2e exist in UAR; KnowMe gates add
  a11y-gate, tauri-ui-review, content-block-ui, hybrid-design-tokens, reference-ui-fidelity skills.

## CROSS-TOOL PROGRESS

- `android-litertlm-runtime`, `litertlm-native-adapter`, `litertlm-device-proof`,
  `document-uar-karpathy-knowledge-strategy`: DONE (codex, parent phase).
- `embed-uar-mobile-offline`: IN_PROGRESS 0/1 (codex) — the active change; **this UI redesign is not part of it** and
  should be sequenced as its own change to avoid colliding with in-flight kernel work in the same submodule.

## SPEC GAP SUMMARY

- The redesign is **unspecified**: no phase goal, OpenSpec change, or spec file covers UAR-UI→KnowMe convergence.
  A new change (or new phase) is required before execute; this assessment is its seed.
- **TanStack Query conflict** must be resolved explicitly: either the redesign adopts the KnowMe constraint and
  migrates server state to the already-vendored PEM, or the change records a deliberate, scoped exemption for the
  UAR submodule. Blocking constraint `preserve-entity-management` currently says no.
- Token strategy undecided: (a) port KnowMe token values into UAR's existing Tailwind 3.4 setup (smallest diff,
  honors "keep the CSS/Tailwind mechanism"), or (b) migrate UAR frontend to Tailwind 4 `@theme` matching KnowMe's
  CSS-first pattern (bigger, but current best practice — re-add shadcn components from the v4 registry rather than
  hand-migrating). Web research below supports (b) if a version bump is acceptable, (a) if minimal risk is priority.
- Skill-name mapping for the operator's request: "ui ux pro max / impeccable" correspond in this environment to the
  `frontend-design`, `frontend-ui-engineering`, `teach-impeccable`, `critique`, and `polish` skills, plus the
  project gates `reference-ui-fidelity`, `hybrid-design-tokens`, `tauri-ui-review`, `a11y-gate`, and
  `content-block-ui`. Plan/execute should invoke them explicitly.
- Branding: KnowMe monogram/wordmark (`desktop/src/shared/components/KnowMeLogo.tsx`, Space Grotesk 700, ember "Me")
  should replace UAR wordmark on product surfaces; admin console may keep UAR identity.

## WEB RESEARCH (operator-requested, 2026-07-21)

- assistant-ui is designed for reskinning: copied shadcn-style component files + CSS variables such as
  `--aui-thread-max-width` ([assistant-ui changelog](https://www.assistant-ui.com/blog/2025-01-31-changelog),
  [shadcn theming docs](https://ui.shadcn.com/docs/theming)) — matches the plan of re-skinning UAR's
  `components/assistant-ui/*` to KnowMe idioms rather than replacing the library.
- Tailwind 3→4 migration path: official upgrade tool, move config into CSS `@theme`, and **re-add shadcn components
  from the v4 registry instead of manually migrating component code**
  ([BuildMVPFast guide](https://www.buildmvpfast.com/blog/tailwind-v4-shadcn-ui-migration-breaking-changes-guide-2026),
  [PkgPulse](https://www.pkgpulse.com/blog/tailwind-css-v4-whats-new-migration),
  [Zippystarter](https://zippystarter.com/blog/guides/migrating-tailwind3-to-tailwind4-with-shadcn)).
- Sharing a design system across apps: the current standard is a **shadcn registry / monorepo shared package**
  ([shadcn monorepo docs](https://ui.shadcn.com/docs/monorepo),
  [openstatus registry writeup](https://www.openstatus.dev/blog/shadcn-component-registry)). KnowMe already has the
  seeds of this (`packages/gen-ui-react`, shared tokens skill); a `@knowme/ui-tokens` registry package is the
  durable end-state, but copying tokens is the pragmatic first step across a submodule boundary.

## BUILD HEALTH

- build check: **UNKNOWN** — UAR frontend `node_modules` absent; full `cargo check --workspace` not run (assess is
  read-only; the configured build-health command spans rust+desktop+mobile and is out of scope for this UI assess).
  Submodule is clean at `e2299ae feat: route embedded model capabilities`.
- known violations: TanStack Query 5 in UAR frontend vs blocking constraint `preserve-entity-management` (see above).
  Stale root Tailwind/PostCSS configs (dead code). No other AGENTS.md violations observed in the UI layer.
- test coverage: **PARTIAL** — UAR frontend has vitest (29 test files), Storybook, Playwright e2e; KnowMe desktop has
  feature `__tests__`. No cross-surface visual-parity tests exist.

## CONSTRAINT CHECK

- AGENTS.md violations: TanStack Query presence (blocking, if KnowMe rules are applied to the submodule); otherwise
  UAR's layering, shadcn usage, and Flat 2.0 direction are compliant in spirit.
- constraints.md: `flat-2-ui` (warning) — UAR lacks KnowMe's *enforced* no-border/no-shadow CSS; `shared-ui-components`
  — satisfied (shadcn on both); `preserve-entity-management` — conflict noted; `cross-platform-parity` — the
  redesigned UAR UI must keep working in browser, Docker, and the Tauri webview from the same bundle.

## GOAL PROGRESS (active phase goals — unaffected by this assessment)

- Embed one mobile-safe UAR kernel behind Rust FFI: PARTIAL (kernel landed in submodule; host integration pending)
- Register KnowMe local models as UAR providers/models: PARTIAL (capabilities routed at e2299ae)
- Offline agents/skills/tools/memory/policy/AG-UI/A2UI: NOT MET (0/1 change complete)
- BYOK-only remote providers: NOT VERIFIED in this assess
- Certification with evidence: NOT MET
- KBD/wiki/learning-loop persistence: PARTIAL (this assessment + prior handoffs)

## RECOMMENDED NEXT STEP

Create a dedicated change (proposal: `uar-ui-knowme-convergence`) — either as a sibling change in this phase after
`embed-uar-mobile-offline` lands, or as a new top-level phase — with milestones: (1) token port + stale-config purge,
(2) thread/composer/message reskin to KnowMe idioms via reference-ui-fidelity, (3) shell/nav/titlebar adoption,
(4) TanStack Query → PEM decision + execution, (5) a2ui-uar re-token, (6) gates: hybrid-design-tokens, a11y-gate,
tauri-ui-review, content-block-ui. Sizing: major — ~34.5k lines of UI, but high token/component similarity keeps it
mechanical in most areas.

ASSESSMENT COMPLETE
