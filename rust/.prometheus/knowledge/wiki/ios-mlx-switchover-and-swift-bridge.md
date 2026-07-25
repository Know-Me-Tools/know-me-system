---
type: Reference
id: ios-mlx-switchover-and-swift-bridge
title: iOS local inference switchover — llama.cpp to MLX-Swift over a C vtable
tags:
- knowme
- model-runtime-per-device
- ios
- mlx
- mlx-swift
- ffi
- local-inference
links:
- mlx-engine-policy
- desktop-macos-mlx-floor-raise-and-ios-thinking-split
- ios-device-certification-local-and-embedded-uar
sources:
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-25T21:40:00+00:00
created_at: 2026-07-25T21:40:00+00:00
updated_at: 2026-07-25T21:40:00+00:00
revision: 0
---

## Context

- **Project:** KnowMe
- **Phase:** `model-runtime-per-device`
- **Captured:** `2026-07-25`
- **Status:** `verified`

## Intent

Make on-device inference actually work on iPhone. llama.cpp was the slowest
measured runtime on iOS and its Rust binding compiled a large, brittle C++ graph
into the staticlib; separately the phase's original Qwen3.5-9B target was never
viable inside iOS jetsam budgets, which produced total iOS inference failure.

## Observed state

- Benchmarks (iPhone 17 Pro, 2B-class): MLX-Swift 46.4 tok/s decode vs
  llama.cpp 37.6; MLX wins Qwen3.5-2B at 61 vs 39.
- MLX's memory footprint is heavier than the weight file (~3 GB for a 2B model),
  so the RAM preflight and the increased-memory-limit entitlement are
  load-bearing, not optional.
- No Rust↔Swift FFI existed. Android's `JNI_OnLoad` / `set_java_vm` pattern had
  no iOS counterpart.

## Decision and delta

- **Runtime:** MLX-Swift, with Qwen3-4B 4-bit as the default. The 9B goal was
  formally abandoned on jetsam evidence.
- **Bridge direction:** Swift registers a `#[repr(C)]` vtable into Rust at app
  launch (the inverse of `set_java_vm`), chosen over Rust-declared `extern "C"`
  implemented in Swift because it degrades link-safely on host/test builds —
  everything is `Option<vtable>` at runtime — and mirrors the Android gate-report
  pattern.
- **Streaming is real, not faked:** MLX streams token-by-token, so Swift invokes
  a per-delta callback whose JSON feeds an `UnboundedSender<StreamEvent>` — the
  channel shape `litert_lm.rs` already used. (Android's LiteRT lane fabricates
  streaming from one response; iOS does not.)
- **Rust keeps ownership** of catalog, download/verify, memory preflight, prompt
  and tool-context assembly, and event normalization. Swift binds the native
  inference API and nothing else (TJ-ARCH-MOB-001).
- Multi-file `MlxArtifactSet` catalog support was added because MLX models are HF
  repos (config + tokenizer + safetensors shards), not single GGUF files;
  downloads stay 100% in Rust with per-file resume, SHA-256, and `.part` rename.

## Verification

Device-certified on iPhone 17 Pro Max: Qwen3-4B loads and generates. An
architecture mismatch (`unsupportedModelType`) surfaced first and was traced to
the model repoint (Qwen3.5 → Qwen3-4B), alongside five build fixes:
source-file registration, automatic signing, `rethrows`→`throws`, escaping
`emit`/`maxTokens`/`Message` closures, and the model repoint itself.

## Failure/lesson

A benchmark advantage is not a port. The decisive work was the *seam*: choosing
vtable-registration over declared-extern made the whole engine host-testable
with a `MockMlxBridge`, which is why the Rust half could be certified before any
device existed in the loop. Pick the FFI direction that keeps the pure side
testable.

## Next experiment

Re-tune `estimated_peak_bytes` and the jetsam fraction from measured on-device
values rather than estimates.
