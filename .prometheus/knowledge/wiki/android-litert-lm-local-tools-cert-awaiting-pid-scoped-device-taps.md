---
type: Reference
id: android-litert-lm-local-tools-cert-awaiting-pid-scoped-device-taps
title: Android LiteRT-LM local tools cert awaiting PID-scoped device taps
tags:
- model-runtime-per-device
- android-device-cert
- litert-lm
- local-tools
- gemma
- device-logs
- cargokit
- ndk
links:
- ios-cert-monitor-holding-for-local-send-with-benign-nw-path-noise
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-25T22:46:00.167846+00:00
created_at: 2026-07-25T22:46:00.167846+00:00
updated_at: 2026-07-25T22:46:00.167846+00:00
revision: 0
---

## Phase context

- Phase: `model-runtime-per-device`
- Project: `KnowMe`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system`
- Captured: `2026-07-25T22:40:22Z`
- Position: `model-runtime-per-device › litertlm-local-tools › device-verify`
- Status: `awaiting_operator`
- Device lane: Samsung Android local inference and tool-call certification

## Phase goals

- Ship `Qwen3.5-9B Q4_K_M` with matching `mmproj` vision support on iPhone 17 Pro Max using Metal; related iOS monitoring is tracked separately in [iOS cert monitor holding for Local send with benign nw_path noise](/ios-cert-monitor-holding-for-local-send-with-benign-nw-path-noise.md).
- Ship Android local text-agent inference through LiteRT-LM `Gemma 4 E2B`, using CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Current state

- KnowMe is running on the Samsung device as a release build, debugger-free.
- App PID: `14261`.
- The previous log monitor flood was caused by an over-broad filter matching Android system internals:
  - `gate` matched `CheckOpsDelegateDispatcher`.
  - `jni` matched system libraries.
- The flooding monitor was killed.
- New monitor `bbh3665mr` is scoped to KnowMe's PID only, preventing Android system chatter from reaching the certification logs.

## Verified build state

- Gradle build completed the `cargokit` NDK cross-compile for `aarch64-linux-android`.
- This compile-verifies shortlist change `e07002b` on the Android target.
- Host test suite passed: `69/69`.
- Rust-side implementation is considered verified; remaining work is on-device behavior certification.

## Operator certification steps

### 1. Local LiteRT-LM lane

Operator should tap **On-device → Local** and send:

```text
Say hello in one short sentence.
```

Expected first-run behavior:

- Downloads the `Gemma-4 E2B` artifact, approximately `3 GB`.
- One-time download; Wi-Fi recommended.
- Expected log progression:
  1. `Downloading`
  2. verification
  3. `ModelLoading`
  4. streamed token deltas

### 2. Tool-call certification

Operator should ask a prompt requiring a tool, for example:

```text
search my memory for the truck
```

This certifies the `litertlm-local-tools` path:

1. shortlist
2. gate: schema + permission checks
3. execution
4. streamed tool events
5. continuation

Mutating tools must prompt for confirmation before execution.

### 3. Memory across restart

Operator should:

1. Tell the app a fact.
2. Force-quit the app.
3. Reopen the app.
4. Ask for the fact back.

This verifies memory persistence across app restart.

## Next action

Await operator taps for Local send, tool-call prompt, and memory-across-restart. Certify from PID-scoped Samsung logs emitted by KnowMe PID `14261`.

# Citations

1. stdin
2. manual:KnowMe/model-runtime-per-device