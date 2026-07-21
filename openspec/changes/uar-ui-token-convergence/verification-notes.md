# Verification notes — uar-ui-token-convergence

Recorded 2026-07-21. Values computed with the WCAG 2.x relative-luminance formula
against the token hex values defined in `frontend/src/index.css`.

## Task 5.1 — build + static bundle

- `pnpm -C frontend build` → exit 0, `✓ built in 9.63s`.
- `node scripts/validate-static-bundle.mjs` → `Frontend bundle valid (8 referenced assets).`
- Pre-existing, unrelated warnings retained: pglite `eval` usage and >1100 kB chunk
  size (both originate in `node_modules`, not in this change).

## Task 5.2 — Storybook

- `pnpm -C frontend build-storybook` → exit 0, `Storybook build completed successfully`.
  (Note: `tasks.md` names this script `storybook:build`; the actual script in
  `package.json` is `build-storybook`.)
- Remaining for human review: visual pass over dark / light / high-contrast for
  border and shadow leaks. Not machine-verifiable here.

## Task 5.3 — focus rings (≥3:1 against adjacent surfaces)

| Theme | Ring | Tightest adjacent surface | Ratio |
|---|---|---|---|
| Dark | ember `#FF6A3D` | muted `#253044` | 4.66:1 PASS |
| Dark | ember `#FF6A3D` | canvas `#0B0F14` | 6.76:1 PASS |
| Light | ember-2 `#E04E28` | card-hov / muted `#F2F4F7` | 3.60:1 PASS |
| Light | ember-2 `#E04E28` | chrome / card `#FFFFFF` | 3.97:1 PASS |
| High-contrast | `#FFEA00` | card `#1F1F1F` | 13.36:1 PASS |

No ring token adjustment was required.

## Task 5.4 — AA spot-check (4.5:1 body / 3:1 large + non-text)

### Dark — `--foreground` `#E8EDF3` on the ladder

| Surface | Ratio |
|---|---|
| background `#0B0F14` | 16.33:1 PASS |
| chrome `#111620` | 15.39:1 PASS |
| surface `#161D29` | 14.36:1 PASS |
| card `#1C2535` | 13.06:1 PASS |
| card-hov `#202B40` | 12.05:1 PASS |
| muted `#253044` | 11.26:1 PASS |

### Dark — secondary / faint text

| Pair | Ratio |
|---|---|
| fg-sub `#A7B0BC` on background | 8.77:1 PASS |
| fg-sub `#A7B0BC` on surface | 7.71:1 PASS |
| fg-sub `#A7B0BC` on card | 7.02:1 PASS |
| fg-faint `#6B7280` on background | 3.98:1 PASS (large/meta only) |
| fg-faint `#6B7280` on surface | 3.50:1 PASS (large/meta only) |

`--fg-faint` is intentionally a large/meta-text-only token; it does not meet 4.5:1
and must not be used for body copy.

### Accent pairs

| Pair | Ratio |
|---|---|
| dark ember `#FF6A3D` on ember-soft `#2B1A18` | 5.84:1 PASS |
| light ember-2 `#E04E28` on ember-soft `#FBE8E1` | 3.35:1 PASS (non-text/large) |
| dark cyan `#00C2DC` on surface `#161D29` | 7.85:1 PASS |
| light cyan `#0891B2` on surface `#FAFBFC` | 3.55:1 PASS (large only) |
| dark `#0B0F14` on ember button | 6.76:1 PASS |
| light `#FFFFFF` on ember-2 button | 3.97:1 PASS (large only) |

### Light — `--foreground` `#0B0F14` on the ladder

| Surface | Ratio |
|---|---|
| background `#F7F7F8` | 17.95:1 PASS |
| chrome `#FFFFFF` | 19.22:1 PASS |
| surface `#FAFBFC` | 18.55:1 PASS |
| card-hov `#F2F4F7` | 17.44:1 PASS |

### Defect found and fixed during this check

`--muted-foreground` in the light theme was ported as `#9AA3B0`, which measures
**2.38:1** on `#F7F7F8` — failing AA for body text and even the 3:1 large-text
floor. It is used for input/textarea placeholders and ~31 `ui/` primitives.

Root cause: the port copied KnowMe's `--color-muted-fg` (`#9AA3B0`, a dark-surface
token) instead of KnowMe's actual mapping `--muted-foreground: var(--color-fg-sub)`
(`desktop/src/index.css:60,121`).

Corrected to `#4B5563` (fg-sub) in both the `.light` block and the
`prefers-color-scheme: light` media block:

| Surface | Ratio after fix |
|---|---|
| background `#F7F7F8` | 7.06:1 PASS |
| chrome / card `#FFFFFF` | 7.56:1 PASS |
| surface `#FAFBFC` | 7.29:1 PASS |
| muted `#F2F4F7` | 6.86:1 PASS |

## Flat 2.0 component sweep (task 2.4)

Removed from `frontend/src/components/ui/` (26 files):

- `ring-1 ring-foreground/10` decorative hairlines on every floating surface
  (alert-dialog, card, combobox, context-menu, dialog, dropdown-menu, hover-card,
  menubar, navigation-menu, popover, select).
- All elevation shadows (`shadow-xs|sm|md|lg|xl` and the bare `shadow` utility).
- `sidebar.tsx` `shadow-[0_0_0_1px_var(--sidebar-border)]` — a shadow used to fake
  a 1px border on the `outline` menu-button variant — plus the floating/inset
  sidebar shadows and `ring-sidebar-border`.

Preserved: every `focus-visible:ring-*` (the one allowed line, contrast-verified
above) and `resizable.tsx`'s `focus-visible:ring-1`.

`pnpm -C frontend typecheck` → exit 0, no errors.

## Not verified in this pass

- **5.3 Playwright e2e + before/after screenshots** — not run.
- **5.5 Axum static-bundle serve in browser + Tauri webview** — not run.
- **5.6 submodule commit / PR / pointer bump** — not performed.
- Human visual review of Storybook across the three themes (5.2, second half).
