---
type: Reference
id: litert-lm-cancellation-wording-aligned-for-android-certification
title: LiteRT-LM cancellation wording aligned for Android certification
tags:
- model-runtime-per-device
- litert-lm
- android-certification
- cancellation
- gemma
- steer-bar
- tool-calls
links:
- ios-cert-monitor-holding-for-local-send-with-benign-nw-path-noise
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-26T17:39:51.284884+00:00
created_at: 2026-07-26T17:39:51.284884+00:00
updated_at: 2026-07-26T17:39:51.284884+00:00
revision: 0
---

## Phase context

- Phase: `model-runtime-per-device`
- Project: `KnowMe`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system`
- Captured: `2026-07-26T17:35:04Z`
- Position: `model-runtime-per-device › embedded-uar-offline-agents › litertlm-device-proof`
- Status: `executing`

## Phase goals

- Ship `Qwen3.5-9B Q4_K_M` with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM `Gemma 4 E2B`, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Current state

- Build still in progress; waiting on the monitor/APK rebuild before device certification.
- Progress marker: `changes 0/1`.
- APK rebuild started at `12:34` for the cancellation certification run.
- `gen_ui_protocol` passed after fixture update: `46/46`.

## Cancellation wording fix

A pre-device issue was found in the Android LiteRT-LM cancellation path:

- LiteRT-LM cancel path emitted: `Stopped by you`
- `chat.rs` `publish_cancelled` path emitted: `Cancelled by user`

Both strings represented the same user action, but the winning path depended on a race. This would have produced inconsistent UI/status text and a certification failure that could look like a product bug.

Decision/fix:

- Align LiteRT-LM cancellation wording to the existing `chat.rs` wording: `Cancelled by user`.
- Fix landed in commit `450de77`.
- `certify-local-cancellation.sh` asserts on `Cancelled by user`; the aligned wording keeps the certification script and UI status consistent.
- Verified `markRunCancelled` carries `event.message` into `statusMessage`, which the steer bar renders.

## Required verification

After the build lands, run Android local cancellation certification:

1. Run `certify-local-cancellation.sh`.
2. Certification must PASS.
3. `logcat` must show `RunCancelled`.
4. `logcat` must not show `generation failed`.
5. Manually stop mid-tool-call and verify:
   - the tool block survives cancellation;
   - the steer bar reports the kept count;
   - the visible cancellation text matches `Cancelled by user`.

## Open work

- Install and run the cancellation certification on device.
- Check both script verdict and logcat evidence.
- Verify steer bar behavior and preserved tool block on screen.
- Attachment-flow scripting remains open.
- iOS airplane-mode certification task `5.5` remains open alongside the iOS local inference certification lane, which has recently been monitored in [iOS cert monitor holding for Local send with benign nw_path noise](/ios-cert-monitor-holding-for-local-send-with-benign-nw-path-noise.md).

# Citations

1. stdin
2. manual:KnowMe/model-runtime-per-device