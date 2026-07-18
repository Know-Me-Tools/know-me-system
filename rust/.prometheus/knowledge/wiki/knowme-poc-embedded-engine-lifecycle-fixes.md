---
type: Reference
id: knowme-poc-embedded-engine-lifecycle-fixes
title: KnowMe PoC embedded engine lifecycle fixes
tags:
- hybrid-mobile
- knowme-poc
- tauri
- pglite
- embedded-lifecycle
- rust
- ci-verification
links:
- hybrid-mobile-architecture-poc-phase-goals-and-current-status
- hybrid-mobile-poc-phase-codegen-and-ci-execution-context
sources:
- stdin
- manual:Hybrid Mobile Architecture/phase-codegen-and-ci-verification
timestamp: 2026-07-16T09:48:06.207452+00:00
created_at: 2026-07-16T09:48:06.207452+00:00
updated_at: 2026-07-16T09:48:06.207452+00:00
revision: 0
---

## Context

- **Phase:** `phase-codegen-and-ci-verification`
- **Project:** Hybrid Mobile Architecture
- **KBD root:** `~/Projects/hybrid-mobile-architecture-src`
- **Captured:** `2026-07-16T09:47:08Z`
- **Phase source:** `manual:Hybrid Mobile Architecture/phase-codegen-and-ci-verification`

This update continues the working KnowMe proof-of-concept effort described in [Hybrid Mobile Architecture PoC Phase Goals and Current Status](/hybrid-mobile-architecture-poc-phase-goals-and-current-status.md) and follows the execution context in [Hybrid Mobile PoC Phase Codegen and CI Execution Context](/hybrid-mobile-poc-phase-codegen-and-ci-execution-context.md).

## Phase objective

As of `2026-07-15`, the phase objective was revised: the end result must be a working proof-of-concept application, not just codegen and CI verification.

The PoC must be built under:

```text
apps/<name>/
```

It must use repository scaffolds and skills, guided by KnowMe reference documentation in:

```text
docs/reference-app/
```

The PoC is expected to prove the skill package end-to-end and demonstrate the broadest practical range of supported capabilities:

- Streaming `ContentBlock` chat
- PEM entity management
- SurrealDB graph-RAG memory
- Local-first sync
- Cross-platform delivery from one Rust core:
  - Flutter mobile
  - Tauri desktop
  - Web

Supporting objectives from the original phase scope remain required and are proven through the PoC:

- Run real codegen on the PoC:
  - `flutter_rust_bridge_codegen generate`
  - `dart run build_runner build`
  - full `flutter pub get`
  - full `pnpm install`
- Confirm pre-codegen warnings clear after generated code and sibling packages exist.
- Resolve or work around the PEM install blocker: `@prometheus-ags/entity-graph-core@workspace:*` is unresolvable outside the PEM monorepo.
- Verify the PoC builds and runs on at least one target per surface:
  - macOS Tauri desktop
  - iOS simulator or Android emulator for Flutter
- Wire CI to run on every push:
  - `cargo clippy --workspace`
  - `audit.sh all`
  - boundary test suites against the PoC

## Implemented changes

All 7 plan items were implemented and verified with clean Rust builds and linting across the full `apps/knowme-poc/rust` workspace and the desktop Tauri crate.

### Tauri single-instance startup

Updated:

- `apps/knowme-poc/desktop/src-tauri/Cargo.toml`
- `apps/knowme-poc/desktop/src-tauri/src/lib.rs`

Change:

- Added `tauri-plugin-single-instance`.
- Registered the plugin first.
- A second app launch now focuses the existing window instead of racing the first process for the config database lock.

### PGlite singleton and lock handling

Updated:

- `apps/knowme-poc/rust/crates/gen_ui_db/src/relational/postgres.rs`
- `apps/knowme-poc/rust/crates/tauri-plugin-gen-ui/src/commands.rs`

Changes:

- `PgliteStore` is now a process-wide `OnceCell` singleton.
- Store instances are cloneable and use an `Arc`-backed server handle.
- `run_migrations` retries once after a short delay when opening the store fails.
- If the retry also fails, the command surfaces a clear locked-by-another-process error.

Regression coverage:

- Added `opening_pglite_store_twice_in_one_process_reuses_the_singleton` to prove the double-open case succeeds in one process.

### Agent state double-init visibility

Updated:

- `apps/knowme-poc/rust/crates/gen_ui_agent/src/state.rs`

Change:

- A second `AgentState::init()` call now emits `tracing::warn!` instead of silently swallowing the duplicate initialization.

### Graph store singleton

Updated:

- `apps/knowme-poc/rust/crates/gen_ui_db_graph/src/store.rs`

Change:

- `GraphStore` received the same singleton treatment as `PgliteStore`.
- This prepares the graph store for mobile FFI wiring, where repeated initialization becomes load-bearing.

### Documentation updates

Updated:

- `docs/pglite-oxide-tauri-hybrid.md`
- `references/rust/patterns.md`
- `CLAUDE.md`
- `AGENTS.md`

Changes:

- Added an **Embedded engine lifecycle** section documenting the singleton pattern for embedded engines.
- Cross-referenced the lifecycle pattern from Rust patterns documentation.
- Fixed stale MSRV references from `1.95+` to `1.96+`.
- Rewrote the `CLAUDE.md` `pglite-oxide` section to match the corrected WASI/desktop-only reality.

## Verification

Completed successfully:

```sh
cargo build
cargo clippy -- -D warnings
```

Verification scope:

- Full `apps/knowme-poc/rust` workspace
- Desktop Tauri crate
- New regression test for same-process PGlite double-open behavior

## Remaining manual check

The desktop app was not launched with:

```sh
npm run tauri dev
```

Reason: the session could not drive a windowed environment. The Rust-level lifecycle fixes are verified by compilation, linting, and targeted regression testing, but a local end-to-end UI startup check is still recommended.

# Citations

1. stdin
2. manual:Hybrid Mobile Architecture/phase-codegen-and-ci-verification
