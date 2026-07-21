## Why

Agent-generated `uar.a2ui/1` surfaces render through the vendored `a2ui-uar` package (`frontend/packages/a2ui-uar/`), which ships its **own** pre-convergence token set in `src/styles.css` (old navy `#0F1923` dark, ad-hoc light, high-contrast) and lets its host page's background show through — the surface paints `color` but never `background`. That unpainted surface is the root cause of the only failing vitest a11y test in the repo (white high-contrast foreground measured against the light page canvas at 1.07:1, documented in `uar-ui-chat-reskin/verification-notes.md`). With tokens, chat, and shell converged, A2UI surfaces are now the only product surface still rendering the old design system.

## What Changes

- Port the KnowMe token values into `packages/a2ui-uar/src/styles.css` for all three package themes (`data-a2ui-theme="dark|light|high-contrast"`), matching `frontend/src/index.css` (KnowMe ladder, ember/ember-soft, cyan, status ramp, radius scale) — the package keeps its scoped `.uar-a2ui-surface` theme mechanism (it must work embedded anywhere), only the values change.
- **Fix the unpainted surface (a11y defect)**: `.uar-a2ui-surface` paints `background: hsl(var(--background))` and `color: hsl(var(--foreground))` per active theme, so every theme is self-contained and contrast-valid regardless of host page. Surfaces embedded in chat inherit rounding/overflow cleanly (the chat A2UI block already wraps them on `bg-card`).
- Flat 2.0 enforcement inside the package: strip border/shadow utilities from the package's component set (`TextField`, `Card`, `ChoicePicker`, `Divider`, etc.) in favor of ladder fills; `--border: transparent` inside the surface scope; dividers become spacing, never `<hr>`-style lines.
- Re-align `frontend/src/features/a2ui/a2ui-surface-renderer.tsx` container chrome to the ladder (its `border border-border bg-card` wrapper becomes plain `bg-card`).
- Verify the admin `A2uiTestingPage` fixtures render correctly on the new package themes (admin keeps its CRT shell; the A2UI surfaces inside it render the package themes).
- **No protocol changes** — `uar.a2ui/1` schema, surface processor, and event flow are untouched; this is theme values, one background-paint fix, and component chrome.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `uar-web-ui-design-tokens`: the `a2ui-uar` package themes become value-identical ports of the KnowMe token set, and agent-generated surfaces render Flat 2.0 with self-contained, contrast-valid theme backgrounds.
- `uar-chat-surface`: the embedded A2UI artifact container aligns to background-only separation (already partially restyled in `uar-ui-chat-reskin`).

## Impact

- `frontend/packages/a2ui-uar/src/styles.css` (token values for 3 themes + background paint), package components under `src/components/` (border/shadow purge), `frontend/src/features/a2ui/a2ui-surface-renderer.tsx` (container chrome).
- Fixes the failing storybook a11y test (`protocol.stories.tsx > High Contrast Long Translation`) — vitest returns to 153/153.
- Admin `A2uiTestingPage` fixture rendering verified manually.
- Depends on `uar-ui-token-convergence` (merged, PR #170); completes the visual convergence of all product surfaces.
- Skill gates: `hybrid-design-tokens`, `a11y-gate`, `content-block-ui`.
