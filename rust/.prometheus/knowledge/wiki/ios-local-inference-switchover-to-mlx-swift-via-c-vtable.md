---
type: Reference
id: ios-local-inference-switchover-to-mlx-swift-via-c-vtable
title: iOS local inference switchover to MLX-Swift via C vtable
tags:
- knowme
- ios
- mlx-swift
- ffi
- local-inference
- model-runtime-per-device
links:
- ios-mlx-switchover-and-qwen3-arch-fix
- ios-device-certification-local-and-embedded-uar
- desktop-macos-mlx-floor-raise-and-ios-thinking-split
sources:
- ios-mlx-switchover-and-swift-bridge.md
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-25T21:45:21.667797+00:00
created_at: 2026-07-25T21:45:21.667797+00:00
updated_at: 2026-07-25T21:45:21.667797+00:00
revision: 0
---

## Context

- **Project:** KnowMe
- **Phase:** `model-runtime-per-device`
- **Captured:** 2026-07-25
- **Status:** verified

## Intent

Make on-device inference work reliably on iPhone. The prior iOS lane failed because:

- `llama.cpp` was the slowest measured iOS runtime and pulled a large, brittle C++ graph into the Rust staticlib.
- The original Qwen3.5-9B target was not viable within iOS jetsam budgets, causing total local inference failure.

## Observed state

- On iPhone 17 Pro, 2B-class benchmarks favored MLX-Swift:
  - MLX-Swift: **46.4 tok/s** decode
  - `llama.cpp`: **37.6 tok/s** decode
  - Qwen3.5-2B: MLX **61 tok/s** vs `llama.cpp` **39 tok/s**
- MLX memory usage exceeds the raw weight file size; a 2B model can require about **3 GB** at runtime.
- RAM preflight and the `com.apple.developer.kernel.increased-memory-limit` entitlement are load-bearing for MLX on iOS.
- There was no existing Rust↔Swift FFI equivalent to Android's `JNI_OnLoad` / `set_java_vm` registration pattern.

## Decisions

### Runtime and model

- Use **MLX-Swift** for iOS local inference.
- Set **Qwen3-4B 4-bit** as the default iOS model.
- Formally abandon the prior **Qwen3.5-9B** goal on jetsam evidence.

The later physical-device bring-up and model architecture correction are covered in [iOS MLX switchover + Qwen3.5→Qwen3-4B arch fix + build fixes](/ios-mlx-switchover-and-qwen3-arch-fix.md).

### FFI bridge direction

Swift registers a `#[repr(C)]` vtable into Rust at app launch. This is the inverse of Android's `set_java_vm` pattern.

Chosen over Rust-declared `extern "C"` functions implemented in Swift because:

- Host/test builds degrade safely: the Rust side can keep the bridge as `Option<vtable>` at runtime.
- The pure Rust side remains testable with `MockMlxBridge` before device availability.
- The shape mirrors Android gate-report registration rather than relying on hard link-time Swift symbols.

### Streaming behavior

Streaming is real, not synthesized:

- MLX streams token-by-token.
- Swift invokes a per-delta callback.
- Callback JSON feeds an `UnboundedSender<StreamEvent>`.
- The channel shape matches the existing `litert_lm.rs` lane.

By contrast, Android's LiteRT lane fabricates streaming from a single response; iOS MLX does not.

### Ownership boundary

Rust remains owner of:

- model catalog
- download and verification
- memory preflight
- prompt assembly
- tool-context assembly
- event normalization

Swift owns only the native MLX inference binding. This preserves the TJ-ARCH-MOB-001 boundary.

### MLX artifact catalog

Added multi-file `MlxArtifactSet` catalog support because MLX models are Hugging Face repositories, not single GGUF files. A model may include:

- config files
- tokenizer files
- one or more safetensors shards

Downloads remain fully in Rust with:

- per-file resume
- SHA-256 verification
- `.part` temporary files followed by atomic rename

## Verification

Device-certified on **iPhone 17 Pro Max**:

- Qwen3-4B loads.
- Qwen3-4B generates tokens.
- The first blocking runtime issue was `unsupportedModelType`, traced to the model repoint from Qwen3.5 to Qwen3-4B.

Build/device fixes needed during certification:

1. Register Swift source file in the Xcode target.
2. Enable automatic signing.
3. Change `rethrows` to `throws`.
4. Mark `emit`, `maxTokens`, and `Message` closures as escaping where required.
5. Repoint the model from Qwen3.5 to Qwen3-4B.

Full real-hardware lane certification, including the distinction between direct local inference and embedded-UAR, is documented in [iOS device certification — Local and embedded-UAR lanes on real hardware](/ios-device-certification-local-and-embedded-uar.md).

## Related implementation note

The later iOS reasoning-rendering fix kept the same MLX lane but changed token classification so Qwen3 `<think>...</think>` output becomes `ThinkingDelta` rather than plain `TextDelta`; see [Desktop macOS MLX floor raise + iOS thinking-split](/desktop-macos-mlx-floor-raise-and-ios-thinking-split.md).

## Lesson

A benchmark advantage is not enough to complete a runtime port. The decisive engineering work was the seam: vtable registration made the Rust half host-testable and allowed certification of catalog, preflight, and event normalization before physical-device execution was available.

Choose the FFI direction that keeps the pure side testable.

## Next experiment

Re-tune `estimated_peak_bytes` and the jetsam fraction from measured on-device values rather than estimates.

# Citations

1. ios-mlx-switchover-and-swift-bridge.md
2. manual:KnowMe/model-runtime-per-device