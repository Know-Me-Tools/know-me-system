---
type: Reference
id: knowme-desktop-uar-surrealkv-persistence-certified-by-wal-durability-gate
title: KnowMe desktop UAR SurrealKV persistence certified by WAL durability gate
tags:
- knowme
- embedded-uar-offline-agents
- uar
- surrealkv
- desktop
- durable-persistence
- runtime-verification
links:
- knowme-embedded-uar-surrealkv-persistence-passes-android-durability-gate
- embedded-uar-offline-agents-executor-session-complete-change-unknown
sources:
- stdin
timestamp: 2026-07-27T19:27:36.623080+00:00
created_at: 2026-07-27T19:27:36.623080+00:00
updated_at: 2026-07-27T19:27:36.623080+00:00
revision: 0
---

## Context

- Phase: `embedded-uar-offline-agents`
- Project: KnowMe
- Repository root: `/Users/gqadonis/Projects/know-me/know-me-system`
- Captured: `2026-07-27T19:19:50Z`
- Position: `embedded-uar-offline-agents › embed-uar-mobile-offline`
- Status: `implementation_ready`

Desktop durable persistence is **certified** for the embedded UAR offline-agent path. This complements the Android certification in [KnowMe embedded UAR SurrealKV persistence passes Android durability gate](/knowme-embedded-uar-surrealkv-persistence-passes-android-durability-gate.md), while leaving iOS persistence uncertified.

## Desktop durability evidence

`uar-db` did not exist before this run because the old bundle predates the persistence change. Its creation is direct evidence that the new code path executed rather than reusing a leftover store.

| Evidence | Result |
|---|---|
| Store created on boot | `LOCK`, `wal/…wal`, `manifest/…manifest` appeared, indicating a real SurrealKV store |
| Survives fast relaunch | 3 files before and after relaunch; no lock error |
| Reopened rather than recreated | WAL grew monotonically across launches: `16199 → 16933 → 23348 → 47079 → 53642` bytes; never truncated |
| Binary contains persistence code markers | `cannot open persistent store`, `surrealkv://`, and `uar-db` all present |

The WAL monotonicity check is the durable-persistence proof. File count alone is insufficient because an empty recreated store can still contain the same three filenames (`LOCK`, WAL, manifest), matching the disk shape of a working persistent store while losing contents.

## Negative control correction

The first negative control was invalid:

- It truncated the WAL to `0`.
- The gate checked for shrinkage, but no value can be less than `0`.
- The app rewrites the WAL on boot, so this only tested arithmetic, not gate behavior.

The valid negative control injected a wipe **between the gate's two phases** with a `PATH` shim that counted launches:

- WAL changed from `47079` bytes to `9167` bytes.
- Gate emitted `WAL SHRANK` and `PERSISTENCE FAIL`.
- Process exited with status `1`.
- File count remained `3`, proving the old file-count-only check would have passed a destructive wipe.

## Build watcher correction

The build watcher initially keyed on `.app` directory mtime. That timestamp changed at `14:10` while `tauri build` was still compiling `surrealdb_core` and `wasmtime`, so it was not a valid completion signal.

Correct behavior:

- Re-arm the watcher on the build process exit, not `.app` mtime.
- The build finished cleanly in `18m16s`.
- The binary was final, so no artifact was lost; only the completion signal was wrong.

## Task status

Task 5.3 is marked partial (`[/]`), not complete (`[x]`). Persistence is certified, but the task also covers:

- desktop prompting
- tool loop
- A2UI interaction
- cancellation
- persistence

Only the persistence surface is certified. Marking the full task complete would hide unverified surfaces, which is the false-positive checkbox this re-audit is meant to prevent.

## Remaining verification gaps

- The adversarial re-audit of the remaining 13 tasks is still running.
- iOS persistence remains uncertified.
- iOS shares `open_persistence` with other platforms but boots through `gen_ui_ffi`, same as Android; the Android pass is useful supporting evidence but not conclusive for iOS.
- Remaining Task 5.3 desktop surfaces still require certification: prompting, tool loop, A2UI, and cancellation.

This record provides runtime proof and should be treated differently from completion-marker-only records such as [embedded-uar-offline-agents executor session complete, change unknown](/embedded-uar-offline-agents-executor-session-complete-change-unknown.md), which identify no verified runtime behavior.

# Citations

1. [1] stdin