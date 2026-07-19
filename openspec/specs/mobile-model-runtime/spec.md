# mobile-model-runtime Specification

## Purpose

Define the current mobile local-inference runtime policy. Android uses LiteRT-LM
with Gemma 4 E2B as the universal recovery baseline. iOS retains the existing
Qwen/llama.cpp lane until it is re-certified separately.

## Requirements

### Requirement: Android selects LiteRT-LM Gemma 4 E2B by default

The Android local runtime MUST select a pinned LiteRT-LM `.litertlm` artifact
without requiring model or backend decisions in Flutter.

#### Scenario: Android selects the generic LiteRT-LM artifact

- **WHEN** the runtime is initialized on Android
- **THEN** it selects `gemma-4-e2b-it-litertlm`
- **AND** it resolves `gemma-4-E2B-it.litertlm`
- **AND** it treats CPU as the universal correctness baseline

#### Scenario: Device-specific artifacts are optimization candidates only

- **WHEN** the device reports Qualcomm SM8750
- **THEN** the runtime may expose `gemma-4-e2b-it-litertlm-sm8750`
- **AND** it MUST NOT select that artifact unless correctness, thermal, and
  performance certification has passed on the physical device

### Requirement: Android does not use llama.cpp OpenCL as production default

The Android production lane MUST NOT depend on llama.cpp OpenCL, Vulkan, or
vendor OpenCL dynamic-library loading for startup or baseline inference.

#### Scenario: Android build uses the default local runtime

- **WHEN** the Android FFI crate is built
- **THEN** it enables `local-litert-lm`
- **AND** it does not enable `local-llama-opencl`

### Requirement: Verified resumable model artifacts

The runtime MUST pin every model artifact by immutable repository revision, byte
length, and SHA-256/X-Linked-ETag, and MUST never activate an incomplete or
corrupt file.

#### Scenario: Interrupted download resumes safely

- **WHEN** a partial `.litertlm.part` file exists
- **THEN** the runtime requests the remaining byte range and appends only a `206`
  response
- **AND** it verifies final byte length and digest before activation

#### Scenario: Corrupt artifact is rejected

- **WHEN** an existing model file fails its pinned length or checksum
- **THEN** the runtime removes or replaces that artifact
- **AND** inference does not load it

### Requirement: Local tool and event semantics remain canonical

The Android LiteRT-LM provider MUST preserve the shared structured
`InferenceProvider` request and event contract.

#### Scenario: Tool schemas are available to local inference

- **WHEN** a local request includes registered tool schemas
- **THEN** LiteRT-LM receives structured tool declarations
- **AND** tool execution remains manual and permission-gated by the host

#### Scenario: Provider chunks are normalized

- **WHEN** LiteRT-LM emits text, thinking, tool calls, tool results, errors, or
  custom chunks
- **THEN** the provider maps them into canonical stream events
- **AND** unknown chunks are preserved instead of discarded

### Requirement: Mobile inference lifecycle is visible

The runtime MUST report model preparation, backend selection, errors, and
fallback reasons through shared lifecycle APIs.

#### Scenario: Model preparation cannot reach the Android adapter

- **WHEN** the LiteRT-LM Android SDK adapter is missing or JNI setup has not run
- **THEN** the runtime emits a stable `litert_lm_native_adapter_unavailable`
  error
- **AND** the UI can render that as an explicit error instead of hanging

### Requirement: Android native readiness is device-gated

The Android runtime MUST NOT treat a successful build as proof that local
inference works. It MUST expose a runtime gate report covering library loading,
artifact presence, artifact verification, CPU baseline initialization, native
adapter/delegate loading, and backend self-test.

#### Scenario: Native adapter is not reachable

- **WHEN** the Android LiteRT-LM provider resolves and verifies the model
- **AND** the official LiteRT-LM Android SDK adapter is not reachable
- **THEN** diagnostics include passed gates for FFI selection and artifact
  verification
- **AND** diagnostics mark native adapter loading as failed with
  `litert_lm_native_adapter_unavailable`
- **AND** diagnostics mark backend self-test as skipped

#### Scenario: Native adapter loads on Android

- **WHEN** the Android LiteRT-LM provider verifies the model
- **AND** the Rust JNI adapter initializes the official LiteRT-LM Android SDK on
  CPU
- **THEN** diagnostics mark native adapter loading as passed
- **AND** the runtime can generate a canonical stream containing message start,
  text delta or error, and done events

#### Scenario: Optional acceleration is unavailable or unproven

- **WHEN** a GPU, NPU, or device-specific LiteRT-LM artifact cannot pass the
  runtime self-test and benchmark gate on the physical device
- **THEN** the runtime keeps or falls back to the CPU baseline
- **AND** diagnostics include the fallback reason
- **AND** the UI can show that local inference is degraded rather than hidden

### Requirement: iOS remains separately certified

The Android LiteRT-LM migration MUST NOT silently change the iOS model/runtime
lane.

#### Scenario: iOS initializes local inference

- **WHEN** the runtime is initialized on iOS
- **THEN** it keeps the existing iOS Qwen/llama.cpp selection until the iOS
  re-certification phase replaces it
