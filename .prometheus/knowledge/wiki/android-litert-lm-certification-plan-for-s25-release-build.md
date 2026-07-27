---
type: Reference
id: android-litert-lm-certification-plan-for-s25-release-build
title: Android LiteRT-LM certification plan for S25+ release build
tags:
- model-runtime-per-device
- android-certification
- litert-lm
- gemma-4-e2b
- cargokit
- gradle-ndk
- local-inference
- tool-calls
links:
- memory-fix-install-watch-armed-for-unplug-safe-ios-verification
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-25T22:37:40.397764+00:00
created_at: 2026-07-25T22:37:40.397764+00:00
updated_at: 2026-07-25T22:37:40.397764+00:00
revision: 0
---

## Phase context

- Phase: `model-runtime-per-device`
- Project: `KnowMe`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system`
- Captured: `2026-07-25T22:17:50Z`
- Position: `model-runtime-per-device › litertlm-local-tools › android-build`
- Status: `executing`
- Device: Samsung SM-S936U / S25+, Android 16, `android-arm64`
- Attached device ID: `R5GYB4AZD7A`

## Phase goals

- Ship `Qwen3.5-9B Q4_K_M` with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM `Gemma 4 E2B`, using CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Build verification finding

- `rust_builder/android/build.gradle` applies the `cargokit` plugin.
- Gradle therefore drives the Cargo NDK cross-compile for Android.
- A successful Gradle `assembleRelease` is the required compile verification for the shortlist change on `aarch64-linux-android`; this closes the earlier gap where bare-shell verification could not complete.
- Current build is still running:
  - Build/job identifier: `b5vapctl4`
  - Completion monitor: `byk4cxwn0`
- `I/flutter` logcat lines currently visible are from the previously installed app on the device, not from the new release build.

## Android certification plan

### Automated steps

1. Device confirmed attached: `R5GYB4AZD7A`, `android-arm64`.
2. Run Gradle `assembleRelease`; this invokes `cargokit` and performs the Cargo NDK cross-compile.
3. After successful build, install the release APK.
4. Detach after install so the app runs debugger-free.

### Operator/device-tap steps

1. Open the **Local lane**:
   - Tap **On-device → Local**.
   - Send a prompt.
   - First send lazily downloads the LiteRT-LM `Gemma 4 E2B` artifact.
   - Artifact is the generic CPU build, pinned to its Hugging Face revision.
   - Download size is several GB and should be one-time.
   - Success criterion: LiteRT-LM local generation works on device.
2. Exercise one permitted tool call:
   - Confirms full local tool path: shortlist → gate → execution → streamed tool events → continuation.
   - Gate coverage includes schema and permission checks.
3. Verify memory persistence:
   - Force-quit the app.
   - Reopen it.
   - Confirm earlier facts persist.
   - The memory fix is engine-agnostic and should hold for the Android LiteRT-LM path as well as the iOS runtime path described in [Memory-fix install watch armed for unplug-safe iOS verification](/memory-fix-install-watch-armed-for-unplug-safe-ios-verification.md).

## Model artifact decision

- Certify the generic CPU LiteRT-LM artifact first.
- Treat the SM8750-specific artifact as a later optimization target only after the CPU artifact passes correctness certification.
- Android default is currently `gemma-4-e2b-it-litertlm` with a pinned revision.

## Next action

Wait for `assembleRelease` to finish. If successful, install the release APK, detach, then run the local prompt, permitted tool call, and memory persistence checks on the S25+.

# Citations

1. stdin
2. manual:KnowMe/model-runtime-per-device