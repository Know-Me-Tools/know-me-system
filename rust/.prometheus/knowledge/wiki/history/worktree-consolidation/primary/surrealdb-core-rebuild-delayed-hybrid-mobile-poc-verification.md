<!-- source=primary; branch=main-pre-consolidation; original_sha256=e2a18ba0a72b679ef01c5a94bd5f4c9c2143f84e991b51593be97ae484839236 -->
---
type: Reference
id: surrealdb-core-rebuild-delayed-hybrid-mobile-poc-verification
title: SurrealDB core rebuild delayed Hybrid Mobile PoC verification
tags:
- hybrid-mobile
- knowme-poc
- surrealdb
- ci-verification
- codegen
- flutter-rust-bridge
- tauri
links:
- t8-resume-status-for-hybrid-mobile-poc-codegen-verification
- frb-rust-input-fix-for-transparent-poc-bridge-types
- hybrid-mobile-poc-phase-codegen-and-ci-execution-context
- hybrid-mobile-poc-android-build-reached-gradle-assembledebug
- hybrid-mobile-poc-android-build-monitor-status-at-assembledebug
sources:
- stdin
- manual:Hybrid Mobile Architecture/phase-codegen-and-ci-verification
timestamp: 2026-07-16T14:16:54.817564+00:00
created_at: 2026-07-16T14:16:54.817564+00:00
updated_at: 2026-07-16T14:16:54.817564+00:00
revision: 0
---

## Context

- **Phase:** `phase-codegen-and-ci-verification`
- **Project:** Hybrid Mobile Architecture
- **KBD root:** `~/Projects/hybrid-mobile-architecture-src`
- **Captured:** `2026-07-16T14:02:58Z`
- **Recorded status:** `executing`
- **Source context:** `manual:Hybrid Mobile Architecture/phase-codegen-and-ci-verification`

This status update continues the Hybrid Mobile Architecture KnowMe PoC verification tracked in [T8 Resume Status for Hybrid Mobile PoC Codegen Verification](/t8-resume-status-for-hybrid-mobile-poc-codegen-verification.md), [FRB rust_input fix for transparent PoC bridge types](/frb-rust-input-fix-for-transparent-poc-bridge-types.md), [Hybrid Mobile PoC Phase Codegen and CI Execution Context](/hybrid-mobile-poc-phase-codegen-and-ci-execution-context.md), [Hybrid Mobile PoC Android build reached Gradle assembleDebug](/hybrid-mobile-poc-android-build-reached-gradle-assembledebug.md), and [Hybrid Mobile PoC Android build monitor status at assembleDebug](/hybrid-mobile-poc-android-build-monitor-status-at-assembledebug.md).

## Current status

- Position: `phase-codegen-and-ci-verification`
- Status: `executing`
- A previously suspected hung test was reclassified as a legitimate full recompilation of `surrealdb-core`.
- The test/build run was restarted with a sufficient time budget and without an artificial short timeout.

## Diagnosis correction

The earlier diagnosis of a hung test was based on:

- A stack sample.
- A timeout that was too short for a clean `surrealdb-core` rebuild.

The observed behavior is now understood as expected slow compilation:

- `surrealdb-core` is known to build slowly.
- Its own documentation notes that `surrealdb-core`'s `build.rs` re-runs on any downstream change.
- Recent schema edits triggered that downstream-change path, causing `surrealdb-core` to recompile from scratch.

## Phase objective

As revised on `2026-07-15`, this phase must deliver a working proof-of-concept application, not only codegen and CI verification.

The PoC must be built under:

```text
apps/<name>/
```

It must use repository scaffolds and skills and be based on KnowMe reference material in:

```text
docs/reference-app/
```

The PoC should demonstrate the broadest practical range of supported capabilities:

- Streaming `ContentBlock` chat.
- PEM entity management.
- SurrealDB graph-RAG memory.
- Local-first sync.
- Cross-platform Flutter, Tauri, and web surfaces from one Rust core.

## Supporting verification goals

The PoC must prove the original codegen/CI objectives in passing:

- Run the real codegen pipeline:
  - `flutter_rust_bridge_codegen generate`
  - `dart run build_runner build`
  - `flutter pub get`
  - `pnpm install`
- Confirm pre-codegen warnings clear once generated code and sibling packages exist.
- Resolve or work around the PEM install blocker:

```text
@prometheus-ags/entity-graph-core@workspace:*
```

- Verify at least one real target per surface:
  - macOS Tauri desktop.
  - iOS simulator or Android emulator for Flutter.
- Wire CI to run on every push:
  - `cargo clippy --workspace`
  - `audit.sh all`
  - boundary test suites against the PoC.

## Work awaiting final build/test confirmation

After the current long-running build/test completes, the next step is to commit and push the verified work, including:

- FRB `Result` literal fix.
- Mobile boot-order wiring.
- Shared memory module.
- Desktop `GraphStore`.
- Two SurrealDB schema bug fixes.

# Citations

1. stdin
2. manual:Hybrid Mobile Architecture/phase-codegen-and-ci-verification