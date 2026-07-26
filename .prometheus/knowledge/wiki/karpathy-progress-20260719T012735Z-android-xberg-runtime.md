---
type: Reference
id: karpathy-progress-20260719T012735Z-android-xberg-runtime
title: "Android Xberg and Vulkan runtime integration"
tags:
- karpathy-progress
- android
- xberg
- failed
links: []
sources:
- conversation:operator-agent
timestamp: 2026-07-19T01:27:35Z
created_at: 2026-07-19T01:27:35Z
updated_at: 2026-07-19T01:27:35Z
revision: 1
---

## Intent

Install a fresh KnowMe Android build on the connected Samsung SM-S936U with the device-specific Qwen runtime and shared Xberg document-processing pipeline.

## Observed state

The Android device `R5GYB4AZD7A` was connected and contained no installed `tools.knowme.knowme` package. Flutter started an arm64 debug build from `$REPO_ROOT/mobile`; CargoKit used `$REPO_ROOT/mobile/build/gen_ui_ffi/build`, which was separate from the previously populated Rust target directory.

## Decision and delta

The current model-runtime, mobile identity/icon, and Xberg document-pipeline work is preserved as an auditable checkpoint before repairing the device build. Android remains the only claimed deployment surface for this record.

## Verification

`flutter run -d R5GYB4AZD7A --debug` failed after 9m41s during `assembleDebug`. The native build reached llama.cpp Vulkan compilation but did not produce or install an APK.

## Failure/lesson

CargoKit did not receive the Vulkan compatibility header configuration used by `scripts/android/build.sh`. Compilation of `ggml-vulkan.cpp` failed with `fatal error: 'vulkan/vulkan.hpp' file not found`. A successful standalone Rust build does not prove Flutter's independent CargoKit build path.

## Next experiment

Expose the Vulkan-Hpp headers to the CargoKit/CMake invocation through the shared Rust build configuration, rerun the warm Flutter build, then verify package installation, foreground launch, Rust-core readiness, persistence, and one streamed chat response.
