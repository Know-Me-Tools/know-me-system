---
type: Reference
id: ios-cert-monitor-still-awaiting-local-send-taps
title: iOS cert monitor still awaiting local send taps
tags:
- model-runtime-per-device
- ios-device-cert
- mlx
- think-split
- device-logs
- flutter-release
links:
- ios-cert-monitor-awaiting-local-think-split-send-on-iphone-17-pro-max
- m5a-subagent-alive-during-embedded-uar-policy-implementation
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-24T21:17:20.630595+00:00
created_at: 2026-07-24T21:17:20.630595+00:00
updated_at: 2026-07-24T21:17:20.630595+00:00
revision: 0
---

## Phase context

- Phase: `model-runtime-per-device`
- Project: `KnowMe`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system`
- Captured: `2026-07-24T21:15:58Z`
- Position: `model-runtime-per-device › ios-device-cert › awaiting_taps`
- Status: `monitoring`
- Device lane: iPhone 17 Pro Max local inference certification

## Phase goals

- Ship `Qwen3.5-9B Q4_K_M` with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM `Gemma 4 E2B`, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Current state

The iOS device-certification monitor remains live and is still waiting for operator taps, continuing the state from [iOS cert monitor awaiting local think-split send on iPhone 17 Pro Max](/ios-cert-monitor-awaiting-local-think-split-send-on-iphone-17-pro-max.md).

- App is installed on the iPhone.
- Cert monitor: `bd8ijfg47` live.
- Observed logs: benign network noise only, specifically another `nw_path` event.
- No certification lane markers observed yet:
  - no `send_local`
  - no MLX load marker
  - no `ThinkingDelta`
  - no clean `TextDelta`
  - no `RunFinished`
  - no error marker
- Status remains `monitoring`; no send has been certified.

## Required operator action

Operator must validate the installed app and trigger the local inference lane:

1. Open/validate the installed app on device.
2. Tap `On-device → Local`.
3. Send a prompt.

## Certification criteria after send

Once the local send occurs, certify the device log sequence:

1. `send_local`
2. MLX model load
3. `ThinkingDelta`
4. clean `TextDelta`
5. `RunFinished`

If this passes, optional follow-up certification can cover Orchestrator / embedded-UAR plumbing, related to the embedded runtime policy work tracked in [M5a subagent alive during embedded UAR policy implementation](/m5a-subagent-alive-during-embedded-uar-policy-implementation.md).

# Citations

1. stdin
2. manual:KnowMe/model-runtime-per-device