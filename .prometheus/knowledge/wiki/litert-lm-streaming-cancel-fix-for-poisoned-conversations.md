---
type: Reference
id: litert-lm-streaming-cancel-fix-for-poisoned-conversations
title: LiteRT-LM streaming cancel fix for poisoned conversations
tags:
- model-runtime-per-device
- litert-lm
- android-inference
- streaming
- cancellation
- conversation-lifecycle
- gemma
links:
- ios-cert-monitor-holding-for-local-send-with-benign-nw-path-noise
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-26T21:37:25.802519+00:00
created_at: 2026-07-26T21:37:25.802519+00:00
updated_at: 2026-07-26T21:37:25.802519+00:00
revision: 0
---

## Phase context

- Phase: `model-runtime-per-device`
- Project: `KnowMe`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system`
- Captured: `2026-07-26T21:31:23Z`
- Position: `model-runtime-per-device › embedded-uar-offline-agents › litertlm-device-proof`
- Status: `executing`
- Android target: local text-agent inference through LiteRT-LM `Gemma 4 E2B`, CPU as universal correctness baseline
- iOS target remains `Qwen3.5-9B Q4_K_M` with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.

## Verified behavior

- Streaming output is working on SM-S936U after deploy `a295896`.
  - Observed **1,177 incremental chunks** for one response.
  - Chunk cadence: approximately `30–60ms` apart.
  - Chunk size: approximately `3–5` characters.
  - Previous behavior was exactly one event containing the entire response.
- Partial text survives cancellation once generation has produced tokens.
  - Confirmed earlier at `chars=2127`.
  - UI showed preserved essay text above: `Cancelled by user — kept 1 block, still in context`.

## Bug found by deploy

A repeated cancel/send flow exposed a latent LiteRT-LM conversation lifecycle bug missed by a single-cancel test:

```text
LiteRtLmJniException: Failed to call nativeSendMessage: CANCELLED: Task cancelled
```

Root cause:

- `cancelProcess()` leaves the current LiteRT-LM conversation permanently cancelled.
- `startStream` created a new conversation without closing the old one.
- Subsequent sends immediately failed because the previous cancelled conversation poisoned following turns.
- The same latent leak existed in the `sendMessage` fallback path.

Fix applied:

- Close the previous conversation before creating a new one in `startStream`.
- Apply the same close-before-create behavior in the `sendMessage` fallback.
- Rebuild is in progress after the fix.

## Cancellation caveats

- Cancelling during prefill preserves no partial text.
  - Long-history threads can spend seconds in prefill before the first token exists.
  - If cancellation happens before token emission, there is no text to preserve; this is an engine/runtime constraint, not a UI preservation failure.
- `certify-local-cancellation.sh` has not yet been re-run against the fix.
  - Its current fixture cancels during the prefill window on a long-history thread.
  - The script therefore reports `FAIL` for a legitimate reason.
  - The assertion was intentionally left unchanged rather than weakened to force a pass.
  - Required follow-up: seed the certification fixture with a short-history thread so cancellation lands after prefill and exercises the preservation path.

## Next verification steps

1. Install the rebuilt APK.
2. Verify two consecutive cancel-then-send cycles:
   - each cancellation preserves already-streamed text;
   - neither cancellation poisons the following turn;
   - the next send starts cleanly without `CANCELLED: Task cancelled`.
3. Decide whether to reseed `certify-local-cancellation.sh` with a short-history fixture.
4. Attachment-flow scripting remains open.
5. iOS airplane-mode certification remains open and relates to the existing local-send certification lane, e.g. [iOS cert monitor holding for Local send with benign nw_path noise](/ios-cert-monitor-holding-for-local-send-with-benign-nw-path-noise.md).

# Citations

1. stdin
2. manual:KnowMe/model-runtime-per-device