---
type: Reference
id: ios-mlx-switchover-and-qwen3-arch-fix
title: iOS MLX switchover + Qwen3.5→Qwen3-4B arch fix + build fixes
tags:
- knowme
- model-runtime-per-device
- ios
- mlx
- mlx-swift
- qwen3
- code-signing
- device-certification
links:
- ios-think-split-segmenter
- desktop-macos-mlx-floor-raise-and-ios-thinking-split
- mlx-engine-policy
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-23T01:30:00+00:00
created_at: 2026-07-23T01:30:00+00:00
updated_at: 2026-07-23T01:30:00+00:00
revision: 0
---

## Intent

Run KnowMe's iOS Flutter app on Travis's iPhone 17 Pro Max and make on-device
local inference actually produce tokens, after a total iOS-inference failure with
the prior llama.cpp/Qwen3.5-9B target.

## Observed state

- Engine policy set to MLX on Apple platforms (iOS + macOS), llama.cpp on
  Windows/Linux, LiteRT-LM on Android. iOS binds MLX-Swift over a C vtable FFI
  (`mobile/ios/Runner/KnowMeMlxBridge.swift` ↔
  `$REPO_ROOT/rust/crates/gen_ui_inference/src/ios_mlx_ffi.rs`
  `knowme_mlx_register_bridge`).
- First device build surfaced a chain of failures, each caught only by actually
  building/signing/running for the physical device.

## Decision and delta

Six fixes, committed on `feat/ios-mlx-inference`:
1. **`b568fa2`** — registered `KnowMeMlxBridge.swift` in the Runner target's
   Sources build phase (it was on disk but not in the pbxproj, so
   `registerKnowMeMlxBridge()` failed to compile).
2. **`28dcab9`** — switched Runner to automatic code signing (team 48VVJ9AXAR).
   The manual "KnowMe Development" profile predated the
   `com.apple.developer.kernel.increased-memory-limit` entitlement (added for
   MLX's large unified-memory footprint) and couldn't sign.
3. **`e0dec9c`** — `runBlocking` must be `throws`, not `rethrows`: the async
   body's error is captured into a `Result` across the Task/semaphore boundary,
   so Swift can't attribute the re-throw to the parameter closure.
4. **`c1d0c23`** — `emit` made `@escaping` (captured by the escaping async body);
   `maxTokens` wired into `GenerateParameters(maxTokens:)`; `messages` typed
   `[[String: Any]]` to match `MLXLMCommon.Message = [String: Any]`.
5. **`ff25b0d`** — THE runtime fix. On-device load failed with
   `unsupportedModelType("qwen3_5")`: MLXLLM's `LLMModelFactory`
   (mlx-swift-examples 2.29.1, the latest tag) registers `qwen2`/`qwen3`/
   `qwen3_moe` but NOT the newer `qwen3_5` architecture, and no released tag adds
   it. Repointed both iOS MLX catalog entries to `qwen3`-arch models with freshly
   verified pinned revisions + per-file SHA-256: 4B default →
   `mlx-community/Qwen3-4B-4bit @4dcb3d10`; 8B opt-in →
   `mlx-community/Qwen3-8B-4bit @545dc425`. The `model_id` strings stay
   `qwen3.5-*` as stable persisted identities; the artifacts are Qwen3.
6. **`fc7b8d0`** — macOS desktop default local model must be the MLX id, not the
   GGUF id (found via the desktop runtime cert; see
   [[desktop-macos-mlx-floor-raise-and-ios-thinking-split]]).

## Verification

- `--no-codesign` build: full compile+link green (Rust staticlib + MLX-Swift +
  the `knowme_mlx_register_bridge` vtable symbol).
- Signed install: `flutter run --release -d 00008150-…` installed + launched on
  the iPhone 17 Pro Max.
- On device: the local model downloaded and generated tokens (operator-confirmed
  token generation). MLXLLM 2.29.1 registry inspected directly to confirm
  `qwen3` supported / `qwen3_5` not.

## Failure/lesson

Compilation cannot certify runtime behavior. Five of six fixes were only
observable by building+signing+running for the physical device; the decisive one
(`unsupportedModelType`) required an actual on-device model load. Model
architecture must be validated against the *pinned* MLX-Swift registry, not
assumed from the model name — a newer architecture silently fails at load.

## Next experiment

Certify that the "Local" tab (direct MLX engine) produces an actual streamed
response bubble on device, distinct from the embedded-UAR lane (iOS defaults to
the UAR lane). See [[ios-local-embedded-lane-certification]].
