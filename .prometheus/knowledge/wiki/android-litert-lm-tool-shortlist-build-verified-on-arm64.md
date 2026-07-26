---
type: Reference
id: android-litert-lm-tool-shortlist-build-verified-on-arm64
title: Android LiteRT-LM tool shortlist build verified on arm64
tags:
- model-runtime-per-device
- android
- litert-lm
- gemma
- cargokit
- ndk
- tool-calls
- flutter-release
links:
- memory-fix-install-watch-armed-for-unplug-safe-ios-verification
- m5b-install-monitor-armed-for-model-runtime-picker-verification
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-25T22:38:47.266749+00:00
created_at: 2026-07-25T22:38:47.266749+00:00
updated_at: 2026-07-25T22:38:47.266749+00:00
revision: 0
---

## Phase context

- Phase: `model-runtime-per-device`
- Project: `KnowMe`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system`
- Captured: `2026-07-25T22:37:18Z`
- Position: `model-runtime-per-device › litertlm-local-tools › installing`
- Status: `executing`
- Device lane: Samsung/Android local text-agent inference certification

## Phase goals

- Ship `Qwen3.5-9B Q4_K_M` with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM `Gemma 4 E2B`, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Current state

- Gradle `assembleRelease` completed successfully.
- Built artifact: `app-release.apk`, size `181.8 MB`.
- Install is in progress on Android.
- Install monitor `b65r9jjh7` is armed.
- The build was not stuck: Android Studio JVM activity and advancing logs indicated progress while the first-session NDK/cargokit path cross-compiled the Rust workspace for `aarch64-linux-android`.

## Verification completed

The green Android release build compile-verifies the bounded `litertlm-local-tools` shortlist change on `aarch64-linux-android`.

- Change verified: bounded tool-shortlist change `e07002b`.
- Target verified: `aarch64-linux-android`.
- Build path: Gradle release build using cargokit/NDK.
- Gap closed: previous verification covered host and iOS targets only; Android arm64 is now compile-verified.

## Next operator actions

After install completes:

1. Detach `flutter run` so the app runs debugger-free.
2. In the app, tap **On-device → Local**.
3. Send a prompt.
   - First send lazily downloads the LiteRT-LM `Gemma 4 E2B` artifact.
   - Artifact is several GB and one-time, so first-token latency is expected on the first run.
4. Trigger one permitted tool call to certify the complete path:
   - shortlist
   - gate validation: schema + permission
   - execution
   - streamed tool events
   - continuation
5. Force-quit and reopen the app, then confirm earlier facts persist.
   - The memory persistence fix is engine-agnostic and should hold on Android as well.

## Related context

This Android lane runs alongside the iOS model-runtime device workstream, including prior install and detach guidance in [Memory-fix install watch armed for unplug-safe iOS verification](/memory-fix-install-watch-armed-for-unplug-safe-ios-verification.md) and picker verification work in [M5b install monitor armed for model runtime picker verification](/m5b-install-monitor-armed-for-model-runtime-picker-verification.md).

# Citations

1. stdin
2. manual:KnowMe/model-runtime-per-device