---
type: Reference
id: karpathy-progress-20260725T151238Z-model-runtime-per-device
title: "Memory bug fix: UAR-lane conversation history + session isolation + chat paste"
tags:
- karpathy-progress
- model-runtime-per-device
- in_progress
sources:
- conversation:operator-agent
timestamp: 2026-07-25T15:12:38Z
created_at: 2026-07-25T15:12:38Z
updated_at: 2026-07-25T15:12:38Z
revision: 1
---

## Intent

Fixed the UAR/Orchestrator lane 'forgetting basic things'. Root cause: two defects. (1) send_uar (chat.rs) dropped the Flutter-supplied history entirely and relied on the embedded in-process, NON-DURABLE SessionStore (HashMap) — after a cold start the session was empty so the model only saw the current message (amnesia). (2) session_id used config.session_id.or(conversation_id), letting a global uar_session_id setting collapse all conversations into one shared session (cross-conversation context bleed). FIX.1 (UAR 7e3e09e/08a6f4c, ADR0016): SeedMessage + RunManager::start_run_with_history / start_run_with_policy_and_history seed an EMPTY session from host history before the current message (idempotent — warm sessions not re-seeded); Runtime::start_run_with_history on the SDK; SeedMessage re-exported from SDK root. FIX.2 (KnowMe b86f43b): UarRunRequest gains history: Vec<UarSeedMessage>; send_uar builds seed messages from the InferenceMessage history it now receives and passes them; embedded session keyed strictly by conversation_id (uar_session_id only honored on service path). Also fixed chat paste (a9ae966): shad.TextField defaulted contextMenuBuilder=null which suppressed the platform Paste/Copy toolbar — restored via TextField.nativeContextMenuBuilder() on the composer + 2 memory fields. Local lane already sent full history and was unaffected.

## Observed state and verification

UAR: cold_started_session_is_seeded_from_supplied_history passes (cold seed + no warm re-seed); full embedded lib suite 9 passed; cargo check server-full green; ADR0016 + OpenSpec seed-embedded-session-history validate --strict. KnowMe: gen_ui_agent lib 68/68; cargo check --target aarch64-apple-ios green; fmt clean. Pushed: UAR 46544c2..08a6f4c (Prometheus-AGS), KnowMe a9ae966..b86f43b (Know-Me-Tools).

## Decision and lesson

Status: in_progress. Preserve evidence, distinguish compile proof from runtime proof, and do not narrow the active goal.

## Next experiment

Build+deploy release to iPhone bundling paste fix + memory fix; operator verifies: (a) paste works in chat/memory inputs; (b) Orchestrator lane remembers earlier turns within a conversation AND after force-quit/reopen; (c) switching conversations no longer bleeds context. Then remaining plan: M7-react, knowledge-capture D.
