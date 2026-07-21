# Verification notes — uar-ui-a2ui-retoken

Recorded 2026-07-21 (kimi). Branch `uar-ui-a2ui-retoken` in the UAR submodule,
based on `origin/main` (dc158a8, post PR #171).

## Build gates

- `pnpm -C frontend typecheck` (`tsc -b`) → exit 0.
- `pnpm -C frontend lint` (`eslint .`) → exit 0.
- `pnpm -C frontend build` → `✓ built in 9.75s`; `scripts/validate-static-bundle.mjs` →
  "Frontend bundle valid (8 referenced assets)".
- `pnpm -C frontend build-storybook` → "Storybook build completed successfully".
- `pnpm -C frontend test` (vitest, 32 files) → **153 passed / 0 failed**. The
  previously failing `protocol.stories.tsx > High Contrast Long Translation` axe
  test now passes (the unpainted-surface defect is fixed).

## What landed

- `packages/a2ui-uar/src/styles.css` — all three scoped themes (`default`/light,
  `dark`, `high-contrast`) ported to KnowMe values (HSL channels mirroring
  `frontend/src/index.css`), new vocabulary added (`--chrome`, `--card-hov`,
  `--fg-sub`, `--fg-faint`, `--ember`, `--ember-soft`, `--cyan`, status ramp),
  `--border: transparent` (except HC, where visible lines are the feature).
- **The a11y fix**: `.uar-a2ui-surface` now paints `background: hsl(var(--background))`
  alongside `color`. A theme that controls text color but not its canvas is not a
  theme — the host canvas no longer leaks into axe measurements (1.07:1 → pass).
- Component purge (package-local primitives): `ui/button` outline → `bg-surface`/
  `bg-card-hov`; `ui/card` loses `shadow-xs ring-1`; `ui/checkbox` filled
  `bg-surface`; `ui/input` + `TextField` textarea filled `bg-surface` (dead `dark:`
  variants dropped); `ChoicePicker` listbox `bg-surface`. `Divider`/`Separator`
  stay token-driven (`bg-border` — invisible in light/dark, visible in HC by design).
- `features/a2ui/a2ui-surface-renderer.tsx` Card container → plain `bg-card`.

## Defect found and fixed during the contrast spot-check (task 4.5)

Dark `--muted-foreground` was `#4B6080` — **2.07:1** on `--muted` `#253044`
(placeholders, meta chips). Root cause identical to the light-theme defect fixed
during token-convergence: KnowMe maps `--muted-foreground: var(--color-fg-sub)`,
so the correct dark value is `#A7B0BC` (fg-sub), not `--color-muted-fg`. Corrected
in **both** `frontend/src/index.css` `:root` and the a2ui-uar dark theme (6.0:1 on
muted after fix).

## Contrast spot-check (WCAG relative luminance)

| Pair | Ratio |
|---|---|
| dark fg on canvas / card | 16.3 / 13.1 PASS |
| dark fg-sub on canvas | 8.8 PASS |
| dark muted-fg on muted (after fix) | ~6.0 PASS |
| dark ember on ember-soft | 5.8 PASS |
| dark ring on card | 5.4 PASS |
| dark cyan on surface | 7.9 PASS |
| dark primary-fg on ember | 6.8 PASS |
| light fg on canvas / card | 18.0 / 19.2 PASS |
| light fg-sub / muted-fg on muted | 7.1 / 6.9 PASS |
| light ember on ember-soft | 3.4 PASS (non-text/large; matches KnowMe) |
| light ring on card | 4.0 PASS |
| light cyan on surface | 3.6 PASS (large; matches KnowMe) |
| light primary-fg on ember | 4.0 PASS (large; matches KnowMe) |
| HC fg on card / yellow on black | 16.5 / 17.0 PASS |

## Deferred

- **4.4 admin `A2uiTestingPage` fixture review** — needs the dev server + backend;
  folded into `uar-ui-verify-gates`. Static expectation: fixtures pass explicit
  `theme` props and render the new package themes inside the CRT shell.
