## Why

The UAR web UI (`rust/vendor/universal-agent-runtime/frontend/`) runs its own by-convention Flat 2.0 theme (`#0F1923` navy, ember `#FF6A3D`) that diverges from KnowMe's binding UI standard (`docs/knowme-ui-ux-standard.md`): different surface ladder, radius, font stack, selection/hover tokens, and no enforced no-border/no-shadow CSS. Every subsequent redesign change (chat reskin, shell/navigation, A2UI re-token) depends on a single shared token foundation. This change establishes that foundation inside the UAR frontend while keeping its Tailwind 3.4 mechanism.

## What Changes

- Port the complete KnowMe design-token set into `frontend/src/index.css` and `frontend/tailwind.config.ts`: 4-step surface ladder (`#0B0F14/#111620/#161D29/#1C2535` dark), hover/muted surfaces, ember + ember-soft selection surface, cyan AI/status semantics, status ramp, light theme values, radius base `0.75rem` with the KnowMe scale.
- Add KnowMe's Flat 2.0 **enforcement** CSS: `--border: transparent`, global `border-transparent`, `box-shadow: none` on `.aui-root`, markdown border neutralization, ember focus rings.
- Adopt the KnowMe font stack: Geist Variable (body, via `@fontsource-variable/geist`), Space Grotesk (display), JetBrains Mono (meta/code); add the `.eyebrow` utility class.
- Rebase UAR's light and high-contrast themes onto the new ladder; keep UAR's existing theme-store mechanism (`uar-theme` localStorage key, 4 modes) unchanged.
- Delete the stale repo-root `tailwind.config.ts` + `postcss.config.cjs` (dead Material-3-purple system pointing at nonexistent paths).
- Replace the UAR wordmark with the KnowMe monogram/wordmark on product surfaces (chat shell, about page); UAR identity is retained inside `/admin`.
- Record two operator decisions as durable exemptions: (1) TanStack Query 5 remains in the UAR frontend as a scoped exemption to KnowMe's `preserve-entity-management` blocking constraint; (2) the `/admin` console keeps its scoped CRT terminal theme.
- **No wire-protocol, state-management, or Tailwind-version changes.** Presentation layer only.

## Capabilities

### New Capabilities

- `uar-web-ui-design-tokens`: The KnowMe token system, Flat 2.0 enforcement, and font stack as the single styling source for the UAR web UI across browser, Docker, and Tauri-webview deployments.

### Modified Capabilities

- None. (Chat, shell, and A2UI visual changes are separate changes built on this one.)

## Impact

- `rust/vendor/universal-agent-runtime/frontend/src/index.css` (rewritten token definitions), `frontend/tailwind.config.ts` (token bindings), `frontend/index.html` (font loading), `frontend/package.json` (fontsource dependency).
- Repo-root stale `tailwind.config.ts` / `postcss.config.cjs` deleted.
- Shared logo/wordmark component on product surfaces; `/admin` untouched.
- All 56 shadcn primitives and chat/admin screens render on the new tokens — visual regression risk is front-loaded here and verified via Storybook + Playwright baseline review in this change (full gate suite runs in `uar-ui-verify-gates`).
- Submodule discipline: all edits land in the `rust/vendor/universal-agent-runtime` submodule on their own branch; the parent repo only receives a submodule pointer bump after verification.
