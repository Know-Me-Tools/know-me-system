<!-- source=primary; branch=main-pre-consolidation; original_sha256=73add1f6a0e0c5f7960b48d87b22a8951fb43c843a7d9ca737eaaf93413bc7de -->
---
type: Reference
id: knowme-tauri-dev-build-clean-after-branding-and-startup-fixes
title: KnowMe Tauri Dev Build Clean After Branding and Startup Fixes
tags:
- hybrid-mobile
- knowme-poc
- tauri
- branding
- startup-commands
- ci-verification
- phase-status
links:
- knowme-poc-codegen-and-tauri-verification-c-102
- phase-codegen-and-ci-verification-session-status
- knowme-poc-branding-tauri-menu-and-startup-command-fixes
sources:
- stdin
timestamp: 2026-07-16T01:20:03.479698+00:00
created_at: 2026-07-16T01:20:03.479698+00:00
updated_at: 2026-07-16T01:20:03.479698+00:00
revision: 0
---

## Context

- **Project:** Hybrid Mobile Architecture
- **Phase:** `phase-codegen-and-ci-verification`
- **KBD root:** `~/Projects/hybrid-mobile-architecture-src`
- **Captured:** `2026-07-16T01:15:20Z`
- **Source phase record:** `manual:Hybrid Mobile Architecture/phase-codegen-and-ci-verification`
- **Status:** `execute_in_progress`

This session continues the KnowMe PoC work for the revised `phase-codegen-and-ci-verification` objective. The phase now targets a working proof-of-concept app, not only codegen and CI verification, as tracked in [KnowMe PoC Codegen and Tauri Verification C-102](/knowme-poc-codegen-and-tauri-verification-c-102.md) and [Phase Codegen and CI Verification Session Status](/phase-codegen-and-ci-verification-session-status.md).

## Revised Phase Goal

As of `2026-07-15`, the phase deliverable is a working PoC app under:

```text
apps/<name>/
```

The PoC must use repository scaffolds and skills, be based on KnowMe reference documentation in:

```text
docs/reference-app/
```

and demonstrate the broadest practical capability set:

- Streaming `ContentBlock` chat
- PEM entity management
- SurrealDB graph-RAG memory
- Local-first sync
- Cross-platform Flutter, Tauri, and web clients from one Rust core
- Feature subset selected via web research on showcase-app practices and 2026 on-device AI feasibility

## Supporting Objectives

Original codegen and CI goals remain supporting objectives proven through the PoC:

- Run the real codegen pipeline:
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

## Current Verification Result

The fresh Tauri development build is confirmed clean after the four latest fixes from [KnowMe PoC Branding, Tauri Menu, and Startup Command Fixes](/knowme-poc-branding-tauri-menu-and-startup-command-fixes.md):

- No errors appear anywhere in the log.
- Rust compiled cleanly.
- Vite is serving successfully on:

```text
http://localhost:1420
```

- Native Tauri binary is still running:

```text
PID 18543
```

## Fixes Included in the Fresh Launch

The clean launch includes these changes:

1. Real KnowMe `K`-monogram icon.
2. `Exit` menu item.
3. Draggable branded titlebar with OS chrome removed.
4. Migration stub commands so startup no longer fails.

## Current Position

```text
phase: phase-codegen-and-ci-verification
status: execute_in_progress
state: fresh dev build confirmed clean
```

## Next Action

The native `KnowMe` window should now be visible. Manual inspection is needed for:

- app icon correctness
- branded titlebar drag behavior
- `Exit` menu behavior
- devtools placement

After inspection, choose whether to commit these fixes or continue with the next planned changes beginning at `C-103+`.

# Citations

1. [1] stdin