---
type: Reference
id: knowme-poc-scribe-feature-and-ci-workflow-verification-status
title: KnowMe PoC Scribe feature and CI workflow verification status
tags:
- hybrid-mobile
- knowme-poc
- ci-verification
- codegen
- tauri
- flutter
- scribe
- ios-simulator
links:
- hybrid-mobile-poc-phase-goals-for-codegen-and-ci-verification
- hybrid-mobile-poc-phase-codegen-and-ci-execution-context
- knowme-poc-embedded-engine-lifecycle-fixes
sources:
- stdin
- manual:Hybrid Mobile Architecture/phase-codegen-and-ci-verification
timestamp: 2026-07-16T10:08:43.791128+00:00
created_at: 2026-07-16T10:08:43.791128+00:00
updated_at: 2026-07-16T10:08:43.791128+00:00
revision: 0
---

## Context

- **Phase:** `phase-codegen-and-ci-verification`
- **Project:** Hybrid Mobile Architecture
- **KBD root:** `~/Projects/hybrid-mobile-architecture-src`
- **Captured:** `2026-07-16T10:04:20Z`
- **Recorded status:** `executing`
- **Source context:** `manual:Hybrid Mobile Architecture/phase-codegen-and-ci-verification`

This update continues the KnowMe proof-of-concept phase tracked in [Hybrid Mobile PoC phase goals for codegen and CI verification](/hybrid-mobile-poc-phase-goals-for-codegen-and-ci-verification.md), [Hybrid Mobile PoC Phase Codegen and CI Execution Context](/hybrid-mobile-poc-phase-codegen-and-ci-execution-context.md), and [KnowMe PoC embedded engine lifecycle fixes](/knowme-poc-embedded-engine-lifecycle-fixes.md).

## Phase objective

As revised on `2026-07-15`, the phase deliverable is a working proof-of-concept application, not merely pipeline verification. Code generation and CI remain supporting objectives that the PoC must prove in passing.

The PoC must be implemented under:

```text
apps/<name>/
```

It must use repository scaffolds and skills, guided by KnowMe reference documentation in:

```text
docs/reference-app/
```

Required capability coverage includes:

- Streaming `ContentBlock` chat
- PEM entity management
- SurrealDB graph-RAG memory
- Local-first sync
- Cross-platform Flutter, Tauri, and web delivery from one Rust core
- Feature subset selected through web research around showcase-app best practices and 2026 on-device AI feasibility

## Supporting verification goals

The PoC is expected to prove the original codegen/CI goals:

- Run the real code generation pipeline:
  - `flutter_rust_bridge_codegen generate`
  - `dart run build_runner build`
  - full `flutter pub get`
  - full `pnpm install`
- Confirm pre-codegen warnings clear after generated code and sibling packages exist.
- Resolve or work around the PEM install blocker: `@prometheus-ags/entity-graph-core@workspace:*` is unresolvable outside the PEM monorepo.
- Verify the PoC builds and runs on at least one real target per surface:
  - macOS Tauri desktop
  - iOS simulator or Android emulator for Flutter
- Wire CI to run on every push:
  - `cargo clippy --workspace`
  - `audit.sh all`
  - boundary test suites against the PoC

## Current session progress

- Fixed the iOS build blocker by booting a valid simulator after a runtime-cleanup issue.
- Diagnosed the `cargokit` `gen_ui_db_graph` / `once_cell` error as a race against concurrent local edits, not a genuine upstream/tooling bug.
- Built and wired the full **C-107 Scribe feature**:
  - Fixed five `whisper-rs` / `cpal` API-version mismatches in recovered worktree code.
  - Moved the completed `gen_ui_audio` crate into the main checkout.
  - Added FFI surface in `gen_ui_ffi::api::scribe`.
  - Added Tauri commands in `tauri-plugin-gen-ui`.
- Completed the **C-110 CI workflow**:
  - Added `.github/workflows/knowme-poc-ci.yml`.
  - YAML validation passed.
  - Corrected scaffold/codegen invocation paths in the workflow.

## In-flight verification

Two background builds are running:

- `tauri-plugin-gen-ui` check
- `gen_ui_ffi` iOS-target build

## Next actions

1. Wait for the two background build results.
2. If both are clean, commit C-107 and C-110 work.
3. Relaunch the iOS simulator build to close T11.
4. Proceed to T12: decision-log documentation.
5. Mark C-103 complete once the remaining verification and documentation gates are closed.

# Citations

1. stdin
2. manual:Hybrid Mobile Architecture/phase-codegen-and-ci-verification
