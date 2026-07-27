---
type: Reference
id: android-litert-lm-r8-fix-verified-through-gemma-model-load
title: Android LiteRT-LM R8 fix verified through Gemma model load
tags:
- model-runtime-per-device
- android-litertlm-runtime
- r8
- gemma
- local-inference
- device-verify
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-25T23:06:15.781264+00:00
created_at: 2026-07-25T23:06:15.781264+00:00
updated_at: 2026-07-25T23:06:15.781264+00:00
revision: 0
---

## Phase context

- Phase: `model-runtime-per-device`
- Project: `KnowMe`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system`
- Captured: `2026-07-25T23:05:21Z`
- Position: `model-runtime-per-device › android-litertlm-runtime › device-verify`
- Status: `monitoring`
- Device lane: Android local text-agent inference via LiteRT-LM `Gemma 4 E2B`

## Phase goals

- Ship `Qwen3.5-9B Q4_K_M` with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM `Gemma 4 E2B`, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Verification result

The Android R8 fix is confirmed on device: the LiteRT-LM bridge class now resolves and execution proceeds past `loadClass` into real model loading.

Observed successful load path:

```text
LiteRT-LM model loading model=gemma-4-e2b-it-litertlm
liblitertlm_jni.so ... ok
LitertLmLoader::Initialize
mmap_status is ok   status: OK   major_version: 1 minor_version: 5
section: LlmMetadataProto | SP_Tokenizer | model_type: tf_lite_embedder
```

Confirmed facts:

- No `ClassNotFoundException` remains for the bridge class.
- `liblitertlm_jni.so` loads successfully.
- `LitertLmLoader::Initialize` runs.
- Model artifact is already present on device: `mmap_status is ok` occurred immediately, so no new multi-GB download was required.
- LiteRT-LM model format version `1.5.0` loaded cleanly.
- Metadata and tokenizer sections parsed:
  - `LlmMetadataProto`
  - `SP_Tokenizer`
- Reported model type: `tf_lite_embedder`.

## Native loader diagnostic

The following log line is benign in this run:

```text
E  No implementation found for void NativeLibraryLoader.nativeCheckLoaded()
   — is the library loaded, e.g. System.loadLibrary?
```

Rationale:

- The line appears at `19:05:01.391`.
- Approximately 20 ms later, `liblitertlm_jni.so` loads successfully and initialization proceeds.
- This matches a probe-before-load pattern: LiteRT-LM calls `nativeCheckLoaded()` to test whether the native library is already loaded, catches the expected missing-native failure, then performs the actual load.
- The subsequent successful `liblitertlm_jni.so ... ok` and model initialization prove this diagnostic is not the failure condition.

## Current certification state

The lane is not yet fully certified end-to-end. Monitoring is waiting for decisive generation markers:

- Generation start
- Token deltas
- `RunFinished`

CPU initialization for a 3B-class model can take some time, so the current state remains `monitoring` rather than failed.

## Next actions

1. Continue watching Android device logs until generation markers appear.
2. Certify the Agent lane end-to-end once `RunFinished` is observed.
3. Have the operator open the Local tab and run a tool-call prompt.
4. Verify tool-agent plumbing sequence:
   - shortlist
   - gate
   - execution
   - continuation

# Citations

1. stdin
2. manual:KnowMe/model-runtime-per-device