---
type: Reference
id: android-litert-lm-cpu-certification-reaches-graph-initialization
title: Android LiteRT-LM CPU certification reaches graph initialization
tags:
- model-runtime-per-device
- android-litert-lm
- device-verify
- cpu-baseline
- gemma
- runtime-gates
links:
- m5b-install-monitor-armed-for-model-runtime-picker-verification
- memory-fix-install-watch-armed-for-unplug-safe-ios-verification
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-25T23:07:00.049334+00:00
created_at: 2026-07-25T23:07:00.049334+00:00
updated_at: 2026-07-25T23:07:00.049334+00:00
revision: 0
---

## Phase context

- Phase: `model-runtime-per-device`
- Project: `KnowMe`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system`
- Captured: `2026-07-25T23:05:56Z`
- Position: `model-runtime-per-device › android-litertlm-runtime › device-verify`
- Status: `monitoring`
- Device lane: Android local text-agent inference certification

## Phase goals

- Ship `Qwen3.5-9B Q4_K_M` with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM `Gemma 4 E2B`, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Current state

LiteRT-LM model initialization is progressing beyond the prior R8 fix. The runtime is loading and compiling model graphs for the Android certification lane.

Observed initialization markers:

- Accelerators registered:
  - GPU: statically linked and registered
  - CPU: registered
- Embedder loaded:
  - Size: `103.8 MB`
  - Type: `TF_LITE_EMBEDDER`
  - Signature: `embedder`
  - Compiled ops: `8`
- Auxiliary graphs initialized, including `eoi` / end-of-audio-or-vision graphs.
- Flatbuffer models are initializing directly.

The lane has not yet reached final certification. Decisive markers still pending:

- Prefill/decode start
- Token deltas
- `RunFinished`

First-load graph compilation on CPU for a 3B-class model can take significant time before the first token appears.

## Expected NPU warnings

The following warnings are expected and are not certification failures:

```text
NPU accelerator could not be loaded ... kLiteRtStatusErrorInvalidArgument
You should provide DispatchLibraryDir option to use NPU
```

Rationale:

- The current certification target is the generic CPU artifact.
- NPU is intentionally not wired for this pass.
- GPU/NPU and `SM8750`-specific artifacts are later optimizations after the generic CPU path is certified.
- Therefore, missing NPU dispatch-library configuration is the expected runtime state.

## Certification plan

1. Continue monitoring until generation begins.
2. Confirm prefill/decode and token deltas.
3. Confirm `RunFinished` to certify Android Agent lane end-to-end on the CPU baseline.
4. Have the operator open the Local tab.
5. Run a tool-call prompt to verify:
   - shortlist
   - runtime gate
   - tool execution
   - continuation after tool execution

This Android lane is the counterpart to the iOS per-device runtime work tracked in entries such as [M5b install monitor armed for model runtime picker verification](/m5b-install-monitor-armed-for-model-runtime-picker-verification.md) and [Memory-fix install watch armed for unplug-safe iOS verification](/memory-fix-install-watch-armed-for-unplug-safe-ios-verification.md).

# Citations

1. stdin
2. manual:KnowMe/model-runtime-per-device