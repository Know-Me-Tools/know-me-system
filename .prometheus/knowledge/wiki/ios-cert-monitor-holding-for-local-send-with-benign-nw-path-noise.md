---
type: Reference
id: ios-cert-monitor-holding-for-local-send-with-benign-nw-path-noise
title: iOS cert monitor holding for Local send with benign nw_path noise
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
timestamp: 2026-07-24T21:25:59.477127+00:00
created_at: 2026-07-24T21:25:59.477127+00:00
updated_at: 2026-07-24T21:25:59.477127+00:00
revision: 0
---

## Phase context

- Phase: `model-runtime-per-device`
- Project: `KnowMe`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system`
- Captured: `2026-07-24T21:25:04Z`
- Position: `model-runtime-per-device › ios-device-cert › awaiting_taps`
- Status: `monitoring`
- Device lane: iPhone 17 Pro Max local inference certification

## Phase goals

- Ship `Qwen3.5-9B Q4_K_M` with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM `Gemma 4 E2B`, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Current state

The iOS device-certification lane remains blocked on operator action, continuing the same waiting condition as [iOS cert monitor awaiting Local send after benign nw_path churn](/ios-cert-monitor-awaiting-local-send-after-benign-nw-path-churn.md).

- App is installed on the iPhone.
- Cert monitor is live.
- Observed logs contain benign network noise only, including `nw_path` churn.
- No lane markers have appeared.
- No Local send / think-split evidence has been captured yet.

## Next action

Operator must use the app's **Local** lane and send a prompt from the device. After that, certify the think-split from device logs.

# Citations

1. stdin
2. manual:KnowMe/model-runtime-per-device