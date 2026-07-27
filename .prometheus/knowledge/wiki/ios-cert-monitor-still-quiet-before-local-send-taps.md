---
type: Reference
id: ios-cert-monitor-still-quiet-before-local-send-taps
title: iOS cert monitor still quiet before local send taps
tags:
- model-runtime-per-device
- ios-device-cert
- device-logs
- think-split
- mlx
- flutter-release
links:
- ios-cert-monitor-still-awaiting-local-send-taps
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-24T21:18:00.471107+00:00
created_at: 2026-07-24T21:18:00.471107+00:00
updated_at: 2026-07-24T21:18:00.471107+00:00
revision: 0
---

## Phase context

- Phase: `model-runtime-per-device`
- Project: `KnowMe`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system`
- Captured: `2026-07-24T21:17:06Z`
- Position: `model-runtime-per-device › ios-device-cert › awaiting_taps`
- Status: `monitoring`
- Device lane: iPhone 17 Pro Max local inference certification

## Phase goals

- Ship `Qwen3.5-9B Q4_K_M` with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM `Gemma 4 E2B`, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Current state

The iOS device-certification lane remains in the same waiting state as [iOS cert monitor still awaiting local send taps](/ios-cert-monitor-still-awaiting-local-send-taps.md): the app is installed, the cert monitor is live, and no local-send marker has appeared yet.

- Cert monitor: `bd8ijfg47` live.
- Observed logs: benign network noise only, including `nw_path` events.
- No lane markers observed.
- No send has occurred yet.
- Operator action is still required: tap **On-device → Local**, then send.

## Next steps

1. Wait for operator taps on the device.
2. When the local send occurs, certify the think-split from device logs.
3. Optionally continue to Orchestrator / embedded-UAR certification after the iOS local path is verified.

# Citations

1. stdin
2. manual:KnowMe/model-runtime-per-device