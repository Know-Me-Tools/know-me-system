<!-- source=primary; branch=main-pre-consolidation; original_sha256=05467f8745b2e94dfbbdc060fd7ccde9f19c5e2cc201cb569d3deef07d363bfb -->
---
type: Reference
id: hybrid-mobile-architecture-poc-phase-goals-and-current-status
title: Hybrid Mobile Architecture PoC Phase Goals and Current Status
tags:
- hybrid-mobile
- proof-of-concept
- codegen
- ci-verification
- tauri
- flutter
- surrealdb
- pem
sources:
- stdin
- manual:Hybrid Mobile Architecture/phase-codegen-and-ci-verification
timestamp: 2026-07-16T01:12:52.208039+00:00
created_at: 2026-07-16T01:12:52.208039+00:00
updated_at: 2026-07-16T01:12:52.208039+00:00
revision: 0
---

## Phase Context

- **Phase:** `phase-codegen-and-ci-verification`
- **Project:** Hybrid Mobile Architecture
- **Repository root:** `~/Projects/hybrid-mobile-architecture-src`
- **Captured:** `2026-07-16T00:45:26Z`
- **Status:** `execute_in_progress`
- **Current state:** Tauri development app is running and visible in a native window titled `knowme-poc`.

## Revised Primary Goal

As of `2026-07-15`, the phase objective was revised from pipeline verification alone to delivering a working proof-of-concept application.

The PoC must be built under:

```text
apps/<name>/
```

It should use the repository scaffolds and skill packages, guided by the KnowMe reference documentation in:

```text
docs/reference-app/
```

Reference materials include the functional specification, moodboard, and user journeys.

## Proof-of-Concept Requirements

The PoC must demonstrate the skill package end-to-end and showcase the broadest practical range of supported capabilities, including:

- Streaming `ContentBlock` chat
- PEM entity management
- SurrealDB graph-RAG memory
- Local-first sync
- Cross-platform delivery from one Rust core:
  - Flutter mobile
  - Tauri desktop
  - Web

The final feature subset should be selected using web research into showcase-app best practices and 2026-era on-device AI feasibility.

## Supporting Objectives

The original codegen and CI verification goals remain as supporting objectives, to be proven through the PoC.

### Codegen Pipeline

Run the real codegen pipeline on the PoC:

```sh
flutter_rust_bridge_codegen generate
dart run build_runner build
flutter pub get
pnpm install
```

Expected outcome:

- Pre-codegen warnings should clear once generated code and sibling packages exist.

### PEM Install Blocker

Resolve or work around the PEM package resolution blocker:

```text
@prometheus-ags/entity-graph-core@workspace:*
```

Problem:

- The dependency is unresolvable outside the PEM monorepo.

### Runtime Verification

Verify that the PoC builds and runs on at least one real target per surface:

- macOS Tauri desktop
- iOS simulator or Android emulator for Flutter

### CI Wiring

Configure CI to run the following on every push:

```sh
cargo clippy --workspace
audit.sh all
```

Also run the boundary test suites against the PoC.

## Current Session State

The app is live in a native Tauri window titled:

```text
knowme-poc
```

The visible UI is currently the placeholder root route. Chat and memory feature wiring are planned for `C-103+`.

Developer tools can be opened from:

```text
View → Toggle Developer Tools
```

or with:

```text
Cmd+Alt+I
```

## Next Step

Resume the implementation loop into:

- `C-103`: live chat
- Remaining planned feature changes

# Citations

1. stdin
2. manual:Hybrid Mobile Architecture/phase-codegen-and-ci-verification