---
type: Reference
id: mlx-cancel-verification-rerun-after-premature-stop-test-failure
title: MLX cancel verification rerun after premature stop-test failure
tags:
- model-runtime-per-device
- mlx
- ios-device
- local-inference
- cancel-generation
- qwen
- rust-tests
links:
- memory-fix-install-watch-armed-for-unplug-safe-ios-verification
- m5b-install-monitor-armed-for-model-runtime-picker-verification
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-27T01:03:52.783293+00:00
created_at: 2026-07-27T01:03:52.783293+00:00
updated_at: 2026-07-27T01:03:52.783293+00:00
revision: 0
---

## Phase context

- Phase: `model-runtime-per-device`
- Project: `KnowMe`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system`
- Captured: `2026-07-27T00:46:13Z`
- Position: `model-runtime-per-device › embedded-uar-offline-agents › litertlm-device-proof`
- Status: `executing`

## Phase goals

- Ship `Qwen3.5-9B Q4_K_M` with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM `Gemma 4 E2B`, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## iPhone deployment and gate status

The app was built, signed with team `48VVJ9AXAR`, installed, and launched on an iPhone 17 Pro Max.

Runtime gates 1–5 passed:

```text
backend: mlx-metal
modelId: qwen3.5-4b-mlx-4bit
```

This continues the iOS local inference certification/runtime path previously tracked in entries such as [Memory-fix install watch armed for unplug-safe iOS verification](/memory-fix-install-watch-armed-for-unplug-safe-ios-verification.md) and [M5b install monitor armed for model runtime picker verification](/m5b-install-monitor-armed-for-model-runtime-picker-verification.md).

## Failed verification run

The first verification run failed because the test stopped too early, before any streamed text existed:

```text
KnowMe intents: stopping mid-stream with 1 blocks / 0 chars
Expected: a value greater than <0>  Actual: <0>
the stop discarded every character the model had produced
```

Root cause:

- The test wait condition was `content.isNotEmpty`.
- That condition became true on the first **thinking** block.
- The test then stopped generation before any `TextBlock` existed.
- The assertion that partial text should be preserved was valid, but the precondition was invalid.

Fix:

- The wait condition now requires at least `80` streamed text characters.
- `_charCount` counts only `TextBlock`s, so thinking blocks cannot satisfy the stop precondition.

## MLX cancel evidence status

Earlier wording implied that the MLX `cancel_generation` override had been validated on device. That was incorrect: the failed run never reached the cancel path.

Current evidence:

- Rust tests prove that the cancel wiring calls the bridge.
- There is not yet device evidence that a real MLX generation stops and preserves partial text.
- The active rerun must verify MLX cancellation behavior on the iPhone.

Required rerun evidence:

- Mid-stream MLX stop preserves already-streamed text.
- Four intents are visible.
- Resume continues after cancellation.

If the rerun still preserves zero text, investigate whether MLX buffers tokens internally instead of emitting deltas as they arrive.

## Commit `21e5602`

Committed changes:

- MLX cancel override.
- Shared `Cancelled by user` wording.
- Two Rust tests.
- Mock bridge now counts `cancel` calls.

Rationale for the mock bridge call count:

- The trait default is a no-op.
- A call count is the only signal distinguishing a correctly wired cancel call from a silently ignored no-op default.

Test result:

```text
26/26 pass with --features local-mlx
```

## Persisted template exception was not a live regression

An older turn in the conversation contained:

```text
TemplateException("System message must be at the beginning.")
```

This was determined to be a persisted record from before commit `82d0c69` on `2026-07-26 04:14`, not a current regression.

Evidence:

- Later turns in the same conversation succeeded with `914` and `744` characters.
- `hoist_system_messages` remains wired at `gen_ui_context/src/lib.rs:876`.

## Open work

- Wait for and report the iPhone rerun result.
- Attachment-flow certification remains open.
- iOS airplane-mode task `5.5` remains open.
- Queue-while-running steering remains deferred until a Rust mid-run injection point exists.

# Citations

1. stdin
2. manual:KnowMe/model-runtime-per-device