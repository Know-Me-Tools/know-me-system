ASSESSMENT: on-device-qwen35
Project: KnowMe
Date: 2026-07-18T20:22:42Z
Codebase baseline: Mobile local chat is functional through a Rust `InferenceProvider` backed by llama.cpp, but it is pinned to text-only Qwen2.5-0.5B with CPU defaults and a 128-token output ceiling.
Cross-tool progress: none

IMPLEMENTATION STATUS
- Per-device model catalog: [MISSING] — one compile-time Qwen2.5-0.5B catalog entry is shared by iOS and Android; there is no platform/device selection or Qwen3.5 artifact metadata.
- Verified model acquisition: [PARTIAL] — the existing download is revision- and SHA-256-pinned and atomically published, but it is not resumable, reads the entire artifact for hashing, exposes no progress, and has no projector artifact.
- iOS Metal inference: [STUB] — `llama-cpp-2` exposes Metal and GPU-layer APIs, but the mobile dependency does not enable the Metal feature and model loading uses default CPU parameters.
- Android accelerated inference: [STUB] — `llama-cpp-2` exposes Vulkan and backend discovery, but the Android build does not enable Vulkan, request GPU layers, test correctness/performance, or persist a CPU fallback decision.
- Useful generation limits: [MISSING] — boot persists 2,048 context tokens and 128 output tokens; the agent defaults repeat the 128-token limit.
- Multimodal image inference: [MISSING] — Flutter passes attachment paths through the correct Widget -> provider -> repository -> FFI layering, but the local agent path converts binary images to textual metadata. `InferenceProvider::generate` accepts only `&str`; no image/projector contract reaches llama.cpp `mtmd`.
- Structured tool calling: [MISSING] — cloud requests receive MCP tool definitions, while local inference receives a flattened prompt and streams text only. There is no schema injection, constrained decoding, structured parser, permission gate, tool execution, or continuation loop for local inference.
- Physical-device verification: [MISSING] — the existing integration test checks that any local assistant bubble finishes; it does not assert the selected model, acceleration backend, vision response, tool call, throughput, memory pressure, or thermal fallback.

CROSS-TOOL PROGRESS
- NONE — no implementation changes are recorded in the new phase progress file.

SPEC GAP SUMMARY
- The functional specification requires llama.cpp mobile inference with Metal on iOS and Vulkan on Android, but both acceleration features are disabled in the current mobile dependency graph.
- The specification's older Gemma-3n mobile default conflicts with the researched Qwen3.5 per-device decision and needs a dedicated OpenSpec change before this phase is considered aligned.
- The model library requirement calls for download, benchmark, RAM allocation, and acceleration management; only a silent first-use download exists today.
- The chat attachment surface exists, but local image understanding does not; multimodal data is discarded before reaching the inference engine.
- Tool-capable local models cannot exercise KnowMe tools because only the cloud lane builds tool definitions and executes the cloud streaming protocol.

BUILD HEALTH
- Rust inference check: [PASS] — `cargo check -p gen_ui_inference --features local-llama` completed successfully.
- Rust inference tests: [PASS] — `cargo test -p gen_ui_inference --features local-llama` completed, but the crate contains zero tests.
- Flutter analyze: [FAIL] — 62 existing issues originate in the vendored `mobile/rust_builder/cargokit/build_tool` package because its independent package dependencies are not resolved from the mobile analyzer invocation. No phase code has been changed yet.
- known violations: the configured blocking `flutter analyze` health command currently fails before this phase starts; vendored llama binding warnings also prevent treating the local inference crate as warning-clean.
- test coverage: [MINIMAL] — one physical integration flow exists for generic local text completion; no unit tests cover downloads, catalog resolution, acceleration selection, multimodal inputs, or tools.

CONSTRAINT CHECK
- AGENTS.md violations: NONE in the current local-inference data flow; inference and networking remain in Rust and Flutter attachment flow follows Widget -> Riverpod provider -> Repository -> FFI.
- constraints.md violations: build-health-passes is already failing because of the cargokit analyzer scope; tests-for-new-behavior is unsatisfied for the planned surface.

GOAL PROGRESS
- Qwen3.5-9B Q4_K_M plus projector on iOS/Metal: [NOT MET] — no artifact or Metal feature is configured.
- Qwen3.5-4B Q4_K_M plus projector on Android/Vulkan with CPU fallback: [NOT MET] — no artifact, Vulkan feature, preflight, or fallback is configured.
- Verified resumable downloads and device preflight: [PARTIAL] — SHA-256 and atomic publication exist for one legacy model; resume, progress, per-platform selection, and preflight are absent.
- Raise local context and output limits: [NOT MET] — current persisted defaults are 2,048 and 128.
- Multimodal image input and host-side structured tool calling: [NOT MET] — local attachments are textual fallbacks and local tools are not wired.
- Physical iPhone and Android verification: [NOT MET] — no phase-specific device evidence exists.

ASSESSMENT COMPLETE
