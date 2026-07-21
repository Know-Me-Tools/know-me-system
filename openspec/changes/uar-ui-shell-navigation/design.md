## Context

With tokens (PR #170) and the chat surface (PR #171) on the KnowMe standard, the remaining structural gap is the app shell. Today `App.tsx` mounts `OfflineBanner` + `TopNav` + routes; the top nav is a desktop header with inline links; narrow screens get no destination navigation; the chat page owns a border-separated thread-list sidebar and an agent toolbar; and the Tauri webview uses default OS decorations even though it loads an in-process Axum URL (no native window chrome of its own).

Reference: `desktop/src/app/AppShell.tsx` + `desktop/src/app/navigation.ts` — one `DESTINATIONS` inventory rendered as a `w-64` chrome rail at `md:` and a bottom bar below it, active destination = `bg-ember-soft` + ember text, rail-footer status card ("Ready · UAR · local fallback"). UAR destinations differ from KnowMe's product (Chat/Admin/About vs Home/Chat/Hands/Memory/Models) — the pattern ports, the inventory adapts.

## Goals / Non-Goals

**Goals:**

- One `DESTINATIONS` inventory (`frontend/src/app/navigation.ts`) consumed by every nav renderer.
- Wide layouts (`md:`+): left rail on `--chrome` with brand header and ember-tinted active destinations.
- Narrow layouts: bottom navigation bar, `min-h-14` targets, `env(safe-area-inset-bottom)` padding, same inventory and active idiom.
- Persistent readiness lane: rail-footer card with server readiness (from `useHealthz`) and deployment mode text (embedded/local vs remote URL), status never conveyed by color alone (dot + label text).
- Chat-level chrome (thread-list sidebar, agent toolbar) separates by background, not borders; hamburger toggle lives in the chat header on narrow only.
- Branded titlebar in the Tauri shell: `decorations: false` + web `Titlebar` with drag region and window controls; zero footprint in plain browsers.

**Non-Goals:**

- No route changes (react-router config untouched), no new destinations, no settings screen.
- No changes to `admin-shell.tsx` (admin's inner icon rail stays CRT-themed).
- No thread-list behavior changes (search/rename/delete/new stay; container re-skin only).
- No native Rust/titlebar code beyond the `tauri.conf.json` decorations flag.
- No full responsive screenshot matrix or Tauri launch certification (`uar-ui-verify-gates`).

## Decisions

### One inventory, two renderers, width-based switch

`navigation.ts` exports `DESTINATIONS: readonly { path, label, icon }[]` = Chat (`/threads`, MessageSquare), Admin (`/admin`, Settings2), About (`/about`, Info). `AppShell` renders the rail (`hidden md:flex`) and the bottom bar (`md:hidden`) from the same array — CSS-width based, no OS/form-factor sniffing (mobile-navigation skill rule). Active test: `pathname.startsWith(path)` for `/threads` and `/admin`, exact for `/about`; active renderer sets `aria-current="page"`.

### The rail owns brand + readiness; the bottom bar owns nothing else

Rail: header = KnowMeLogo + wordmark (28px, already in `components/KnowMeLogo.tsx`); below it a mono tagline line. Footer status card (`bg-surface rounded-xl p-4 text-xs`): green dot + "Ready"/"Degraded" text from `useHealthz`, second line with deployment mode (`Embedded · local` when served from the Tauri/localhost origin, `Remote · <host>` otherwise — derived from `window.location`, no new services). Bottom bar: destinations only, safe-area padding.

### Thread-list sidebar and agent toolbar are chat chrome, not shell

`chat-page.tsx` keeps `LeftSidebar` and the `AgentSelector` toolbar. Their containers drop `border-r`/`border-b` in favor of ladder backgrounds (sidebar on `--chrome`, toolbar row on `--background`, chat canvas on `--background`). The hamburger (currently in `TopNav`) moves into the agent toolbar row, rendered `md:hidden` only, toggling the existing `useUiState` mobile sidebar. `top-nav.tsx` is deleted; `App.tsx` mounts `OfflineBanner` + `Titlebar` + `AppShell`.

### Titlebar lives in web chrome

The Tauri app points its webview at the in-process Axum server, so native decoration replacement is a web concern: `tauri.conf.json` sets `"decorations": false` on the window; `Titlebar.tsx` renders a 9 (`h-9`) drag strip with `data-tauri-drag-region`, the KnowMe monogram, and min/max/close buttons that call `getCurrentWindow()` from `@tauri-apps/api` — rendered only when `window.__TAURI_INTERNALS__` exists, otherwise it returns null. Platform caveat per `tauri-custom-titlebar`: on macOS keep traffic-light buttons by using `titleBarStyle: "Overlay"` instead of removing decorations; the change implements both per the skill's matrix.

### Admin destination is a normal destination

Reaching `/admin` uses the same inventory entry; the CRT theme activation stays inside `admin-page.tsx` as today.

## Risks / Trade-offs

- **Bottom bar + chat composer stacking**: on narrow, the composer sits above the bottom nav — the chat page's root must fill the shell's `main` column without a second scroll context; verified in the two-width smoke pass (320 + 1440) in this change.
- **`decorations: false` affects only the Tauri desktop shell**; the much more common Docker/browser deployments are untouched. If a platform misbehaves, the fallback is reverting the single config flag.
- **Deleting `TopNav`** removes the inline-link header some muscle memory relies on; acceptable — the rail/bottom bar is the standard.
- **Status lane data**: `useHealthz` polls `/healthz`; readiness text must stay honest when the server is degraded (dot + text, no color-only states).
