---
type: Reference
id: hybrid-mobile-poc-ios-skia-verification-status
title: Hybrid Mobile PoC iOS Skia Verification Status
tags:
- hybrid-mobile
- proof-of-concept
- ios-verification
- flutter
- impeller
- skia
- codegen
- ci-verification
links:
- t8-resume-status-for-hybrid-mobile-poc-codegen-verification
- hybrid-mobile-poc-phase-codegen-and-ci-execution-context
- frb-rust-input-fix-for-transparent-poc-bridge-types
- hybrid-mobile-poc-phase-goals-and-verification-scope
- hybrid-mobile-poc-phase-goals-for-codegen-and-ci-verification
sources:
- stdin
- manual:Hybrid Mobile Architecture/phase-codegen-and-ci-verification
timestamp: 2026-07-17T01:37:10.862037+00:00
created_at: 2026-07-17T01:37:10.862037+00:00
updated_at: 2026-07-17T01:37:10.862037+00:00
revision: 0
---

## Context

- **Phase:** `phase-codegen-and-ci-verification`
- **Project:** Hybrid Mobile Architecture
- **KBD root:** `$REPO_ROOT`
- **Captured:** `2026-07-17T01:36:26Z`
- **Recorded status:** `executing`
- **Source context:** `manual:Hybrid Mobile Architecture/phase-codegen-and-ci-verification`

This update continues the Hybrid Mobile Architecture proof-of-concept/codegen verification effort tracked in [T8 Resume Status for Hybrid Mobile PoC Codegen Verification](/t8-resume-status-for-hybrid-mobile-poc-codegen-verification.md), [Hybrid Mobile PoC Phase Codegen and CI Execution Context](/hybrid-mobile-poc-phase-codegen-and-ci-execution-context.md), [FRB rust_input fix for transparent PoC bridge types](/frb-rust-input-fix-for-transparent-poc-bridge-types.md), [Hybrid Mobile PoC phase goals and verification scope](/hybrid-mobile-poc-phase-goals-and-verification-scope.md), and [Hybrid Mobile PoC phase goals for codegen and CI verification](/hybrid-mobile-poc-phase-goals-for-codegen-and-ci-verification.md).

## Revised phase objective

As of `2026-07-15`, the phase deliverable is a working proof-of-concept application, not merely pipeline verification. Code generation and CI remain supporting objectives that the PoC must prove in passing.

The PoC must be built under:

```text
apps/<name>/
```

It must use repository scaffolds and skills and be based on KnowMe reference documentation in:

```text
docs/reference-app/
```

Reference inputs include the functional spec, moodboard, and user journeys.

## Required capability coverage

The PoC should showcase the broadest practical range of supported Hybrid Mobile Architecture capabilities, with the exact feature subset selected from showcase-app best practices and 2026 on-device AI feasibility research:

- Streaming `ContentBlock` chat
- PEM entity management
- SurrealDB graph-RAG memory
- Local-first sync
- Cross-platform Flutter, Tauri, and web surfaces driven from one Rust core

## Supporting verification goals

The PoC must prove the original codegen/CI goals in passing:

- Run the real codegen pipeline on the PoC:
  - `flutter_rust_bridge_codegen generate`
  - `dart run build_runner build`
  - full `flutter pub get`
  - full `pnpm install`
- Confirm pre-codegen warnings clear once generated code and sibling packages exist.
- Resolve or work around the PEM install blocker:

```text
@prometheus-ags/entity-graph-core@workspace:*
```

- Verify the PoC builds and runs on at least one real target per surface:
  - macOS Tauri desktop
  - iOS simulator or Android emulator for Flutter
- Wire CI to run on every push:
  - `cargo clippy --workspace`
  - `audit.sh all`
  - boundary test suites against the PoC

## Current iOS verification status

- The Impeller-disabled run is building.
- A waiter should capture screenshots as soon as the chat frame paints under Skia.
- If the Skia screenshot shows the chat screen with the bottom tab bar:
  - T5 iOS verification can close.
  - The Impeller-on-simulator presentation bug should remain documented.
  - Temporary probes should be removed.
- If the Skia screenshot does not show the expected chat screen and bottom tab bar:
  - collect a VM-service stack dump as the next diagnostic.

## Citations

1. stdin
2. manual:Hybrid Mobile Architecture/phase-codegen-and-ci-verification
