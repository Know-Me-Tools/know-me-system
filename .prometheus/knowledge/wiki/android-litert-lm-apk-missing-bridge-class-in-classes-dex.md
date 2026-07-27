---
type: Reference
id: android-litert-lm-apk-missing-bridge-class-in-classes-dex
title: Android LiteRT-LM APK missing bridge class in classes.dex
tags:
- model-runtime-per-device
- android-litert-lm
- apk-packaging
- dexing
- uar
- gemma
- device-cert
links:
- memory-fix-install-watch-armed-for-unplug-safe-ios-verification
- ios-cert-monitor-holding-for-local-send-with-benign-nw-path-noise
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-25T22:58:09.921389+00:00
created_at: 2026-07-25T22:58:09.921389+00:00
updated_at: 2026-07-25T22:58:09.921389+00:00
revision: 0
---

## Context

- Phase: `model-runtime-per-device`
- Project: `KnowMe`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system`
- Captured: `2026-07-25T22:49:05Z`
- Position: `model-runtime-per-device › android-litertlm-runtime › blocked-packaging`
- Status: `blocked`

## Phase goals

- Ship `Qwen3.5-9B Q4_K_M` with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM `Gemma 4 E2B`, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Certified working on Android

- `cargokit`/NDK cross-compile is green for `aarch64-linux-android`.
- Tool-shortlist change `e07002b` is compile-verified for Android.
- Embedded UAR runs on Android.
- UAR resolves:
  - `provider=knowme-local-litert-lm`
  - `model=gemma-4-e2b-it-litertlm`
- Android 3-tier policy and provenance plumbing are active.
- Model artifact is present.
- Runtime load is attempted at the expected model path.

## Blocker

Android local inference does **not** work on the Samsung device because `KnowMeLiteRtLmBridge` is missing from the APK dex output.

Evidence:

- `KnowMeLiteRtLmBridge` compiles successfully into both debug and release `.class` outputs.
- The generated APK `classes.dex` does **not** contain the bridge descriptor.
- Dex enumeration shows only `MainActivity` from that package.
- R8/ProGuard are not configured, so this is not explained by shrinker removal.
- The class is being lost between Kotlin compilation and dexing.

## Impact

- Blocks `android-litertlm-runtime` despite its recorded `complete` state in `progress.json`.
- Invalidates the recorded `complete` state for `litertlm-device-proof`; that proof could not have covered this runtime path while the bridge class was absent from `classes.dex`.
- Blocks Samsung certification of:
  - local-lane generation
  - tool call flow: shortlist → gate → execution → continuation

## Required fix path

First debugging moves:

1. Inspect app module dexing inputs for the affected release/debug variants.
2. Verify whether `KnowMeLiteRtLmBridge` is reachable from the release variant compile/runtime classpath.
3. Rebuild the APK.
4. Re-enumerate `classes.dex`.
5. Confirm the bridge descriptor appears in dex output.
6. Re-run Samsung certification once packaged.

Fast falsifiable test:

```sh
# After rebuild, inspect APK dex descriptors and verify KnowMeLiteRtLmBridge exists.
# Expected: descriptor for KnowMeLiteRtLmBridge appears in classes.dex.
# Current failure: only MainActivity appears from the package.
```

## Related device lanes

This Android blocker is separate from the iOS runtime/device-certification lane, which has been tracked through install and local-send monitoring such as [Memory-fix install watch armed for unplug-safe iOS verification](/memory-fix-install-watch-armed-for-unplug-safe-ios-verification.md) and [iOS cert monitor holding for Local send with benign nw_path noise](/ios-cert-monitor-holding-for-local-send-with-benign-nw-path-noise.md).

# Citations

1. stdin
2. manual:KnowMe/model-runtime-per-device