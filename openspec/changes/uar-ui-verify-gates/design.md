## Context

The UAR→KnowMe convergence shipped four changes that each deferred live verification. This change is the certification pass. Environment: a pre-existing backend instance on `:1906` (serving the freshly built static bundle from the working tree), Vite dev on `:8080` (Playwright-managed), and `stub-llm` for provider-backed specs via their dedicated real-server config.

## Goals / Non-Goals

**Goals:**

- e2e suite tells the truth: failures are product bugs, environment gaps, or timing — never broken selectors or impossible waits.
- Screenshot matrix exists and is inspected: 3 routes × 4 widths × 2 themes.
- a11y-gate checklist completed against the running app with evidence.
- Any defect found by the gates is fixed in this change (theme boot init, ThemeToggle, e2e infrastructure).

**Non-Goals:**

- No new UI features; no shell redesign (that was #172).
- No CI workflow changes (the repo's CI orchestration is out of scope; configs are fixed, pipelines are not).
- No Tauri binary launch certification (manual; config/logic inspection only).
- No embedding-capable backend provisioning for `knowledge-rag-real`.

## Decisions

### `networkidle` is the wrong wait for an SSE app — use condition waits

With the entity-sync SSE stream open, `page.waitForLoadState("networkidle")` never resolves and every test using it times out deterministically (6 specs). Replaced with `domcontentloaded` plus explicit element-visibility waits with real budgets (`{ timeout: 15000 }`) — this tests what users see, not network quiescence that never comes.

### Strict-mode violations get `.first()` only where any-match is semantically right

`text=Providers` matches 3 elements on `/admin` (rail item, section group, context panel) — the assertion intent is "Providers appears", so `.first()` is the honest fix, not a page change. Where intent was component-specific (agent selector), the locator was corrected to the component's actual contract (`aria-label="Select agent"` — the spec's `text=Default Agent` never matched the real label "Default Assistant").

### Boot-time theme application is a product bug, fixed at the root

`theme-store.ts` applies the stored theme only at module-init, and nothing imported the store at boot — `index.html`'s `class="dark"` always won, and `ThemeToggle` was rendered nowhere, so light/high-contrast were unreachable. Fixed with a side-effect import in `main.tsx` (root cause) and a visible `ThemeToggle` in the top nav (reachability). Found by the screenshot matrix, not by any test — exactly what the matrix is for.

### Boot budget reflects measured cold start

Backend cold boot (SurrealKV init + MCP stdio spawn) measured ~66s on this machine; the real-server config's 60s timeout raced it. Raised to 180s. Config-level Playwright expect timeout raised to 12s for cold Vite transforms under suite load.

## Risks / Trade-offs

- **Longer timeouts slow failure discovery** — accepted; retries already cost more.
- **Environment-dependent specs stay red in the default config** (`*-real` without their dedicated config; `knowledge-rag-real` needs embeddings). Documented in verification-notes rather than hidden with skips.
- **Verification was performed against the main+PR173 bundle** (shell-nav #172 unmerged at capture time); the matrix is honest about that and will be re-run cheaply once #172 merges (the rail/bottom-bar screenshots then replace the top-nav ones).
