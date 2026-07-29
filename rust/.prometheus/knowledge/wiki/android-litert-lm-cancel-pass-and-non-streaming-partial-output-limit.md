---
type: Reference
id: android-litert-lm-cancel-pass-and-non-streaming-partial-output-limit
title: Android LiteRT-LM cancel pass and non-streaming partial-output limit
tags:
- knowme
- android
- litert-lm
- local-inference
- cancellation
- uar
- device-certification
- model-runtime-per-device
links:
- android-litert-lm-local-tab-fix-and-remaining-device-proof-gap
- knowme-mode-picker-fixes-and-unresolved-device-certification
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-26T21:10:15.284560+00:00
created_at: 2026-07-26T21:10:15.284560+00:00
updated_at: 2026-07-26T21:10:15.284560+00:00
revision: 0
---

## Context

- **Project:** KnowMe
- **Phase:** `model-runtime-per-device`
- **KBD root:** `/Users/gqadonis/Projects/know-me/know-me-system`
- **Captured:** `2026-07-26T17:49:13Z`
- **Position:** `model-runtime-per-device › embedded-uar-offline-agents › litertlm-device-proof`
- **Status:** `executing`
- **Device under test:** Samsung Android device / `SM-S936U`
- **Runtime lane:** Android local text-agent inference through LiteRT-LM Gemma 4 E2B, CPU baseline

This follows the Android LiteRT-LM proof work in [Android LiteRT-LM Local tab fix and remaining device proof gap](/android-litert-lm-local-tab-fix-and-remaining-device-proof-gap.md) and the broader mode/runtime certification tracked in [KnowMe mode picker fixes and unresolved device certification](/knowme-mode-picker-fixes-and-unresolved-device-certification.md).

## Phase goals

- Ship Qwen3.5-9B Q4_K_M with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM Gemma 4 E2B with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Certification result

**Android cancellation fix: PASS, with a non-streaming limitation.**

Device logs provide the decisive evidence:

```text
12:40:58.584  LiteRT-LM generation starting
12:41:03.407  LiteRT-LM generation cancellation requested
12:41:03.423  LiteRT-LM generation cancelled by user
```

Observed results:

- `generation failed` appeared **zero** times.
- Cancellation reaches the engine and is classified as user cancellation, not a terminal generation error.
- Interrupt latency was **5–16 ms** across two runs.
- Previous behavior in the same scenario logged `error=terminal ... Java exception was thrown` and continued to complete a `1506` character run after the stop tap.
- The steer bar renders correctly with all three accessible names present in the UI dump:
  - `Steer`
  - `Retry`
  - `Discard`

## Important limitation: Android LiteRT-LM does not preserve partial answer text

A harder manual test cancelled a long generation after approximately 12 seconds, after the model had clearly been running. Result:

- UI showed `Cancelled by user`.
- No kept-count was reported.
- The cancelled turn contained **zero content blocks**.

This is expected for the current Android local lane and is not a UI bug.

Root cause:

- LiteRT-LM is currently used as a **non-streaming** engine.
- `TextDelta` is pushed exactly once at `rust/crates/gen_ui_inference/src/litert_lm.rs:421`.
- That `TextDelta` is emitted only after the single blocking `generate_text` call returns.
- If cancellation occurs before `generate_text` returns, the error arm runs instead.
- Therefore no text crosses the JNI boundary before cancellation, leaving nothing to preserve.
- The missing `generation timing` line in the long-cancel run corroborates that generation did not complete normally.

Corrected conclusion:

- Partial content preservation is valid in the Flutter/protocol layers and for streaming engines such as MLX.
- It is **not** valid for partial answer text in the current Android LiteRT-LM local lane.
- The existing zero-block branch is correct because it avoids claiming kept work that never reached downstream layers.

## Fix status

| Area | Status | Notes |
|---|---:|---|
| Cancel-vs-error classification | ✅ Device-proven | `cancellation requested` followed by `cancelled by user`; `generation failed` count is `0`. |
| Partial tool-call preservation | ✅ Implemented and unit-tested | Not exercised on Android because the current LiteRT-LM path is non-streaming. |
| Steer bar | ✅ Device-proven | Renders with accessible `Steer`, `Retry`, and `Discard` names. |
| Partial answer text preservation on Android | ⚠️ Not supported currently | Requires LiteRT-LM incremental streaming/callback API support. |

## Decision needed

Choose one Android-lane policy:

1. **Accept and document the limitation**
   - Android LiteRT-LM preserves thinking/tool blocks that reach downstream layers.
   - Android LiteRT-LM does not preserve partial answer text when cancellation occurs before blocking `generate_text` returns.

2. **Implement incremental Android streaming**
   - Adopt LiteRT-LM async/callback generation if the AAR exposes it.
   - Downstream Flutter/protocol/tool-call machinery is already implemented and tested.
   - Remaining gap is only the engine emission granularity.

## Remaining open work

- Decide whether to pursue LiteRT-LM async/callback streaming or document the non-streaming limitation as accepted.
- Attachment-flow scripting remains open.
- iOS airplane-mode certification task `5.5` remains open.

# Citations

1. stdin
2. manual:KnowMe/model-runtime-per-device