## Context

KnowMe currently has two incomplete halves: mobile local inference through `InferenceProvider`, and full agent behavior in UAR through an HTTP SDK. The UAR Rust SDK's `RuntimeBuilder` does not build application state, so selecting its `embedded` feature still cannot start a direct run. Mobile therefore defaults to `CloudService`, contrary to the sovereign/offline product contract.

UAR already contains the domain engine needed for the repair: `RunManager`, `Orchestrator::from_driver`, `RunManager::with_llm_driver`, provider/model registry, persistence traits, skill and native-tool registries, normalized event history, cancellation, governance, and A2UI continuation. KnowMe already adapts structured UAR LLM requests to its local inference providers.

## Goals / Non-Goals

**Goals:**

- One real UAR library API for desktop, Android, and iOS.
- Full offline agent execution with a host-supplied local LLM.
- Identical run, tool, skill, policy, memory, AG-UI, and A2UI semantics across embedded and HTTP deployments.
- Durable mobile state and stable direct FFI operations.
- Android and iOS differ only at the local-model driver implementation.
- Optional remote/BYOK providers remain available without becoming a prerequisite.

**Non-Goals:**

- Running an HTTP listener or sidecar inside a mobile application.
- Making every process-based or remote MCP transport work offline on mobile.
- Replacing Android LiteRT-LM or iOS Metal inference in this change.
- Duplicating UAR agent semantics in Flutter, React, or KnowMe stores.

## Decisions

### UAR exposes a transport-independent `EmbeddedRuntime`

UAR core will construct an application kernel containing its run manager, provider registry, persistence, skills, native tools, MCP registry, A2UI registry/replay backbone, sessions, and embedding interface. The SDK wraps and re-exports that kernel. Axum routes and the SDK HTTP client become adapters over equivalent operations.

Alternative: populate the broad server `AppState` in the SDK. Rejected because it couples mobile to listeners, rate limiters, filesystem watchers, compiler services, server auth, and other transport/deployment concerns.

### Host drivers are first-class providers

`EmbeddedRuntimeBuilder` requires a host `LlmDriver` plus provider/model metadata for offline runs. The builder registers that metadata in UAR and attaches the driver to `RunManager`. This preserves UAR-owned prompt, policy, agent, tool, and event behavior while allowing KnowMe to own hardware-specific model execution.

Alternative: call local inference outside UAR and project output afterward. Rejected because tools, skills, policy, A2UI continuation, and agent branching would not actually control inference.

### Shared kernel, platform-specific inference driver

`gen_ui_agent` implements the UAR host-driver contract over `InferenceProvider`. Android supplies the LiteRT-LM provider; iOS supplies the Metal/Qwen provider. Both are linked into the existing Flutter Rust library and accessed through the same repository/FFI surface.

Alternative: separate UAR integrations per mobile OS. Rejected because it guarantees semantic drift and duplicate bugs.

### Production embedded mode requires durable persistence

The builder accepts `Arc<dyn PersistenceLayer>`. In-memory persistence is allowed only when explicitly enabled for tests/development. Mobile production uses a durable provider in the application sandbox and stores agents, policies, settings, skills, knowledge, conversations, and replay metadata without a server.

### Direct streams use canonical UAR events

The SDK exposes run creation plus broadcast/history subscription directly. It does not create a parallel embedded event schema. KnowMe projects `NormalizedEvent` into its canonical AG-UI/A2UI blocks exactly as it does for service events.

### Mobile feature profile excludes server transports

The UAR and SDK add an `embedded-mobile` feature profile. Server binaries, Axum routes, process MCP transports, watchers, and desktop-only services are gated away wherever cross-compilation proves they are unnecessary. One long-lived Tokio runtime/kernel is created during Rust-core startup, not per chat request.

## Risks / Trade-offs

- **UAR has a broad unconditional dependency graph** → Gate transport-only modules incrementally and make Android/iOS target checks mandatory.
- **Persistence implementations may not be mobile-safe** → inject the durable provider and run restart/restoration tests on both platforms; fail closed rather than silently use ephemeral state.
- **Local models vary in tool-call quality** → UAR retains schema validation, bounded repair, approval policy, and explicit error events.
- **Native inference may not support immediate cancellation** → UAR cancellation remains authoritative; the host driver discards late output and unloads safely when necessary.
- **Agent features may invoke network tools** → resource capability metadata identifies network requirements and offline policy disables unavailable resources with visible events.
- **AGPL linking obligations** → the embedded SDK already documents that enabling the UAR runtime links the AGPL runtime; distribution follows that license.

## Migration Plan

1. Add and test the UAR core embedded builder with a deterministic host driver and in-memory persistence.
2. Replace the SDK stub with the real core and keep server startup behind an explicit server feature.
3. Add direct SDK tests for run streaming, tool loop, cancellation, A2UI continuation, and persistence operations.
4. Enable `embedded-mobile` in KnowMe and implement its UAR `LlmDriver` adapter over `InferenceProvider`.
5. Change mobile runtime selection to embedded-first and remove implicit cloud fallback.
6. Inject Android and iOS local providers and register model capabilities.
7. Cross-compile both mobile targets, then certify offline device/simulator prompts, tool calls, agents, persistence, and network-disabled startup.
8. Retain HTTP service mode as an explicit optional setting. Rollback is feature selection back to service mode; no persisted records are deleted.

## Open Questions

- Which existing durable UAR persistence provider has the smallest verified iOS/Android footprint after cross-compilation, or whether KnowMe should provide a thin adapter over its already-open local database.
- Whether process/remote MCP transports should be split into a separate crate after the first mobile feature-gating pass.
