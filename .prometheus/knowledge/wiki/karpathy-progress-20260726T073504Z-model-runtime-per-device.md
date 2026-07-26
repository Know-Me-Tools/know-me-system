---
type: Reference
id: karpathy-progress-20260726T073504Z-model-runtime-per-device
title: "Local mode certified on Samsung; restart persistence is unstarted milestone 3.1"
tags:
- karpathy-progress
- model-runtime-per-device
- in-progress
sources:
- conversation:operator-agent
timestamp: 2026-07-26T07:35:04Z
created_at: 2026-07-26T07:35:04Z
updated_at: 2026-07-26T07:35:04Z
revision: 1
---

## Intent

The bare-RunPolicy fix is certified on device. Selecting Local changes the trigger label and routes the send to the on-device Gemma model, bypassing the UAR entirely. Instrumented tracing then settled the remaining restart question definitively: the write is correct (policy saving chat_mode=local -> policy saved chat_mode=local, same conversation id), but after a force-stop the load of that same conversation id returns a policy with NO chat_mode. Cause is not a defect in this change: UarEmbeddedRuntime::build_local wires InMemoryProvider (RwLock<HashMap>) as the PersistenceLayer, with an explicit source comment that the next persistence milestone swaps it for the mobile durable adapter. Conversation policies therefore live in RAM and die with the process. This is openspec/changes/embed-uar-mobile-offline task 3.1 (inject durable sandboxed persistence), still unchecked — so restart survival was never implemented, not broken. Stopped patching per the debugging rule that repeated failed fixes signal architecture rather than a bug; the durable adapter is the real work and belongs to that milestone.

## Observed state and verification

Device SM-S936U R5GYB4AZD7A, release APK 03:31. Local selected: uiautomator content-desc=Local on the trigger; sheet exposes selected=true/false per row. Routing proof from logcat: 'LiteRT-LM native adapter loaded model=gemma-4-e2b-it-litertlm', 'LiteRT-LM generation starting', 'LiteRT-LM generation produced text chars=9 tool_calls=0', with NO 'Universal Agent Runtime run' line; reply rendered in-thread. Persistence proof: 'conversation policy saving chat_mode=local' then 'saved chat_mode=local' for c65da1c4-0649-4cd9-82e5-eff21a6a9c88; after force-stop, 'conversation policy loaded' for the SAME id with chat_mode absent while the thread's messages reloaded. Source: rust/crates/gen_ui_agent/src/uar.rs:878-882 InMemoryProvider; vendor/.../persistence/providers/memory.rs:30 RwLock<HashMap>; openspec/changes/embed-uar-mobile-offline/tasks.md:17 task 3.1 unchecked. cargo test -p gen_ui_agent --lib 72 passed. Diagnostic tracing reverted after use.

## Decision and lesson

Status: in-progress. Preserve evidence, distinguish compile proof from runtime proof, and do not narrow the active goal.

## Next experiment

Durable embedded persistence (embed-uar-mobile-offline task 3.1) to make per-conversation mode survive restart — swap InMemoryProvider for the sandboxed durable adapter at uar.rs:878. Also still outstanding: per-conversation isolation check across two threads within one session, and iOS device certification once the iPhone is reattached.
