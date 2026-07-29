---
type: Reference
id: karpathy-progress-20260726T232311Z-model-runtime-per-device
title: "Stopped-turn four intents + post-cancel engine rebuild"
tags:
- karpathy-progress
- model-runtime-per-device
- in-progress
sources:
- conversation:operator-agent
timestamp: 2026-07-26T23:23:11Z
created_at: 2026-07-26T23:23:11Z
updated_at: 2026-07-26T23:23:11Z
revision: 1
---

## Intent

Cancelling a LiteRT-LM generation poisoned native state that outlives the Kotlin Conversation wrapper, so the next turn threw 'CANCELLED: Task cancelled' with zero output. Engine exposes no reset (javap: only initialize/close/createConversation), and closing the conversation was already proven insufficient, so cancel() now flags a rebuild that the next send performs lazily. Also classified cancels on the blocking sendToolResults path, which had none and surfaced the raw JNI exception as terminal. Then shipped four explicit intents on a stopped turn: Steer and Resume primary (they keep the user's work), Retry and Discard subordinate. The intent rides in message content because InferenceMessage.name is dropped by the local prompt builder and an unknown role string degrades to user silently.

## Observed state and verification

Device SM-S936U Android 16, Local lane confirmed via the mode trigger. certify-local-second-send.sh: cycle 1 stopped mid-stream kept chars=718, log 'rebuilding LiteRT-LM engine after cancellation', cycle 2 chars=23. certify-local-steer-intents.sh: kept chars=632, all four intents plus the kept-work line present, RESUME chars=11607 with history tail 'user:124,assistant:632'. gen_ui_protocol 46/46; mobile chat suite 39/39; flutter analyze clean. Contrast 15.39:1 dark / 19.22:1 light. Commits 7706500, 3cb4128.

## Decision and lesson

Status: in-progress. Preserve evidence, distinguish compile proof from runtime proof, and do not narrow the active goal.

## Next experiment

Attachment-flow certification and iOS airplane-mode (task 5.5) remain open. Queue-while-running steering stays deferred pending a Rust mid-run injection point.
