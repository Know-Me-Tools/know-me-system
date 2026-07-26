---
type: Reference
id: pr-8-waypoint-fix-and-embed-uar-mobile-offline-resume-plan
title: 'PR #8 waypoint fix and embed UAR mobile offline resume plan'
tags:
- model-runtime-per-device
- uar
- orchestrator-state
- surrealdb
- mobile-offline
- worktree-cleanup
- android-inference
sources:
- stdin
timestamp: 2026-07-26T08:30:48.914041+00:00
created_at: 2026-07-26T08:30:48.914041+00:00
updated_at: 2026-07-26T08:30:48.914041+00:00
revision: 0
---

## Phase context

- Phase: `model-runtime-per-device`
- Project: `KnowMe`
- KBD root/worktree: `/Users/gqadonis/Projects/know-me/know-me-system/.claude/worktrees/frosty-goodall-4cb66c`
- Captured: `2026-07-26T08:19:37Z`
- Position: `model-runtime-per-device › embed-uar-mobile-offline`
- Status: `implementation_ready`
- Progress: `changes 4/5`

## Phase goals

- Ship `Qwen3.5-9B Q4_K_M` with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM `Gemma 4 E2B`, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## PR state and waypoint fix

- PR #7 was merged at `2026-07-26T08:16Z`, but did **not** include the waypoint fix.
  - Merge captured commit `1bfa380` only.
  - Commit `fb896e8` was pushed after the merge.
- Result: `origin/main` still contains the stale destructive waypoint:

```text
exactNextCommand: "/opsx:new uar-ui-shell-navigation"
```

- Risk: if left on `main`, the orchestrator would continue emitting the destructive `/opsx:new uar-ui-shell-navigation` command.
- PR #8 (`https://github.com/Know-Me-Tools/know-me-system/pull/8`) carries the fix:
  - One file changed.
  - Five lines changed.
  - Cherry-picked cleanly onto merge commit `54c5441`.
  - Verified non-no-op: `main` genuinely still has the stale `exactNextCommand` value.

## PR #7 contents now on main

- `uar-ui-shell-navigation` reached `17/17`.
- Verification completed at:
  - `320px`
  - `1440px`
  - dark mode
  - light mode
- Live DOM measurement showed `0px` overlap.
- Composer, hamburger, and drawer gaps were recorded honestly in `verification-notes.md` rather than hidden.

## Safe cleanup after PR #8 merges

After PR #8 merges, no unique work should remain in the frosty-goodall worktree. Remove it manually because the operation is irreversible:

```bash
git worktree remove --force /Users/gqadonis/Projects/know-me/know-me-system/.claude/worktrees/frosty-goodall-4cb66c
```

- Reclaims approximately `2.1G`.
- `--force` is required because of an incidental dirty submodule pointer.
- Approximately `892M` is `node_modules` plus a built `dist/`.
- Anyone returning to the UAR frontend should run:

```bash
pnpm install
pnpm -r build
```

- The non-obvious build command applies in `frontend/packages/prometheus-entity-management`.
- Main checkout is currently on `feat/ios-mlx-inference` with uncommitted changes; it was intentionally not touched.

## Orchestrator issues still present on main

These were flagged in PR #8 but not fixed:

1. `position.json` is stale.
   - It reports `embed-uar-mobile-offline` at `0/0`.
   - `tasks.md` reports `8/22`.
   - Treat `position.json` as derived state; regenerate with `kbd_position_sync` rather than hand-editing.
2. `progress.json` is missing for the phase.
   - Likely cause of state no longer advancing automatically.
   - Fix should happen at the source of phase progress generation.

## Task 3.1 resume plan

Resume after PR #8 merges and the worktree is removed.

Implementation direction:

- Reuse `SurrealDbProvider`.
  - It connects through `engine::any`.
  - It normalizes a bare path into `surrealkv://<path>`.
  - It skips auth for embedded endpoints.
  - No mobile-specific adapter is needed.
- Root cause of current blocker:
  - `gen_ui_agent/Cargo.toml` enables `in-memory-backend`.
  - It does **not** enable `surreal-backend`.
  - `providers/surreal.rs` is cfg-gated behind `surreal-backend`.

Open questions before coding:

1. The referenced pinning tests do not exist on this branch. Decide whether to write them fresh.
2. Enabling `surreal-backend` pulls `dep:surrealdb` into the mobile build. Confirm whether the size impact is acceptable.

Next implementation steps:

1. Merge PR #8 so `main` stops emitting destructive `/opsx:new`.
2. Remove the stale worktree to reclaim `2.1G`.
3. Resume Task 3.1 on `main`.
4. Answer the two open questions.
5. Swap `InMemoryProvider` for `SurrealDbProvider`.
6. Run the Samsung restart gate.

# Citations

1. stdin