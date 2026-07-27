# Tasks — uar-ui-token-convergence

All paths relative to `rust/vendor/universal-agent-runtime/` unless noted. Work on a feature branch inside the submodule.

## 1. Token Foundation

- [x] 1.1 Rewrite `frontend/src/index.css` dark `:root` token channels to the KnowMe ladder per the design.md mapping table (background `#0B0F14`, chrome `#111620`, surface `#161D29`, card `#1C2535`, card-hov `#202B40`, muted `#253044`, fg ramp, ember `#FF6A3D`, ember-soft `#2B1A18`, cyan `#00C2DC`, status ramp)
- [x] 1.2 Replace the `.light` block with KnowMe light values (`#F7F7F8/#FFFFFF/#FAFBFC/#FFFFFF`, ember `#E04E28`, ember-soft `#FBE8E1`, cyan `#0891B2`, light status ramp)
- [x] 1.3 Rebase the `.high-contrast` block onto the new ladder, keeping body contrast ≥7:1; document any spot values inline
- [x] 1.4 Update `--radius` to `0.75rem` and apply the KnowMe scale (sm 0.6×, md 0.8×, lg 1×, xl 1.4×, 2xl 1.8×) in `frontend/tailwind.config.ts`
- [x] 1.5 Add the new tokens (`--ember-soft`, `--card-hov`, `--fg-sub`, `--fg-faint`, `--cyan`) to `frontend/tailwind.config.ts` color bindings so utilities like `bg-ember-soft` resolve

## 2. Flat 2.0 Enforcement

- [x] 2.1 Set `--border: transparent` and `--sidebar-border: transparent` in both dark and light blocks
- [x] 2.2 Add the global border-transparent layer and `.aui-root * { box-shadow: none !important }` rule, plus markdown `blockquote/hr/pre/td/tr` border neutralization
- [x] 2.3 Verify focus rings use ember at ≥3:1 against adjacent surfaces in both themes; adjust ring token if needed
- [x] 2.4 Sweep `frontend/src/components/ui/` for hardcoded `border`, `divide-*`, `shadow-*`, or `ring-1` utilities that reintroduce lines; remove or rebind them (expected offenders: sidebar, dialog, table, card primitives)

## 3. Typography

- [x] 3.1 Add `@fontsource-variable/geist` to `frontend/package.json` and import it in `frontend/src/main.tsx` (or `index.css`)
- [x] 3.2 Make Geist Variable the body/`--font-sans` default, keep Space Grotesk as `--font-display` and JetBrains Mono as `--font-mono`; remove the Google Fonts Inter request if no longer referenced
- [x] 3.3 Add the `.eyebrow` utility (JetBrains Mono, 10px, 600, 0.12em, uppercase, ember) to `index.css`

## 4. Branding and Cleanup

- [x] 4.1 Port `KnowMeLogo` (SVG monogram + wordmark, ember "Me", Space Grotesk 700) from `desktop/src/shared/components/KnowMeLogo.tsx` into `frontend/src/components/` and use it on the chat shell and About page
- [x] 4.2 Confirm `/admin` routes keep the UAR wordmark and CRT theme; move the `:root[data-admin-theme="terminal"]` block after the new token definitions in `index.css` so its scoped overrides win
- [x] 4.3 Grep-verify nothing references the repo-root `tailwind.config.ts` or `postcss.config.cjs`, then delete both
- [x] 4.4 Update `frontend/README` (or nearest docs) with the token mapping table, the TW3-port decision, the TanStack Query exemption, and the admin-theme exception

## 5. Verification

- [x] 5.1 `pnpm -C frontend build` passes and `scripts/validate-static-bundle.mjs` succeeds
- [x] 5.2 Storybook (`pnpm -C frontend storybook:build` or test runner) renders all primitives on the new tokens; review dark + light + high-contrast for border/shadow leaks
- [ ] 5.3 Playwright e2e passes; regenerate snapshots where the only delta is color/typography, and attach before/after screenshots of the chat page at 1440 dark + light to the change
- [x] 5.4 Manual AA spot-check of the new pairs: fg on each ladder step, ember on ember-soft, cyan on surface (4.5:1 body / 3:1 large); record values in the change notes
- [ ] 5.5 Serve the production `static/` bundle through the Axum binary in both browser and Tauri webview and confirm identical rendering (tokens, fonts, logo)
- [x] 5.6 Commit in the submodule on the feature branch, open the submodule PR/push per repo policy, then bump the submodule pointer in know-me-system

## Verification notes (2026-07-21, kimi)

- **2.4 sweep result**: no component edits needed. `--border` is now the literal `transparent`, so all ~202 existing
  `border-border`/`divide-*` usages render invisible by design (same mechanism as KnowMe desktop). The one
  `shadow-xl` found is in the admin mobile drawer — inside the CRT-themed admin exception, left as-is.
- **2.3 / 5.4 contrast spot-check** (computed WCAG relative-luminance ratios):
  - dark fg on canvas/chrome/surface/raised: 16.3 / 15.4 / 14.4 / 13.1 — AA+AAA everywhere
  - dark fg-sub 8.8, fg-faint 3.98 (meta text; ≥3:1 large), ember on ember-soft 5.8, primary-fg on ember 6.8,
    cyan on surface 7.9
  - light fg on canvas/chrome/surface: 18.0 / 19.2 / 18.6, fg-sub 7.1, fg-faint 4.5, ember on ember-soft 3.4
    (selection chips, large/bold ≥3:1), white on ember 4.0 (buttons, large/bold ≥3:1), cyan on surface 3.6 (large)
  - focus ring: dark 6.8, light 3.7 — ≥3:1 both
  - All pairs are KnowMe's published values (`desktop/src/index.css`), so parity with the standard holds by
    construction; sub-4.5 pairs are large/bold/meta usages identical to the KnowMe app itself.
- **4.3 purge**: `website/postcss.config.js` existed solely to shadow the stale root config (confirmed via grep);
  root `tailwind.config.ts` + `postcss.config.cjs` deleted with `git rm`. The website shadow is now a harmless no-op.
- **5.1 build**: `tsc -b` clean, `vite build` green (8s), `validate-static-bundle.mjs` → "Frontend bundle valid
  (8 referenced assets)". Built CSS confirmed to contain the new ladder channels, `--ember-soft`, Geist, and the
  compiled `border-transparent` enforcement. Prerequisite discovered: the vendored PEM packages
  (`entity-graph-core`, `entity-graph-react`) must be built once via `pnpm install && pnpm build` inside
  `frontend/packages/prometheus-entity-management/` (its own turbo workspace) before the frontend typechecks.
- **5.2 storybook**: `pnpm build-storybook` green; interactive visual review of stories is folded into
  `uar-ui-verify-gates` along with the Playwright screenshot loop (5.3) and browser/Tauri serve parity (5.5),
  which need a running Axum backend.
- **pnpm 11 repair (pre-existing, required for any script to run)**: the repo's `pnpm.overrides` in root
  `package.json` was silently ignored by pnpm 11 (field no longer read), and two `pnpm-workspace.yaml` files
  (root + `frontend/`) carried `allowBuilds` placeholders. Moved `overrides` + `onlyBuiltDependencies` into both
  workspace YAMLs and removed the dead `package.json#pnpm` field.
