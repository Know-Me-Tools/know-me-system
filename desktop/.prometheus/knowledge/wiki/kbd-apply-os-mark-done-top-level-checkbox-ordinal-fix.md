---
type: Reference
id: kbd-apply-os-mark-done-top-level-checkbox-ordinal-fix
title: kbd-apply os_mark_done Top-Level Checkbox Ordinal Fix
tags:
- kbd-apply
- openspec
- checkbox-ordinals
- awk
- regression-tests
- prometheus-entity-management
sources:
- stdin
- manual:prometheus-entity-management/phase-v3-universal-platform-evolution
- ~/.claude/skills/kbd-apply/kbd-apply.sh
- ~/.claude/skills/kbd-apply/tests/os_mark_done_ordinals.bats
timestamp: 2026-07-16T20:37:33.992516+00:00
created_at: 2026-07-16T20:37:33.992516+00:00
updated_at: 2026-07-16T20:37:33.992516+00:00
revision: 0
---

## Context

- **Project:** `prometheus-entity-management`
- **Phase:** `phase-v3-universal-platform-evolution`
- **KBD root:** `~/Projects/prometheus/prometheus-entity-management/.claude/worktrees/dazzling-wing-870ab7`
- **Captured:** `2026-07-16T20:36:34Z`

## Phase goals in scope

The broader phase aims to evolve `@prometheus-ags/prometheus-entity-management` from a React-only entity graph into a cross-platform, multi-framework, AI-native, local-first, peer-syncing, code-generating ecosystem.

### P0 foundation goals

- Extract `entity-graph-core` as a framework-agnostic Zustand store/engine/transports/adapters/CRUD relations package with zero React dependencies.
- Migrate existing React bindings to `entity-graph-react` as a peer consumer.
- Reduce bundle size and improve tree-shaking via `sideEffects: false` and optional sub-packages for `table/`, `view/`, and `crud/`.
- Add list virtualization with `@tanstack/react-virtual` via `useVirtualizedEntityList`.
- Add parallel query support via `useEntityQueries`.
- Implement column resizing to unblock the stubbed `getResizeHandler`.
- Add SSR dehydration/rehydration via `dehydrateGraph()` / `rehydrateGraph()`, compatible with Next.js 15+ App Router streaming.
- Define a Schema Definition Language consumed by all code generators, using `schema.json` / `entity-graph.toml`.

### P1 web binding goals

- Add `entity-graph-svelte` for Svelte 5 using runes-based wrappers (`$state`, `$derived`, `$effect`) around the vanilla Zustand store.
- Add `entity-graph-solid` for SolidJS using `createResource`-based bindings with fine-grained reactivity.

## Bug fixed: mismatched checkbox ordinals

`kbd-apply.sh` had a numbering mismatch in `os_mark_done`.

### Root cause

- `os_mark_done` used an awk checkbox-matching pattern that counted checkboxes at any indentation:

```awk
^[[:space:]]*-[[:space:]]*\[[ xX]\]
```

- `os_list` and `os_progress` defer to:

```bash
openspec instructions apply --json
```

- Empirical verification against `openspec` `1.6.0` showed the JSON API counts only **top-level, non-indented** checkboxes.
- Nested sub-bullets under a parent task are excluded from `openspec` numbering entirely.
- Because `os_mark_done` counted nested checkbox lines while `os_list` did not, the ordinal shown to the user could mutate the wrong line.

### Fix

Changed both awk patterns in `os_mark_done` from indentation-tolerant matching to top-level-only matching:

```diff
- ^[[:space:]]*-[[:space:]]*\[[ xX]\]
+ ^-[[:space:]]*\[[ xX]\]
```

This makes `os_mark_done` count only unindented checkbox lines, matching `openspec instructions apply --json` behavior.

## Verification

A fixture reproduced the reported scenario:

- 8 top-level tasks
- 5 nested sub-bullets under task 3

Results:

- **Before fix:** `mark-done 8` flipped the 5th nested sub-bullet instead of top-level task 8.
- **After fix:** ordinals 1–8 map to the correct top-level task lines.
- Nested sub-bullets are never touched by `os_mark_done` ordinal selection.

## Regression coverage

Added regression suite:

```text
~/.claude/skills/kbd-apply/tests/os_mark_done_ordinals.bats
```

Coverage:

- 4 passing Bats tests.
- Exact nested-fixture scenario.
- Includes a test that calls the real `os_list`, backed by the `openspec` CLI, to confirm its count and ordinals remain in lockstep with `os_mark_done`.

## Explicit non-changes

No changes were made to:

- `sk_mark_done` — Spec Kit adapter.
- `nk_mark_done` — native-kbd adapter.

Rationale: both already source numbering from the same file they mutate, so they do not have the split-source-of-truth issue seen in `os_list`/`os_mark_done`.

# Citations

1. stdin
2. manual:prometheus-entity-management/phase-v3-universal-platform-evolution
3. ~/.claude/skills/kbd-apply/kbd-apply.sh
4. ~/.claude/skills/kbd-apply/tests/os_mark_done_ordinals.bats
