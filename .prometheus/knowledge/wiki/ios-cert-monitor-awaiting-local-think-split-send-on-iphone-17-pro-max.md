---
type: Reference
id: ios-cert-monitor-awaiting-local-think-split-send-on-iphone-17-pro-max
title: iOS cert monitor awaiting local think-split send on iPhone 17 Pro Max
tags:
- model-runtime-per-device
- ios-device-cert
- mlx
- think-split
- device-logs
- flutter-release
links:
- ios-device-cert-awaiting-taps-after-knowme-install-on-iphone-17-pro-max
- ios-local-mlx-think-split-certification-plan-for-knowme
sources:
- stdin
timestamp: 2026-07-24T21:16:11.631811+00:00
created_at: 2026-07-24T21:16:11.631811+00:00
updated_at: 2026-07-24T21:16:11.631811+00:00
revision: 0
---

## Phase context

- Phase: `model-runtime-per-device`
- Project: `KnowMe`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system`
- Captured: `2026-07-24T21:15:16Z`
- Position: `model-runtime-per-device › ios-device-cert › awaiting_taps`
- Status: `monitoring`
- Device: iPhone 17 Pro Max

## Phase goals

- Ship `Qwen3.5-9B Q4_K_M` with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM `Gemma 4 E2B`, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Current state

The iOS device certification lane remains blocked on operator taps after the app install covered in [iOS device cert awaiting taps after KnowMe install on iPhone 17 Pro Max](/ios-device-cert-awaiting-taps-after-knowme-install-on-iphone-17-pro-max.md).

- Cert monitor: `bd8ijfg47` live.
- Observed logs: benign `nw_path` cache-miss noise only.
- No lane markers have appeared yet, so no local send has happened.
- Operator action required on the phone:
  1. Validate KnowMe launches.
  2. Open Chat.
  3. Select **On-device → Local**.
  4. Send: `Say hello in one short sentence.`

## Certification criteria

The monitor should stay quiet on routine network noise and only react when a real local generation marker appears.

Expected successful device-log sequence:

```text
send_local
MLX load / ModelLoading
ThinkingDelta
clean TextDelta
RunFinished
```

This certifies the iOS local MLX think-split path planned in [iOS local MLX think-split certification plan for KnowMe](/ios-local-mlx-think-split-certification-plan-for-knowme.md).

## Next steps

- Wait for operator to perform the on-device Local send.
- Certify the think-split from device logs.
- If clean, close P1 `#19` / task B.
- Optional follow-up: certify Orchestrator / embedded-UAR path after local lane validation.

# Citations

1. stdin