---
type: Reference
id: desktop-local-inference-blocked-by-stale-mlx-model-download
title: Desktop local inference blocked by stale MLX model download
tags:
- model-runtime-per-device
- mlx
- desktop-runtime
- local-inference
- tauri
- cancel-generation
- accessibility
links:
- memory-fix-install-watch-armed-for-unplug-safe-ios-verification
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-27T05:24:40.016841+00:00
created_at: 2026-07-27T05:24:40.016841+00:00
updated_at: 2026-07-27T05:24:40.016841+00:00
revision: 0
---

## Phase context

- Phase: `model-runtime-per-device`
- Project: `KnowMe`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system`
- Captured: `2026-07-27T03:19:35Z`
- Position: `model-runtime-per-device › embedded-uar-offline-agents › litertlm-device-proof`
- Status: `executing`

## Phase goals

- Ship `Qwen3.5-9B Q4_K_M` with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM `Gemma 4 E2B`, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Desktop runtime result

The app built and ran cleanly for roughly one hour with zero build/runtime startup errors, but the desktop MLX lane could not be certified because the local model artifact is incomplete:

```text
model-cache/mlx/qwen3.5-4b-mlx-4bit/
  model.safetensors.part   1,149,317,551 bytes   Jul 22 15:07
```

- Only a stale `model.safetensors.part` exists.
- No complete `model.safetensors` is available.
- Therefore there is no local model available for generation and no valid mid-stream cancellation test could be run.
- The dev server was stopped rather than left running.

A second automation blocker was found: the dev binary runs without a bundle identifier. `osascript` returns `missing value`, so the screen-control tool cannot resolve the process as an app. UI-driving this lane requires a packaged `.app` build.

## Verified work

Code is committed as `44ee525`.

Validation completed:

- Vitest: `52/52` passing across 11 files.
- TypeScript: `tsc` clean.
- `gen_ui_inference`:
  - MLX-C: `7/7` passing.
  - llama.cpp: `15/15` passing.

New coverage includes:

- Terminal cancelled state.
- Partial-content preservation.
- Resume history-replay invariant.
- Both cancel-lane routes.

Three engine gaps were closed. All shared the same failure mode as the mobile bugs: a no-op trait default compiled cleanly and still returned `Ok(true)`, masking missing runtime behavior.

## Accessibility fix

The accessibility gate caught a real contrast failure:

- Light-theme primary color contrast was `3.97:1`.
- That was acceptable for the existing 32 icon-only uses of the variant.
- It failed once a 14px label was added because text requires at least `4.5:1`.
- The fix raised contrast to `4.84:1` and was scoped locally rather than changing a token used by 32 existing call sites.

## Not verified

- **macOS desktop MLX runtime:** blocked by the incomplete `qwen3.5-4b-mlx-4bit` download. This was the requested certification gate and is not met.
- **Windows/Linux llama.cpp runtime:** fixed and unit-tested, but no hardware was available for runtime verification.
- **Browser WebLLM end-to-end:** requires WebGPU and a model download.

The implementation and engine fixes are well-tested at code level, but runtime certification is incomplete for desktop inference. In particular, two of the three engine fixes require real generation tests for full confidence: setting a cancellation flag is not equivalent to proving generation actually stops.

## Next actions

1. Delete the stale `model.safetensors.part` and re-trigger the `qwen3.5-4b-mlx-4bit` download from the app model UI.
2. Run `tauri:dev` after a complete model is present.
3. Confirm the Local lane generates.
4. Stop generation mid-stream and record:
   - preserved character count;
   - rendered `Cancelled by user` state.
5. Exercise Resume and Steer flows.
6. Re-check the tool UI because the message status change touches every message.
7. Continue open certification items from earlier device work, including attachment-flow certification and iOS airplane-mode verification in the same model-runtime lane as [Memory-fix install watch armed for unplug-safe iOS verification](/memory-fix-install-watch-armed-for-unplug-safe-ios-verification.md).

# Citations

1. stdin
2. manual:KnowMe/model-runtime-per-device