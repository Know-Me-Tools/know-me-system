---
type: Reference
id: ios-local-inference-cert-monitor-interrupted-during-benign-nw-path-noise
title: iOS local inference cert monitor interrupted during benign nw_path noise
tags:
- model-runtime-per-device
- ios-device-cert
- device-logs
- local-inference
- nw-path
- mlx
links:
- ios-cert-monitor-holding-for-local-send-with-benign-nw-path-noise
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-24T21:35:13.594859+00:00
created_at: 2026-07-24T21:35:13.594859+00:00
updated_at: 2026-07-24T21:35:13.594859+00:00
revision: 0
---

## Phase context

- Phase: `model-runtime-per-device`
- Project: `KnowMe`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system`
- Captured: `2026-07-24T21:25:49Z`
- Position: `model-runtime-per-device › ios-device-cert › awaiting_taps`
- Status: `monitoring`
- Device lane: iPhone 17 Pro Max local inference certification

## Phase goals

- Ship `Qwen3.5-9B Q4_K_M` with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM `Gemma 4 E2B`, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Current state

The iOS device-certification lane remains blocked on operator action, continuing the same wait condition as [iOS cert monitor holding for Local send with benign nw_path noise](/ios-cert-monitor-holding-for-local-send-with-benign-nw-path-noise.md).

- Cert monitor is still armed.
- The previous turn was interrupted; it is unclear whether the interrupt was meant to redirect the work.
- Observed device logs still contain only benign network and agent-generation churn:
  - `nw_path`
  - agent-generation churn
- No Local-tab send has been observed.
- No local inference lane markers have appeared:
  - `send_local`
  - `ThinkingDelta`
  - `ModelLoading`
  - `TextDelta`
  - `RunFinished`
- No errors have been reported.

## Operational issue

The current monitor wakes on high-volume benign log lines, creating noise without additional certification signal. This does not indicate model-runtime progress because no Local send or generation markers are present.

## Proposed monitor adjustment

If certification should continue, tighten the monitor to ignore benign `nw_path` / agent-generation noise and alert only on meaningful lane markers or errors:

```text
send_local
ModelLoading
ThinkingDelta
TextDelta
RunFinished
errors
```

This keeps the cert lane armed while avoiding wakeups until the operator taps **On-device → Local** and sends.

## Decision needed

Choose one path:

1. **Continue iOS certification**
   - Re-scope monitor to lane-markers-only.
   - Keep waiting for Local-tab send.
2. **Redirect / drop cert hold**
   - Stop holding on iOS cert monitoring.
   - Resume another workstream such as `M5b` pickers or `M7-react` macOS settings.

# Citations

1. stdin
2. manual:KnowMe/model-runtime-per-device