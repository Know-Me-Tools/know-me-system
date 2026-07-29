---
type: Reference
id: android-litert-lm-stop-recovery-and-steer-intent-certification
title: Android LiteRT-LM stop recovery and steer intent certification
tags:
- model-runtime-per-device
- android-device-cert
- litert-lm
- local-inference
- cancellation
- steer-intents
- uiautomator
links:
- ios-cert-monitor-holding-for-local-send-with-benign-nw-path-noise
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-27T00:41:09.748809+00:00
created_at: 2026-07-27T00:41:09.748809+00:00
updated_at: 2026-07-27T00:41:09.748809+00:00
revision: 0
---

## Phase context

- Phase: `model-runtime-per-device`
- Project: `KnowMe`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system`
- Captured: `2026-07-26T23:24:44Z`
- Position: `model-runtime-per-device › embedded-uar-offline-agents › litertlm-device-proof`
- Status: `executing`

## Phase goals

- Ship `Qwen3.5-9B Q4_K_M` with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM `Gemma 4 E2B`, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Outcome

Android LiteRT-LM local inference certification is complete for the exercised stop/recovery and steering paths on physical device `SM-S936U` / `R5GYB4AZD7A`.

Verified evidence:

- Local lane confirmed.
- Mid-stream stop preserved partial assistant output: **632 chars** in steer-intents certification and **718 chars** in second-send certification.
- Stopped-turn action bar exposed all four intents:
  - **Steer**
  - **Resume**
  - **Retry**
  - **Discard**
- Resume continued from replayed history ending `user:124, assistant:632` and produced **11,607 chars**.
- Post-cancel second send rebuilt the engine and generated cycle-2 text.
- Test/analysis status:
  - `46/46` Rust protocol tests passed.
  - `39/39` mobile chat tests passed.
  - Analyze clean.
  - Contrast: `15.39:1` dark, `19.22:1` light.

## Stop/cancel engine fix

Commit `7706500` fixed a stop poisoning issue:

- Previous behavior: after stop, the next turn failed with `CANCELLED: Task cancelled` and emitted zero output.
- `javap` confirmed LiteRT-LM `Engine` exposes no reset API.
- Closing the `Conversation` was insufficient; cancellation state lived behind the native handle.
- New behavior:
  - `cancel()` marks the engine as cancelled.
  - Stop remains instant.
  - The next send lazily rebuilds the engine before generation.

A second cancellation defect was fixed at the same time:

- `sendToolResults` used a blocking path without cancel classification.
- Stopping a turn after a tool call surfaced the raw JNI exception as a terminal error.
- The path now classifies cancellation consistently.

## Stopped-turn intent UI

Commit `3cb4128` added four intents for stopped turns:

- Primary intents preserve user work:
  - **Steer**
  - **Resume**
- Subordinate intents:
  - **Retry**
  - **Discard**

`Retry` is labelled as replacing the response and discarding work so far because `regenerateResponse` is destructive in place: it throws away the partial assistant content that the rest of the stopped-turn bar preserves.

## Intent transport constraint

The stopped-turn intent must be encoded in message content for Local inference parity.

Rejected alternatives:

- `InferenceMessage.name`: dropped by the local prompt builder.
- New role string: silently degrades to `user`.

Either alternative could appear to work on Orchestrator while failing invisibly on Local, so intent metadata rides in message content instead.

## Certification scripts and instrumentation

Existing Android certification scripts under `scripts/android/` cover:

- Backend/memory/thermal checks.
- Cancellation.
- Markdown/Mermaid.
- Permitted tool call.

Load and throughput instrumentation landed on `2026-07-26` in commit `c92fc80`:

- `load_ms` measured on adapter load.
- `generate_ms`, `chars`, and `chars_per_second` measured on generation.
- Metrics harvested into `write-local-certification-report.sh`.

Two new physical-device certifications passed:

- `certify-local-second-send.sh`
  - Verifies post-cancel engine rebuild.
  - Observed kept chars: `718`.
  - Confirmed cycle-2 generation after rebuild.
- `certify-local-steer-intents.sh`
  - Verifies all four stopped-turn intents are present.
  - Observed kept chars: `632`.
  - `Resume` continued and reached `11,607` chars.

## False-pass issue in UI dumping

Three certification checks initially appeared green due to stale UI XML.

Root cause:

- `uiautomator dump` cannot snapshot while the app is streaming.
- The old `|| true` wrapper swallowed dump failures.
- Scripts then consumed stale XML from a previous run.
- A run that finished uncancelled could falsely pass because the stale dump still contained a `Stop` control.

Fix:

- `dump_ui` now fails loudly in the relevant scripts instead of returning stale XML.

Implication:

- Any earlier device certification number captured during streaming through a swallowed `uiautomator dump` failure should be re-checked.

## Remaining work

- Attachment-flow certification still has no script.
- iOS airplane-mode certification task `5.5` remains unrun; this belongs to the broader iOS local inference certification lane, related to prior iOS device-cert monitoring such as [iOS cert monitor holding for Local send with benign nw_path noise](/ios-cert-monitor-holding-for-local-send-with-benign-nw-path-noise.md).
- Queue-while-running steering remains deferred because Rust currently has no mid-run injection point.

# Citations

1. stdin
2. manual:KnowMe/model-runtime-per-device