<!-- source=agent-a6bf13877ab890979; branch=worktree-agent-a6bf13877ab890979; original_sha256=7ce207c951bc4466b5016dfe6a907419d5234cf9e885e56973fb6f86bde29c8c -->
---
type: Reference
id: knowme-poc-codegen-and-tauri-verification-c-102
title: KnowMe PoC Codegen and Tauri Verification C-102
tags:
- hybrid-mobile
- knowme-poc
- codegen
- tauri
- flutter
- rust-workspace
- ci-verification
sources:
- stdin
- manual:Hybrid Mobile Architecture/phase-codegen-and-ci-verification
timestamp: 2026-07-16T00:45:50.089071+00:00
created_at: 2026-07-16T00:45:50.089071+00:00
updated_at: 2026-07-16T00:45:50.089071+00:00
revision: 0
---

## Phase Context

- **Project:** Hybrid Mobile Architecture
- **Phase:** `phase-codegen-and-ci-verification`
- **KBD root:** `$REPO_ROOT`
- **Captured:** 2026-07-16T00:43:21Z
- **Status:** `execute_in_progress`
- **Latest completed checkpoint:** C-102 merged; loop stopped by user choice
- **Commit:** `86e7d1d` pushed to `main`

## Revised Phase Goal

As of 2026-07-15, the phase end result is a **working proof-of-concept application**, not merely codegen and CI verification.

The PoC must be built in `apps/<name>/` using repository scaffolds and skills, based on the KnowMe reference documentation under `docs/reference-app/`.

The PoC should prove the skill package end-to-end and showcase the broadest practical set of supported capabilities:

- Streaming `ContentBlock` chat
- PEM entity management
- SurrealDB graph-RAG memory
- Local-first sync
- Cross-platform Flutter, Tauri, and web clients from one Rust core
- Feature subset selected via web research on showcase-app practices and 2026 on-device AI feasibility

## Supporting Objectives

The original codegen/CI goals remain supporting objectives and should be proven through the PoC:

- Run the real codegen pipeline against the PoC:
  - `flutter_rust_bridge_codegen generate`
  - `dart run build_runner build`
  - full `flutter pub get`
  - full `pnpm install`
- Confirm pre-codegen warnings clear once generated code and sibling packages exist.
- Resolve or work around the PEM install blocker:
  - `@prometheus-ags/entity-graph-core@workspace:*` is unresolvable outside the PEM monorepo.
- Verify the PoC builds and runs on at least one real target per surface:
  - macOS Tauri desktop
  - iOS simulator or Android emulator for Flutter
- Wire CI to run on every push:
  - `cargo clippy --workspace`
  - `audit.sh all`
  - boundary test suites against the PoC

## C-102 Completion Summary

C-102 was the first full codegen run against the KnowMe PoC scaffold. It is complete and verified.

### Rust Workspace

- Workspace size: 13 crates
- Verification command: `cargo check --workspace`
- Result: clean
- Fixes applied:
  - Corrected `pub use` visibility bugs in `gen_ui_ffi` API modules.
  - Fixed a doc-comment ordering bug introduced by `flutter_rust_bridge` codegen in `lib.rs`.

### Flutter Mobile

- Verification command: `flutter analyze`
- Result: 0 errors, 0 warnings
- Fixes applied:
  - Replaced stale `Mutation<T>()` usage that is not available on Riverpod 3.3.2's public surface.
  - Removed leftover counter-app boilerplate.

### Tauri Desktop

- Verification commands/results:
  - `tsc --noEmit`: clean
  - `tauri dev`: launches and runs with a rendering window and zero console errors
- Fixes applied:
  - Added missing `build.rs`.
  - Added missing `tauri-build` dependency/configuration.
  - Fixed hardcoded `app_lib` crate-name mismatch.
  - Added missing app icons.
  - Added missing Tauri v2 capabilities file.
    - Without this file, frontend `event.listen()` was silently denied.
  - Built two external npm packages from source:
    - `@prometheus-ags/gen-ui-react`
    - `@flint/react`
  - Reason for source build: pnpm's git fetch for `@flint/react` only pulled `package.json` and `SKILL.md`.

### Web

- The same Vite bundle was verified rendering cleanly in a plain browser tab.

### Developer Tools Menu

Added a native menu item:

- **View → Toggle Developer Tools**

The menu item was wired into:

- The live app
- The scaffold script template, so future scaffolds inherit the fix

## Scaffold Propagation

All fixes were applied in both places:

- The scaffold scripts/templates for future generated apps
- The already scaffolded PoC copy at `apps/knowme-poc/`

## Remaining Work

The following checkpoints remain pending and were paused so the working app can be reviewed first:

- C-103: live chat end-to-end
- C-104: memory graph-RAG
- C-105: local model inference
- C-106: local-first sync
- C-107: Whisper
- C-108: MCP/agent
- C-109: settings
- C-110: CI

# Citations

1. stdin
2. manual:Hybrid Mobile Architecture/phase-codegen-and-ci-verification