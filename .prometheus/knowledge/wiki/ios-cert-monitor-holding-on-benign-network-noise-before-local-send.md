---
type: Reference
id: ios-cert-monitor-holding-on-benign-network-noise-before-local-send
title: iOS cert monitor holding on benign network noise before local send
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
timestamp: 2026-07-24T21:18:39.584979+00:00
created_at: 2026-07-24T21:18:39.584979+00:00
updated_at: 2026-07-24T21:18:39.584979+00:00
revision: 0
---

## Phase context

- Phase: `model-runtime-per-device`
- Project: `KnowMe`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system`
- Captured: `2026-07-24T21:17:48Z`
- Position: `model-runtime-per-device › ios-device-cert › awaiting_taps`
- Status: `monitoring`
- Device lane: iPhone 17 Pro Max local inference certification

## Phase goals

- Ship `Qwen3.5-9B Q4_K_M` with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM `Gemma 4 E2B`, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Current state

The iOS device-certification lane remains in the same waiting state as [iOS cert monitor still quiet before local send taps](/ios-cert-monitor-still-quiet-before-local-send-taps.md): the app is on the iPhone, cert monitoring is live, and no local-send lane marker has appeared.

- Cert monitor: `bd8ijfg47` live.
- Observed logs: benign network noise only, including another `nw_path` event.
- No `send_local` marker observed.
- No error observed.
- No action should be taken until an operator tap produces a real cert marker or an error.

## Certification criteria after operator taps Local and sends

Watch device logs for the complete local think-split sequence:

1. `send_local`
2. MLX model load
3. `ThinkingDelta`
4. Clean `TextDelta`
5. `RunFinished`

Successful observation of this sequence certifies the iOS local think-split path for the installed build.

## Next actions

- Wait for the operator to validate install, tap **Local**, and send.
- Certify the think-split sequence from device logs once the send occurs.
- After local-send certification, optionally certify Orchestrator / embedded-UAR behavior.

# Citations

1. stdin
2. manual:KnowMe/model-runtime-per-device