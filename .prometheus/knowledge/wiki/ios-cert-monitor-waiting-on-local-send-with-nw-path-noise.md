---
type: Reference
id: ios-cert-monitor-waiting-on-local-send-with-nw-path-noise
title: iOS cert monitor waiting on Local send with nw_path noise
tags:
- model-runtime-per-device
- ios-device-cert
- device-logs
- think-split
- mlx
- flutter-release
links:
- ios-cert-monitor-awaiting-local-send-after-benign-nw-path-churn
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-24T21:25:24.624657+00:00
created_at: 2026-07-24T21:25:24.624657+00:00
updated_at: 2026-07-24T21:25:24.624657+00:00
revision: 0
---

## Phase context

- Phase: `model-runtime-per-device`
- Project: `KnowMe`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system`
- Captured: `2026-07-24T21:24:42Z`
- Position: `model-runtime-per-device › ios-device-cert › awaiting_taps`
- Status: `monitoring`
- Device lane: iPhone 17 Pro Max local inference certification

## Phase goals

- Ship `Qwen3.5-9B Q4_K_M` with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM `Gemma 4 E2B`, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Current state

The iOS device-certification lane remains blocked on operator action, continuing the same wait state as [iOS cert monitor awaiting Local send after benign nw_path churn](/ios-cert-monitor-awaiting-local-send-after-benign-nw-path-churn.md).

- App is installed on the iPhone.
- Cert monitor: `bd8ijfg47` live.
- Observed logs: benign `nw_path` network churn only.
- No lane markers have appeared.
- Awaiting operator to validate install, tap `Local`, and send.

## Next steps

1. Operator sends from the `Local` tab on device.
2. Certify the think-split from device logs once the local-send marker appears.
3. Optionally proceed to Orchestrator / embedded-UAR certification after local inference is verified.

# Citations

1. [1] stdin
2. [2] manual:KnowMe/model-runtime-per-device