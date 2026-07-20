# PLAN: model-runtime-per-device

Project: KnowMe
Date: 2026-07-18
OpenSpec available: YES
Changes to implement: 5

## CHANGE LIST (ordered)

1. `android-litertlm-runtime`: Replace Android Qwen/llama.cpp/OpenCL with LiteRT-LM Gemma 4 E2B.
   - Scope: Rust inference core, Android FFI feature selection, model cache, diagnostics
   - Depends on: NONE
   - Recommended agent: Codex
   - Est. complexity: High
   - Complexity score: High
   - Model class: frontier
   - Customer value: HIGH
   - Details: Android defaults to `gemma-4-e2b-it-litertlm` from `litert-community/gemma-4-E2B-it-litert-lm`, revision `9262660a1676eed6d0c477ab1a86344430854664`, file `gemma-4-E2B-it.litertlm`, size `2,588,147,712`, digest `181938105e0eefd105961417e8da75903eacda102c4fce9ce90f50b97139a63c`. CPU is the universal correctness baseline. Qualcomm SM8750-specific artifacts are optimization candidates only after physical-device proof.

2. `litertlm-native-adapter`: Link the official LiteRT-LM Android SDK behind `InferenceProvider`.
   - Scope: Native Android build, Rust/JNI adapter, streaming callbacks, prompt/template handling
   - Depends on: `android-litertlm-runtime`
   - Recommended agent: Codex
   - Est. complexity: High
   - Complexity score: High
   - Model class: frontier
   - Customer value: HIGH
   - Details: Use the official Android AAR through a Rust-owned JNI bridge so Flutter remains UI-only. Preserve structured messages, LiteRT-LM prompt-template application, constrained decoding, multimodal content shape, and manual tool-call loops. Direct C++ vendoring remains a future option if the Android SDK cannot expose required capabilities.

3. `litertlm-local-tools`: Add bounded structured tool-call continuation for Android local inference.
   - Scope: Rust agent/inference orchestration, MCP registry integration, AG-UI/A2UI events, parser and policy tests
   - Depends on: `litertlm-native-adapter`
   - Recommended agent: Codex
   - Est. complexity: High
   - Complexity score: High
   - Model class: frontier
   - Customer value: HIGH
   - Details: Send only shortlisted registered tool declarations. Detect model tool calls, validate schema and permissions, execute manually through the existing registry, stream tool events, and continue with tool results. Mutating tools require confirmation.

4. `litertlm-device-proof`: Verify the complete Android text, document, and safe-tool path on physical devices.
   - Scope: Rust diagnostics, Flutter integration tests, Android build/deployment evidence
   - Depends on: `litertlm-local-tools`
   - Recommended agent: Manual
   - Est. complexity: High
   - Complexity score: High
   - Model class: frontier
   - Customer value: HIGH
   - Details: Certify the attached Samsung first with the generic artifact on CPU, then test GPU/NPU and SM8750-specific artifacts only as optimizations. Record load time, throughput, backend, memory, thermal state, cancellation, Markdown/Mermaid, Xberg/OCR attachment flow, and one permitted tool call.

5. `document-uar-karpathy-knowledge-strategy`: Create and publish the evidence-backed UAR RAG and Karpathy wiki strategy.
   - Scope: architecture and research documents only
   - Depends on: NONE
   - Recommended agent: Roo Code (Architect mode)
   - Est. complexity: M
   - Complexity score: Medium
   - Model class: medium
   - Customer value: HIGH
   - Details: Review and finalize the assessment draft in `docs/strategies/knowledge-base-karpathy-wikis/README.md` and its persistent research package. Preserve current-source evidence, platform-specific placement, artifact authority, learning-loop gates, sync/privacy/licensing constraints, explicit contradictions, and a strict no-code acceptance boundary. The full change contract is in `knowledge-base-karpathy-wikis-plan.md`.

## EXECUTION ROUND ORDER

Round 1: `android-litertlm-runtime`
Round 2: `litertlm-native-adapter`
Round 3: `litertlm-local-tools`
Round 4: `litertlm-device-proof`

Independent document round: `document-uar-karpathy-knowledge-strategy` may run
in parallel with any runtime round because it changes architecture and research
artifacts only.

## SUPERSEDED WORK

Android Qwen GGUF, llama.cpp OpenCL, Vulkan, MTMD/projector support, and direct
vendor OpenCL dynamic-library loading are no longer the Android production plan.
They may remain only as quarantined developer fallback experiments until
LiteRT-LM certification is complete.

## DOCUMENT-ONLY SUPPLEMENT

The ordered creation/finalization plan for the UAR RAG and Karpathy wiki
strategy is `knowledge-base-karpathy-wikis-plan.md`. Its OpenSpec change is
`document-uar-karpathy-knowledge-strategy`.

## COMMANDS TO RUN

`/opsx:new document-uar-karpathy-knowledge-strategy`

## PLAN COMPLETE
