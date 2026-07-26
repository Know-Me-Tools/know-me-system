---
type: Reference
id: m5b-install-monitor-armed-for-model-runtime-picker-verification
title: M5b install monitor armed for model runtime picker verification
tags:
- model-runtime-per-device
- flutter-run
- ios-device
- model-picker
- install-monitor
- qwen
- litert-lm
links:
- ios-cert-monitor-holding-for-local-send-with-benign-nw-path-noise
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-25T14:46:08.897764+00:00
created_at: 2026-07-25T14:46:08.897764+00:00
updated_at: 2026-07-25T14:46:08.897764+00:00
revision: 0
---

## Phase context

- Phase: `model-runtime-per-device`
- Project: `KnowMe`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system`
- Captured: `2026-07-25T14:45:00Z`
- Position: `model-runtime-per-device › M5b › installing`
- Status: `executing`

## Phase goals

- Ship `Qwen3.5-9B Q4_K_M` with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM `Gemma 4 E2B`, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Current state

- Install-complete monitor `bt85ptbtr` is armed.
- `flutter run` should be detached immediately after the app launches, making the device unplug-safe.
- Next operator verification target: confirm **3 pickers** and their **precedence** after app launch.
- This continues the iOS model-runtime device workstream after earlier local-send certification monitoring such as [iOS cert monitor holding for Local send with benign nw_path noise](/ios-cert-monitor-holding-for-local-send-with-benign-nw-path-noise.md).

## Next steps

1. Wait for app launch.
2. Detach `flutter run` once launch is confirmed.
3. Operator verifies:
   - all 3 pickers are present;
   - picker precedence is correct.

# Citations

1. stdin
2. manual:KnowMe/model-runtime-per-device