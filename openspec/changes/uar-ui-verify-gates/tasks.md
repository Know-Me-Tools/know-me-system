# Tasks — uar-ui-verify-gates

## 1. e2e Suite Repairs

- [x] 1.1 Replace `waitForLoadState("networkidle")` with condition-based waits in all chat specs (SSE streams keep network open forever; 6 specs timed out deterministically)
- [x] 1.2 Fix strict-mode/multi-match selectors (`text=Providers` → `.first()`; smoke + admin specs)
- [x] 1.3 Correct the agent-selector locator to `aria-label="Select agent"` (spec's `text=Default Agent` never matched the real "Default Assistant" label)
- [x] 1.4 Raise per-assertion timeouts in admin specs (8000/5000 → 15000) and config-level `expect.timeout` to 12000 for cold Vite transforms
- [x] 1.5 Raise `playwright.real-server.config.ts` webServer boot budget 60s → 180s (measured cold boot ~66s)

## 2. Defects Found by the Gates (fixed)

- [x] 2.1 Light/high-contrast themes unreachable: `theme-store` never initialized at boot (nothing imported it) and `ThemeToggle` was rendered nowhere → side-effect store import in `main.tsx` + toggle rendered in the top nav
- [x] 2.2 Real-server e2e configs raced backend cold boot (task 1.5)

## 3. Screenshot Matrix

- [x] 3.1 Capture `/threads`, `/about`, `/admin` at 320/768/1024/1440 in dark AND light (24 PNGs under `screenshots/`)
- [x] 3.2 Visually inspect the matrix against the KnowMe standard (Flat 2.0, ladder separation, no border/shadow leaks, light theme correctness, narrow-width nav behavior)

## 4. a11y-gate Checklist (WCAG 2.2 AA)

- [x] 4.1 Contrast: storybook axe suite 153/153 + computed WCAG ratio tables (this and prior changes' notes)
- [x] 4.2 Accessible names: 0 buttons without accessible names on /threads
- [x] 4.3 Keyboard walk: logical Tab order across shell → theme toggle → sidebar → agent toolbar → composer CTA; ember focus rings present (rapid-Tab alpha readings are transition artifacts)
- [x] 4.4 Live regions present for streaming/status (aria-live count ≥ 1 on chat)
- [x] 4.5 Target sizes: 0 interactive elements under 24px
- [x] 4.6 Status not by color alone (dot + text in readiness/status indicators); reduced-motion rules present in both themes and the a2ui package
- [x] 4.7 Images: brand SVG is `aria-hidden` with adjacent text wordmark

## 5. Cross-Change Closures

- [x] 5.1 Admin A2UI fixture page renders re-tokened package themes (closes uar-ui-a2ui-retoken task 4.4)
- [x] 5.2 Full vitest suite green (153/153) on the final tree
- [x] 5.3 Record all evidence in verification-notes.md

## 6. Environment-Limited (documented, not hidden)

- [x] 6.1 `provider-route-real` — red under the default config (points at the pre-existing :1906 backend), GREEN under its dedicated real-server config (13.8s pass)
- [x] 6.2 `knowledge-rag-real` — needs an embedding-capable backend; stub-llm serves chat completions only
- [x] 6.3 `runtime-event-replay:67` — SSE-driven replay timing flake (passes in isolation, flakes under suite load)
- [x] 6.4 Tauri window launch (drag/controls/macOS lights) — manual verification; config + titlebar logic inspected only
