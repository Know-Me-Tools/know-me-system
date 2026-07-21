## ADDED Requirements

### Requirement: Embedded runtime is a complete in-process UAR

The UAR Rust library SHALL construct a transport-independent embedded runtime that owns agent execution, run lifecycle, provider selection, skills, tools, policies, persistence, AG-UI events, and A2UI state without starting an HTTP server.

#### Scenario: Host builds an embedded runtime

- **WHEN** a host supplies a local LLM driver, provider/model metadata, and durable persistence
- **THEN** the builder returns a ready embedded runtime with non-null run, provider, agent, skill, tool, persistence, and A2UI services
- **AND** no socket or sidecar is created

#### Scenario: Embedded runtime is not configured with a local driver

- **WHEN** offline embedded mode is built without a host LLM driver
- **THEN** construction fails with a stable configuration error
- **AND** it does not silently select an HTTP or cloud provider

### Requirement: Host local models are registered UAR providers

The embedded runtime SHALL accept a host `LlmDriver` and provider/model capability metadata and SHALL expose that model through the same registry and routing policy used for remote providers.

#### Scenario: Offline local provider is the default

- **WHEN** the host registers an enabled local provider/model as default
- **THEN** the UAR default and orchestrator agents resolve to that model without credentials or network access
- **AND** effective run policy identifies the local provider and model

#### Scenario: Optional remote providers are absent

- **WHEN** no user BYOK credentials are configured
- **THEN** local agent execution remains fully available
- **AND** remote providers are not selected as implicit fallback routes

### Requirement: Direct and HTTP execution preserve event semantics

The embedded API SHALL expose canonical normalized run events, replay, cancellation, approval, errors, and A2UI continuations with semantics equivalent to the HTTP/AG-UI adapter.

#### Scenario: Direct run streams an agent loop

- **WHEN** an embedded host starts an agent run and subscribes directly
- **THEN** lifecycle, thinking, text, skill activation, tool, citation, memory, artifact, error, cancellation, and completion events retain their order and payloads
- **AND** unknown custom events are preserved as artifacts

#### Scenario: A2UI action continues a run

- **WHEN** a user action targets an A2UI artifact from a completed or paused embedded run
- **THEN** UAR creates a real continuation run in the same conversation
- **AND** the action, state patches, tool effects, and resulting answer are replayable

### Requirement: Embedded data survives application restart

Production embedded mode SHALL persist agents, skills, settings, policies, memories, knowledge bases, conversations, run terminal state, and A2UI component/interaction data inside the application sandbox.

#### Scenario: Application restarts offline

- **WHEN** the mobile process is killed after persisted configuration and conversations exist
- **THEN** the next offline startup restores those records without contacting a service
- **AND** built-in default and orchestrator agents remain present and non-deletable

#### Scenario: Ephemeral persistence is requested for production

- **WHEN** a production embedded build is configured with only process-local persistence
- **THEN** runtime construction fails with a durable-storage-required error

### Requirement: Embedded runtime supports both mobile operating systems

The same UAR core and SDK API SHALL build and operate on `aarch64-linux-android` and `aarch64-apple-ios`; only the injected inference driver MAY vary by platform.

#### Scenario: Android operates without connectivity

- **WHEN** the Android application has its local model installed and all network access is disabled
- **THEN** the embedded default and orchestrator agents can complete prompts, tool loops, cancellation, and A2UI interactions

#### Scenario: iOS operates without connectivity

- **WHEN** the iOS application has its local model installed and all network access is disabled
- **THEN** the same embedded APIs can complete prompts, tool loops, cancellation, and A2UI interactions

### Requirement: Mobile embedding does not start transport-only services

The mobile feature profile MUST NOT require Axum listeners, sidecars, process-spawn MCP transports, filesystem watchers, or remote provider initialization to construct the embedded runtime.

#### Scenario: Mobile library initializes

- **WHEN** the shared Rust library initializes on Android or iOS
- **THEN** it creates one long-lived async runtime and embedded UAR kernel
- **AND** initialization performs no network I/O and binds no ports
