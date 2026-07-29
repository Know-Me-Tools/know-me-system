---
type: Reference
id: android-litert-lm-cancellation-and-tool-call-certification-fixes
title: Android LiteRT-LM cancellation and tool-call certification fixes
tags:
- model-runtime-per-device
- litert-lm
- android-local-inference
- cancellation
- jni-bridge
- certification-scripts
- tool-calls
links:
- ios-cert-monitor-holding-for-local-send-with-benign-nw-path-noise
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-26T17:26:58.894070+00:00
created_at: 2026-07-26T17:26:58.894070+00:00
updated_at: 2026-07-26T17:26:58.894070+00:00
revision: 0
---

## Phase context

- Phase: `model-runtime-per-device`
- Project: `KnowMe`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system`
- Captured: `2026-07-26T16:52:32Z`
- Position: `model-runtime-per-device › embedded-uar-offline-agents › litertlm-device-proof`
- Status: `executing`

## Phase goals

- Ship `Qwen3.5-9B Q4_K_M` with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM `Gemma 4 E2B`, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Cancellation defect: two independent blockers

The chat stop/cancel path was ineffective for active LiteRT-LM generation because cancellation never reached the engine, and the Kotlin bridge serialized `cancel()` behind the long-running generation call.

### Rust cancellation path did not reach engine

Observed path before fix:

- Chat stop button called `cancelRun`.
- `cancelRun` invoked `state::cancel_run`.
- `state::cancel_run` set an `AtomicBool` that `chat.rs` only polled at phase boundaries.
- JNI `cancel()` was reachable only through `cancelModelOperation`, which controls model download, not active chat generation.

Result: the stop button and engine-level JNI cancellation were disconnected.

Fix:

- Added `cancel_generation()` to the `InferenceProvider` trait.
- Provided a default no-op implementation so existing engines are not forced to implement it immediately.
- Implemented `cancel_generation()` for LiteRT-LM.
- Called `cancel_generation()` from `state::cancel_run`.

Important semantic boundary:

- `cancel_generation()` intentionally does **not** set `model_cancelled`.
- It does **not** publish a `Cancelling` model/download state.
- The model remains loaded; only the current generation is cancelled.
- Reusing `cancel_model_operation` semantics would incorrectly make later prepare/load paths appear cancelled.

### Kotlin bridge deadlock blocked cancellation

`KnowMeLiteRtLmBridge.cancel()` was annotated `@Synchronized` on the same object as `sendMessage`. Since `sendMessage` holds that monitor for the entire generation, `cancel()` would block until generation completed, then cancel nothing.

Fix:

- Removed `@Synchronized` from `cancel()` only.
- Made `conversation` `@Volatile` so lock-free reads by `cancel()` remain safe.

## Tool-call certification script defect

Initial diagnosis claimed the tool-call script scraped before the UI settled. Code review showed this was incorrect.

Actual defect:

- The parent gate required all of:
  - `LiteRT-LM generation starting`
  - produced-text marker
  - expected UI substrings
- On an Orchestrator-lane run, the first two local-engine markers can never appear.
- Therefore the script polled until timeout even when the expected text was already rendered on screen.

Fix:

- Made rendered UI authoritative for pass/fail because it proves the answer reached the user regardless of lane.
- Local-engine log markers are still reported when present as corroborating evidence.
- Added `REQUIRE_LOCAL_ENGINE=1` for certification runs where the on-device local path itself must be proven.
- Updated child script log assertions to accept either lane's markers.

## Validation status

- Host `cargo check`: green.
- Both affected scripts pass syntax validation with `bash -n`.
- APK rebuild was in progress at capture time.

## Next steps

- Install the rebuilt APK.
- Re-run `certify-local-cancellation` on device.
- Re-run `certify-local-tool-call` on device.
- Complete attachment-flow scripting.
- Complete iOS airplane-mode certification; this continues the broader iOS device-certification lane previously blocked on local-send/operator actions such as [iOS cert monitor holding for Local send with benign nw_path noise](/ios-cert-monitor-holding-for-local-send-with-benign-nw-path-noise.md).

# Citations

1. stdin
2. manual:KnowMe/model-runtime-per-device