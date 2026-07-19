PLAN: on-device-qwen35
Project: KnowMe
Date: 2026-07-18T20:23:58Z
OpenSpec available: YES
Changes to implement: 4

CHANGE LIST (ordered)
1. qwen35-model-runtime: Replace the legacy model with verified per-platform Qwen3.5 artifacts and accelerated text inference.
   - Scope: Rust inference | Rust agent configuration | iOS/Android native build
   - Depends on: NONE
   - Recommended agent: Codex
   - Est. complexity: L
   - Complexity score: High
   - Model class: frontier
   - Customer value: HIGH
   - Details: Add a typed model catalog for iOS 9B and Android 4B, resumable streaming downloads with incremental SHA-256 verification, 8K context and 1,024-token output defaults, Metal/Vulkan build features, GPU offload selection, and safe CPU fallback. Preserve the existing absolute-GGUF escape hatch and Rust-core boundary.

2. qwen35-mobile-vision: Deliver real local image understanding with matching multimodal projectors.
   - Scope: Rust inference trait | Rust agent attachments | FFI contract | existing Flutter attachment flow
   - Depends on: qwen35-model-runtime
   - Recommended agent: Codex
   - Est. complexity: L
   - Complexity score: High
   - Model class: frontier
   - Customer value: HIGH
   - Details: Extend the inference seam with typed local requests carrying validated image paths, acquire and verify the matching projector, initialize llama.cpp mtmd, and pass visual embeddings into generation. Retain bounded textual extraction for non-image documents and add focused contract tests.

3. qwen35-local-tools: Add structured local tool calls and a bounded host-side execution loop.
   - Scope: Rust agent | inference formatting/parsing | MCP tool policy | AG-UI/A2UI events
   - Depends on: qwen35-model-runtime
   - Recommended agent: Codex
   - Est. complexity: L
   - Complexity score: High
   - Model class: frontier
   - Customer value: HIGH
   - Details: Supply the same permitted MCP tool schemas used by the cloud lane to the local model, parse Qwen structured calls, publish tool-use/results through the existing event adapter, execute through the existing permission-aware tool registry, and cap iterations. Do not expose unrestricted filesystem or shell capability merely because the model requests it.

4. qwen35-device-proof: Prove model identity, acceleration/fallback, vision, and useful output on both physical phones.
   - Scope: Rust diagnostics | Flutter integration tests | iOS/Android deployment evidence
   - Depends on: qwen35-model-runtime, qwen35-mobile-vision, qwen35-local-tools
   - Recommended agent: Codex
   - Est. complexity: M
   - Complexity score: Medium
   - Model class: medium
   - Customer value: HIGH
   - Details: Expose non-sensitive runtime diagnostics, add deterministic integration prompts for Markdown/Mermaid, image description, and a safe tool call, deploy to Travis's iPhone and the attached Android phone, and record throughput/memory/backend evidence. A failed Vulkan correctness probe must select CPU and remain a passing degraded mode.

EXECUTION ROUND ORDER
Round 1: qwen35-model-runtime
Round 2: qwen35-mobile-vision, qwen35-local-tools
Round 3: qwen35-device-proof

COMMANDS TO RUN
/opsx:new qwen35-model-runtime
/opsx:new qwen35-mobile-vision
/opsx:new qwen35-local-tools
/opsx:new qwen35-device-proof

TRADE-OFFS AND SCOPE CUTS
- The Android Hexagon NPU/LiteRT-LM path is deferred; this phase uses the existing llama.cpp abstraction with Vulkan correctness testing and CPU fallback.
- The iPhone 9B model must pass an app-memory preflight; an automatic 4B fallback is required rather than allowing iOS jetsam termination.
- PDF/Office binary understanding remains document extraction/preview work; this phase's native multimodal scope is raster image input.

PLAN COMPLETE
