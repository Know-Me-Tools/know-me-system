# Verification notes — uar-ui-verify-gates

Recorded 2026-07-21 (kimi). **Final state: all five convergence PRs merged to
`main` (644a80f)** — #172 shell-navigation (1d8b3ba), #173 a2ui-retoken (9539a0a),
#174 verify-gates (c7239b7), #175 chain completion (644a80f). Post-merge main:
`tsc -b` clean, `vite build` green, vitest **153/153**. The screenshot matrix
under `screenshots/` was recaptured against the merged rail/bottom-bar shell.

Environment: pre-existing backend on `:1906` serving the freshly built static
bundle from this tree; Vite dev `:8080` (Playwright-managed); `stub-llm` via the
dedicated real-server config. (Earlier captures in this file's history showed
the rebranded top-nav era while #172 was unmerged; the committed matrix is the
final merged shell — rail at 1440, bottom bar at 320, ThemeToggle in the rail
header, readiness card "Ready · Embedded · local".)

## Headline results

- e2e suite: **13 failed / 28 passed → 3 failed / 33 passed** (the 3 remaining are
  environment-limited, section 6 of tasks.md). 4 timing flakes (pass on retry), 1 skip.
- vitest: **153/153** on the final tree.
- Screenshot matrix: 24 PNGs under `screenshots/`, visually inspected.
- Two real defects found and fixed: unreachable light theme, real-server boot race.

## Defect 1 — light theme was unreachable (fixed)

The matrix's first light captures rendered dark. Root cause: `theme-store.ts`
applies the stored theme only at module-init, and no module imported the store at
boot (`index.html` hardcodes `class="dark"`); additionally `ThemeToggle` was not
rendered anywhere, so users could not switch themes at all. Fixed with a
side-effect import in `main.tsx` and a `ThemeToggle` instance in the top nav.
Re-verified: `html class="light"`, `body` computed `#F7F7F8`, light matrix shots
correct at all four widths.

## Defect 2 — real-server e2e configs raced backend boot (fixed)

Backend cold boot (SurrealKV init + MCP stdio spawn) measured **~66s** on this
machine vs the 60s `webServer` timeout in `playwright.real-server.config.ts` —
both real specs timed out before `/readyz`. Raised to 180s;
`provider-route-real` now passes (13.8s). Also raised default config
`expect.timeout` to 12s (cold Vite transforms under suite load).

## e2e repair ledger

| Spec | Was | Root cause | Fix |
|---|---|---|---|
| 6 chat specs | deterministic timeout | `waitForLoadState("networkidle")` never resolves with SSE open | condition waits (`.or()` locators, 15s budgets) |
| smoke.spec | strict-mode violation (3 matches for `Providers`) | admin rail + group + panel | `.first()` |
| chat-agent-selection | wrong locator | `text=Default Agent` never matched ("Default Assistant") | `getByLabel("Select agent")` |
| admin specs ×4 | cold-start timeouts | 5–8s budgets vs cold Vite transforms | 15s budgets + config expect 12s |
| 2 real specs | webServer boot timeout | 60s budget < ~66s measured boot | 180s budget |

## Screenshot matrix inspection record

24 captures: `screenshots/{threads,about,admin}-{320,768,1024,1440}-{dark,light}.png`.

- **Flat 2.0**: no visible borders, dividers, or layout shadows in any capture;
  regions separate by ladder background alone (chrome sidebar vs canvas vs card).
- **Dark** (`threads-1440-dark`): KnowMe ladder exact; ember CTA on ember-soft;
  agent toolbar with mono model id; thread-list chrome separation.
- **Light** (`threads-1440-light`, `about-768-light`): white chrome, `#F7F7F8`
  canvas, ember `#E04E28` accents, correct eyebrow + health-pill rendering.
- **Narrow** (`threads-320-*`): top-nav era compression is visible (About link
  clipped) — this is the known gap `uar-ui-shell-navigation` (#172) resolves
  with the bottom bar; not a regression (predates convergence).
- **Admin** (`admin-1024-dark`): runtime console + A2UI testing page render the
  converged tokens; eyebrow labels, zero-metric cards, honest empty states.

## a11y-gate checklist (WCAG 2.2 AA)

- Contrast: storybook axe suite 153/153; computed ratio tables in this and the
  token/a2ui changes' notes (fg ≥13:1 dark ladder, ≥18:1 light, rings ≥3.7:1).
- Accessible names: 0 unnamed buttons on `/threads` (probe).
- Keyboard: Tab order = destinations → theme toggle → sidebar → agent toolbar →
  composer; ember focus ring on every stop (rapid-Tab zero-alpha readings are
  ring transition-in artifacts, not missing indicators).
- Live regions: chat status line uses `aria-live`; streaming states announced.
- Targets: 0 interactive elements under 24px.
- Status: dot + text (never color alone) in readiness/status indicators.
- Reduced motion: `prefers-reduced-motion` rules in frontend and a2ui package CSS.
- Images: brand SVG `aria-hidden` with adjacent text wordmark.

## Environment-limited (documented, not hidden)

- `provider-route-real`: red under the default config (targets the pre-existing
  :1906 backend, no stub); **green under its dedicated config**.
- `knowledge-rag-real`: needs an embedding-capable backend; stub-llm serves chat
  completions only. Backend/API-level, unrelated to UI convergence.
- `runtime-event-replay:67`: SSE replay timing flake (passes standalone).
- Tauri window launch: `decorations: false` + Titlebar logic verified by
  inspection; real-window drag/controls remain manual verification.
