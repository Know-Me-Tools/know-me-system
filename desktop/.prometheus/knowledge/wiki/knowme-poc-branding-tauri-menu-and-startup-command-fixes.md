---
type: Reference
id: knowme-poc-branding-tauri-menu-and-startup-command-fixes
title: KnowMe PoC Branding, Tauri Menu, and Startup Command Fixes
tags:
- hybrid-mobile
- knowme-poc
- tauri
- branding
- startup-commands
- web-compatibility
- phase-status
links:
- phase-codegen-and-ci-verification-session-status
- knowme-poc-codegen-and-tauri-verification-c-102
sources:
- stdin
- manual:Hybrid Mobile Architecture/phase-codegen-and-ci-verification
timestamp: 2026-07-16T01:14:35.577225+00:00
created_at: 2026-07-16T01:14:35.577225+00:00
updated_at: 2026-07-16T01:14:35.577225+00:00
revision: 0
---

## Context

- **Project:** Hybrid Mobile Architecture
- **Phase:** `phase-codegen-and-ci-verification`
- **KBD root:** `~/Projects/hybrid-mobile-architecture-src`
- **Captured:** `2026-07-16T01:12:35Z`
- **Status:** `execute_in_progress`
- **Position:** `phase-codegen-and-ci-verification`

This session continues the phase described in [Phase Codegen and CI Verification Session Status](/phase-codegen-and-ci-verification-session-status.md) and follows the revised PoC objective tracked in [KnowMe PoC Codegen and Tauri Verification C-102](/knowme-poc-codegen-and-tauri-verification-c-102.md).

## Revised Phase Goal

As of `2026-07-15`, the phase end result is a **working proof-of-concept app**, not only code generation and CI verification.

The PoC must be built under:

```text
apps/<name>/
```

It must use repository scaffolds and skills, and be based on KnowMe reference documentation under:

```text
docs/reference-app/
```

The PoC should prove the skill package end to end and showcase the broadest practical supported capability set:

- Streaming `ContentBlock` chat
- PEM entity management
- SurrealDB graph-RAG memory
- Local-first sync
- Cross-platform Flutter, Tauri, and web clients from one Rust core
- Feature subset selected via web research on showcase-app best practices and 2026 on-device AI feasibility

## Supporting Objectives

The original codegen/CI scope remains as supporting objectives to be proven via the PoC:

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
  - macOS Tauri desktop
  - iOS simulator or Android emulator for Flutter
- Wire CI to run on every push:
  - `cargo clippy --workspace`
  - `audit.sh all`
  - Boundary test suites against the PoC

## Completed Session Work

Four requested app fixes were completed and verified on the web target.

### 1. Custom Branded Titlebar

- Disabled native window decorations with `decorations: false`.
- Replaced native chrome with the `Titlebar` component.
- Titlebar displays the KnowMe logo and wordmark.
- Window controls are platform-aware:
  - macOS traffic-light controls on the left.
  - Windows-style controls on the right.

### 2. File/View/Help Menu

Implemented a real application menu structure:

- **File**
  - Includes **Exit**.
- **View**
  - Includes **Toggle Developer Tools** as the last item.
- **Help**
  - Includes **About**.

### 3. KnowMe Branding

Applied KnowMe brand assets and tokens from `docs/reference-app/`:

- Exact color tokens, including:
  - `--ember`: `#FF6A3D`
  - Dark navy background: `#0B0F14`
- Fonts:
  - Space Grotesk
  - Inter
  - Roboto
  - JetBrains Mono
- Real K-monogram logo mark is now used as the app icon.
- Regenerated platform icon sets for:
  - macOS
  - Windows
  - iOS
  - Android

The K logo mark and `KnowMe` wordmark now render clearly and correctly in the branded titlebar, remain stable across reloads, and produce no console errors on the verified web target.

### 4. Startup Migration Command Fix

Registered the following Tauri commands as real stub commands so the frontend startup sequence no longer fails:

```text
run_migrations
load_seeds
attach_sync_shapes
memory_ingest
memory_search
entity_runtime_start
entity_runtime_stop
```

These stubs unblock startup while preserving the expected command boundary for later implementation.

## Bugs Found and Fixed During Web Testing

Testing the web target surfaced two native-context assumptions that crashed the app outside Tauri.

### Unconditional Tauri Window API Call

- Problem: `getCurrentWindow()` was called at module/init scope.
- Impact: Crashed the web app because no native Tauri context exists.
- Fix: Gate Tauri-specific calls behind `isTauri()`.

### Unconditional Tauri Event Listener

- Problem: `listen()` was called at module/init scope.
- Impact: Crashed the web app outside native Tauri.
- Fix: Gate Tauri event listener setup behind `isTauri()`.

## Verification Status

Verified on the web target:

- Branded titlebar renders correctly.
- KnowMe logo and wordmark render correctly.
- Menu/titlebar-related code produces no console errors.
- Startup no longer fails on missing migration/runtime commands.
- Tauri API calls are guarded outside native context.

Not yet verified in the currently running native Tauri window:

- The native Tauri window with PID `75130` is still showing the pre-fix bundle.
- Tauri's window process did not hot-reload the same way as the browser tab.
- Restarting the dev server or reloading the native window is required to see the fixes live in native Tauri.

## Next Action

Reload the native Tauri window or restart the dev server, then verify the same four fixes in the native desktop surface.

# Citations

1. stdin
2. manual:Hybrid Mobile Architecture/phase-codegen-and-ci-verification

## Consolidated source variants

### Variant from `agent-a6bf13877ab890979`

Original path: `apps/knowme-poc/desktop/.prometheus/knowledge/wiki/knowme-poc-branding-tauri-menu-and-startup-command-fixes.md`  
Original SHA-256: `1014ad9ed304830f4197462f92c14fd819d81ecf86408b50aece80d230446b3a`

## Context

- **Project:** Hybrid Mobile Architecture
- **Phase:** `phase-codegen-and-ci-verification`
- **KBD root:** `$REPO_ROOT`
- **Captured:** `2026-07-16T01:12:35Z`
- **Status:** `execute_in_progress`
- **Position:** `phase-codegen-and-ci-verification`

This session continues the phase described in [Phase Codegen and CI Verification Session Status](/phase-codegen-and-ci-verification-session-status.md) and follows the revised PoC objective tracked in [KnowMe PoC Codegen and Tauri Verification C-102](/knowme-poc-codegen-and-tauri-verification-c-102.md).

## Revised Phase Goal

As of `2026-07-15`, the phase end result is a **working proof-of-concept app**, not only code generation and CI verification.

The PoC must be built under:

```text
apps/<name>/
```

It must use repository scaffolds and skills, and be based on KnowMe reference documentation under:

```text
docs/reference-app/
```

The PoC should prove the skill package end to end and showcase the broadest practical supported capability set:

- Streaming `ContentBlock` chat
- PEM entity management
- SurrealDB graph-RAG memory
- Local-first sync
- Cross-platform Flutter, Tauri, and web clients from one Rust core
- Feature subset selected via web research on showcase-app best practices and 2026 on-device AI feasibility

## Supporting Objectives

The original codegen/CI scope remains as supporting objectives to be proven via the PoC:

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
  - macOS Tauri desktop
  - iOS simulator or Android emulator for Flutter
- Wire CI to run on every push:
  - `cargo clippy --workspace`
  - `audit.sh all`
  - Boundary test suites against the PoC

## Completed Session Work

Four requested app fixes were completed and verified on the web target.

### 1. Custom Branded Titlebar

- Disabled native window decorations with `decorations: false`.
- Replaced native chrome with the `Titlebar` component.
- Titlebar displays the KnowMe logo and wordmark.
- Window controls are platform-aware:
  - macOS traffic-light controls on the left.
  - Windows-style controls on the right.

### 2. File/View/Help Menu

Implemented a real application menu structure:

- **File**
  - Includes **Exit**.
- **View**
  - Includes **Toggle Developer Tools** as the last item.
- **Help**
  - Includes **About**.

### 3. KnowMe Branding

Applied KnowMe brand assets and tokens from `docs/reference-app/`:

- Exact color tokens, including:
  - `--ember`: `#FF6A3D`
  - Dark navy background: `#0B0F14`
- Fonts:
  - Space Grotesk
  - Inter
  - Roboto
  - JetBrains Mono
- Real K-monogram logo mark is now used as the app icon.
- Regenerated platform icon sets for:
  - macOS
  - Windows
  - iOS
  - Android

The K logo mark and `KnowMe` wordmark now render clearly and correctly in the branded titlebar, remain stable across reloads, and produce no console errors on the verified web target.

### 4. Startup Migration Command Fix

Registered the following Tauri commands as real stub commands so the frontend startup sequence no longer fails:

```text
run_migrations
load_seeds
attach_sync_shapes
memory_ingest
memory_search
entity_runtime_start
entity_runtime_stop
```

These stubs unblock startup while preserving the expected command boundary for later implementation.

## Bugs Found and Fixed During Web Testing

Testing the web target surfaced two native-context assumptions that crashed the app outside Tauri.

### Unconditional Tauri Window API Call

- Problem: `getCurrentWindow()` was called at module/init scope.
- Impact: Crashed the web app because no native Tauri context exists.
- Fix: Gate Tauri-specific calls behind `isTauri()`.

### Unconditional Tauri Event Listener

- Problem: `listen()` was called at module/init scope.
- Impact: Crashed the web app outside native Tauri.
- Fix: Gate Tauri event listener setup behind `isTauri()`.

## Verification Status

Verified on the web target:

- Branded titlebar renders correctly.
- KnowMe logo and wordmark render correctly.
- Menu/titlebar-related code produces no console errors.
- Startup no longer fails on missing migration/runtime commands.
- Tauri API calls are guarded outside native context.

Not yet verified in the currently running native Tauri window:

- The native Tauri window with PID `75130` is still showing the pre-fix bundle.
- Tauri's window process did not hot-reload the same way as the browser tab.
- Restarting the dev server or reloading the native window is required to see the fixes live in native Tauri.

## Next Action

Reload the native Tauri window or restart the dev server, then verify the same four fixes in the native desktop surface.

# Citations

1. stdin
2. manual:Hybrid Mobile Architecture/phase-codegen-and-ci-verification
