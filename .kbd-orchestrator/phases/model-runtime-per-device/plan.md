# PLAN: model-runtime-per-device

Project: KnowMe
Date: 2026-07-18
OpenSpec available: YES
Changes to implement: 4

## CHANGE LIST (ordered)

1. `qwen35-model-runtime`: Ship the device-aware Qwen3.5 model catalog and accelerated runtime foundation.
   - Scope: Rust inference core, Cargo target features, model cache, FFI diagnostics
   - Depends on: NONE
   - Recommended agent: Codex
   - Est. complexity: L
   - Complexity score: High
   - Model class: frontier
   - Customer value: HIGH
   - Details: Replace the single Qwen2.5 entry with iPhone Qwen3.5-9B and Android Qwen3.5-4B Q4_K_M entries, each with pinned revision, byte length, SHA-256, and matching backend policy. Implement resumable atomic downloads, context/output defaults, Metal/Vulkan feature selection, CPU fallback, and non-sensitive runtime diagnostics. Preserve absolute user-supplied GGUF loading.

2. `qwen35-mobile-vision`: Add typed local image requests through llama.cpp mtmd.
   - Scope: Rust inference contract, projector catalog/download, mtmd binding, FFI attachment validation
   - Depends on: `qwen35-model-runtime`
   - Recommended agent: Codex
   - Est. complexity: High
   - Complexity score: High
   - Model class: frontier
   - Customer value: HIGH
   - Details: Add validated image inputs without moving inference or networking into Flutter. Download and verify the exact mmproj paired with the selected platform model, initialize mtmd, and return typed errors for unsupported or oversized media. Keep text-only requests backward compatible.

3. `qwen35-local-tools`: Add bounded structured tool-call continuation for local Qwen3.5.
   - Scope: Rust agent/inference orchestration, MCP registry integration, AG-UI/A2UI events, parser and policy tests
   - Depends on: `qwen35-model-runtime`
   - Recommended agent: Codex
   - Est. complexity: L
   - Complexity score: High
   - Model class: frontier
   - Customer value: HIGH
   - Details: Render only registered tool schemas using the model chat template, parse strict structured calls, enforce permission and argument/output/iteration limits, execute through the existing MCP registry, and continue with tool results. Emit tool-call and tool-result events through the existing protocol so Flutter can render them.

4. `qwen35-device-proof`: Verify the complete text, vision, and safe-tool path on physical devices.
   - Scope: Rust diagnostics, Flutter integration tests, iOS/Android build and deployment evidence
   - Depends on: `qwen35-mobile-vision`, `qwen35-local-tools`
   - Recommended agent: Manual
   - Est. complexity: L
   - Complexity score: High
   - Model class: frontier
   - Customer value: HIGH
   - Details: Build and deploy to Travis's iPhone 17 Pro Max and the Galaxy S25 Ultra, record selected model/backend, load time, representative throughput, memory/fallback decisions, and results for Markdown/Mermaid, image understanding, and one permitted tool. Fail the release gate if Vulkan produces corrupted output or if the selected artifact is not verified.

## EXECUTION ROUND ORDER

Round 1: `qwen35-model-runtime`
Round 2 (parallel): `qwen35-mobile-vision`, `qwen35-local-tools`
Round 3: `qwen35-device-proof`

## COMMANDS TO RUN

```text
/opsx:apply qwen35-model-runtime
/opsx:apply qwen35-mobile-vision
/opsx:apply qwen35-local-tools
/opsx:apply qwen35-device-proof
```

## SCOPE CUT / TRADE-OFF

Gemma 4 through LiteRT-LM/QNN is explicitly deferred. It may be benchmarked later as an Android-specific backend, but adopting it in this phase would introduce a second runtime before the llama.cpp Qwen3.5 path is proven and would not satisfy the current unified Rust inference contract.

## PLAN COMPLETE
