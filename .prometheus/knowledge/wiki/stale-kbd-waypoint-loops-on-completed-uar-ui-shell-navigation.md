---
type: Reference
id: stale-kbd-waypoint-loops-on-completed-uar-ui-shell-navigation
title: Stale KBD waypoint loops on completed uar-ui-shell-navigation
tags:
- model-runtime-per-device
- kbd-waypoint
- orchestrator-state
- uar-ui
- offline-agents
- mobile-persistence
- samsung-gate
sources:
- stdin
timestamp: 2026-07-26T08:01:40.581214+00:00
created_at: 2026-07-26T08:01:40.581214+00:00
updated_at: 2026-07-26T08:01:40.581214+00:00
revision: 0
---

## Context

- Phase: `model-runtime-per-device`
- Project: `KnowMe`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system/.claude/worktrees/frosty-goodall-4cb66c`
- Captured: `2026-07-26T08:00:18Z`
- Position: `model-runtime-per-device › embedded-uar-offline-agents › embed-uar-mobile-offline`
- Status: `implementation_ready`
- Progress: changes `4/5`

## Phase goals

- Ship `Qwen3.5-9B Q4_K_M` with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM `Gemma 4 E2B`, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Root cause

`current-waypoint.json` is stale state from `2026-07-21`, written by a different tool (`sourceTool: kimi`). Its `exactNextCommand` is hardcoded to:

```bash
/opsx:new uar-ui-shell-navigation
```

The hook reads `exactNextCommand` verbatim on every turn, so it repeatedly emits the same command regardless of whether the user complies or explains the state. Because `/opsx:new uar-ui-shell-navigation` targets an already-completed change, running it would be destructive and could overwrite completed work.

## Stale waypoint fields

| Field | Current stale value | Actual state |
|---|---|---|
| `exactNextCommand` | `/opsx:new uar-ui-shell-navigation` | Change already exists and is complete; `/opsx:new` would overwrite it |
| `nextPendingChange` | `uar-ui-shell-navigation` | No longer pending |
| `status` | `implementation_ready` | Non-terminal, so the gate treats every stop as premature |
| `updatedAt` | `2026-07-21T18:40:00Z` | 5 days stale at capture time |

## Correct remediation

Do **not** hand-edit orchestrator state to silence the gate. The owning KBD skill should reconcile the waypoint against real phase progress:

```bash
/kbd-status
```

If `/kbd-status` does not clear the stale pointer, run:

```bash
/kbd-goal-check
```

`/kbd-goal-check` evaluates whether the phase stopping condition is met.

## Actual workstream state

### `uar-ui-shell-navigation`

- Complete: `17/17` tasks.
- `openspec validate` is clean.
- Smoke verification passed at:
  - `320px` and `1440px`
  - dark and light themes
  - `0px` overlap measured on the live DOM
- Verification notes recorded two remaining gaps:
  - Composer, hamburger, and drawer do not mount without a backend; deferred to `uar-ui-verify-gates`.
  - Theme toggle is unreachable below `768px`.

### `embed-uar-mobile-offline` task 3.1

- Still unchecked.
- Not blocked.
- Submodule is fully present; the earlier claim that it was uninitialized was a path error.
- This is the durable sandboxed persistence work originally assigned.
- Samsung restart bug remains live.
- Remaining task 3.1 work:
  - Read `PersistenceLayer` trait.
  - Read `providers/surreal.rs`.
  - Decide reuse-vs-write based on evidence.
  - Implement adapter.
  - Keep the two pinned-shape tests green.
  - Run Samsung restart verification gate.

## Recommended next sequence

1. Run `/kbd-status` to reconcile `current-waypoint.json` and stop the stale hook loop.
2. Resume `embed-uar-mobile-offline` task 3.1 for durable sandboxed persistence.

# Citations

1. stdin