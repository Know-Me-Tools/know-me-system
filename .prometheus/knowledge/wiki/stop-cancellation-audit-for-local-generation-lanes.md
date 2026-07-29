---
type: Reference
id: stop-cancellation-audit-for-local-generation-lanes
title: Stop cancellation audit for local generation lanes
tags:
- model-runtime-per-device
- generation-cancellation
- mlx-c
- llama-cpp
- webllm
- tauri
- accessibility
- database-migrations
links:
- memory-fix-install-watch-armed-for-unplug-safe-ios-verification
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-27T03:20:01.978011+00:00
created_at: 2026-07-27T03:20:01.978011+00:00
updated_at: 2026-07-27T03:20:01.978011+00:00
revision: 0
---

## Phase context

- Phase: `model-runtime-per-device`
- Project: `KnowMe`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system`
- Captured: `2026-07-27T02:14:44Z`
- Position: `model-runtime-per-device › embedded-uar-offline-agents › litertlm-device-proof`
- Status: `executing`
- Commit: `44ee525`

## Phase goals

- Ship `Qwen3.5-9B Q4_K_M` with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM `Gemma 4 E2B`, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Key finding

The cancellation audit was required before UI polish: on three of four desktop/web lanes, **Stop did not actually stop generation**, so there was no preserved partial output to Steer or Resume from. The root cause across engines was a no-op `cancel_generation` default that hid mobile/engine bugs.

## Engine fixes

### macOS / mlx-c

- Existing code already had:
  - a per-token cancellation flag,
  - `Cancelled` event emission.
- Missing piece: engine override for `cancel_generation`.
- Before the fix, the cancellation flag was reachable only through `cancel_model_operation`, which is for model download cancellation rather than generation cancellation.
- Fix was a one-line override connecting Stop to the existing per-token generation cancellation path.

### Windows/Linux / llama.cpp

- Previous token loop had no user-cancel concept.
- Loop exited only on:
  - EOG,
  - `max_tokens`,
  - dropped receiver.
- Added a dedicated generation-cancel flag, kept separate from `model_cancelled`.
- Rejected `llama_set_abort_callback` for this pass:
  - it exists in `llama-cpp-sys-2`,
  - it is not exposed by the safe wrapper,
  - using it would require raw FFI.
- Per-token checks now cover stop-mid-answer behavior.
- Limitation documented: this does **not** interrupt a long prefill.

### Browser / WebLLM

Problems found:

- `interruptGenerate()` was installed but never called.
- `cancelActiveRun` branched two ways while `runInferenceTurn` branched three ways.
- `streamLocalWeb` silently dropped lifecycle events.
- Stop posted to a remote service for a run the remote service had never heard of, while local tokens continued streaming.
- UI displayed “Cancelling”, making the broken cancellation path appear functional.

## UI and interaction corrections

- Initial plan incorrectly placed the action bar in the `RunCancelled` data renderer.
  - `thread.tsx` already imports the store and attaches Regenerate/Delete via `ActionBarPrimitive`.
  - A data renderer has no message id, so Retry/Discard cannot be wired there.
- Steer behavior was corrected: Steer must **arm the composer**, not immediately send.

## Database migration issue

- `CREATE TABLE IF NOT EXISTS` does not add new columns to existing databases.
- Adding `intent` only to the create-table schema caused existing databases to miss the column.
- The next INSERT failed for **every** message.
- Fix: add an idempotent `ALTER` migration for `intent`.

## Accessibility fixes

- Light-theme primary color contrast was `3.97:1`.
- This was acceptable for the existing 32 icon-only uses of the variant (`3:1` non-text UI requirement), but failed for a new 14px text label requiring `4.5:1`.
- Fixed contrast to `4.84:1` with a scoped change instead of changing a token used by 32 call sites.
- `RunCancelled` was missing `aria-live`; added live-region behavior for cancellation status.

## Verification status

Green checks:

- Vitest: `52/52`
- TypeScript: clean `tsc`
- `gen_ui_inference`:
  - `7/7` mlx-c
  - `15/15` llama

Known non-signal:

- `pnpm lint` is pre-existing broken because the repo has no ESLint config.

Not yet verified:

- Real Tauri desktop run was still building at capture time due to cold Rust build.
- llama.cpp cancellation change is unit-tested only; there was **no Windows or Linux hardware run** at capture time.
- iOS airplane-mode and attachment-flow certification remained open; related iOS local-runtime certification/install context includes [Memory-fix install watch armed for unplug-safe iOS verification](/memory-fix-install-watch-armed-for-unplug-safe-ios-verification.md).

## Next verification steps

1. Complete real `tauri:dev` run on the mlx-c lane.
2. Confirm the local lane is selected.
3. Stop mid-stream.
4. Record kept characters and rendered “Cancelled by user”.
5. Verify Resume.
6. Verify Steer arms composer rather than sending immediately.
7. Confirm existing tool UI still renders, because the status change touches every message.
8. Record with the karpathy recorder.

# Citations

1. stdin
2. manual:KnowMe/model-runtime-per-device