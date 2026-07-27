---
type: Reference
id: karpathy-progress-20260725T213147Z-model-runtime-per-device
title: "Memory fix CERTIFIED on device: cross-restart memory + no cross-conversation bleed"
tags:
- karpathy-progress
- model-runtime-per-device
- verified
sources:
- conversation:operator-agent
timestamp: 2026-07-25T21:31:47Z
created_at: 2026-07-25T21:31:47Z
updated_at: 2026-07-25T21:31:47Z
revision: 1
---

## Intent

Operator verified on iPhone 17 Pro Max (release build, detached): the UAR/Orchestrator lane now remembers earlier turns ACROSS an app force-quit/restart, and separate conversations no longer bleed context into each other. This certifies both defects fixed: (1) send_uar previously dropped the host-supplied history entirely and relied on the non-durable in-process SessionStore, so a cold start meant the model saw only the current message; now UarRunRequest.history carries the thread and RunManager::start_run_with_history seeds an EMPTY session from it (idempotent — warm sessions never re-seeded). (2) config.session_id.or(conversation_id) let a global uar_session_id collapse all conversations into one shared session; the embedded path now keys strictly by conversation_id. Chat paste fix shipped in the same build.

## Observed state and verification

Device verification by operator on release build installed from commit b86f43b (UAR submodule 08a6f4c): memory persists across restart; new conversation does not know the other thread's facts. Pre-deploy: UAR cold_started_session_is_seeded_from_supplied_history + embedded suite 9/9; gen_ui_agent 68/68; iOS cfg check; fmt; ADR0016 + OpenSpec seed-embedded-session-history validate --strict.

## Decision and lesson

Status: verified. Preserve evidence, distinguish compile proof from runtime proof, and do not narrow the active goal.

## Next experiment

M7-react (#41): macOS React settings drop the UAR service-URL form when the runtime is embedded. Then knowledge-capture D (#29): 7 karpathy Reference records + pk ingest + prometheus/pk lint gate.
