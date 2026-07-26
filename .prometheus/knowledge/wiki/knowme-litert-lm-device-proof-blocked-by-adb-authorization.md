---
type: Reference
id: knowme-litert-lm-device-proof-blocked-by-adb-authorization
title: KnowMe LiteRT-LM device proof blocked by adb authorization
tags:
- knowme
- model-runtime-per-device
- litert-lm
- android
- ios
- device-certification
- proof-gap
- uar
links:
- knowme-model-runtime-per-device-phase-status-and-proof-gaps
- litert-lm-device-proof-instrumentation-added-but-certification-blocked
- uar-durable-persistence-blocked-by-uninitialized-submodule
- knowme-uar-persistence-task-remains-blocked-by-submodule-checkout
sources:
- stdin
timestamp: 2026-07-26T08:08:13.494945+00:00
created_at: 2026-07-26T08:08:13.494945+00:00
updated_at: 2026-07-26T08:08:13.494945+00:00
revision: 0
---

## Context

Phase: `model-runtime-per-device` for KnowMe. Current position: `model-runtime-per-device › litertlm-device-proof`; status: `executing`; progress: `changes 4/5`.

This updates the active proof-gap status tracked in [KnowMe model-runtime-per-device phase status and proof gaps](/knowme-model-runtime-per-device-phase-status-and-proof-gaps.md) and supersedes any claim that `litertlm-device-proof` is complete.

## Phase goals

- Ship Qwen3.5-9B Q4_K_M with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM Gemma 4 E2B, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Authoritative status

- Session: No. 4 of 5.
- `changes_completed: 4 / 5`.
- `active_change: litertlm-device-proof`.
- `execute_complete: false`.
- Change 5, `litertlm-device-proof`, is **not finished**.

A prior completion claim for this change was false. The record's `status_corrections` reverted it because the Android release path crashed at bridge load until 2026-07-25; therefore, the claimed device evidence could not have existed. Do not mark the change complete from script existence alone.

## Device-proof evidence matrix

`litertlm-device-proof` requires physical-device evidence for 9 items:

| Item | State |
|---|---|
| Backend, memory, thermal | Script exists; needs physical device run |
| Cancellation | Script exists; needs physical device run |
| Markdown/Mermaid | Script exists; needs physical device run |
| Permitted tool call | Script exists; needs physical device run |
| Load time | Instrumented this turn via `load_ms` |
| Throughput | Instrumented this turn via `generate_ms` / `chars_per_second` |
| Attachment flow | No script exists; remaining implementation work |

Summary:

- 7 checks have tooling ready and are blocked only on device access.
- 2 timing/throughput metrics were instrumented in commit `c92fc80`.
- 1 check, attachment flow, still needs a script written.

## Current blocker

Samsung device `R5GYB4AZD7A` reports `adb unauthorized`.

Required action on the device screen:

- Tap **Allow USB debugging?**.
- Enable **Always allow from this computer** so authorization survives reconnects.

Without `adb shell`, the certification workflow cannot:

- Install the instrumented APK.
- Launch the app.
- Read logs.
- Run the 7 ready physical-device checks.

## Confirmed completed behavior

Android Local mode is genuinely device-proven on the Samsung:

- Local mode label applies.
- Requests route to on-device Gemma through LiteRT-LM.
- No UAR run is triggered for that path.
- Per-conversation isolation holds across threads.

This aligns with the previous instrumentation/blocking record in [LiteRT-LM device proof instrumentation added but certification blocked](/litert-lm-device-proof-instrumentation-added-but-certification-blocked.md).

## Still open outside Change 5

### iOS Local mode certification

- iOS Local mode remains uncertified.
- App is built, installed, and running on iPhone 17 Pro Max.
- Observed PID: `26732`.
- Process is stable.
- `devicectl` provides no screen control, so certification requires manual taps on the phone.

### Per-conversation mode persistence

Per-conversation mode still resets after app restart because `InMemoryProvider` is used at:

```text
rust/crates/gen_ui_agent/src/uar.rs:878
```

This is task 3.1 under `embed-uar-mobile-offline`, tracked separately in [UAR durable persistence blocked by uninitialized submodule](/uar-durable-persistence-blocked-by-uninitialized-submodule.md) and [KnowMe UAR persistence task remains blocked by submodule checkout](/knowme-uar-persistence-task-remains-blocked-by-submodule-checkout.md).

## Next steps

1. Authorize USB debugging on Samsung `R5GYB4AZD7A`.
2. Reinstall the instrumented APK.
3. Run the certification suite for the 7 ready checks.
4. Write the missing attachment-flow script.
5. Perform iOS tap-through certification on iPhone 17 Pro Max.
6. Only then consider closing `litertlm-device-proof`.

# Citations

1. [1] stdin