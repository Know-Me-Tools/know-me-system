---
type: Reference
id: ios-mlx-cancellation-fix-verified-resume-test-harness-tap-fixed
title: iOS MLX cancellation fix verified; resume test harness tap fixed
tags:
- model-runtime-per-device
- ios-mlx
- cancellation
- flutter-tests
- resume-flow
- device-certification
- qwen
links:
- memory-fix-install-watch-armed-for-unplug-safe-ios-verification
- ios-cert-monitor-holding-for-local-send-with-benign-nw-path-noise
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-27T01:11:40.244052+00:00
created_at: 2026-07-27T01:11:40.244052+00:00
updated_at: 2026-07-27T01:11:40.244052+00:00
revision: 0
---

## Phase context

- Phase: `model-runtime-per-device`
- Project: `KnowMe`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system`
- Captured: `2026-07-27T01:03:27Z`
- Position: `model-runtime-per-device › embedded-uar-offline-agents › litertlm-device-proof`
- Status: `executing`
- Device lane: iPhone 17 Pro Max local inference/runtime verification

## Phase goals

- Ship `Qwen3.5-9B Q4_K_M` with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM `Gemma 4 E2B`, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Confirmed on iPhone

The iOS MLX cancellation path is confirmed working on device.

Observed cancelled generation record:

```text
{role: assistant, status: cancelled, statusMessage: Cancelled by user, blocks: 2, chars: 139}
```

Key result:

- A mid-stream stop reached a real MLX generation.
- The generation settled as `cancelled` with the shared cancellation wording.
- Partial output was preserved: `139` characters.
- Before the override fix, the stop did not reach the engine.
- Runtime gates 1–5 passed with:
  - `backend: mlx-metal`
  - `modelId: qwen3.5-4b-mlx-4bit`

Also confirmed on device: all four post-cancel intents rendered with the reassurance line that content is still in context:

- Steer
- Resume
- Retry
- Discard

This continues the iOS device runtime certification path after earlier install and local-send monitoring work such as [Memory-fix install watch armed for unplug-safe iOS verification](/memory-fix-install-watch-armed-for-unplug-safe-ios-verification.md) and [iOS cert monitor holding for Local send with benign nw_path noise](/ios-cert-monitor-holding-for-local-send-with-benign-nw-path-noise.md).

## Test failures diagnosed

### Run 2: stopped on thinking block, not text

Run 2 stopped at `1 blocks / 0 chars` because the harness waited on the first streamed *thinking* block instead of streamed text.

Fix applied:

- Require at least `80` streamed text characters before triggering the stop.

### Run 3: Resume button present but not hittable

Run 3 timed out after 10 minutes at Resume. The transcript still had only 2 messages, meaning no resume turn was created.

Diagnostic log:

```text
derived an Offset (Offset(242.7, 712.0)) that would not hit test on the specified widget
```

Root cause:

- `Resume` existed in the widget tree but was outside the viewport.
- `tap` found the widget and emitted a warning, but did not perform a real tap.
- Earlier `find.text` assertions for Steer/Resume/Retry/Discard passed because they only proved tree presence, not reachability.

Harness fixes applied:

- Call `ensureVisible` before tapping `Resume`.
- Call `ensureVisible` for each of the four action buttons so presence-without-reachability cannot pass.
- Set `hitTestWarningShouldBeFatal = true` so missed taps fail immediately instead of surfacing later as unrelated timeouts.

This is the same failure class as the Android `uiautomator` stale-dump issue: the harness reported success while doing nothing.

## Dismissed non-regression

A persisted older turn in the conversation contains:

```text
TemplateException("System message must be at the beginning.")
```

This is not considered a live regression because:

- Later turns in the same thread succeeded with `914` and `744` characters.
- The relevant fix, commit `82d0c69`, predates those later successful turns.

## Current next steps

- Report Run 4 result; expectation is that `Resume` creates a continuation turn and the end-to-end test passes.
- If `Resume` still produces no output after a verified hittable tap, investigate whether `sendResume` reaches Rust on the MLX lane.
- Remaining open items:
  - Attachment-flow certification.
  - iOS airplane-mode task `5.5`.
  - Queue-while-running steering, deferred until a Rust mid-run injection point exists.

# Citations

1. stdin
2. manual:KnowMe/model-runtime-per-device