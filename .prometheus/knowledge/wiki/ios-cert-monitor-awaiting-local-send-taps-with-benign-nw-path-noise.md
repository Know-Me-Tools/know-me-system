---
type: Reference
id: ios-cert-monitor-awaiting-local-send-taps-with-benign-nw-path-noise
title: iOS cert monitor awaiting local send taps with benign nw_path noise
tags:
- model-runtime-per-device
- ios-device-cert
- device-logs
- think-split
- mlx
- flutter-release
links:
- ios-cert-monitor-still-quiet-before-local-send-taps
- ios-cert-monitor-awaiting-local-send-after-benign-network-churn
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-24T21:23:44.205592+00:00
created_at: 2026-07-24T21:23:44.205592+00:00
updated_at: 2026-07-24T21:23:44.205592+00:00
revision: 0
---

## Phase context

- Phase: `model-runtime-per-device`
- Project: `KnowMe`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system`
- Captured: `2026-07-24T21:19:06Z`
- Position: `model-runtime-per-device › ios-device-cert › awaiting_taps`
- Status: `monitoring`
- Device lane: iPhone 17 Pro Max local inference certification

## Phase goals

- Ship `Qwen3.5-9B Q4_K_M` with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM `Gemma 4 E2B`, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Current state

The iOS certification lane remains blocked on operator taps, continuing the same wait state as [iOS cert monitor still quiet before local send taps](/ios-cert-monitor-still-quiet-before-local-send-taps.md) and [iOS cert monitor awaiting local send after benign network churn](/ios-cert-monitor-awaiting-local-send-after-benign-network-churn.md).

- App is installed on the iPhone.
- Cert monitor: `bd8ijfg47` live.
- Logs show benign network noise only; latest observed event was another `nw_path` event.
- No lane markers have appeared.
- No local send has been observed.
- No certification action is available until operator interaction occurs.

## Next actions

1. Operator validates the install on-device.
2. Operator opens `Local` and sends a prompt.
3. Certify the local think-split from device logs after the local-send marker appears.
4. Optionally certify Orchestrator / embedded-UAR path after local inference certification.

# Citations

1. stdin
2. manual:KnowMe/model-runtime-per-device