## Why

The UAR web UI has a desktop-only shell: a top nav bar with inline links and a chat-only left sidebar, no navigation at all on narrow screens (the top-nav links shrink but the pattern is a desktop header, not the KnowMe width-based rail/bottom-bar idiom), no persistent model/lane/readiness lane, and no branded window chrome when running inside the Tauri shell (the webview has default OS decorations). `docs/knowme-ui-ux-standard.md` §5 mandates a single destination inventory rendered as a rail on wide layouts and a bottom bar on narrow ones, ember-tinted active states, and an always-visible readiness lane.

## What Changes

- Add a single `DESTINATIONS` inventory (Chat `/threads`, Admin `/admin`, About `/about`) shared by all navigation renderers (new `frontend/src/app/navigation.ts`).
- Replace `components/layout/top-nav.tsx` with a KnowMe-style `AppShell` (`frontend/src/app/AppShell.tsx`): left rail on `--chrome` at `md:` and up (brand header with KnowMeLogo/wordmark, destination buttons with `bg-ember-soft` + ember text active state), bottom navigation bar below `md` (safe-area aware, `min-h-14` targets), and a rail-footer status card showing server readiness + deployment mode (embedded/local vs remote), never color-alone.
- Keep the chat thread-list sidebar (`left-sidebar.tsx`) and agent-selector toolbar as chat-level chrome, but re-skin their containers to the ladder (chrome/surface backgrounds instead of border-separated regions); the mobile sidebar hamburger moves into the chat page header (narrow only).
- Add a branded titlebar for the Tauri shell: `decorations: false` in `src-tauri/tauri.conf.json`, plus a web-chrome `Titlebar` component (drag region via `data-tauri-drag-region`, min/max/close through `@tauri-apps/api` when running inside Tauri, hidden in plain browsers) per the `tauri-custom-titlebar` skill.
- `OfflineBanner` stays, repositioned above the shell.
- **No routing, state, or service changes.** react-router routes are untouched; this is shell layout and window chrome only.

## Capabilities

### New Capabilities

- `uar-shell-navigation`: Single-inventory, width-adaptive shell navigation (rail/bottom bar), persistent readiness lane, and branded Tauri titlebar for the UAR web UI.

### Modified Capabilities

- `uar-web-ui-design-tokens`: shell chrome consumes `--chrome`/`--surface` ladder steps and `ember-soft` active states; remaining border-separated shell containers move to background-color separation.

## Impact

- New: `frontend/src/app/navigation.ts`, `frontend/src/app/AppShell.tsx`, `frontend/src/components/Titlebar.tsx`.
- Reworked: `frontend/src/App.tsx` (shell mount), `frontend/src/pages/chat-page.tsx` (hamburger + container chrome), `frontend/src/components/layout/left-sidebar.tsx` (container re-skin), `src-tauri/tauri.conf.json` (decorations).
- Deleted: `frontend/src/components/layout/top-nav.tsx` (superseded by AppShell).
- Admin keeps its inner icon rail (`admin-shell.tsx`) unchanged — the CRT-themed exception; only the outer shell destination to reach it changes.
- Depends on `uar-ui-token-convergence` (merged, PR #170) and follows `uar-ui-chat-reskin` (merged, PR #171).
- Skill gates: `mobile-navigation` (bottom-bar pattern), `tauri-custom-titlebar` (window chrome), `reference-ui-fidelity`.
- Verification note: the Tauri window-config change is verified by config inspection + browser-hidden titlebar logic; a full Tauri launch belongs to `uar-ui-verify-gates`.
