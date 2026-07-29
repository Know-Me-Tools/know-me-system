---
type: Reference
id: uar-submodule-current-mobile-surrealkv-persistence-is-wiring-work
title: UAR submodule current; mobile SurrealKV persistence is wiring work
tags:
- uar
- surrealdb
- surrealkv
- durable-persistence
- mobile-runtime
- submodules
- hybrid-app-architecture
links:
- uar-hybrid-app-architecture-phase-status-and-m5a-handoff
- karpathy-progress-20260726T073504Z-model-runtime-per-device
sources:
- stdin
- manual:universal-agent-runtime/uar-hybrid-app-architecture
timestamp: 2026-07-26T15:52:06.858154+00:00
created_at: 2026-07-26T15:52:06.858154+00:00
updated_at: 2026-07-26T15:52:06.858154+00:00
revision: 0
---

## Context

Phase: `uar-hybrid-app-architecture` for `universal-agent-runtime` at:

`/Users/gqadonis/Projects/know-me/know-me-system/rust/vendor/universal-agent-runtime`

This continues the hybrid app architecture work tracked in [UAR hybrid app architecture phase status and M5a handoff](/uar-hybrid-app-architecture-phase-status-and-m5a-handoff.md) and addresses the durable mobile persistence gap previously identified in [Local mode certified on Samsung; restart persistence is unstarted milestone 3.1](/karpathy-progress-20260726T073504Z-model-runtime-per-device.md).

## Phase goals

- Fix the P0 desktop data-loss bug by replacing random-port-per-launch in `src-tauri/src/lib.rs` with a stable, persisted localhost port so IndexedDB, `localStorage`, and service-worker origins survive restarts.
- Convert the Tauri shell from embedding the UAR server in-process to spawning the existing `uar-sidecar` binary via `externalBin` / `plugin-shell`, exposing its fixed port to the operating system.
- Align local-first data storage with the hybrid mobile architecture per-target matrix:
  - Web: keep PGlite `idb://`.
  - Desktop: move data layer into Rust via `pglite-oxide`.
  - Mobile: use SQLite + `sqlite-vec` via `gen_ui_core` FFI.
  - Constraint: `pglite-oxide` has no iOS/Android support per corrected architecture notes.
- Establish the Flutter mobile target where UAR runs entirely on-device using embedded SurrealDB / `surrealkv` persistence, per `TJ-ARCH-MOB-001`.
- Run `/impeccable audit` and `/impeccable critique` across the React frontend to produce a scored UI/UX defect inventory covering brittleness, stalls, freeze paths, and admin console UX.
- Execute prioritized fixes with `/impeccable polish` / `/impeccable harden`.
- Absorb six supplemental Admin/Agents UI changes from `uar-grade-a-upgrade-2026-07`:
  - `sw-scheme-safe-caching`
  - `model-warning-clarity`
  - `provider-first-model-picker`
  - `edit-panel-verification`
  - `governance-reconciliation`
  - `freeze-diagnostics`
- Evaluate migration of the frontend toolchain to TypeScript 7.0 native compiler only after verifying release status, ecosystem compatibility, and migration path from 5.9.3 per dependency-verification rules 22/23.

## Submodule state

No UAR submodule update is available.

- Current pointer: `08a6f4c`
- Branch: `desktop-stable-port-opsx`
- Remote sync: `0` commits ahead, `0` commits behind
- `08a6f4c` is the newest non-Dependabot commit on the entire remote as of `2026-07-25`.
- Commit message: `feat(sdk): re-export SeedMessage from the SDK crate root`
- Next most recent branch is `main` from `2026-07-22`.
- KnowMe's recorded submodule pointer matches the submodule's actual `HEAD`.

Conclusion: updating the pointer would require pointing at a commit that does not exist on the checked remote.

## Nested worktrees found inside the submodule

Two agent worktrees exist inside the UAR submodule. Both branches are at `0` commits ahead of the current branch.

One worktree contains uncommitted changes:

```text
.claude/worktrees/agent-a191a5716dd7fef4c
 M src/embedded.rs               (+171)
 M src/uar/domain/agent_store.rs (+38)
 M src/uar/domain/policy.rs      (+200)
```

Inspection result:

- All modified lines are additive test code inside `mod tests` blocks.
- Tests cover policy resolution against `InMemoryProvider`, an `effective_policy_for_run` helper, and a `built_in_agents_are_protected` assertion.
- No production code changed.
- No durable storage code changed.
- Changes were left uncommitted to avoid misattributing another session's work.

The other worktree contains only an untracked directory:

```text
openspec/changes/embedded-run-policy-and-admin-surface/
```

Remaining repository dirt is in nested submodule:

```text
frontend/packages/prometheus-entity-management
```

That nested submodule should not be touched as part of mobile durable persistence work.

## Durable persistence finding

The mobile restart-loss issue is likely a wiring problem, not a missing provider implementation.

A complete `SurrealDbProvider` already exists:

- Size: approximately 1343 lines.
- Implements all 45 `PersistenceLayer` methods.
- Gated behind the `surreal-backend` feature.
- Constructor explicitly supports embedded endpoints including:
  - SurrealKV
  - in-memory
- Embedded endpoints require no authentication, matching the on-device use case.

This changes the likely scope of `openspec/changes/embed-uar-mobile-offline` task 3.1 from "write durable provider" to "wire `build_local` to durable embedded SurrealKV".

## Required verification before implementation

Before changing `build_local`, confirm:

1. `normalize_endpoint` accepts a sandboxed file path suitable for mobile app storage.
2. The `surreal-backend` feature builds for mobile targets without pulling in unsupported server-only dependencies:
   - `aarch64-linux-android`
   - `aarch64-apple-ios`
3. The KnowMe mobile build can enable or reach `surreal-backend` for embedded UAR.
4. `build_local` can be pointed at an app-sandboxed SurrealKV file instead of `InMemoryProvider`.

## Current position

```text
Position: model-runtime-per-device › embedded-uar-offline-agents › embed-uar-mobile-offline
Status: executing
Progress: changes 4/5
```

Latest verified state:

- UAR submodule needs no update.
- Pointer `08a6f4c` on `desktop-stable-port-opsx` is current with remote.
- Agent worktree changes are additive tests only and remain uncommitted.
- Durable persistence scope is likely limited to wiring an existing SurrealKV-capable provider into `UarEmbeddedRuntime::build_local`.

# Citations

1. stdin
2. manual:universal-agent-runtime/uar-hybrid-app-architecture