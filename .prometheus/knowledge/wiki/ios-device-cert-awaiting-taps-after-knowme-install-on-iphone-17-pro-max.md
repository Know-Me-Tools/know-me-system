---
type: Reference
id: ios-device-cert-awaiting-taps-after-knowme-install-on-iphone-17-pro-max
title: iOS device cert awaiting taps after KnowMe install on iPhone 17 Pro Max
tags:
- model-runtime-per-device
- ios-device-cert
- mlx
- think-split
- embedded-uar
- flutter-release
- device-logs
links:
- ios-local-mlx-think-split-certification-plan-for-knowme
- m5a-embedded-uar-policy-guidance-after-fastembed-fix
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-24T21:15:38.939276+00:00
created_at: 2026-07-24T21:15:38.939276+00:00
updated_at: 2026-07-24T21:15:38.939276+00:00
revision: 0
---

## Phase context

- Phase: `model-runtime-per-device`
- Project: `KnowMe`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system`
- Captured: `2026-07-24T21:05:10Z`
- Position: `model-runtime-per-device › ios-device-cert › awaiting_taps`
- Status: `app_installed`
- Device: Travis's iPhone 17 Pro Max

## Phase goals

- Ship `Qwen3.5-9B Q4_K_M` with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM `Gemma 4 E2B`, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Current state

KnowMe has been signed, installed, and launched on the iPhone 17 Pro Max.

- Install/launch result: `Installing and launching... 5.2s`.
- Build contents:
  - iOS think-split fix.
  - M1–M7 embedded runtime work.
  - Updated `liter-llm` integration.
  - Expanded provider catalog, approximately 297 providers.
  - Pinned **On-device** provider.
- Device-log cert monitor is armed: `bd8ijfg47`.
- Previous build context is tracked in [iOS local MLX think-split certification plan for KnowMe](/ios-local-mlx-think-split-certification-plan-for-knowme.md).

## Operator validation sequence

### Step 0 — Validate Flutter install

1. Unlock the iPhone and open **KnowMe**.
2. Confirm it reaches the home screen with no crash.
3. Tap **Chat**.
   - This validates embedded-UAR boot with the new `liter-llm` integration and expanded provider catalog.
4. Optional: open **Settings → Model providers**.
   - Expected: expanded catalog around 297 providers, including OpenCode Zen/Go, plus pinned **On-device** provider.

### Steps 1–4 — iOS think-split cert

1. In **Chat → On-device**, tap the **Local** tab.
   - Important: iOS defaults to UAR; tapping **Local** selects the direct-MLX local lane.
2. Send: `Say hello in one short sentence.`
   - Model is already downloaded, so response should be fast.
3. Pass criteria:
   - Reasoning appears in a separate collapsible **Thinking** block.
   - Reasoning is not mixed into the final answer.
   - No bare `</think>` tag appears.
   - Final answer appears as clean content, e.g. a short `Hello...` sentence.
4. Report result or provide screenshot.

## Log-based certification criteria

Cert monitor should observe the direct local MLX path and clean split-stream events:

```text
send_local
MLX load
ThinkingDelta
TextDelta
RunFinished
```

Expected interpretation:

- `send_local`: confirms the Local tab/direct path was used.
- `MLX load`: confirms local MLX runtime engaged.
- `ThinkingDelta`: confirms reasoning was split into a thinking stream.
- Clean `TextDelta`: confirms answer text excludes raw think tags.
- `RunFinished`: confirms the run completed normally.

## Optional follow-up

After local think-split cert passes, tap **Orchestrator** and send a prompt to certify the embedded-UAR lane. The embedded-UAR policy/runtime work is related to the implementation tracked in [M5a embedded UAR policy guidance after fastembed fix](/m5a-embedded-uar-policy-guidance-after-fastembed-fix.md).

## Closure target

Completing the install validation and iOS think-split cert closes P1 `#19` / task B `#27` for this phase.

# Citations

1. stdin
2. manual:KnowMe/model-runtime-per-device