# Verification notes — uar-ui-shell-navigation

Recorded 2026-07-21 (kimi). Branch `uar-ui-shell-navigation` in the UAR submodule,
based on `origin/main` (dc158a8, post PR #171).

## Build gates

- `pnpm -C frontend typecheck` (`tsc -b`) → exit 0.
- `pnpm -C frontend lint` (`eslint .`) → exit 0.
- `pnpm -C frontend build` → `✓ built in 8.41s`; `scripts/validate-static-bundle.mjs` →
  "Frontend bundle valid (8 referenced assets)".
- `pnpm -C frontend test` (vitest, 32 files) → **152 passed / 1 failed** — the same
  pre-existing `a2ui-uar` high-contrast story defect documented in
  `uar-ui-chat-reskin/verification-notes.md` (scoped to `uar-ui-a2ui-retoken`).

## What landed

- `src/app/navigation.ts` — single `DESTINATIONS` (Chat/Admin/About) + shared active test.
- `src/app/AppShell.tsx` — rail (`hidden md:flex w-60 bg-chrome`) and bottom bar
  (`md:hidden`, `min-h-14` targets, `env(safe-area-inset-bottom)` padding) from the one
  inventory; distinct `aria-label`s ("Main (rail)" / "Main (bottom bar)") per the
  mobile-navigation skill; active = `bg-ember-soft` + ember text + `aria-current`.
- Readiness lane — rail-footer card: dot + Ready/Unreachable text from `useHealthz`
  (never color-alone), `Embedded · local` vs `Remote · <host>` from `window.location`,
  version suffix when available.
- `App.tsx` — mounts `Titlebar` + `OfflineBanner` + `AppShell`; `top-nav.tsx` deleted
  (`git rm`). Mobile sidebar hamburger moved into the chat toolbar (`md:hidden`).
- Chat chrome — thread-list sidebar on `--chrome` (no border-r), agent toolbar on
  `--background` (no border-b), mobile drawer now full-height (`top-12` removed with
  the top nav), active thread = `bg-ember-soft` (the 3px left-border active marker is
  gone), search field on `--surface`.
- `components/Titlebar.tsx` — adapted from the KnowMe reference: `isTauri()`-gated
  (returns null in browsers, no Tauri API touched), `data-tauri-drag-region` +
  explicit `startDragging()` on left-button mousedown, macOS traffic lights left /
  Windows-Linux controls right, aria-labeled buttons, `bg-chrome h-9 select-none`.
  **Deviation**: UAR does not register `tauri-plugin-os` on the Rust side, so macOS
  detection uses a UA check behind `isTauri()` instead of `platform()` — avoids a
  Rust plugin dependency for control ordering.
- `src-tauri/tauri.conf.json` — `"decorations": false`, window title
  "KnowMe — Universal Agent Runtime".
- Bonus Flat 2.0 cleanup in scope-adjacent product surface: `about-page.tsx` cards,
  stack table, and health pill lost their remaining border classes.

## Grep gate (task 5.4)

`App.tsx`, `AppShell.tsx`, `chat-page.tsx`, `left-sidebar.tsx` — zero `border-r`/
`border-b`/`border-l-[` region separators. Remaining `border-*` hits in the repo are
the admin CRT exception, ui primitives bound to the transparent `--border` token, or
markdown-content rules neutralized by the `.aui-md` CSS — all by design.

## Deferred

- **5.3 interactive smoke (320px/1440px, dark+light)** — needs a running backend for
  meaningful chat states; folded into `uar-ui-verify-gates`. Static checks done here:
  bottom bar and rail both derive from one inventory; chat `main` fills the shell
  column; composer sits inside the chat page above the bottom bar.
- **Tauri window verification** — `decorations: false` + titlebar logic verified by
  inspection only; a real Tauri launch (drag, controls, macOS traffic lights) belongs
  to `uar-ui-verify-gates`.
