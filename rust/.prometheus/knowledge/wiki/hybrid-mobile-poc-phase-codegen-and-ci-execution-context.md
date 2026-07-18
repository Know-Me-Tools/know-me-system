---
type: Reference
id: hybrid-mobile-poc-phase-codegen-and-ci-execution-context
title: Hybrid Mobile PoC Phase Codegen and CI Execution Context
tags:
- hybrid-mobile
- proof-of-concept
- codegen
- ci-verification
- tauri
- flutter
- surrealdb
- pem
links:
- hybrid-mobile-architecture-poc-phase-goals-and-current-status
- hybrid-mobile-poc-codegen-and-ci-verification-session
sources:
- stdin
- manual:Hybrid Mobile Architecture/phase-codegen-and-ci-verification
timestamp: 2026-07-16T08:35:06.455050+00:00
created_at: 2026-07-16T08:35:06.455050+00:00
updated_at: 2026-07-16T08:35:06.455050+00:00
revision: 0
---

## Phase Metadata

- **Phase:** `phase-codegen-and-ci-verification`
- **Project:** Hybrid Mobile Architecture
- **Repository root:** `~/Projects/hybrid-mobile-architecture-src`
- **Captured:** `2026-07-16T08:31:04Z`
- **Source:** `manual:Hybrid Mobile Architecture/phase-codegen-and-ci-verification`
- **Recorded status:** `executing`

This update belongs to the broader [Hybrid Mobile Architecture PoC Phase Goals and Current Status](/hybrid-mobile-architecture-poc-phase-goals-and-current-status.md) effort and follows related execution notes in [Hybrid Mobile PoC Codegen and CI Verification Session](/hybrid-mobile-poc-codegen-and-ci-verification-session.md).

## Revised Phase Objective

As of `2026-07-15`, the phase end result was revised from pipeline verification alone to a working proof-of-concept application. Code generation and CI verification remain supporting objectives that the PoC must prove in passing.

The PoC must be implemented under:

```text
apps/<name>/
```

It must use the repository scaffolds and skills, based on KnowMe reference documentation in:

```text
docs/reference-app/
```

Reference inputs include:

- Functional specification
- Moodboard
- User journeys

## Required PoC Capability Coverage

The app must prove the skill package end-to-end and showcase the broadest practical range of supported capabilities, including:

- Streaming `ContentBlock` chat
- PEM entity management
- SurrealDB graph-RAG memory
- Local-first sync
- Cross-platform delivery from one Rust core:
  - Flutter mobile
  - Tauri desktop
  - Web

The feature subset should be selected using web research on showcase-app best practices and 2026 on-device AI feasibility.

## Supporting Verification Goals

The original codegen/CI scope is retained as proof criteria for the PoC:

- Run the real codegen pipeline on the PoC:
  - `flutter_rust_bridge_codegen generate`
  - `dart run build_runner build`
  - full `flutter pub get`
  - full `pnpm install`
- Confirm pre-codegen warnings clear after generated code and sibling packages exist.
- Resolve or work around the PEM install blocker:

```text
@prometheus-ags/entity-graph-core@workspace:*
```

  This dependency is currently unresolvable outside the PEM monorepo.
- Verify the PoC builds and runs on at least one real target per surface:
  - macOS Tauri desktop
  - iOS simulator or Android emulator for Flutter
- Wire CI to run on every push:
  - `cargo clippy --workspace`
  - `audit.sh all`
  - boundary test suites against the PoC

## Current Execution State

- **Position marker:** `phase-codegen-and-ci-verification`
- **Status:** `executing`
- **Next step:** wait for two background Explore agents before drafting the data-architecture plan:
  - current-architecture map
  - existing repository guidance survey

## Citations

1. stdin
2. manual:Hybrid Mobile Architecture/phase-codegen-and-ci-verification
