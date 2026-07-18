---
type: Reference
id: hybrid-mobile-poc-phase-goals-for-codegen-and-ci-verification
title: Hybrid Mobile PoC phase goals for codegen and CI verification
tags:
- hybrid-mobile
- proof-of-concept
- codegen
- ci-verification
- flutter
- tauri
- pem
- surrealdb
links:
- t8-resume-status-for-hybrid-mobile-poc-codegen-verification
- hybrid-mobile-poc-phase-codegen-and-ci-execution-context
- knowme-poc-embedded-engine-lifecycle-fixes
sources:
- stdin
- manual:Hybrid Mobile Architecture/phase-codegen-and-ci-verification
timestamp: 2026-07-16T10:04:40.726164+00:00
created_at: 2026-07-16T10:04:40.726164+00:00
updated_at: 2026-07-16T10:04:40.726164+00:00
revision: 0
---

## Context

- **Phase:** `phase-codegen-and-ci-verification`
- **Project:** Hybrid Mobile Architecture
- **KBD root:** `~/Projects/hybrid-mobile-architecture-src`
- **Captured:** `2026-07-16T09:58:08Z`
- **Recorded status:** `executing`
- **Source context:** `manual:Hybrid Mobile Architecture/phase-codegen-and-ci-verification`

This phase continues the Hybrid Mobile Architecture KnowMe proof-of-concept work tracked in [T8 Resume Status for Hybrid Mobile PoC Codegen Verification](/t8-resume-status-for-hybrid-mobile-poc-codegen-verification.md), [Hybrid Mobile PoC Phase Codegen and CI Execution Context](/hybrid-mobile-poc-phase-codegen-and-ci-execution-context.md), and [KnowMe PoC embedded engine lifecycle fixes](/knowme-poc-embedded-engine-lifecycle-fixes.md).

## Revised phase objective

As of `2026-07-15`, the phase objective was revised by user direction: the deliverable is a working proof-of-concept application, not only codegen and CI verification. Code generation and CI checks remain supporting objectives that the PoC must prove in passing.

The PoC must be built under:

```text
apps/<name>/
```

It must use this repository's scaffolds and skills, guided by KnowMe reference documentation in:

```text
docs/reference-app/
```

Reference material includes the KnowMe functional specification, moodboard, and user journeys.

## Primary proof-of-concept requirements

The PoC must prove the skill package end-to-end and showcase the broadest practical range of supported capabilities, including:

- Streaming `ContentBlock` chat
- PEM entity management
- SurrealDB graph-RAG memory
- Local-first sync
- Cross-platform delivery from one Rust core:
  - Flutter mobile
  - Tauri desktop
  - Web

Feature subset selection should be informed by web research into showcase-app best practices and 2026 on-device AI feasibility.

## Supporting verification goals

The original codegen/CI goals remain in scope as supporting objectives:

- Run the real codegen pipeline on the PoC:
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

## Current status

The phase is marked `executing` in the Prometheus position marker:

```text
Position: phase-codegen-and-ci-verification | status: executing
```

# Citations

1. stdin
2. manual:Hybrid Mobile Architecture/phase-codegen-and-ci-verification
