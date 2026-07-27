---
type: Reference
id: embedded-uar-session-memory-seeding-and-conversation-isolation
title: Embedded UAR session memory seeding and conversation isolation
tags:
- knowme
- uar
- embedded-session
- memory-system
- conversation-history
- session-isolation
- model-runtime-per-device
links:
- mobile-embedded-uar-provider-config-without-service-url
- mobile-embedded-uar-provider-config-and-parity
- three-tier-model-routing-and-response-provenance-in-knowme
sources:
- uar-embedded-session-memory-fix.md
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-25T21:46:57.509134+00:00
created_at: 2026-07-25T21:46:57.509134+00:00
updated_at: 2026-07-25T21:46:57.509134+00:00
revision: 0
---

## Context

- **Project:** KnowMe
- **Phase:** `model-runtime-per-device`
- **Captured:** `2026-07-25`
- **Status:** verified
- **Area:** UAR/Orchestrator embedded session memory

## Problem

The agent appeared to "forget basic things" stated earlier in the same thread. The failure was not context-window trimming: conversation history was not reliably reaching the model on the embedded UAR path.

The Local lane was already correct because it assembled `history + user_message` into each request.

## Root causes

Two independent defects existed on the UAR/Orchestrator lane:

1. **History was never sent from the host**
   - `send_uar` passed only the current message.
   - It relied on the embedded `SessionStore`, an in-process `HashMap<String, Session>`.
   - The embedded store accumulated turns only while the process stayed warm.
   - After a cold start, the session store was empty, so the model saw only the latest message even though the UI displayed the full conversation.
   - `UarRunRequest` had no `history` field, so there was no API seam for host-owned durable history.

2. **Conversations could share one session**
   - The session key used `config.session_id.or(conversation_id)`.
   - `config.session_id` came from a single global `uar_session_id` setting, inherited from the service-era contract.
   - When `uar_session_id` was set, `.or()` short-circuited and every conversation shared the same embedded session.
   - Result: context could bleed between unrelated threads.

This was related to the mobile embedded UAR work where embedded devices should not depend on service-era configuration such as a service URL or global session setting; see [Mobile embedded UAR provider config without service URL](/mobile-embedded-uar-provider-config-without-service-url.md).

## Decision

Separate durable history ownership from runtime session ownership:

- **Host owns durable history.** The app persists and supplies previous conversation turns.
- **Runtime owns live session state.** Embedded UAR keeps in-process session state while warm.
- **Seeding bridges the two.** Host-supplied history is replayed into an empty runtime session only when needed.

No persistence dependency was added to the runtime.

## Implementation delta

### UAR: seed messages and history-aware run APIs

ADR 0016 introduced:

```rust
SeedMessage {
    role,
    content,
    tool_call_id,
}
```

New APIs:

- `start_run_with_history`
- `start_run_with_policy_and_history`

Behavior:

- Host-supplied turns are replayed into the embedded session only when `message_count() == 0`.
- If the session is already warm, re-passing history is a no-op.
- This makes seeding idempotent and prevents duplicate context.
- Existing `start_run` delegates with an empty seed, preserving behavior for existing callers.

### KnowMe: pass history and isolate conversations

- `UarRunRequest` gained a `history` field.
- `send_uar` now receives the history it previously dropped and converts it into `SeedMessage` values.
- Embedded session keys are now based strictly on `conversation_id`.
- `uar_session_id` is honored only on the service path, where a caller-chosen remote session id remains a valid contract.

This aligns with the broader embedded/service split described in [Mobile embedded-UAR provider config (local⊕cloud) + cross-platform parity](/mobile-embedded-uar-provider-config-and-parity.md) and the model-routing/provenance work in [Three-tier model routing and response provenance in KnowMe](/three-tier-model-routing-and-response-provenance-in-knowme.md).

## Verification

Test coverage:

- `cold_started_session_is_seeded_from_supplied_history`
  - Asserts an empty session request contains seeded prior turns plus the current input.
  - Asserts a second turn on the warm session includes prior turns exactly once.
- Embedded suite: `9/9` passing.
- `gen_ui_agent`: `68/68` passing.
- iOS configuration compiles.

Operator device certification on a release build:

- Memory persists across force-quit/restart.
- A new conversation no longer knows facts from another thread.

## Lesson

When model memory appears broken, first inspect the actual request payload. LLM calls are stateless; conversation memory is exactly the message list sent to the model unless a runtime explicitly persists state.

## Future work

Consider persisting the embedded session store itself so warm-path history does not depend on host replay.

# Citations

1. uar-embedded-session-memory-fix.md
2. manual:KnowMe/model-runtime-per-device