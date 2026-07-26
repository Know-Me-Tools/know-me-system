---
type: Reference
id: memory-fix-install-watch-armed-for-unplug-safe-ios-verification
title: Memory-fix install watch armed for unplug-safe iOS verification
tags:
- model-runtime-per-device
- memory-fix
- flutter-run
- ios-device
- install-monitor
- local-inference
links:
- m5b-install-monitor-armed-for-model-runtime-picker-verification
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-25T21:00:32.781604+00:00
created_at: 2026-07-25T21:00:32.781604+00:00
updated_at: 2026-07-25T21:00:32.781604+00:00
revision: 0
---

## Phase context

- Phase: `model-runtime-per-device`
- Project: `KnowMe`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system`
- Captured: `2026-07-25T20:59:16Z`
- Position: `model-runtime-per-device › memory-fix › installing`
- Status: `executing`
- Device lane: iPhone 17 Pro Max local inference/runtime verification

## Phase goals

- Ship `Qwen3.5-9B Q4_K_M` with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM `Gemma 4 E2B`, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Current state

- Install watch `bfvndwn3e` is armed.
- `flutter run` should be detached as soon as the app launches so the device is unplug-safe.
- This continues the iOS device runtime install/verification path after prior picker-install monitoring in [M5b install monitor armed for model runtime picker verification](/m5b-install-monitor-armed-for-model-runtime-picker-verification.md).

## Next verification steps

After app launch and `flutter run` detach, the operator should verify:

1. Paste behavior works as expected.
2. Within-conversation memory persists correctly.
3. Cold-start memory survives force-quit and relaunch.
4. No memory leaks or bleed occur across conversations.

# Citations

1. stdin
2. manual:KnowMe/model-runtime-per-device