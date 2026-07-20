# Analysis — Embedded UAR Offline Agents

## Outcome

The existing mobile architecture is not an embedded UAR. KnowMe links the UAR SDK with only `http-client`, selects `CloudService` on Android and iOS, and sends the UAR lane over HTTP. The SDK's `embedded` runtime is also incomplete: `RuntimeBuilder::build` returns `state: None`, while `Runtime::start` launches the Axum server. That API cannot provide offline mobile agent execution.

The corrective architecture is a transport-independent UAR `EmbeddedRuntime` assembled inside the UAR crate and exposed through its Rust SDK. It owns the same `RunManager`, agent artifacts, provider registry, skills, MCP/native tools, policy, persistence, normalized event stream, and A2UI continuation loop as the server. HTTP becomes an adapter over the same application services, not a prerequisite.

## Evidence

- `universal-agent-runtime-sdk/src/runtime.rs` documents direct access but constructs no `AppState`; every component accessor returns `None`.
- UAR already exposes the correct engine seams: `RunManager::with_llm_driver`, `Orchestrator::from_driver`, `ProviderRegistry::register_custom_provider`, normalized event history/replay, cancellation, and A2UI interaction continuation.
- KnowMe already has a local-model bridge from structured UAR requests to `InferenceProvider`, but it is not connected to an embedded UAR kernel.
- Android's official NDK guidance uses one app-owned native library loaded via `System.loadLibrary`, caches JNI state in `JNI_OnLoad`, and cautions against class lookup from native-created threads. The UAR therefore stays inside KnowMe's existing Rust FFI library instead of loading another runtime dynamically.
- iOS uses the same C/Rust library boundary through Xcode linking. The shared embedded kernel is platform-neutral; only its injected local-model driver differs.

## Architecture Decision

1. Add a first-class `EmbeddedRuntimeBuilder` to UAR core.
2. Require a host-supplied `Arc<dyn LlmDriver>` for offline mode and register its provider/model metadata in UAR's provider registry.
3. Allow host-supplied persistence, embeddings, MCP registry, native skills, and A2UI registry. Provide an in-memory deterministic default for tests, while mobile production injects durable storage.
4. Expose direct run, stream/replay, cancellation, interaction, agent, skill, provider, knowledge, and settings operations from the SDK without HTTP.
5. Make the SDK's `Runtime` wrap this real kernel. Keep `start_server` as an explicitly separate server feature.
6. Link the SDK `embedded-mobile` feature into `gen_ui_agent`; implement the host `LlmDriver` over KnowMe's existing `InferenceProvider`.
7. Android injects LiteRT-LM; iOS injects its Metal-backed local provider. Both use identical UAR code and Flutter FFI APIs.

## Risks and Mitigations

- **Broad server dependency graph:** introduce an embedded-mobile feature profile and progressively gate transport-only modules/dependencies. Cross-compile both `aarch64-linux-android` and `aarch64-apple-ios` as release gates.
- **Non-durable in-memory defaults:** production builders must reject ephemeral storage unless explicitly allowed for development/testing.
- **MCP process transports on mobile:** mobile embeds native/in-process tools; remote/stdio MCP remains optional and must advertise network/process requirements.
- **Runtime lifetime:** one application-owned Tokio runtime and one UAR kernel live for the process lifetime; no per-request runtime construction.
- **Event parity:** direct streams use the exact `NormalizedEvent` and A2UI replay backbone used by the HTTP adapter.

## Research Conclusion

No external framework should replace UAR. The strongest option is to adapt the existing UAR because it already owns the domain semantics and host-driver seams. Flutter Rust Bridge remains the established app boundary; Android JNI and iOS linking are packaging details beneath it. Cloud/service UAR remains optional BYOK functionality only.
