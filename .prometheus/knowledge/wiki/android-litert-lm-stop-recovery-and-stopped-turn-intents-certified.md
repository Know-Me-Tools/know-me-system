---
type: Reference
id: android-litert-lm-stop-recovery-and-stopped-turn-intents-certified
title: Android LiteRT-LM stop recovery and stopped-turn intents certified
tags:
- knowme
- model-runtime-per-device
- litert-lm
- android-device-cert
- stop-recovery
- local-inference
- chat-ui
sources:
- stdin
timestamp: 2026-07-26T23:25:10.443819+00:00
created_at: 2026-07-26T23:25:10.443819+00:00
updated_at: 2026-07-26T23:25:10.443819+00:00
revision: 0
---

## Phase context

- Phase: `model-runtime-per-device`
- Project: `KnowMe`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system`
- Captured: `2026-07-26T23:24:00Z`
- Status: completed for the Android LiteRT-LM lane and current stopped-turn UX work

## Phase goals

- Ship `Qwen3.5-9B Q4_K_M` with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM `Gemma 4 E2B`, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Runtime blocker fixed

Commit: `7706500` in `rust/crates/gen_ui_inference/src/litert_lm.rs`.

Problem:

- Calling stop during LiteRT-LM generation poisoned the rest of the session.
- The next turn failed with `CANCELLED: Task cancelled` and produced zero output.
- `javap` showed LiteRT-LM `Engine` exposes no reset API.
- Closing the `Conversation` was already proven insufficient because cancellation state remained behind the native handle.

Resolution:

- `cancel()` now marks the engine as cancelled.
- The next send lazily rebuilds the engine, keeping the stop operation instant while restoring subsequent-turn correctness.

Secondary defect fixed:

- `sendToolResults` was a blocking path without cancel classification.
- Stopping a turn after a tool call emitted surfaced the raw JNI exception as a terminal error.
- The stop/cancel classification was fixed for this path as part of the same certification work.

## Stopped-turn UX intents

Commit: `3cb4128` in `mobile/lib/features/chat/presentation/screens/chat_screen.dart`.

Stopped turns now expose four intents:

- **Steer**: primary; preserves the user's partial work and lets the user guide continuation.
- **Resume**: primary; continues from the preserved partial response.
- **Retry**: subordinate; destructive in-place regeneration.
  - Label explicitly states that it "replaces this response and discards the work so far" because `regenerateResponse` discards the existing partial.
- **Discard**: subordinate; removes the stopped work.

Implementation constraint:

- The steering/resume intent must be carried in message content.
- `InferenceMessage.name` is dropped by the local prompt builder.
- A new role string silently degrades to `user` locally.
- Either alternative would work through Orchestrator but fail invisibly on Local, so content-level intent encoding is required for portable local/orchestrated behavior.

## Device certification evidence

Device: `SM-S936U` with Local lane confirmed.

Observed results:

- Stopped mid-stream while preserving **632 chars**.
- UI showed all four intents and the kept-work line.
- **Resume** generated **11,607 chars**, continuing from history ending `user:124, assistant:632`.
  - This verifies the replay behavior required by the design.
- Second send after stop rebuilt the engine and generated text in cycle 2.
- Rust protocol tests: `46/46` passing.
- Mobile chat tests: `39/39` passing.
- Static analysis: clean.
- Contrast checks:
  - Dark mode: `15.39:1`
  - Light mode: `19.22:1`

## Certification caveat: stale UI dumps caused false passes

Three certification checks initially passed falsely.

Root cause:

- `uiautomator dump` cannot snapshot reliably while the app is streaming.
- The previous `|| true` wrapper returned stale XML after dump failure.
- A run that actually finished uncancelled could report a pass because the stale dump still contained a Stop control.

Mitigation:

- `dump_ui` now fails loudly instead of returning stale XML.
- Any prior device metric captured during streaming should be rechecked if it depended on `uiautomator dump` output.

## Still open

- Attachment-flow certification.
- iOS airplane-mode certification task `5.5`.
- Queue-while-running steering remains deferred because Rust lacks the required mid-run injection point.

# Citations

1. stdin