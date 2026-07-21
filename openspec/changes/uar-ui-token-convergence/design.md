## Context

The UAR frontend and the KnowMe desktop app share a design DNA (Flat 2.0, ember `#FF6A3D`, shadcn/ui, assistant-ui) but diverged in execution. KnowMe's standard is binding per `AGENTS.md` and `docs/knowme-ui-ux-standard.md`; the operator has decided the UAR web UI must adopt it. The operator further decided (2026-07-21): **port token values into UAR's existing Tailwind 3.4 setup** (no Tailwind 4 migration), grant a **scoped TanStack Query exemption** for the vendored submodule, and **keep the admin console's CRT terminal theme** as a deliberate exception.

Authoritative source values live in `desktop/src/index.css` (Tailwind 4 `@theme` + `body.light` overrides). The target files are `rust/vendor/universal-agent-runtime/frontend/src/index.css` (368 lines, `hsl(var(--token))` pattern) and `frontend/tailwind.config.ts`. Because UAR stays on Tailwind 3.4, the port maps KnowMe's hex values into UAR's existing HSL-channel CSS-variable pattern rather than copying KnowMe's CSS-first `@theme` verbatim.

## Goals / Non-Goals

**Goals:**

- One token source inside the UAR frontend whose rendered values match the KnowMe standard exactly (dark + light).
- Flat 2.0 enforced by CSS, not by convention: no borders, divider lines, decorative outlines, or layout shadows; regions differ only by background color.
- KnowMe font stack and `.eyebrow` utility available to every component.
- No visual regression inside `/admin` (CRT theme keeps its scoped tokens).
- Browser, Docker, and Tauri-webview deployments render identically from the same `static/` bundle.

**Non-Goals:**

- Migrating to Tailwind 4 or CSS-first `@theme` (deferred; recorded trade-off: two-way token sync until a shared token package exists).
- Replacing TanStack Query, Zustand organization, routing, or any state layer.
- Changing AG-UI/A2UI wire protocols or message-part types.
- Re-skinning individual screens (chat thread, shell/nav) — those are `uar-ui-chat-reskin` and `uar-ui-shell-navigation`.
- WCAG certification of the whole app (runs in `uar-ui-verify-gates`); this change only keeps its own edits at AA contrast.

## Decisions

### Port values into the existing `hsl(var())` TW3 pattern

UAR's `tailwind.config.ts` consumes tokens as `hsl(var(--background))` etc. We keep that mechanism and replace the channel values, adding the tokens KnowMe has that UAR lacks (`--ember-soft`, `--card-hov`, `--fg-faint`, cyan/status aliases). Mapping (dark / light):

| Token (UAR name) | Dark | Light |
|---|---|---|
| `--background` (canvas) | `#0B0F14` | `#F7F7F8` |
| `--chrome` (sidebar/header) | `#111620` | `#FFFFFF` |
| `--surface` | `#161D29` | `#FAFBFC` |
| `--card` (raised) | `#1C2535` | `#FFFFFF` |
| `--card-hov` (new) | `#202B40` | `#F2F4F7` |
| `--muted` | `#253044` | `#F2F4F7` |
| `--muted-foreground` | `#4B6080` | `#9AA3B0` |
| `--foreground` | `#E8EDF3` | `#0B0F14` |
| `--fg-sub` (new) | `#A7B0BC` | `#4B5563` |
| `--fg-faint` (new) | `#6B7280` | `#6B7280` |
| `--primary` / `--ember` | `#FF6A3D` | `#E04E28` |
| `--ember-soft` (new) | `#2B1A18` | `#FBE8E1` |
| `--cyan` | `#00C2DC` | `#0891B2` |
| `--success` / `--warning` / `--error` | `#22C55E`/`#F59E0B`/`#EF4444` | `#16A34A`/`#D97706`/`#DC2626` |
| `--radius` | `0.75rem` (scale: sm 0.6×, md 0.8×, lg 1×, xl 1.4×, 2xl 1.8×) | same |

Alternative: migrate to Tailwind 4 `@theme` now. Rejected by operator decision — larger diff, every component file touched, and the redesign's later changes already carry visual risk.

### Flat 2.0 enforcement moves from convention to CSS

Adopt KnowMe's enforcement block verbatim (adapted to UAR's variable names): `--border: transparent`, `--sidebar-border: transparent`, a global `* { border-color: transparent }` layer, `box-shadow: none !important` scoped to `.aui-root`, and markdown `blockquote/hr/pre/td/tr` border neutralization. Focus rings use ember at ≥3:1 against the adjacent surface.

Alternative: fix borders component-by-component. Rejected — 56 shadcn primitives plus assistant-ui skins make per-component fixes unenforceable; the CSS-level rule is exactly why KnowMe's version works.

### Theme mechanism stays UAR's; light values become KnowMe's

Keep `frontend/src/stores/theme-store.ts` (dark/light/high-contrast/system, `uar-theme` key, class on `<html>`). Replace the `.light` token block contents with the KnowMe light values. Rebase `.high-contrast` onto the new ladder at ≥7:1 body contrast (UAR-specific accessibility feature KnowMe lacks — kept). No key rename, no persistence migration.

### Fonts: Geist Variable body, Space Grotesk display, JetBrains Mono meta

Add `@fontsource-variable/geist` to `frontend/package.json`; keep JetBrains Mono and Space Grotesk (already loaded). Inter remains available as fallback but is no longer the body default. Add the `.eyebrow` utility (JetBrains Mono, 10px/600, 0.12em tracking, uppercase, ember).

### Stale root configs are deleted, not reconciled

Repo-root `tailwind.config.ts` + `postcss.config.cjs` reference `./src/**/*.rs` and a nonexistent `./web/` — nothing consumes them. Deleting them removes a trap for future contributors. Verified by grep before deletion.

### Branding swap is surface-scoped

Port `KnowMeLogo` (SVG monogram + Space Grotesk wordmark, ember "Me") into UAR's shared components and use it on the chat shell and About page. `/admin` keeps UAR identity and its CRT tokens. The admin theme block `:root[data-admin-theme="terminal"]` is moved after the new token definitions so its scoped overrides always win on admin routes.

### Submodule commit discipline

All work happens on a branch inside `rust/vendor/universal-agent-runtime` (its own git repo, remote `git@github.com:Prometheus-AGS/universal-agent-runtime.git`). The parent know-me-system repo gets exactly one submodule-pointer bump after this change verifies.

## Risks / Trade-offs

- **Front-loaded visual regression**: every screen changes at once when the ladder values shift. Mitigation: Storybook stories + Playwright screenshots reviewed in this change before merge; the dedicated gate change (`uar-ui-verify-gates`) does the full 4-width × 2-theme loop later.
- **TW3/TW4 mechanism drift** between UAR and desktop tokens: accepted; token values are the contract, and this file is the mapping table for future syncs.
- **High-contrast rebase** may need spot values beyond the ladder; keep each spot value ≥7:1 and document additions inline in `index.css`.
