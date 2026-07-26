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

## Interactive smoke — 2026-07-26 (claude)

Frontend-only run: `pnpm -C frontend dev --port 1906` (no backend). Setup required
building the `workspace:*` dep first — `@prometheus-ags/prometheus-entity-management`
ships `src/` but resolves via `dist/`, so `pnpm -r build` in
`frontend/packages/prometheus-entity-management` is a prerequisite for `dev` to boot.
Without it Vite fails import-analysis on `src/entities/bootstrap.ts` and no shell paints.

### Verified (task 3.3 — layout stacking)

Measured on the live DOM at 320x720, not eyeballed:

- `main` bottom = 648px, bottom-bar top = 648px → **overlap 0px**. The bar is a flex
  sibling of the content row, so content cannot be occluded by it.
- `document.body.scrollWidth` 320 ≤ viewport 320 → no horizontal overflow.
- Bottom bar `padding-bottom: 8px` resolved (`env(safe-area-inset-bottom)` = 0 on
  desktop Chrome; the `calc(.5rem + …)` term is present and correct).
- Rail `display: none` at 320px; present at 1440px. Breakpoint is Tailwind `md:`
  (768px), so both smoke widths sit clearly on either side.

### Verified (task 5.3 — shell idiom, 4 combinations)

1440px dark, 1440px light, 320px dark, 320px light — all captured.

- Wide: rail with brand header (logo + wordmark + mono tagline), 3 destinations,
  readiness card pinned to rail footer.
- Narrow: rail hidden, bottom bar with icon+label for all 3 destinations.
- Active state resolves correctly in **both** themes — dark `rgb(42,26,24)` /
  light cream, text `rgb(255,106,61)`; `bg-ember-soft text-ember` + `aria-current="page"`.
- Readiness card shows dot **plus** text ("Unreachable" / "Embedded · local") — the
  never-color-alone rule holds. "Unreachable" is expected with no backend.
- Flat 2.0 holds: rail/main separate by background color only, no borders or dividers.
- Destinations expose real `<a href>` elements; the two navs keep distinct
  `aria-label`s ("Main (rail)" / "Main (bottom bar)").

### NOT verified — needs a running backend

The chat page short-circuits to a `No Model Configured` guard without a provider, so
the chat chrome never mounts: **0 textareas, 0 composer elements, 0 hamburger buttons**
in the DOM at 320px. Therefore these remain unverified:

- Composer clearing the bottom nav *with the composer actually rendered* (3.3 verifies
  the shell geometry that makes occlusion impossible, not the composer itself).
- Hamburger button and mobile sidebar drawer open/close (5.3).

These are the same backend-dependent items the 2026-07-21 apply notes folded into
`uar-ui-verify-gates`. 3.3 and 5.3 are marked done for the shell-layout scope they own;
the composer/drawer interaction check stays with `uar-ui-verify-gates`.

### Finding — theme toggle unreachable at narrow widths

`ThemeToggle` is mounted only inside the rail (`AppShell.tsx:86`), which is
`hidden md:flex`. At <768px there is **no** theme control anywhere in the shell
(confirmed: accessibility tree at 320px lists only the 3 destination links and the
page's own button). Dark/light both render correctly at 320px, but a narrow-viewport
user cannot switch between them. Not in this change's task list — filed as a follow-up
rather than fixed here.
