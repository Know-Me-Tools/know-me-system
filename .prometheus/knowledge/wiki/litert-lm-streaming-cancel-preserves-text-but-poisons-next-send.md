---
type: Reference
id: litert-lm-streaming-cancel-preserves-text-but-poisons-next-send
title: LiteRT-LM streaming cancel preserves text but poisons next send
tags:
- model-runtime-per-device
- litert-lm
- android-local-inference
- streaming-cancel
- gemma
- native-session
- device-cert
links:
- ios-cert-monitor-holding-for-local-send-with-benign-nw-path-noise
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-26T23:24:22.720880+00:00
created_at: 2026-07-26T23:24:22.720880+00:00
updated_at: 2026-07-26T23:24:22.720880+00:00
revision: 0
---

## Phase context

- Phase: `model-runtime-per-device`
- Project: `KnowMe`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system`
- Captured: `2026-07-26T21:37:06Z`
- Position: `model-runtime-per-device › embedded-uar-offline-agents › litertlm-device-proof`
- Status: `executing`
- Device lane: Android local inference on `SM-S936U`

## Phase goals

- Ship `Qwen3.5-9B Q4_K_M` with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM `Gemma 4 E2B`, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Verified behavior

### Cycle 1: cancel now preserves partial streamed text

On the **Local** lane, a streaming generation cancel preserved partial text:

```text
16:35:11.241  cancellation requested
16:35:11.254  streaming stopped by user chars=1784
```

Observed result:

- `1,784` characters preserved.
- `359` incremental chunks emitted before stop.
- Interrupt latency: `13ms`.
- Previous behavior before this change: cancel preserved `0` characters.

### Cycle 2: next send fails after cancel

Sending again after a cancel fails with no streamed output:

```text
LiteRtLmJniException: Failed to call nativeSendMessage: CANCELLED: Task cancelled
```

Observed result:

- `0` chunks emitted.
- One stop works per app session.
- The next send is dead, making repeat use worse than the previous non-preserving cancel behavior.

## Root-cause finding

A suspected stale Kotlin `Conversation` wrapper was tested and rejected:

- Closing the `Conversation` in both code paths did **not** fix the cycle-2 failure.
- The build was rebuilt, reinstalled, and retested before rejecting the hypothesis.
- `javap` inspection of the AAR showed `Engine` exposes **no cancel or reset API**, only `createConversation`.
- Conclusion: `cancelProcess()` cancels task state in the native session below the Kotlin wrapper. A freshly created `Conversation` still shares that poisoned native state, so closing the wrapper cannot clear cancellation.

The attempted conversation-close fix was not committed because it does not resolve the defect.

## Testing caveat

An earlier `chars=0` cycle-1 reading was a testing error:

- The run was on the **Orchestrator** lane.
- The streaming path is **Local-only**.
- Lane confirmation is required before interpreting stream/cancel metrics.

## Ship decision

Do **not** ship the streaming path as-is.

Rationale: replacing "cancel preserves nothing" with "cancel works once, then poisons the next turn" is a net regression for real use.

## Recommended fix order

1. **Recreate the `Engine` after cancel**
   - `Engine` has no reset API, so close+reload may be the only way to clear native task state.
   - Earlier `load_ms` measurements were `243–291ms`, likely tolerable but must be measured after implementing.
2. **Check for a newer LiteRT-LM release**
   - Look for reset/cancel semantics that clear task state per conversation.
3. **Fallback to blocking `sendMessage` for the turn after cancel**
   - Preserves streaming for the common case while avoiding a poisoned next streamed send.

## Open follow-up work

- Try recreating `Engine` after cancel and measure reload cost.
- If reload is too slow, use blocking generation for the turn following a cancel.
- Attachment-flow scripting remains open.
- iOS airplane-mode certification remains open, continuing the iOS device-certification lane tracked in entries such as [iOS cert monitor holding for Local send with benign nw_path noise](/ios-cert-monitor-holding-for-local-send-with-benign-nw-path-noise.md).

# Citations

1. stdin
2. manual:KnowMe/model-runtime-per-device