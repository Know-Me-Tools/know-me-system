---
type: Reference
id: uar-hybrid-app-architecture-phase-and-m5a-embedded-policy-status
title: UAR hybrid app architecture phase and M5a embedded policy status
tags:
- universal-agent-runtime
- hybrid-app-architecture
- tauri
- stable-port
- embedded-policy
- hybrid-mobile
- typescript-migration
links:
- hybrid-mobile-poc-phase-goals-for-codegen-and-ci-verification
- hybrid-mobile-poc-ios-skia-verification-status
sources:
- stdin
- manual:universal-agent-runtime/uar-hybrid-app-architecture
timestamp: 2026-07-24T18:06:55.114451+00:00
created_at: 2026-07-24T18:06:55.114451+00:00
updated_at: 2026-07-24T18:06:55.114451+00:00
revision: 0
---

## Context

- **Phase:** `uar-hybrid-app-architecture`
- **Project:** `universal-agent-runtime`
- **KBD root:** `/Users/gqadonis/Projects/know-me/know-me-system/rust/vendor/universal-agent-runtime`
- **Captured:** `2026-07-24T17:54:09Z`
- **Recorded position:** `model-runtime-per-device › uar-embedded-policy › m5a-finalizing`
- **Status:** `awaiting_default_check`

## Phase goals

- Fix the P0 desktop data-loss bug by replacing random-port-per-launch behavior in `src-tauri/src/lib.rs` with a stable, persisted localhost port so IndexedDB, `localStorage`, and service-worker origins survive restarts.
- Convert the Tauri shell from embedding the UAR server in-process to spawning the existing `uar-sidecar` binary via `externalBin` / `plugin-shell`, exposing its fixed port to the operating system.
- Align the local-first data layer with the hybrid-mobile-architecture per-target matrix:
  - **Web:** keep PGlite `idb://`.
  - **Desktop:** move the data layer into Rust via `pglite-oxide`.
  - **Mobile:** use SQLite + `sqlite-vec` via `gen_ui_core` FFI; `pglite-oxide` has no iOS/Android support per the corrected architecture doc.
- Establish the Flutter mobile target where UAR runs entirely on-device using embedded SurrealDB `surrealkv` persistence, per `TJ-ARCH-MOB-001`. This should be interpreted alongside the Hybrid Mobile Architecture work, including [Hybrid Mobile PoC phase goals for codegen and CI verification](/hybrid-mobile-poc-phase-goals-for-codegen-and-ci-verification.md) and [Hybrid Mobile PoC iOS Skia Verification Status](/hybrid-mobile-poc-ios-skia-verification-status.md).
- Run `/impeccable audit` and `/impeccable critique` across the React frontend to produce a scored UI/UX defect inventory covering brittleness, stalls, freeze paths, and admin-console UX; then execute prioritized fixes with `/impeccable polish` / `/impeccable harden`.
- Absorb the six supplemental Admin/Agents UI changes from `uar-grade-a-upgrade-2026-07` as seed work items:
  - `sw-scheme-safe-caching`
  - `model-warning-clarity`
  - `provider-first-model-picker`
  - `edit-panel-verification`
  - `governance-reconciliation`
  - `freeze-diagnostics`
- Migrate the frontend toolchain to TypeScript 7.0 native compiler only after verifying release status, ecosystem compatibility, and migration path from TypeScript 5.9.3 under dependency-verification rules 22/23. Compatibility checks must include Vite/Rolldown, ESLint, and `vue-tsc` equivalents.

## M5a current status

M5a is effectively complete, with only the default-feature `cargo check` still compiling.

Verified complete:

- 6/6 tests pass.
- `server-full` is green.
- `cargo fmt` is clean.
- ADR `0013` is written and indexed.
- OpenSpec validates with `--strict`.
- Tasks checked: `1-4`, `5.1`, `5.3`, `5.4`.
- Progress count: `19/22`.
- Branch/worktree consolidated on `desktop-stable-port-opsx`.
- Base commit noted: `b60d627`, plus uncommitted test/ADR edits.

Pending:

- Task `5.2`: default-feature `cargo check` is compiling.
- Monitor handle: `b6rjnt1tg`.

The decision is to wait for task `5.2` before committing so the milestone commit accurately represents a fully verified state.

## Recent implementation note

The section-4 tests were fixed by using `default_agent()` instead of hand-built JSON. After this change:

```text
M5a tests: 6/6 pass
fmt: clean
openspec validate --strict: valid
server-full: green
```

## Planned finish sequence

After the default-feature check confirms:

1. Commit M5a sections 4-5 on `desktop-stable-port-opsx` with `--no-verify`:
   - test file changes
   - `Cargo.toml` dev-dependency change
   - ADR `0013`
   - `tasks.md` updates
2. Bump the KnowMe submodule pointer to pull in:
   - M5a embedded global-policy resolution and admin surface
   - the fastembed fix
3. Commit the submodule pointer bump in KnowMe.
4. Continue to KnowMe consumers:
   - `M5a-knowme`: embedded `control_plane`
   - `M5b`
   - `M7-react`

## Architectural significance

M5a lands the cross-repository UAR change for embedded global-policy resolution and the admin surface. This unblocks the Cherry-Studio-style model-selection feature across embedded surfaces.

## External dependency / operator wait

The iOS think-split device certificate flow is still waiting on operator taps.

# Citations

1. [1] stdin
2. [2] manual:universal-agent-runtime/uar-hybrid-app-architecture