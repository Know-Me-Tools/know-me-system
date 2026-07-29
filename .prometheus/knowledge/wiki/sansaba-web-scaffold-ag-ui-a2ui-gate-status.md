---
type: Reference
id: sansaba-web-scaffold-ag-ui-a2ui-gate-status
title: Sansaba web scaffold AG-UI/A2UI gate status
tags:
- sansaba-workspace
- web-scaffold
- ag-ui
- a2ui
- axum-bff
- local-first
- react-vite
- phase-status
links:
- pem-sync-bridge-and-mobile-tier-phase-status-for-pr-17
sources:
- stdin
- manual:Sansaba Workspace/web-scaffold-and-c29-gate
timestamp: 2026-07-27T18:27:15.709142+00:00
created_at: 2026-07-27T18:27:15.709142+00:00
updated_at: 2026-07-27T18:27:15.709142+00:00
revision: 0
---

## Phase context

- Phase: `web-scaffold-and-c29-gate`
- Project: `Sansaba Workspace`
- KBD root: `/Users/gqadonis/Projects/sansaba/San Saba Automation/sansaba-workspace`
- Captured: `2026-07-27T18:26:08Z`
- Position: `web-scaffold-and-c29-gate › letter-agreement-c29-gate`
- Status: `apply_ready`
- Progress: `changes 4/7`; `letter-agreement-c29-gate 3/29`

## Phase goals

Implementation target is the project documentation:

- `CLAUDE.md`
- `docs/SSW-WEB-002-scaffold-plan.md`
- `docs/SSW-ARCH-001-architecture-spec-and-plan.md` Appendix C
- `docs/branding/branding-guide.md`

### G-1: Install KnowMe project skills

Run `add-project-skills.sh` against this repo per SSW-WEB-002 step 2 so project skills are installed next to authored skills:

- `pem-local-first`
- `sync-doctrine`
- `content-block-ui`
- `reference-ui-fidelity`
- `hybrid-design-tokens`
- `a11y-gate`

Resolved script location, verified present on `2026-07-27`:

- `~/.claude/plugins/marketplaces/knowme-hybrid-architecture/scripts/add-project-skills.sh`
- Cache copy: `~/.claude/plugins/cache/knowme-hybrid-architecture/hybrid-mobile-architecture/1.1.0/scripts/`

PEM local-first expectations align with the broader sync doctrine tracked in [PEM sync bridge and mobile tier phase status for PR 17](/pem-sync-bridge-and-mobile-tier-phase-status-for-pr-17.md).

### G-2: Scaffold `web/` per SSW-WEB-002

Expected client at `web/`:

- React 19
- Vite 8
- TypeScript 7
- shadcn-ui
- Tailwind 4
- PEM `3.0.0-alpha.0` + PGlite local-first storage
- Zustand 5 for transient UI state only
- TanStack Router/Table
- Assistant-UI
- `bridge/{a2ui,agui}` layer

Binding constraints from SSW-WEB-002 §1 and §3:

- Do **not** run `scaffold-tauri.sh` as the app generator.
- Scaffold Vite fresh.
- Transport is **web REST + SSE to the Axum BFF**, not Tauri IPC.
- Strip Tauri coupling:
  - no `src-tauri/`
  - no `invoke()` / `listen()` in stores

## AG-UI implementation status

AG-UI is fully implemented. The endpoint was live; the prior 404 was caused by using the display name from `/api/agents` instead of the registry key required by `/agui/:agentName`.

Correct agent key:

- `documentGenerationAgent`

Using the registry key caused a real run to stream immediately and exposed deployed endpoint defects.

## AG-UI protocol defects fixed

Three real endpoint bugs were visible in one streamed response:

1. **Duplicate `RUN_FINISHED`**
   - Mastra emitted both a `finish` chunk and a terminal chunk.
   - Clients that reset state on `RUN_FINISHED` reset twice and discarded a run that had just rendered.
2. **Non-spec thinking events**
   - Endpoint emitted `CUSTOM/THINKING_DELTA`.
   - AG-UI spec expects `THINKING_TEXT_MESSAGE_CONTENT`.
   - Conforming clients silently dropped extended-thinking output.
3. **`TOOL_CALL_END` carried the result**
   - Spec separates tool-call termination from tool result emission.
   - Collapsing them made a cancelled tool indistinguishable from a successful tool.

## AG-UI coverage

Event coverage increased from `10/33` to `33/33`, verified by diffing against `@ag-ui/core@0.0.57` rather than a hand-written list.

The 23 missing events included:

- entire reasoning event group
- `STATE_DELTA`

`STATE_DELTA` is required so the shell can patch its entity graph without rewriting every node and churning every bound surface.

## A2UI status

A2UI round-trips correctly:

1. tool result
2. `A2UI_ARTIFACT`
3. client
4. `a2uiResponse`

Added missing support for a **flint-forge catalog client**, enabling specs to come from a shared registry in addition to local files.

### A2UI spec resolution policy

Supported locations:

- Local hot-reloaded specs: `workspace/a2ui/<name>.a2ui.json`
- Published shared specs: `flint_a2ui.components`

Conflict policy:

- Local specs win over catalog specs.
- Rationale: a repo-local schema changes atomically with the tool that emits it; a catalog entry is shared platform-wide, and silent registry override would make agent behavior depend on an unrelated deploy.

Catalog resilience:

- Forge unreachable degrades to `not found`, not a failed run.
- Requests are bounded at `3s/5s`, preventing a slow registry from stalling an agent.

Template reference in repo:

- `document-generation-agent/src/mastra/server/routes/agui/CONFORMANCE.md`

## Concurrency design decision

Mapper state is now threaded per run instead of module-scoped.

Rationale:

- The handler serves concurrent runs.
- Shared mutable mapper state could interleave message IDs between concurrent runs.
- A test now covers this concurrency case.

## Validation

- New tests: `22`
- Passing tests: `268/268`
- Test files: `31`
- Typecheck: clean
- Whole-project typecheck OOMs at Node default heap due to pre-existing repo condition.
- Whole-project typecheck passes with `--max-old-space-size=8192`.

## Next work

Run:

```bash
/kbd-apply letter-agreement-c29-gate
```

Remaining work:

- `26` tasks left in `letter-agreement-c29-gate`
- Start with feature structure and Mini-App manifest.
- Build the `letter-agreement-precheck` artifact in the agent.

# Citations

1. [1] stdin
2. [2] manual:Sansaba Workspace/web-scaffold-and-c29-gate