---
type: Reference
id: ios-local-mlx-think-split-certification-plan-for-knowme
title: iOS local MLX think-split certification plan for KnowMe
tags:
- model-runtime-per-device
- ios-device-build
- mlx
- think-split
- embedded-uar
- device-certification
- flutter-release
links:
- m5a-embedded-uar-policy-guidance-after-fastembed-fix
sources:
- stdin
timestamp: 2026-07-24T21:05:31.857118+00:00
created_at: 2026-07-24T21:05:31.857118+00:00
updated_at: 2026-07-24T21:05:31.857118+00:00
revision: 0
---

## Phase context

- Phase: `model-runtime-per-device`
- Project: `KnowMe`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system`
- Captured: `2026-07-24T20:56:58Z`
- Position: `model-runtime-per-device › ios-device-build › building`
- Status: `build_in_flight`
- Branch: `30e9f41`
- Device: iPhone 17 Pro Max `3CF53A89`
- Build monitor: `bql3g52zz`

## Phase goals

- Ship `Qwen3.5-9B Q4_K_M` with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM `Gemma 4 E2B`, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Current build state

A signed iOS release build was launched with `flutter run --release` and is compiling/installing to the connected iPhone 17 Pro Max.

Included changes:

- iOS think-split fix.
- M1–M7 embedded runtime work.
- `liter-llm` update.
- Expanded provider catalog update with approximately 297 providers.
- Pinned **On-device** provider.

Immediate objective: certify iOS direct-MLX local lane think-split behavior on device, then optionally certify the embedded-UAR orchestrator lane. Embedded-UAR policy/runtime work relates to the earlier [M5a embedded UAR policy guidance after fastembed fix](/m5a-embedded-uar-policy-guidance-after-fastembed-fix.md).

## Step 0: validate Flutter install before certification

After installation and launch complete, the operator must unlock the iPhone and verify:

1. **KnowMe launches** and reaches the home screen without boot crash.
2. **Chat** tab opens.
   - This validates FFI and embedded-UAR boot with the new `liter-llm` and expanded provider catalog.
3. Optional sanity check: **Settings → Model providers** shows:
   - Expanded catalog, including OpenCode Zen/Go among approximately 297 providers.
   - Pinned **On-device** provider.

If any validation fails, stop certification and diagnose from device logs.

## iOS think-split certification procedure

1. Navigate to **Chat → On-device** and tap the **Local** tab.
   - This is critical because iOS defaults to the UAR lane.
   - The Local tab selects the direct-MLX lane: `active_lane='local'`.
2. Send exactly:

   ```text
   Say hello in one short sentence.
   ```

3. Observe the response.

### Pass criteria

The certification passes when:

- Reasoning renders as a separate, collapsible **Thinking** block.
- Reasoning is not mixed into the answer bubble.
- No bare `</think>` tag is visible.
- The final answer, e.g. “Hello…”, appears as clean response content below or after the thinking block.

### Log-side certification signals

The device-log monitor should be armed with `idevicesyslog` once the app is installed. Expected sequence:

1. `send_local` phase sequence.
2. MLX model load.
3. `ThinkingDelta`, proving reasoning stream split.
4. Clean `TextDelta`.
5. `RunFinished`.

## Optional embedded-UAR lane certification

After the direct-MLX Local lane passes:

1. Tap **Orchestrator**.
2. Send another prompt.
3. Confirm the embedded kernel drives the MLX model through the full agent loop.

This optional check validates the embedded-UAR lane separately from the direct local lane.

## Next actions

- Wait for signed release build to land on device.
- Arm `idevicesyslog` certification monitor.
- Operator validates install using Step 0.
- Operator runs Local lane think-split certification.
- Certifier confirms pass/fail from both UI observation and device logs.
- Optionally run Orchestrator/embedded-UAR certification.

# Citations

1. stdin