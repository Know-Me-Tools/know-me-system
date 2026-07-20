## Why

KnowMe's mobile UAR lane currently requires an HTTP service even though mobile devices already host capable local models. The UAR SDK claims embedded support but builds no in-process runtime state, so Android and iOS cannot run real agents offline.

## What Changes

- Replace the UAR SDK's stub embedded runtime with a fully initialized, transport-independent library runtime.
- Expose host injection for local LLM drivers, provider/model metadata, persistence, embeddings, MCP/native tools, skills, and A2UI registries.
- Route agent runs, normalized streaming, cancellation, tool approval, A2UI interaction continuation, agents, knowledge, memory, skills, and settings through direct Rust APIs.
- Embed the same UAR kernel inside KnowMe on Android and iOS through the existing Flutter-to-Rust FFI library.
- Register Android LiteRT-LM and iOS Metal-backed local models as UAR provider/model entries so the default and orchestrator agents function with no network.
- Keep remote UAR and Liter-LLM providers optional and BYOK-only.
- Add durable offline state, cross-compilation gates, and physical/simulator functional certification.
- **BREAKING**: `UarDeploymentMode` on mobile defaults to `EmbeddedLibrary`; service mode is no longer the implicit mobile fallback.

## Capabilities

### New Capabilities

- `embedded-uar-runtime`: A complete in-process UAR application kernel, direct SDK operations, host provider injection, and transport parity.

### Modified Capabilities

- `mobile-model-runtime`: Android and iOS local model runtimes become UAR-registered providers capable of offline agent execution rather than standalone chat-only engines.

## Impact

- UAR core and Rust SDK public APIs and feature profiles.
- KnowMe `gen_ui_agent`, `gen_ui_inference`, `gen_ui_ffi`, Flutter repositories/providers, and generated bindings.
- Android and iOS native build/link configurations.
- UAR persistence, provider catalog, run/event, AG-UI/A2UI, agent, skill, tool, memory, knowledge, and settings integration.
- Desktop embedded-runtime path and HTTP adapter parity tests.
