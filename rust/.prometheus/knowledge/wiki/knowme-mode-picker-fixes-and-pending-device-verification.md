---
type: Reference
id: knowme-mode-picker-fixes-and-pending-device-verification
title: KnowMe mode picker fixes and pending device verification
tags:
- knowme
- model-runtime-per-device
- mode-picker
- local-inference
- flutter
- rust
- device-certification
links:
- knowme-mode-selector-redesign-ms1-status-and-android-catalog-root-cause
- android-litert-lm-local-tab-fix-and-remaining-device-proof-gap
- ios-local-inference-switchover-to-mlx-swift-via-c-vtable
- ios-certification-of-local-and-embedded-uar-lanes-on-hardware
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-26T01:13:38.457906+00:00
created_at: 2026-07-26T01:13:38.457906+00:00
updated_at: 2026-07-26T01:13:38.457906+00:00
revision: 0
---

## Context

- **Project:** KnowMe
- **Phase:** `model-runtime-per-device`
- **Captured:** `2026-07-26T01:12:45Z`
- **Status:** still building; Android release APK pending install/verification; iOS hardware unavailable
- **Related prior work:** overlaps with [KnowMe mode selector redesign MS1 status and Android catalog root cause](/knowme-mode-selector-redesign-ms1-status-and-android-catalog-root-cause.md) and [Android LiteRT-LM Local tab fix and remaining device proof gap](/android-litert-lm-local-tab-fix-and-remaining-device-proof-gap.md).

## Phase goals

- Ship Qwen3.5-9B Q4_K_M with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM Gemma 4 E2B with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

> Later iOS local-inference work moved away from the original 9B target due to jetsam limits; see [iOS local inference switchover to MLX-Swift via C vtable](/ios-local-inference-switchover-to-mlx-swift-via-c-vtable.md). iOS lane behavior and hardware certification context are covered in [iOS certification of Local and embedded-UAR lanes on hardware](/ios-certification-of-local-and-embedded-uar-lanes-on-hardware.md).

## Fixed: “Local does nothing” root causes

Two independent defects caused embedded/local UI surfaces to appear inert:

1. **Embedded catalog shape mismatch**
   - File: `rust/crates/gen_ui_agent/src/control_plane.rs:72`
   - The embedded `control_plane` catalog returned agents as a bare array.
   - The service returns the envelope shape:

     ```json
     { "runtime_agents": [], "federated_agents": [] }
     ```

   - Embedded surfaces parsed the bare-array response as an empty agent list.

2. **Unhandled `setMode` rejection**
   - `setMode` lacked the `try/catch` behavior used by sibling calls.
   - `set_active_lane` deliberately rejected in some conditions.
   - The rejection escaped into an async-void path, so `state.error` was never populated.

## Mode picker redesign shipped

Implemented a redesigned mode/lane trigger:

- A single trigger is labelled with the current selection.
- Tapping it opens a sheet listing:
  - every catalog agent
  - `Local`
- Unavailable lanes are disabled and include an explanation.
- Availability is computed through a new side-effect-free `lane_availability` probe.

## Fixed: per-conversation lane routing gaps

An audit found the earlier per-conversation lane fix only covered `send()`.

The following paths still read the global lane setting:

- `send_with_attachments`
- `send_content`
- `send_interaction`

Impact:

- Sending an attachment from a Local conversation could run on whatever lane was currently set globally.

Resolution:

- Updated all send paths to respect the conversation-specific lane.
- Added a source-level guard so future send paths cannot silently reintroduce global-lane routing.

## Test fixture and widget corrections

### Agent fixture shape

The old toolbar test fixture hand-wrote agents in the service shape with a flat `name` key. This masked the embedded catalog bug because CI asserted against a shape production never emits.

Correction:

- Rewrote the fixture to use the real `AgentArtifact` shape.
- Agent display title now comes from `metadata.title`.

### Picker widget surface

Picker tests exposed a real widget bug:

- `openSheetOverlay` asserts on a `DrawerOverlay` ancestor.
- The Material `ListTile` used in the sheet asserted against the sheet's decorated surface.

Correction:

- Replaced `ListTile` with a flat shadcn row.
- Preserved the 48 px minimum target size.

## Verification status

### Passing

- `cargo test -p gen_ui_agent --lib`: **71/71 pass**
- New `mobile/test/features/chat/chat_mode_picker_test.dart`: **5/5 pass**
- iOS compile verification for `aarch64-apple-ios`: **clean**

### Flutter baseline failures

`flutter test` result:

- **28 pass**
- **3 fail**

The 3 failures also occur with these changes stashed, so they are considered pre-existing and unrelated:

- settings workspace dark/light
- narrow-phone categories

## Outstanding device verification

### Android

- Device: Samsung `SM-S936U`
- State: attached
- Release APK: building
- Next step: install the APK and verify the picker end-to-end on hardware after build completion.

### iOS

- Physical iPhone is not connected.
- `xcrun devicectl` reports both iPhones as `unavailable`.
- `flutter devices` only shows the simulator.
- Runtime iOS certification cannot happen in this session.
- Compile verification for `aarch64-apple-ios` is not runtime proof.

# Citations

1. stdin
2. manual:KnowMe/model-runtime-per-device