---
type: Reference
id: hybrid-mobile-poc-ios-reboot-verification-next-steps
title: Hybrid Mobile PoC iOS Reboot Verification Next Steps
tags:
- hybrid-mobile
- proof-of-concept
- ios-verification
- flutter
- codegen
- ci-verification
- pasteboard-daemon
links:
- hybrid-mobile-poc-ios-skia-verification-status
- t8-resume-status-for-hybrid-mobile-poc-codegen-verification
- hybrid-mobile-poc-phase-codegen-and-ci-execution-context
- frb-rust-input-fix-for-transparent-poc-bridge-types
- hybrid-mobile-poc-phase-goals-and-verification-scope
sources:
- stdin
- manual:Hybrid Mobile Architecture/phase-codegen-and-ci-verification
timestamp: 2026-07-17T01:48:55.876582+00:00
created_at: 2026-07-17T01:48:55.876582+00:00
updated_at: 2026-07-17T01:48:55.876582+00:00
revision: 0
---

## Context

- **Phase:** `phase-codegen-and-ci-verification`
- **Project:** Hybrid Mobile Architecture
- **KBD root:** `$REPO_ROOT`
- **Captured:** `2026-07-17T01:47:15Z`
- **Recorded status:** `executing`
- **Source context:** `manual:Hybrid Mobile Architecture/phase-codegen-and-ci-verification`

This update continues the Hybrid Mobile Architecture proof-of-concept/codegen verification work tracked in [Hybrid Mobile PoC iOS Skia Verification Status](/hybrid-mobile-poc-ios-skia-verification-status.md), [T8 Resume Status for Hybrid Mobile PoC Codegen Verification](/t8-resume-status-for-hybrid-mobile-poc-codegen-verification.md), [Hybrid Mobile PoC Phase Codegen and CI Execution Context](/hybrid-mobile-poc-phase-codegen-and-ci-execution-context.md), [FRB rust_input fix for transparent PoC bridge types](/frb-rust-input-fix-for-transparent-poc-bridge-types.md), and [Hybrid Mobile PoC phase goals and verification scope](/hybrid-mobile-poc-phase-goals-and-verification-scope.md).

## Revised phase objective

As of `2026-07-15`, the phase deliverable is a working proof-of-concept app, not just codegen/CI verification. The codegen and CI tasks remain supporting objectives proven by the PoC.

The PoC must be built under:

```text
apps/<name>/
```

It must use the repository scaffolds and skills, guided by KnowMe reference documentation in:

```text
docs/reference-app/
```

The app should demonstrate the broadest practical range of supported capabilities:

- Streaming `ContentBlock` chat
- PEM entity management
- SurrealDB graph-RAG memory
- Local-first sync
- Cross-platform Flutter/Tauri/web from one Rust core
- Feature subset informed by showcase-app best practices and 2026 on-device AI feasibility research

## Supporting verification goals

The PoC must prove the original phase scope in passing:

- Run the real codegen pipeline:
  - `flutter_rust_bridge_codegen generate`
  - `dart run build_runner build`
  - Full `flutter pub get`
  - Full `pnpm install`
- Confirm pre-codegen warnings clear once generated code and sibling packages exist.
- Resolve or work around the PEM install blocker:
  - `@prometheus-ags/entity-graph-core@workspace:*` is unresolvable outside the PEM monorepo.
- Verify the PoC builds and runs on at least one real target per surface:
  - macOS Tauri desktop
  - iOS simulator or Android emulator for Flutter
- Wire CI to run on every push:
  - `cargo clippy --workspace`
  - `audit.sh all`
  - Boundary test suites against the PoC

## Current iOS verification state

The simulator is being rebooted with a fresh pasteboard daemon. After reboot, the app should relaunch and screenshots should be captured after the UI settles.

Expected close condition for T5 iOS verification:

- Post-reboot screenshot shows the chat screen.
- Bottom Cupertino tab bar is visible.

If the above is true, T5 iOS verification can be closed.

## Immediate next actions

After confirming the post-reboot screenshot:

1. Remove temporary instrumentation/probes:
   - `[shell]`
   - `[chat]`
2. Revert the ineffective `FLTEnableImpeller` plist key.
3. Record the pasteboard-daemon finding in:
   - memory
   - T5 notes
   - execution log
4. Commit the verified changes.
5. Take stock of remaining phase items.

## Citations

1. stdin
2. manual:Hybrid Mobile Architecture/phase-codegen-and-ci-verification
