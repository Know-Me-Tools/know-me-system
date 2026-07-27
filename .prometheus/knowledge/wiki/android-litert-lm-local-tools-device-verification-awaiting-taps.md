---
type: Reference
id: android-litert-lm-local-tools-device-verification-awaiting-taps
title: Android LiteRT-LM local tools device verification awaiting taps
tags:
- model-runtime-per-device
- android-device-cert
- litert-lm
- local-tools
- gemma
- cargokit
- device-logs
links:
- memory-fix-install-watch-armed-for-unplug-safe-ios-verification
- m5b-install-monitor-armed-for-model-runtime-picker-verification
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-25T22:40:46.102642+00:00
created_at: 2026-07-25T22:40:46.102642+00:00
updated_at: 2026-07-25T22:40:46.102642+00:00
revision: 0
---

## Phase context

- Phase: `model-runtime-per-device`
- Project: `KnowMe`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system`
- Captured: `2026-07-25T22:38:29Z`
- Position: `model-runtime-per-device › litertlm-local-tools › device-verify`
- Status: `awaiting_operator`
- Device lane: Samsung `SM-S936U` Android local inference/tool certification

## Phase goals

- Ship `Qwen3.5-9B Q4_K_M` with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM `Gemma 4 E2B`, using CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Current state

- KnowMe is installed and launched on the Samsung `SM-S936U`.
- APK size: `181.8 MB`.
- `flutter run` has been detached; the device is debugger-free and unplug-safe.
- Device-log monitor `bdlcvnn67` is armed for:
  - LiteRT-LM lane markers
  - tool-call markers
  - failure markers
- Operator action is required before certification can complete.

This Android lane is the LiteRT-LM counterpart to the iOS runtime/device verification work tracked in [Memory-fix install watch armed for unplug-safe iOS verification](/memory-fix-install-watch-armed-for-unplug-safe-ios-verification.md) and earlier picker/runtime setup in [M5b install monitor armed for model runtime picker verification](/m5b-install-monitor-armed-for-model-runtime-picker-verification.md).

## Build certification already completed

The Gradle build completed the `cargokit` NDK cross-compile for `aarch64-linux-android` successfully.

Certified by this build:

- Android-target compile verification for the bounded tool-shortlist change `e07002b`.
- Closure of the previous bare-shell verification gap for Android NDK compilation.
- Rust side of `litertlm-local-tools` verified in combination with `69/69` passing host tests.

## Required operator verification steps

Force-quit and reopen KnowMe before testing so the newly installed APK is the active process.

### 1. Local LiteRT-LM lane

1. Open **On-device → Local**.
2. Send:

```text
Say hello in one short sentence.
```

Expected behavior:

- First send downloads the Gemma 4 E2B artifact from Hugging Face.
- Artifact size is approximately `3 GB`.
- Download is one-time; expect a long delay before the first token.
- Wi-Fi is recommended.
- The monitor should observe download and model-load markers in logs.

### 2. Tool-call certification

Ask a prompt requiring a tool, for example:

```text
search my memory for the truck
```

This certifies the full `litertlm-local-tools` path:

1. bounded shortlist
2. gate validation, including schema and permission checks
3. tool execution
4. streamed tool events
5. continuation after tool result

A mutating tool should require user confirmation before execution.

### 3. Memory across restart

1. Tell the app a fact.
2. Force-quit KnowMe.
3. Reopen KnowMe.
4. Ask for the fact back.

Expected behavior:

- The memory seeding fix should persist across restart.
- The fix is engine-agnostic and should hold on Android as well.

## Runtime strategy

- Certify the generic CPU Gemma 4 E2B artifact first.
- Treat the `SM8750`-specific build as a later optimization only after the generic CPU artifact passes correctness certification.

## Next action

Await operator taps:

1. Local lane prompt to trigger first download/load.
2. Tool-call prompt to certify shortlist → gate → execution → streamed events → continuation.
3. Memory persistence prompt across force-quit/reopen.

Certification can be reported from device logs as markers arrive.

# Citations

1. stdin
2. manual:KnowMe/model-runtime-per-device