---
type: Reference
id: uar-hybrid-architecture-s4-s5-verification-and-s6-plan
title: UAR hybrid architecture S4/S5 verification and S6 plan
tags:
- uar
- hybrid-app-architecture
- mobile-runtime
- admin-tests
- gen-ui-agent
- tauri
- ssrf-hardening
- xberg
links:
- uar-hybrid-app-architecture-phase-status-and-m5a-handoff
- uar-submodule-current-mobile-surrealkv-persistence-is-wiring-work
sources:
- stdin
timestamp: 2026-07-28T03:56:09.625629+00:00
created_at: 2026-07-28T03:56:09.625629+00:00
updated_at: 2026-07-28T03:56:09.625629+00:00
revision: 0
---

## Context

Phase: `uar-hybrid-app-architecture` for `universal-agent-runtime` at:

`/Users/gqadonis/Projects/know-me/know-me-system/rust/vendor/universal-agent-runtime`

This continues the broader hybrid-app work tracked in [UAR hybrid app architecture phase status and M5a handoff](/uar-hybrid-app-architecture-phase-status-and-m5a-handoff.md) and follows the mobile persistence wiring context in [UAR submodule current; mobile SurrealKV persistence is wiring work](/uar-submodule-current-mobile-surrealkv-persistence-is-wiring-work.md).

## Phase goals

- Fix the P0 desktop data-loss bug by replacing random-port-per-launch in `src-tauri/src/lib.rs` with a stable, persisted localhost port so IndexedDB, `localStorage`, and service-worker origins survive restarts.
- Convert the Tauri shell from embedding the UAR server in-process to spawning the existing `uar-sidecar` binary via `externalBin` / `plugin-shell`, exposing its fixed port to the OS.
- Align the local-first data layer to the hybrid mobile architecture per-target matrix:
  - Web: keep PGlite `idb://`.
  - Desktop: move the data layer into Rust via `pglite-oxide`.
  - Mobile: use SQLite + `sqlite-vec` through `gen_ui_core` FFI.
  - Constraint: `pglite-oxide` has no iOS/Android support per the corrected architecture notes.
- Establish the Flutter mobile target where UAR runs entirely on-device using embedded SurrealDB / `surrealkv` persistence, per `TJ-ARCH-MOB-001`.
- Run `/impeccable audit` and `/impeccable critique` across the React frontend to produce a scored UI/UX defect inventory covering brittleness, stalls, freeze paths, and admin console UX.
- Execute prioritized fixes with `/impeccable polish` / `/impeccable harden`.
- Absorb six supplemental Admin/Agents UI work items from `uar-grade-a-upgrade-2026-07`:
  - `sw-scheme-safe-caching`
  - `model-warning-clarity`
  - `provider-first-model-picker`
  - `edit-panel-verification`
  - `governance-reconciliation`
  - `freeze-diagnostics`
- Evaluate migration of the frontend toolchain to TypeScript 7.0 native compiler before committing:
  - Verify release status.
  - Verify ecosystem compatibility with Vite/Rolldown, ESLint, and `vue-tsc` equivalents.
  - Define migration path from TypeScript 5.9.3.
  - Follow dependency-verification rules 22/23.

## Current state

### S4: verified and committed

- Commit: `9b894ef`.
- Result: 9/9 admin tests pass.
- Commitlint emitted one warning about a blank line; it was not a failure.

Key properties pinned by tests:

- `ios_never_executes_locally_even_with_scripts`
  - iOS cannot degrade to `Native` under any input combination.
  - This is asserted directly because silent fallback to native execution would be a correctness failure rather than an obvious visible error.
- `unavailability_distinguishes_permanent_from_transient`
  - iOS reports `platform_limitation: true` because it can never spawn processes.
  - Android reports `platform_limitation: false` because the condition is a missing reachable server, not a permanent platform impossibility.
  - UI implication: do not offer retry for the iOS permanent limitation; do not imply permanence for Android transient unavailability.

### S5: compile-clean, test suite pending at capture time

- `gen_ui_agent` builds with control-plane wiring.
- Full `gen_ui_agent` suite was running under the harness at capture time.
- Decision: do not commit S5 until the full suite is green.

### S6: researched and planned, not started

Planned S6 implementation:

- Native fetch tool.
- SSRF hardening.
- `xberg` binary extraction.

Findings:

- UAR already depends on `xberg` behind the `document-intelligence` feature.
  - Binary extraction needs no new dependency in either repository.
- Upstream `robots.txt` handling splits behavior by dual user-agent:
  - `Autonomous`
  - `User-Specified`
- S6 should mirror that dual user-agent model.

## Operational lessons from the stall

The stall had three independent causes:

1. A cross-session Cargo lock: another Claude session, `sansaba` / `fdb-gateway`, held the shared `~/.cargo` cache. The correct action was to wait rather than kill another session's process.
2. A `pkill` killed the process being awaited, leaving a watcher spinning against a log that would never update.
3. `setsid` is unavailable on macOS. The detach attempt died immediately. Use the harness's own background mechanism instead; it survives properly.

## Prometheus position

```text
Position: embedded-uar-offline-agents › embed-uar-mobile-offline
Status: implementation_ready
Progress: changes 0/1
Last: S4 VERIFIED and committed (UAR 9b894ef, 9/9 admin tests incl. iOS-never-native); S5 compiles clean, full gen_ui_agent suite running under the harness
Next: read the gen_ui_agent test result and commit S5; then implement S6 (native fetch tool with SSRF hardening + xberg binary extraction)
```

# Citations

1. stdin