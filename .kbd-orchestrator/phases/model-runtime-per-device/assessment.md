# Assessment: model-runtime-per-device

Project: KnowMe
Date: 2026-07-18
Codebase baseline: The Rust core already exposes a llama.cpp-backed mobile inference seam, but the shipped implementation still downloads and loads a pinned Qwen2.5 0.5B text-only model with CPU-default parameters.

## IMPLEMENTATION STATUS

- Shared inference seam: **PARTIAL** — `gen_ui_types::inference::InferenceProvider` provides load/generate/unload, but `LocalModelSpec` has only a model id and context length; it has no platform, projector, backend, or capability fields.
- Current local model catalog: **PARTIAL** — `gen_ui_inference/src/llama.rs` hard-codes Qwen2.5-0.5B-Instruct-Q4_K_M, one revision, one SHA-256, and one URL. No Qwen3.5 4B/9B entries exist.
- Download integrity: **PARTIAL** — downloads are atomic and SHA-256 verified, but interrupted downloads do not resume and the downloader does not pin or validate content length.
- Model selection: **STUB** — boot migrates/creates a single `qwen2.5-0.5b-instruct-q4` preference. There is no device-aware iOS/Android catalog resolution or safe fallback policy.
- Native acceleration: **STUB** — `LlamaModelParams::default()` and `LlamaContextParams::default()` are used. `local-llama` does not enable the vendored crate's `metal`, `vulkan`, or `mtmd` features, and no GPU-layer policy or backend diagnostics are exposed.
- Multimodal inference: **MISSING** — the vendored binding contains an `mtmd` wrapper, but the application does not download a matching projector, accept typed image input in the inference contract, or merge image embeddings.
- Tool calling/agent continuation: **MISSING** — local generation receives one flat prompt and streams text. No tool schema formatting, structured-call parser, permission-aware execution loop, iteration bound, or tool-result continuation is connected.
- Generation limits: **PARTIAL** — the engine supports caller-supplied values, but current defaults are 2K context and a 128-token ceiling in the mobile call path. This is inadequate for office tasks, Mermaid/code output, and agent loops.
- Runtime diagnostics: **MISSING** — the selected model, backend, fallback decision, memory preflight, and throughput are not published to the mobile UI or integration tests.
- Physical-device verification: **MISSING** — existing OpenSpec task proposals describe iPhone and Android proof, but no current evidence records a Qwen3.5 load, vision request, tool call, backend, or fallback.

## CROSS-TOOL PROGRESS

- NONE — the new phase has no completed or in-progress changes recorded by another tool. Existing `openspec/changes/qwen35-*` proposals and task lists are unexecuted planning artifacts.

## SPEC GAP SUMMARY

- The functional spec describes llama.cpp through `llama-cpp-2` on mobile with Metal on iOS and Vulkan on Android, but the implementation currently uses CPU-default configuration and a Qwen2.5 0.5B model.
- The functional spec still names Gemma-3n-2B as the default mobile model in one roadmap section, while the current phase decision is Qwen3.5-9B for iPhone and Qwen3.5-4B for Android. The model catalog and docs need one authoritative policy.
- The inference trait is text-only. Adding vision and future tool calls requires a backward-compatible typed request/event contract in the Rust core; Flutter must remain a presentation/FFI client.
- The current model downloader is not resumable, so a multi-gigabyte mobile artifact can restart from zero after interruption.

## BUILD HEALTH

- Rust workspace check: **PASS** — `cd rust && cargo check --workspace` completed successfully in 17.11s.
- Known violations/warnings: the vendored `llama-cpp-2` crate emits existing missing-docs, deprecated API, unused-import, and dead-code warnings. No build errors were observed.
- Desktop and Flutter build health: **UNKNOWN** — not run during this fact-finding assessment.
- Test coverage: **MINIMAL** — there are Flutter chat/A2UI tests and local chat integration scaffolding, but no catalog, resume/checksum, backend-selection, projector, tool-parser, or physical-device tests for this phase.

## CONSTRAINT CHECK

- AGENTS.md violations: **NONE FOUND** in the inspected inference path. The current design keeps LLM/networking in Rust and leaves Flutter as the client layer.
- constraints.md violations: **NONE FOUND** in the inspected path. New work must preserve the Rust core boundary, typed interfaces, and focused tests.

## GOAL PROGRESS

- iPhone Qwen3.5-9B Q4_K_M + mmproj + Metal: **NOT MET** — no artifact, projector, feature flag, GPU policy, or runtime path exists.
- Android Qwen3.5-4B Q4_K_M + mmproj + Vulkan/CPU fallback: **NOT MET** — no artifact, projector, Vulkan feature, fallback, or diagnostics exist.
- Device-aware manifests/downloads/integrity/fallback: **PARTIAL** — one atomic SHA-256-verified catalog download exists, but it is neither device-aware nor resumable and has no fallback policy.
- Generation limits and tool-capable model plumbing: **PARTIAL** — shared sampling fields exist, but current mobile defaults are too small and structured tool execution is absent.

## ASSESSMENT COMPLETE
