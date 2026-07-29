---
type: Reference
id: karpathy-progress-20260726T213642Z-model-runtime-per-device
title: "Android streaming preserves partial text; a second cancel is blocked by native session state"
tags:
- karpathy-progress
- model-runtime-per-device
- in-progress
sources:
- conversation:operator-agent
timestamp: 2026-07-26T21:36:42Z
created_at: 2026-07-26T21:36:42Z
updated_at: 2026-07-26T21:36:42Z
revision: 1
---

## Intent

Streaming via Conversation.sendMessageAsync is device-verified: partial text now survives a cancel on the Local lane. Cycle 1 on SM-S936U preserved chars=1784 across 359 incremental chunks with a 13ms interrupt, versus zero preserved before the change. Cycle 2 — sending again after a cancel — FAILS with LiteRtLmJniException 'Failed to call nativeSendMessage: CANCELLED: Task cancelled' and zero chunks. My first hypothesis (a stale Kotlin Conversation object) was WRONG: closing the previous conversation in both startStream and the sendMessage fallback did not fix it, and javap shows Engine exposes no cancel or reset API at all, only createConversation. So cancelProcess() cancels state in the native session BELOW the Kotlin wrapper, which a freshly created Conversation still shares — closing the wrapper cannot clear it. Verified through a full rebuild and reinstall, not inferred. Also corrected a testing error of mine: an earlier cycle-1 attempt showed chars=0 because the conversation was on the Orchestrator lane, not Local; the streaming path is Local-only, so the lane must be set before any of this is meaningful.

## Observed state and verification

Device SM-S936U, APK built 16:31 including the conversation-close attempt. Cycle 1 (Local lane confirmed via content-desc=Local on the mode trigger): 16:35:11.241 cancellation requested -> 16:35:11.254 'streaming stopped by user chars=1784', 359 'produced text' events. Cycle 2: 16:35:57.827 cancellation requested -> 16:35:57.855 System.err LiteRtLmJniException 'Failed to call nativeSendMessage: CANCELLED: Task cancelled', 0 chunks, and a preceding 'generation timing generate_ms=12417 chars=0'. javap on litertlm-android-0.14.0 Engine: only createConversation, no cancel/reset. Earlier mis-run: mode trigger read 'Orchestrator' while 'Stop generating' was still present, so the observed chars=0 was a UAR-lane run, not a LiteRT-LM failure.

## Decision and lesson

Status: in-progress. Preserve evidence, distinguish compile proof from runtime proof, and do not narrow the active goal.

## Next experiment

Fix the second-cancel path. Options in order of preference: (a) recreate the Engine (close + reload) after a cancel, since Engine has no reset and the native cancellation appears to live at or below session scope — costs a model reload, so measure load_ms impact; (b) check whether litertlm 0.14.x has a newer release that resets task state on new conversations; (c) fall back to blocking sendMessage for the turn immediately after a cancel. Until fixed, one stop per app session works and the next send fails, which is worse than the pre-streaming behaviour for repeat use — do not ship the streaming path without resolving this.
