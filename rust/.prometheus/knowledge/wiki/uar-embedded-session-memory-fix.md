---
type: Reference
id: uar-embedded-session-memory-fix
title: Embedded session memory — seeding history and isolating conversations
tags:
- knowme
- model-runtime-per-device
- uar
- embedded
- session
- memory
- context
links:
- mobile-embedded-uar-provider-config
- three-tier-model-selection-and-provenance
- ios-device-certification-local-and-embedded-uar
sources:
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-25T21:44:00+00:00
created_at: 2026-07-25T21:44:00+00:00
updated_at: 2026-07-25T21:44:00+00:00
revision: 0
---

## Context

- **Project:** KnowMe
- **Phase:** `model-runtime-per-device`
- **Captured:** `2026-07-25`
- **Status:** `verified`

## Intent

The agent was "forgetting basic things" — losing facts stated earlier in the same
thread. Establish whether conversation history actually reaches the model.

## Observed state

Two independent defects, both on the UAR/Orchestrator lane. (The Local lane was
already correct: it assembles `history + user_message` into the request.)

1. **History was never sent.** `send_uar` passed only the current message and
   relied on the embedded `SessionStore` — an in-process
   `HashMap<String, Session>` that is **not durable**. Within one process the
   session accumulates turns correctly, but after a cold start it is empty, so
   the model saw only the latest message while the UI still displayed the full
   thread. `UarRunRequest` had no history field at all, so there was no seam.
2. **Conversations could collide.** The session key was
   `config.session_id.or(conversation_id)`, and `config.session_id` came from a
   single global `uar_session_id` setting — a service-era concept. Whenever it
   held a value, `.or()` short-circuited and *every* conversation shared one
   session, bleeding context between threads.

## Decision and delta

- **UAR (ADR 0016):** a `SeedMessage { role, content, tool_call_id }` type and
  `start_run_with_history` / `start_run_with_policy_and_history`, which replay
  host-supplied turns into the session **only when it is empty**
  (`message_count() == 0`). Seeding only an empty session makes the call
  idempotent — a warm session already holds its turns, so re-passing history is a
  no-op and never duplicates context. Existing `start_run` delegates with an
  empty seed, so no caller changed behavior.
- **KnowMe:** `UarRunRequest` gains `history`; `send_uar` now receives the
  history it previously dropped and converts it to seed messages. The embedded
  session is keyed **strictly** by `conversation_id`; `uar_session_id` is honored
  only on the service path, where a caller-chosen session id is a legitimate
  contract with a remote runtime.

The division of responsibility is the point: the **host** owns durable history,
the **runtime** owns live session state, and the seed is the one-way bridge from
the former into an empty instance of the latter. No persistence dependency was
added to the runtime.

## Verification

`cold_started_session_is_seeded_from_supplied_history` asserts both halves: an
empty session's request contains the seeded turns plus the current input, and a
second turn on the warm session shows the prior turns exactly once. Embedded
suite 9/9; `gen_ui_agent` 68/68; iOS cfg compiles.

Device-certified by the operator on a release build: memory now persists across a
force-quit/restart, and a new conversation does not know the other thread's facts.

## Failure/lesson

"The model forgets" was assumed to be a context-window or trimming problem. It
was neither — the history simply never left the app. When memory looks broken,
first prove what the request actually contains; an LLM is stateless per call, so
"memory" is exactly and only the message list you send.

## Next experiment

Consider persisting the embedded session store itself, so warm-path history does
not depend on host replay at all.
