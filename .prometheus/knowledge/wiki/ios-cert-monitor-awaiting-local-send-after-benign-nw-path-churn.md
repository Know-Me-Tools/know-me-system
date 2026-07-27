---
type: Reference
id: ios-cert-monitor-awaiting-local-send-after-benign-nw-path-churn
title: iOS cert monitor awaiting Local send after benign nw_path churn
tags:
- model-runtime-per-device
- ios-device-cert
- device-logs
- think-split
- mlx
- flutter-release
links:
- ios-cert-monitor-still-quiet-before-local-send-taps
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-24T21:24:20.865176+00:00
created_at: 2026-07-24T21:24:20.865176+00:00
updated_at: 2026-07-24T21:24:20.865176+00:00
revision: 0
---

## Phase context

- Phase: `model-runtime-per-device`
- Project: `KnowMe`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system`
- Captured: `2026-07-24T21:23:32Z`
- Position: `model-runtime-per-device › ios-device-cert › awaiting_taps`
- Status: `monitoring`
- Device lane: iPhone 17 Pro Max local inference certification

## Phase goals

- Ship `Qwen3.5-9B Q4_K_M` with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM `Gemma 4 E2B`, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Current state

The iOS device-certification lane remains in the same blocked state as [iOS cert monitor still quiet before local send taps](/ios-cert-monitor-still-quiet-before-local-send-taps.md): app is on the iPhone, cert monitoring is live, and the operator has not yet sent from the Local tab.

- Cert monitor: `bd8ijfg47` live.
- Observed logs: benign network churn only, specifically `nw_path_copy_interface_with_generation` events.
- No `send_local` marker observed.
- No `ThinkingDelta` marker observed.
- No error markers observed.
- No actionable runtime failure is present; continue monitoring until the operator performs the Local send.

## Next validation step

1. Operator validates install, opens the Local tab, and sends a prompt.
2. Certify the think-split path from device logs once `send_local` / `ThinkingDelta` markers appear.
3. Optionally proceed to Orchestrator / embedded-UAR certification after local think-split certification succeeds.

# Citations

1. stdin
2. manual:KnowMe/model-runtime-per-device