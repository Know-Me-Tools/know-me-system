# mobile-model-runtime Specification

## Purpose
TBD - created by archiving change qwen35-model-runtime. Update Purpose after archive.
## Requirements
### Requirement: Per-device Qwen3.5 model selection

The local runtime MUST select the pinned Qwen3.5 model for the current mobile
platform without requiring inference decisions in the Flutter UI layer.

#### Scenario: iOS selects the capable-phone model

- **WHEN** the runtime is initialized on iOS
- **THEN** it selects `qwen3.5-9b-q4-k-m` and its matching F16 multimodal projector
- **AND** it may fall back to the pinned 4B artifact when the 9B artifact cannot fit

#### Scenario: Android selects the thermally appropriate model

- **WHEN** the runtime is initialized on Android
- **THEN** it selects `qwen3.5-4b-q4-k-m` and its matching F16 multimodal projector
- **AND** it enables Vulkan with a CPU fallback when Vulkan is unavailable

### Requirement: Verified resumable model artifacts

The runtime MUST pin every model artifact by immutable repository revision, byte
length, and SHA-256, and MUST never activate an incomplete or corrupt file.

#### Scenario: Interrupted download resumes safely

- **WHEN** a partial `.gguf.part` file exists
- **THEN** the runtime requests the remaining byte range and appends only a `206`
  response
- **AND** it verifies the final byte length and SHA-256 before atomic activation

#### Scenario: Corrupt artifact is rejected

- **WHEN** an existing model file fails its pinned length or checksum
- **THEN** the runtime removes or replaces that artifact
- **AND** inference does not load it

### Requirement: Mobile inference backend diagnostics

The runtime MUST report the selected model, active backend, and any fallback reason
through the Rust inference service.

#### Scenario: GPU backend is selected

- **WHEN** Metal is available on iOS or Vulkan is available on Android
- **THEN** the runtime attempts full GPU layer offload
- **AND** diagnostics identify the selected backend

#### Scenario: CPU fallback is selected

- **WHEN** GPU initialization or memory preflight fails
- **THEN** the runtime retries with CPU execution or the pinned 4B fallback
- **AND** diagnostics include a non-empty fallback reason

### Requirement: Useful local generation defaults

The local runtime MUST use an 8K context and a 1,024-token output ceiling by
default while preserving explicit user preferences.

#### Scenario: Existing legacy preference is migrated

- **WHEN** persisted preferences contain the old 0.5B model or no model
- **THEN** boot migrates the model to the current platform default
- **AND** writes the new context and output defaults

#### Scenario: User preference is preserved

- **WHEN** a user has explicitly selected a supported model or generation limit
- **THEN** boot does not overwrite that selection
