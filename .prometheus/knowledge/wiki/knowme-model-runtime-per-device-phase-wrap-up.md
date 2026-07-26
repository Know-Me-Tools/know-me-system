---
type: Reference
id: knowme-model-runtime-per-device-phase-wrap-up
title: KnowMe model-runtime-per-device phase wrap-up
tags:
- model-runtime-per-device
- knowme
- uar
- embedded-runtime
- ios-inference
- react-settings
- knowledge-capture
links:
- ios-cert-monitor-holding-for-local-send-with-benign-nw-path-noise
- memory-fix-install-watch-armed-for-unplug-safe-ios-verification
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-25T22:00:43.823764+00:00
created_at: 2026-07-25T22:00:43.823764+00:00
updated_at: 2026-07-25T22:00:43.823764+00:00
revision: 0
---

## Phase status

- Phase: `model-runtime-per-device`
- Project: `KnowMe`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system`
- Captured: `2026-07-25T21:49:05Z`
- Status: complete except deferred `P2.3` / issue `#22`
- Position: `model-runtime-per-device › phase-wrapup`

## Goals

- Ship `Qwen3.5-9B Q4_K_M` with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM `Gemma 4 E2B`, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Completed work

### M7-react: runtime-aware settings UI

Commit: `da95881`

- React settings now branch on the live runtime status `targetMode`.
- On macOS embedded runtime:
  - URL/key/Save/Test service form is replaced by an `Embedded · on-device` badge.
  - Kernel readiness is shown.
- On Windows, Linux, and web:
  - Service URL/key/Save/Test form remains available.
- No new FFI was needed because `uar_runtime_status()` already reported `targetMode`.
- Net change was a single-file React change after a test rejected an attempted wider runtime-status change.

#### Failure lesson

Do **not** make `UarRuntimeStatus::current()` report macOS as embedded with `embedded_kernel_available`.

- `UarRuntimeStatus::current()` is a static per-platform projection.
- Reporting `embedded_kernel_available` there would assert a kernel may be initialized when it may not be.
- The correct UI path is to read the live runtime status, which already reports the embedded target mode correctly.

### Knowledge capture D

Commit: `5d28212`

Seven consolidated Reference records were written, index/log updated, ingested, linted, and verified discoverable via `pk search`.

| Record | Non-obvious lesson |
|---|---|
| iOS MLX switchover + Swift vtable | Vtable **registration**, not declared-extern linkage, kept the Rust engine host-testable. |
| PreThink segmenter | Qwen3's template pre-fills the opening `<think>`; the stream only carries the closing tag. |
| Mobile embedded-UAR config | Routing must live in the driver; the kernel binds one `primary_driver` per run. |
| Three-tier selection + provenance | The blank chip came from an empty `ModelRoute` at the producer, not a renderer bug. |
| Embedded session memory | History never left the app; an LLM's memory is exactly the message list sent to it. |
| macOS embedded host | Parity is per deployment mode, not uniform behavior. |
| iOS device certification | The default lane is `uar`, not `local`; certify the lane intended for certification. |

These consolidated records supersede noisy historical cert-monitor snapshots, including device-wait states such as [iOS cert monitor holding for Local send with benign nw_path noise](/ios-cert-monitor-holding-for-local-send-with-benign-nw-path-noise.md) and related install-watch entries such as [Memory-fix install watch armed for unplug-safe iOS verification](/memory-fix-install-watch-armed-for-unplug-safe-ios-verification.md).

## Verification

- `pk lint`: **0 errors**.
- No lint errors referenced the new consolidated records.
- `pk search`: all 7 records ingested and returned as top hits.
- Historical lint warnings remain:
  - 99 warnings total.
  - Mostly from historical cert-monitor snapshots generated while waiting for device taps.
  - These are superseded by the consolidated Knowledge capture D records.
- `prometheus learn --compile` could not run the knowledge step because it requires `prometheus-learn/knowledge` to be built in.
- Effective verification gate for this round was `pk lint`.

## Closed tasks

- Tasks `29`, `36`, and `41` closed.

## Remaining deferred work

Only `P2.3` / issue `#22` remains:

- Wire KnowMe's `embedded_stream_payload` to the UAR encoder.
- UAR encoder already landed in ADR `0012`.
- This removes a duplicate KnowMe encoder.
- No user-visible effect; categorized as low-value deferred de-duplication.

## Current recommendation

Phase work is complete except deferred encoder de-duplication. Next action is either:

1. Complete `P2.3` / `#22`, or
2. Run the phase reflection gate.

# Citations

1. stdin
2. manual:KnowMe/model-runtime-per-device