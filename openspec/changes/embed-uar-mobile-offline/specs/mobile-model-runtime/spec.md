## MODIFIED Requirements

### Requirement: Local tool and event semantics remain canonical

Android LiteRT-LM and iOS local inference providers MUST preserve the shared structured `InferenceProvider` request/event contract and MUST execute as host LLM drivers inside the embedded UAR agent loop.

#### Scenario: Tool schemas are available to local UAR inference

- **WHEN** an embedded UAR run selects registered tools for a local request
- **THEN** the platform local provider receives structured tool declarations
- **AND** UAR performs validation, permission policy, execution, and tool-result continuation

#### Scenario: Provider chunks are normalized

- **WHEN** a mobile local provider emits text, thinking, tool calls, tool results, errors, or custom chunks
- **THEN** the host driver maps them into UAR canonical normalized events
- **AND** skill activation, citations, memory, lifecycle, and A2UI events emitted by UAR remain ordered around model events
- **AND** unknown chunks are preserved instead of discarded

### Requirement: iOS remains separately certified

The Android LiteRT-LM migration MUST NOT silently change the iOS model/runtime lane, but iOS MUST use the same embedded UAR kernel and offline agent contract with its independently certified local driver.

#### Scenario: iOS initializes local inference

- **WHEN** the runtime is initialized on iOS
- **THEN** it keeps the certified iOS Qwen/Metal model selection
- **AND** registers that selection as a local UAR provider/model
- **AND** does not require an HTTP UAR service for agent execution

## ADDED Requirements

### Requirement: Mobile UAR mode defaults to embedded local execution

Android and iOS SHALL select `EmbeddedLibrary` as their default UAR deployment mode and SHALL require an explicit user setting before using a remote UAR service.

#### Scenario: Mobile starts with no network and no cloud keys

- **WHEN** the application starts with a verified local model and no configured remote service
- **THEN** chat exposes local and agent-defined modes through the embedded UAR
- **AND** the default/orchestrator agent is usable offline

#### Scenario: User opts into a remote provider

- **WHEN** the user configures BYOK credentials or an explicit UAR service URL
- **THEN** that remote route becomes an optional selectable provider
- **AND** local embedded execution remains available
