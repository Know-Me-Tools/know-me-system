# Android LiteRT-LM Six-Gate Roadmap

Date: 2026-07-19

This roadmap preserves the current Android direction: LiteRT-LM is the
production recovery lane, Gemma 4 E2B is the default text-agent model, and CPU is
the universal correctness baseline. The Android native-inference postmortem does
not change that backend decision. It changes the proof standard.

## Release principle

Android local inference is complete only when a physical device proves the full
runtime path. A successful Rust, Gradle, or Flutter build is evidence for
compilation only; it is not evidence that the backend loaded, selected the right
delegate, generated correct output, or recovered from failure.

Every Android native backend must therefore produce a gate report:

1. Rust/Flutter FFI library loads.
2. Model artifact is present in the APK or app-private cache.
3. Model artifact matches pinned revision, byte length, and SHA-256.
4. Universal CPU baseline initializes.
5. Optional LiteRT-LM delegate/native adapter initializes without hard vendor
   startup dependencies.
6. Backend self-test passes on the physical device.

The app may expose optional GPU/NPU acceleration only after gate 6 passes and
benchmarks show a real user-visible win. If any optional acceleration gate fails,
the app must fall back visibly to CPU.

## Current checkpoint — 2026-07-19

The Android CPU baseline now passes the first physical-device smoke test on the
attached Samsung `SM S936U` running Android 16/API 36:

- The APK is arm64-only and contains the KnowMe FFI library plus LiteRT-LM JNI
  library.
- The core FFI library has no vendor GPU/OpenCL hard dependency.
- The generic `gemma-4-E2B-it.litertlm` artifact downloaded to app-private cache,
  matched the pinned byte length, passed SHA-256 verification, and was published.
- `MainActivity` loads `libgen_ui_ffi.so` through `System.loadLibrary`, so
  `JNI_OnLoad` captures the JavaVM before Rust calls the Android SDK bridge.
- Rust resolves the LiteRT-LM bridge class through the app classloader when
  running on attached Rust worker threads.
- LiteRT-LM loaded the CPU baseline and generated a visible `OK` response from a
  local prompt.
- LiteRT-LM receives Rust-shortlisted tool declarations through the Android
  SDK's `OpenApiTool` provider path with automatic execution disabled.
- Returned LiteRT-LM `Message.toolCalls` are normalized into canonical
  `ToolCallStarted`, `ToolCallDelta`, and `ToolCallComplete` stream events.

This proves the CPU smoke lane and code-level tool-call candidate preservation.
It does not yet prove that the model reliably chooses tools for certified
prompts. True token-level streaming callbacks, host-side tool execution/repair,
cancellation certification, and broader device coverage remain future gates.

## Roadmap

### R1 — Safe LiteRT-LM baseline

- Keep Android on `LocalRuntimeKind::LiteRtLm`.
- Keep `gemma-4-e2b-it-litertlm` as the default generic artifact.
- Keep `gemma-4-e2b-it-litertlm-sm8750` as a device-specific candidate only.
- Download to a resumable `.part` file and activate only after byte-length and
  SHA-256 verification.
- Report `litert_lm_native_adapter_unavailable` until the official C++
  Conversation API bridge is linked.
- Expose a structured runtime gate report in diagnostics so UI and logs show the
  exact gate reached.

### R2 — Native LiteRT-LM Android adapter

- Link the official LiteRT-LM Android SDK behind the Rust `InferenceProvider`
  through a narrow JNI bridge.
- Keep Flutter as UI only; prompt construction remains owned by the Rust context
  fabric.
- Initialize CPU first.
- Treat GPU/NPU delegates as lazily probed optimization lanes, never startup
  requirements.
- Stream LiteRT-LM text, thinking/reasoning, tool-call candidates, errors, and
  raw/custom chunks into the canonical event stream.
- Run a deterministic backend self-test before marking the model ready.
- Keep direct C++ Conversation API vendoring as a future fallback only if the
  Android SDK cannot expose required streaming/tool/cancellation semantics.

### R3 — Tool-capable local agent loop

- [x] Shortlist registered tools before prompt assembly.
- [x] Provide LiteRT-LM with structured tool declarations.
- [x] Preserve model tool-call candidates as canonical stream events.
- [ ] Parse model tool-call candidates through schema validation.
- Enforce permission policy before execution.
- Allow at most two bounded repair attempts.
- Persist tool-call and tool-result events with the same AG-UI/A2UI content-block
  contract used by other providers.
- Require explicit confirmation for mutating tools.

### R4 — Device certification

- Certify the attached Samsung first using the generic CPU artifact.
- Record gate report, device identity, Android version, backend, load time,
  first-token latency, throughput, memory, thermal state, and cancellation
  behavior.
- Run prompt certification for English replies, office/personal tasks, Mermaid,
  code, constrained JSON, multi-turn history, tool calling, cancellation, and
  restart recovery.
- Certify the SM8750-specific artifact only after the generic CPU path is green.
- Add at least one non-Qualcomm modern Android phone before claiming broad
  Android support.

### R5 — Optional acceleration

- Probe LiteRT-LM GPU/NPU delegates at runtime.
- Do not hard-link or require vendor-specific GPU libraries from the core FFI
  library.
- Run delegate smoke tests on device before activation.
- Select GPU/NPU only when correctness, thermal safety, and median latency or
  throughput improvement are proven.
- Store fleet capability observations separately from conversation content.

## Non-goals for this release train

- Do not return Android production inference to llama.cpp OpenCL.
- Do not make Vulkan the Android production baseline.
- Do not submit images directly to the Android text-agent model; route
  images/PDFs/documents through Xberg/OCR/extraction first.
- Do not claim “works on Android” from build output alone.

## Source-backed adapter constraints

- LiteRT-LM Android initializes an `Engine` from `EngineConfig`, and Google
  explicitly warns model loading can take several seconds and should happen off
  the UI thread.
- Android GPU use requires optional `<uses-native-library>` declarations for
  `libvndksupport.so` and `libOpenCL.so` with `android:required="false"`.
- NPU use may require passing the app native-library directory into the backend
  configuration; it is not part of the CPU baseline.
- The Conversation API works message-in/message-out with structured roles,
  message content, preface/history, and tool declarations.
- Google Maven currently publishes the Android SDK as
  `com.google.ai.edge.litertlm:litertlm-android:0.14.0`.
- Manual tool calling is supported by disabling automatic tool calling, letting
  the host validate and execute tool calls before sending tool responses back.
- Async C++ callbacks deliver partial message chunks and a terminal empty
  message; the adapter must accumulate/normalize chunks into canonical events.

References:

- [LiteRT-LM Android API](https://developers.google.com/edge/litert-lm/android)
- [LiteRT-LM C++ API](https://developers.google.com/edge/litert-lm/cpp)
- [LiteRT-LM API overview](https://developers.google.com/edge/litert-lm/api_overview)
