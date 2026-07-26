---
type: Reference
id: knowme-mode-picker-fixes-and-unresolved-device-certification
title: KnowMe mode picker fixes and unresolved device certification
tags:
- knowme
- model-runtime-per-device
- mode-picker
- flutter
- local-inference
- device-certification
- desktop-stable-port
links:
- knowme-mode-selector-redesign-ms1-status-and-android-catalog-root-cause
- android-litert-lm-local-tab-fix-and-remaining-device-proof-gap
- ios-local-inference-switchover-to-mlx-swift-via-c-vtable
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-26T01:23:34.739699+00:00
created_at: 2026-07-26T01:23:34.739699+00:00
updated_at: 2026-07-26T01:23:34.739699+00:00
revision: 0
---

## Context

- **Project:** KnowMe
- **Phase:** `model-runtime-per-device`
- **Captured:** `2026-07-26T01:13:14Z`
- **KBD root:** `/Users/gqadonis/Projects/know-me/know-me-system`
- **Status:** Android release APK building; iOS hardware unavailable; desktop stable port still executing.
- **Related:** follows the mode selector/root-cause work in [KnowMe mode selector redesign MS1 status and Android catalog root cause](/knowme-mode-selector-redesign-ms1-status-and-android-catalog-root-cause.md) and overlaps Android proof work in [Android LiteRT-LM Local tab fix and remaining device proof gap](/android-litert-lm-local-tab-fix-and-remaining-device-proof-gap.md).

## Phase goals

- Ship Qwen3.5-9B Q4_K_M with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM Gemma 4 E2B with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

> Later iOS local-inference work moved away from the original 9B target because of jetsam limits; see [iOS local inference switchover to MLX-Swift via C vtable](/ios-local-inference-switchover-to-mlx-swift-via-c-vtable.md).

## Fixes and findings

### Toolbar fixture used non-production agent shape

- The old toolbar test fixture hand-wrote agents in the **service** shape with a flat `name` key.
- Production does **not** emit that shape; production uses `AgentArtifact` with `metadata.title`.
- CI missed defect #1 because the assertion targeted fixture-only data rather than production-shaped artifacts.
- The fixture was rewritten to use the real `AgentArtifact` shape and `metadata.title`.

### Picker test exposed widget bug

- Writing picker tests surfaced a real widget defect:
  - `openSheetOverlay` asserts on a `DrawerOverlay` ancestor.
  - The Material `ListTile` asserted against the sheet's decorated surface.
- The row implementation was replaced with a flat shadcn-style row.
- The replacement also preserves the minimum 48 px tap-target floor.

## Device verification state

### Android

- Target device: Samsung `SM-S936U`.
- Device is attached.
- Release APK is currently building.
- Pending action: install the release APK when available and verify the picker end-to-end on hardware.

### iOS

- Physical iPhone is **not connected**.
- `xcrun devicectl` reports both iPhones as `unavailable`.
- `flutter devices` shows only the simulator.
- iOS device certification cannot be completed in this session.
- The change compiles cleanly for `aarch64-apple-ios`, but this is compile verification only, not runtime proof.
- Pending action: connect the iPhone and run the same picker verification on physical hardware.

## Desktop stable port position

Prometheus position snapshot:

- **Position:** `model-runtime-per-device › desktop-stable-port › desktop-stable-port`
- **Status:** `executing`
- **Progress:** changes `4/5`
- **Last completed:**
  - macOS deployment floor raised to 14.0.
  - `mlex` / `mlx-c` lane compiles for the first time.
  - `MlxcEngine` certified.
  - Per-target engine matrix verified by full `tauri-plugin` build:
    - macOS: MLX
    - Windows/Linux: `llama.cpp`
- **Remaining:**
  - Runtime certification: launch desktop app and complete an agent turn.
  - Windows/Linux CI cross-build.

# Citations

1. stdin
2. manual:KnowMe/model-runtime-per-device