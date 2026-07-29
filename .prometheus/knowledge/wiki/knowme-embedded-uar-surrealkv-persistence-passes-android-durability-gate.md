---
type: Reference
id: knowme-embedded-uar-surrealkv-persistence-passes-android-durability-gate
title: KnowMe embedded UAR SurrealKV persistence passes Android durability gate
tags:
- knowme
- embedded-uar-offline-agents
- uar
- surrealkv
- android
- durable-persistence
- openspec
links:
- knowme-uar-persistence-task-remains-blocked-by-submodule-checkout
- embedded-uar-offline-agents-executor-session-complete-change-unknown
sources:
- stdin
timestamp: 2026-07-27T18:44:29.937928+00:00
created_at: 2026-07-27T18:44:29.937928+00:00
updated_at: 2026-07-27T18:44:29.937928+00:00
revision: 0
---

## Context

- Phase: `embedded-uar-offline-agents`
- Project: KnowMe
- Repository root: `/Users/gqadonis/Projects/know-me/know-me-system`
- Captured: `2026-07-27T18:43:32Z`
- Device: `SM-S936U`
- Gate report: `/tmp/knowme-uar-persistence.json`

This updates the earlier blocked persistence state described in [KnowMe UAR persistence task remains blocked by submodule checkout](/knowme-uar-persistence-task-remains-blocked-by-submodule-checkout.md): the Android durable persistence proof for embedded UAR is now passing.

## Result

**Persistence certification passed on Android SM-S936U.**

- Cold start opened the SurrealKV store successfully.
- Fast relaunch opened the SurrealKV store successfully.
- No store lock error occurred.
- No permissions error occurred.
- `force-stop` followed by immediate relaunch with `0s` delay opened cleanly on both gates.

The gate is committed and re-runnable via `certify-durable-uar-persistence.sh`. If future devices, cold boots, or storage-pressure scenarios expose a lock-release race, this gate should catch it.

## Architecture decision: no dedicated connection thread

A dedicated connection-owning thread is **not needed** based on measured device behavior.

Rationale:

- The current architecture opens durable persistence without lock or permission failures on the target Android device.
- `SurrealDbProvider` is already `Send + Sync` behind an `Arc`.
- Required concurrency is handled inside SurrealDB.
- Adding a dedicated connection thread would introduce:
  - queueing,
  - thread lifecycle management,
  - queue backpressure as a new failure mode,
  - additional complexity for a race not reproduced by hardware certification.

A lock race is real in a pathological unit-test scenario: dropping and reopening two handles in the same process within microseconds required retry-with-backoff. On device, however, the OS reaps the process before the new process reaches `open_persistence`, so the background lock release has completed before relaunch.

## Storage backend correction

The durable UAR path uses **SurrealKV**, not RocksDB.

- UAR pins `kv-surrealkv` only.
- `surrealdb-librocksdb-sys` appeared during compilation as a transitive artifact and was previously misattributed as the active backend.
- The code and comment were corrected at commit `6b8f3ae`.

## D1–D4 task status

| Task | State |
|---|---|
| D1 — `surreal-backend` feature | ✅ Cross-compiles for host, `aarch64-linux-android`, and `aarch64-apple-ios` |
| D2 — `open_persistence` swap in `rust/crates/gen_ui_agent/src/uar.rs` | ✅ `InMemoryProvider` removed from durable path; no fallback on failure |
| D3 — path threaded from both boot paths | ✅ `rust/crates/gen_ui_host/src/lib.rs:291` and `rust/crates/gen_ui_ffi/src/api/boot.rs` |
| D4 — device durability proof | ✅ PASS; report at `/tmp/knowme-uar-persistence.json` |

## OpenSpec impact

Task `3.1` in `openspec/changes/embed-uar-mobile-offline/tasks.md` is now eligible to be checked off because its device-certification gate has passed.

## Remaining work

Two follow-ups remain undecided:

1. **Re-audit the other 13 tasks** in `embed-uar-mobile-offline/tasks.md`.
   - A reassessment found four headline claims from the `2026-07-20` assessment were false.
   - Remaining checkboxes should not be trusted without re-verifying each against code.
   - This is a re-audit task, not a build task.
2. **Certify desktop macOS durable persistence.**
   - Code is wired to the same `open_persistence` path and SurrealKV engine.
   - No macOS release-bundle restart certification has been run.
   - Same code path is not considered sufficient evidence without runtime proof.

This record provides runtime proof beyond the earlier completion-marker-only records such as [embedded-uar-offline-agents executor session complete, change unknown](/embedded-uar-offline-agents-executor-session-complete-change-unknown.md).

# Citations

1. [1] stdin