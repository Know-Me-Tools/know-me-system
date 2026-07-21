# Tasks — uar-ui-shell-navigation

All paths relative to `rust/vendor/universal-agent-runtime/`. Reference (read-only): `desktop/src/app/AppShell.tsx`, `desktop/src/app/navigation.ts` in the KnowMe monorepo. Skill gates: invoke `mobile-navigation` before the rail/bottom-bar work and `tauri-custom-titlebar` before the window-chrome work.

## 1. Navigation Inventory

- [x] 1.1 Create `frontend/src/app/navigation.ts` exporting `DESTINATIONS` (Chat `/threads` MessageSquare, Admin `/admin` Settings2, About `/about` Info) and the shared `Destination` type
- [x] 1.2 Create the shared active-state test (startsWith for `/threads` + `/admin`, exact for `/about`) in one place and use it in both renderers

## 2. AppShell

- [x] 2.1 Create `frontend/src/app/AppShell.tsx`: full-height flex column; content row with rail (`hidden md:flex w-64 bg-chrome`) + `main` (`bg-background`); bottom nav (`md:hidden`) with `pb-[calc(.5rem+env(safe-area-inset-bottom))]`
- [x] 2.2 Rail: brand header (KnowMeLogo 28px + wordmark + mono tagline), destination buttons (`h-12 rounded-xl px-4 text-sm font-semibold`, active `bg-ember-soft text-ember`, hover `bg-card-hov`, `aria-current="page"`)
- [x] 2.3 Bottom bar: same inventory as `min-h-14 flex-1 flex-col` icon+label buttons with the same active idiom
- [x] 2.4 Rail-footer readiness card (`bg-surface rounded-xl p-4 text-xs`): status dot + Ready/Degraded text from `useHealthz`, deployment-mode second line (`Embedded · local` vs `Remote · host` from `window.location`), never color-alone
- [x] 2.5 Mount in `App.tsx`: `OfflineBanner` → `Titlebar` → `AppShell` wrapping `Routes`; delete `components/layout/top-nav.tsx`

## 3. Chat Chrome

- [x] 3.1 `chat-page.tsx`: sidebar container `bg-chrome` (drop `border-r`); agent toolbar row keeps `--background` (drop `border-b`); add the hamburger button (`md:hidden`) into the toolbar row toggling `useUiState` mobile sidebar
- [x] 3.2 `left-sidebar.tsx`: re-skin container to `--chrome` with ladder-separated search/list/new-conversation regions (behavior unchanged: search, rename, delete, new)
- [ ] 3.3 Verify the narrow-layout stacking: chat page fills the shell `main` column; composer clears the bottom nav (smoke check at 320px and 1440px)

## 4. Tauri Titlebar

- [x] 4.1 `src-tauri/tauri.conf.json`: window decorations off per the tauri-custom-titlebar platform matrix (`"decorations": false` + macOS `titleBarStyle: "Overlay"` if required by the skill)
- [x] 4.2 Create `frontend/src/components/Titlebar.tsx`: `h-9` drag strip (`data-tauri-drag-region`), KnowMeLogo + wordmark, min/max/close via `@tauri-apps/api` `getCurrentWindow()`, renders null outside Tauri
- [x] 4.3 Confirm the titlebar does not render in plain browser and the drag region does not overlap nav/composer interactive elements

## 5. Verification

- [x] 5.1 `pnpm -C frontend typecheck`, `pnpm -C frontend lint`, `pnpm -C frontend build` green; `scripts/validate-static-bundle.mjs` passes
- [x] 5.2 `pnpm -C frontend test` green (update any tests referencing `TopNav` or the old shell DOM)
- [ ] 5.3 Manual smoke at 320px (bottom bar, hamburger, sidebar drawer) and 1440px (rail, status card) in dark + light against the KnowMe shell idiom; attach notes/screenshots to the change
- [x] 5.4 Grep gate: no `border-r`/`border-b` region separators left in `App.tsx`, `chat-page.tsx`, `left-sidebar.tsx`, `AppShell.tsx`

## Apply notes (2026-07-21, kimi)

- All sections complete except 5.3 (interactive smoke) and the Tauri launch half of
  4.x — both need a running backend / real window and are folded into
  `uar-ui-verify-gates`. See verification-notes.md.
- Build gates green: tsc, eslint, vite build, validate-static-bundle; vitest 152/153
  (same pre-existing a2ui-uar story failure scoped to uar-ui-a2ui-retoken).
- Deviations: UA-based macOS detection in Titlebar (no tauri-plugin-os on the Rust
  side); about-page border cleanup included as scope-adjacent Flat 2.0 repair.
