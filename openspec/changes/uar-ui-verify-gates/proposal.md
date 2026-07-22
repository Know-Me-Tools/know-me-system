## Why

Four convergence changes (tokens #170, chat #171, shell #172, A2UI #173) landed without their live verification loops — every prior change deferred browser/Tauri/e2e verification to this one. Unverified, the convergence could not be certified, and indeed this pass found real defects the static gates missed: the e2e suite had 13 failures, the light theme was completely unreachable (the theme store never initialized at boot and the toggle was rendered nowhere), and the real-server e2e configs timed out before the backend could boot.

## What Changes

- Repair the Playwright e2e suite so it tests the app honestly: replace `waitForLoadState("networkidle")` (never resolves with SSE streams open) with condition-based waits; fix strict-mode/multi-match selectors; align the agent-selector locator with the component's real `aria-label`; raise config-level expect timeout for cold Vite transforms; raise the real-server webServer boot budget (cold boot measures ~66s on dev machines vs the 60s timeout).
- Fix the unreachable light theme: side-effect import of the theme store in `main.tsx` so the stored theme applies at boot, and render `ThemeToggle` in the top nav so themes are user-switchable.
- Produce the screenshot matrix: `/threads`, `/about`, `/admin` × 320/768/1024/1440 × dark/light (24 captures), visually inspected against the KnowMe standard.
- Complete the a11y-gate checklist against the running app: accessible names, keyboard walk with focus indicators, target sizes, live regions, contrast (axe suite + computed tables), reduced motion, status-not-color-alone.
- Verify the admin A2UI fixture page renders the re-tokened package themes (closes the last open task from `uar-ui-a2ui-retoken`).
- Record the full verification evidence in `verification-notes.md`.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `uar-web-ui-design-tokens`: themes are reachable and boot-applied (light theme works end-to-end).
- `uar-chat-surface`: chat e2e specs reflect the converged DOM and pass.
- `uar-shell-navigation`: shell selectors used by e2e are stable.

## Impact

- `frontend/src/main.tsx` (theme boot init), `frontend/src/components/layout/top-nav.tsx` (ThemeToggle), `frontend/e2e/*.spec.ts` (selector/wait repairs), `frontend/playwright.config.ts` (expect timeout), `frontend/playwright.real-server.config.ts` (boot budget).
- 24 screenshots under this change's `screenshots/` directory.
- Deferred by nature: the suite retains environment-dependent specs (`knowledge-rag-real` needs an embedding-capable backend; `provider-route-real` runs green under its dedicated config) and SSE-timing flakes documented in the notes.
- Full Tauri desktop window launch (drag/controls on a real window) remains manual verification — config and titlebar logic are inspected, not machine-verified here.
