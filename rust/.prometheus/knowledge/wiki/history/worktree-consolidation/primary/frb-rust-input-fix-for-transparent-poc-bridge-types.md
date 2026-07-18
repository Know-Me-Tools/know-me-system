<!-- source=primary; branch=main-pre-consolidation; original_sha256=6faabcd3719569e6df838a79827870a2fb1c782027868282c30f5d0281ccd2b7 -->
---
type: Reference
id: frb-rust-input-fix-for-transparent-poc-bridge-types
title: FRB rust_input fix for transparent PoC bridge types
tags:
- hybrid-mobile
- flutter-rust-bridge
- codegen
- proof-of-concept
- flutter
- tauri
- pem
- ci-verification
links:
- t8-resume-status-for-hybrid-mobile-poc-codegen-verification
- hybrid-mobile-poc-phase-codegen-and-ci-execution-context
- hybrid-mobile-poc-phase-goals-for-codegen-and-ci-verification
- hybrid-mobile-poc-phase-goals-and-verification-scope
- android-rust-cross-compile-progress-for-hybrid-mobile-poc
sources:
- stdin
- manual:Hybrid Mobile Architecture/phase-codegen-and-ci-verification
timestamp: 2026-07-16T12:40:51.362104+00:00
created_at: 2026-07-16T12:40:51.362104+00:00
updated_at: 2026-07-16T12:40:51.362104+00:00
revision: 0
---

## Context

- **Phase:** `phase-codegen-and-ci-verification`
- **Project:** Hybrid Mobile Architecture
- **KBD root:** `~/Projects/hybrid-mobile-architecture-src`
- **Captured:** `2026-07-16T12:39:13Z`
- **Recorded status:** `executing`
- **Source context:** `manual:Hybrid Mobile Architecture/phase-codegen-and-ci-verification`

This update continues the Hybrid Mobile Architecture KnowMe PoC verification tracked in [T8 Resume Status for Hybrid Mobile PoC Codegen Verification](/t8-resume-status-for-hybrid-mobile-poc-codegen-verification.md), [Hybrid Mobile PoC Phase Codegen and CI Execution Context](/hybrid-mobile-poc-phase-codegen-and-ci-execution-context.md), [Hybrid Mobile PoC phase goals for codegen and CI verification](/hybrid-mobile-poc-phase-goals-for-codegen-and-ci-verification.md), [Hybrid Mobile PoC phase goals and verification scope](/hybrid-mobile-poc-phase-goals-and-verification-scope.md), and [Android Rust cross-compile progress for Hybrid Mobile PoC](/android-rust-cross-compile-progress-for-hybrid-mobile-poc.md).

## Revised phase objective

As of `2026-07-15`, the phase deliverable is a working proof-of-concept application, not just pipeline verification. Code generation and CI remain supporting objectives that the PoC must prove in passing.

The PoC must be built under:

```text
apps/<name>/
```

It must use the repository scaffolds and skills, guided by KnowMe reference documentation in:

```text
docs/reference-app/
```

Required proof points include:

- Streaming `ContentBlock` chat.
- PEM entity management.
- SurrealDB graph-RAG memory.
- Local-first sync.
- Cross-platform Flutter, Tauri, and web support from one Rust core.
- A feature subset selected via web research into showcase-app best practices and 2026 on-device AI feasibility.

## Supporting verification goals

The PoC must also prove the original codegen/CI phase objectives:

- Run the real codegen pipeline on the PoC:
  - `flutter_rust_bridge_codegen generate`
  - `dart run build_runner build`
  - Full `flutter pub get`
  - Full `pnpm install`
- Confirm pre-codegen warnings clear once generated code and sibling packages exist.
- Resolve or work around the PEM install blocker:

```text
@prometheus-ags/entity-graph-core@workspace:*
```

- Verify the PoC builds and runs on at least one real target per surface:
  - macOS Tauri desktop.
  - iOS simulator or Android emulator for Flutter.
- Wire CI to run on every push:
  - `cargo clippy --workspace`
  - `audit.sh all`
  - Boundary test suites against the PoC.

## FRB transparent type root cause

The root cause of the `MemoryHit` opaque-type problem was identified in Flutter Rust Bridge code generation:

- `flutter_rust_bridge` only `cargo expand`s the self crate, `gen_ui_ffi`, by default.
- Any type re-exported from another crate falls back to an opaque Dart wrapper unless its defining crate/module is explicitly included in `flutter_rust_bridge.yaml`.
- This also applies to types previously assumed to be transparent, such as `EntityRecord`.
- The resulting Dart wrappers have no usable field access.

## Fix applied

The fix was to add the defining crates/modules to the comma-separated `rust_input` list in:

```text
flutter_rust_bridge.yaml
```

This should cause FRB to generate transparent Dart classes with real fields for re-exported cross-crate types.

## Verification in progress

Codegen was re-running at capture time to verify transparent Dart generation for:

- `MemoryHit`
- `RelatedEntity`
- `EntityRecord`
- `ListResult`
- `ViewDescriptor`

Expected result: each type generates as a transparent Dart class with field access, not as an opaque wrapper.

## Next actions

- Await codegen completion.
- Inspect generated Dart bindings for transparent classes and fields.
- Finish mobile bridge wiring:

```text
rust_bridge_provider.dart
```

- Finish the desktop memory backend.
- Re-run full build verification across the PoC targets.

# Citations

1. stdin
2. manual:Hybrid Mobile Architecture/phase-codegen-and-ci-verification