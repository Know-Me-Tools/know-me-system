---
type: Reference
id: hybrid-mobile-poc-phase-goals-and-verification-scope
title: Hybrid Mobile PoC phase goals and verification scope
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
- knowme-poc-scribe-feature-and-ci-workflow-verification-status
- hybrid-mobile-poc-phase-goals-for-codegen-and-ci-verification
sources:
- stdin
- manual:Hybrid Mobile Architecture/phase-codegen-and-ci-verification
timestamp: 2026-07-16T11:17:18.652144+00:00
created_at: 2026-07-16T11:17:18.652144+00:00
updated_at: 2026-07-16T11:17:18.652144+00:00
revision: 0
---

## Context

- **Phase:** `phase-codegen-and-ci-verification`
- **Project:** Hybrid Mobile Architecture
- **KBD root:** `~/Projects/hybrid-mobile-architecture-src`
- **Captured:** `2026-07-16T11:01:35Z`
- **Recorded status:** `executing`
- **Phase source:** `manual:Hybrid Mobile Architecture/phase-codegen-and-ci-verification`

This entry consolidates the phase objective and supporting verification scope for the Hybrid Mobile Architecture KnowMe proof-of-concept. It continues the execution history tracked in [T8 Resume Status for Hybrid Mobile PoC Codegen Verification](/t8-resume-status-for-hybrid-mobile-poc-codegen-verification.md), [Hybrid Mobile PoC Phase Codegen and CI Execution Context](/hybrid-mobile-poc-phase-codegen-and-ci-execution-context.md), [KnowMe PoC embedded engine lifecycle fixes](/knowme-poc-embedded-engine-lifecycle-fixes.md), [KnowMe PoC Scribe feature and CI workflow verification status](/knowme-poc-scribe-feature-and-ci-workflow-verification-status.md), and [Hybrid Mobile PoC phase goals for codegen and CI verification](/hybrid-mobile-poc-phase-goals-for-codegen-and-ci-verification.md).

## Revised phase objective

As of `2026-07-15`, the phase deliverable was revised by user direction: the end result must be a **working proof-of-concept application**, not merely code generation or CI pipeline verification.

The original codegen and CI objectives remain in scope, but only as supporting objectives that the PoC must prove in passing.

## Primary deliverable

Build a proof-of-concept app under:

```text
apps/<name>/
```

The PoC must:

- Use the scaffolds and skills in the repository.
- Be guided by KnowMe reference documentation in:

```text
docs/reference-app/
```

- Use the KnowMe functional spec, moodboard, and user journeys as product inputs.
- Prove the skill package works end-to-end.
- Showcase the broadest practical range of supported capabilities.
- Select the final feature subset using:
  - Web research on showcase-app best practices.
  - 2026 feasibility constraints for on-device AI.

## Capabilities the PoC should demonstrate

The PoC should cover the broadest practical set of supported architecture capabilities:

- Streaming `ContentBlock` chat.
- PEM entity management.
- SurrealDB graph-RAG memory.
- Local-first sync.
- Cross-platform app surfaces from one Rust core:
  - Flutter mobile.
  - Tauri desktop.
  - Web.

## Supporting codegen objectives

The PoC must run the real code generation pipeline:

```sh
flutter_rust_bridge_codegen generate
dart run build_runner build
flutter pub get
pnpm install
```

Verification requirement:

- Confirm that pre-codegen warnings clear once generated code and sibling packages exist.

## PEM install blocker

The phase must resolve or work around the PEM package installation blocker:

```text
@prometheus-ags/entity-graph-core@workspace:*
```

Issue:

- The package is unresolvable outside the PEM monorepo.

Required outcome:

- The PoC dependency installation path must succeed despite this workspace-resolution constraint.

## Build and runtime verification targets

The PoC must build and run on at least one real target for each surface:

- **Desktop:** macOS Tauri desktop.
- **Mobile:** iOS simulator or Android emulator for Flutter.

## CI requirements

CI must run on every push and include:

```sh
cargo clippy --workspace
audit.sh all
```

Additional CI coverage:

- Boundary test suites against the PoC.

## Citations

1. stdin
2. manual:Hybrid Mobile Architecture/phase-codegen-and-ci-verification
