---
type: Reference
id: knowme-mode-selector-redesign-ms1-status-and-android-catalog-root-cause
title: KnowMe mode selector redesign MS1 status and Android catalog root cause
tags:
- knowme
- model-runtime-per-device
- mode-selector
- android
- flutter
- rust
- local-inference
links:
- ios-local-inference-switchover-to-mlx-swift-via-c-vtable
- android-litert-lm-local-tab-fix-and-remaining-device-proof-gap
sources:
- stdin
timestamp: 2026-07-26T01:13:03.206702+00:00
created_at: 2026-07-26T01:13:03.206702+00:00
updated_at: 2026-07-26T01:13:03.206702+00:00
revision: 0
---

## Context

- **Project:** KnowMe
- **Phase:** `model-runtime-per-device`
- **Workstream:** `mode-selector-redesign`
- **Milestone:** `MS1`
- **Captured:** `2026-07-26T00:34:36Z`
- **Status:** `awaiting_steer`
- **Device state:** Samsung/iPhone binaries are unchanged; no new build has been deployed or verified on hardware.

## Phase goals

- Ship Qwen3.5-9B Q4_K_M with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM Gemma 4 E2B with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

> Later iOS work moved away from the original iPhone Qwen3.5-9B target because of jetsam limits; see [iOS local inference switchover to MLX-Swift via C vtable](/ios-local-inference-switchover-to-mlx-swift-via-c-vtable.md). Android local inference context overlaps with [Android LiteRT-LM Local tab fix and remaining device proof gap](/android-litert-lm-local-tab-fix-and-remaining-device-proof-gap.md).

## Corrected diagnosis

The earlier diagnosis that Android was failing because of `NoLocalEngine` was incorrect.

- Android engine registration occurs at boot.
- The actual defect is a **catalog shape mismatch** that makes the agent list empty.
- A Rust-side fix exists, but it is not visible to Dart/Flutter yet and has not been deployed.

## Landed so far

Rust-only changes are complete and compiling:

- Embedded catalog now returns the service-compatible shape:
  - `runtime_agents`
  - `federated_agents`
- Per-conversation lane resolution was added in `send()`.
- `lane_availability()` probe was added.

These changes are not sufficient for user-visible behavior because the Flutter layer cannot yet reach the new probe and the UI fixes have not landed.

## Outstanding work

Required before the current redesign can be tested on-device:

- FFI passthrough for `lane_availability()` so Dart can call the Rust probe.
- Flutter fixes:
  - Read display labels from `metadata.title`.
  - Improve error handling in `setMode`.
  - Load the catalog correctly in new chats.
- Implement the single-button picker sheet.
- Add Rust tests and widget tests.
- Build and install on Samsung/iPhone.
- Verify behavior on physical devices.

## User-visible status

- The app currently installed on devices is still the old binary.
- No device behavior has changed.
- Nothing from this milestone is verified on hardware yet.
- The empty agent list diagnosis is not proven on-device until Flutter integration and deployment occur.

## Steering options

Two paths were identified:

1. **Continue through the full redesign**
   - Complete FFI passthrough.
   - Complete Flutter state/label/error fixes.
   - Build the single-button picker sheet.
   - Add tests.
   - Deploy to devices.

2. **Cut a smaller verification checkpoint first**
   - Apply only the minimal fixes needed to make the existing 3-button toolbar work:
     - Catalog shape parity.
     - `metadata.title` labels.
     - Error surfacing.
   - Deploy that build first.
   - Confirm the catalog-shape root cause on-device before proceeding with the single-button redesign.

The smaller checkpoint gives a faster testable answer and de-risks the redesign by proving the root cause first.

# Citations

1. [1] stdin