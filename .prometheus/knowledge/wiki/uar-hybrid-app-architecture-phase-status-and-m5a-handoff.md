---
type: Reference
id: uar-hybrid-app-architecture-phase-status-and-m5a-handoff
title: UAR hybrid app architecture phase status and M5a handoff
tags:
- uar
- hybrid-app-architecture
- tauri
- mobile-runtime
- embedded-runtime
- openspec
- react-ui
links:
- embedded-uar-provider-routing-and-mobile-parity
- karpathy-progress-20260723T013607Z-model-runtime-per-device
sources:
- stdin
- manual:universal-agent-runtime/uar-hybrid-app-architecture
timestamp: 2026-07-24T15:32:36.999195+00:00
created_at: 2026-07-24T15:32:36.999195+00:00
updated_at: 2026-07-24T15:32:36.999195+00:00
revision: 0
---

## Phase goals

- Fix P0 desktop data loss by replacing random-port-per-launch in `src-tauri/src/lib.rs` with a stable, persisted localhost port so IndexedDB, `localStorage`, and service-worker origins survive restarts.
- Convert the Tauri shell from embedding the UAR server in-process to spawning the existing `uar-sidecar` binary via `externalBin` / `plugin-shell`, exposing its fixed port to the operating system.
- Align local-first persistence with the hybrid mobile architecture per-target matrix:
  - Web: keep PGlite `idb://`.
  - Desktop: move the data layer into Rust via `pglite-oxide`.
  - Mobile: use SQLite + `sqlite-vec` through `gen_ui_core` FFI.
  - Constraint: `pglite-oxide` has no iOS/Android support per the corrected architecture notes.
- Establish the Flutter mobile target where UAR runs entirely on-device using embedded SurrealDB / `surrealkv` persistence, per `TJ-ARCH-MOB-001`.
- Run `/impeccable audit` and `/impeccable critique` across the React frontend to produce a scored UI/UX defect inventory covering brittleness, stalls, freeze paths, and admin console UX.
- Execute prioritized React fixes using `/impeccable polish` / `/impeccable harden`.
- Absorb six supplemental Admin/Agents UI work items from `uar-grade-a-upgrade-2026-07`:
  - `sw-scheme-safe-caching`
  - `model-warning-clarity`
  - `provider-first-model-picker`
  - `edit-panel-verification`
  - `governance-reconciliation`
  - `freeze-diagnostics`
- Evaluate TypeScript 7.0 native compiler migration before committing:
  - Verify current release status.
  - Check ecosystem compatibility: Vite/Rolldown, ESLint, and `vue-tsc` equivalents.
  - Define migration path from TypeScript 5.9.3.
  - Follow dependency-verification rules 22/23.

## Current position

```text
Position: model-runtime-per-device › uar-embedded-policy › m5a-impl-delegated
Status: implementation_in_flight
```

M5a is not complete; implementation has been delegated to a worktree subagent and is currently in flight. The appropriate action is to wait for the M5a subagent completion notification, then verify and integrate.

## Completed and preserved work

- M5a OpenSpec authored and committed at `26ccde2`.
- OpenSpec validation status: `4/4 valid`.
- KnowMe work is intact:
  - 8 commits preserved.
  - Current HEAD: `c005342`.
- Consumers queued after M5a:
  1. `M5a-knowme`
  2. `M5b`
  3. `M7-react`

## Stop-hook handling decision

A stale Stop-hook automation nudge suggested:

```text
/opsx:new desktop-stable-port
```

That command targets a different, already-completed phase: `desktop-stable-port`. It was intentionally not followed because starting an unrelated OpenSpec change would fork the current work and risk conflicting with the in-flight submodule edits.

Decision: hold until the M5a implementation subagent completes. Do not touch the submodule or chat/settings surfaces that queued consumers are expected to modify.

## Required next verification after M5a completes

After the M5a subagent reports completion:

1. Verify with server-full checks.
2. Run default `cargo check`.
3. Run the parity test.
4. Verify or update ADR `0013`.
5. Run OpenSpec strict validation:

```bash
openspec --strict
```

6. Bump the KnowMe submodule pointer.
7. Continue queued consumers:
   - `M5a-knowme`
   - `M5b`
   - `M7-react`

## Mobile runtime architecture linkage

This phase continues the embedded mobile runtime direction described in [Embedded UAR Provider Routing and Mobile Parity](/embedded-uar-provider-routing-and-mobile-parity.md): iOS/Android should not require a UAR service URL and should instead run the embedded in-process UAR kernel with provider/model/key configuration.

The iOS certification track is still blocked on operator interaction: device certificate / build install flow requires operator taps before Local and Orchestrator lanes can be certified, consistent with [iOS device cert build + 5 knowledge records](/karpathy-progress-20260723T013607Z-model-runtime-per-device.md).

## External blocker

- iOS think-split device certificate flow is still awaiting operator taps.

# Citations

1. stdin
2. manual:universal-agent-runtime/uar-hybrid-app-architecture