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
- The APK native libraries have no vendor GPU/OpenCL hard dependency. Public
  Android graphics/runtime libraries required by the LiteRT-LM SDK, such as
  `libEGL.so`, `libGLESv2.so`, `libGLESv3.so`, and `libandroid.so`, are allowed;
  OEM OpenCL/vendor libraries remain forbidden in the startup-critical baseline.
- Mobile boot registers an embedded trusted-local `knowme_diagnostics` MCP
  server. It exposes a read-only, non-destructive, closed-world
  `local_runtime_status` tool for safe automatic execution and an
  `approval_required_echo` tool that is deliberately open-world so physical
  devices can certify the explicit approval/resume path without depending on
  Forge network availability.
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
- The canonical stream envelope now includes `ToolResult`, and the local path
  validates LiteRT-LM tool-call candidates against the advertised shortlist,
  JSON shape, required fields, and shallow property types. It then fails closed
  with a typed tool-result error until permission/mutation policy metadata is
  available for safe host execution.
- MCP `ToolAnnotations` now round-trip from `tools/list` into the shared
  `InferenceTool` contract. This preserves `readOnlyHint`, `destructiveHint`,
  `idempotentHint`, and `openWorldHint` for the future execution policy. These
  hints are treated as advisory only; local auto-execution still requires a
  trusted registry policy and confirmation handling.
- The host-side local tool gate now executes only the narrow safe subset:
  tools from a trusted-local MCP server that are explicitly read-only,
  non-destructive, and closed-world. Untrusted, mutating, destructive, or
  open-world tools produce typed `ToolResult` errors instead of running.
- Safe MCP tool results are now sent back to the active LiteRT-LM conversation
  using the Android API's `Content.ToolResponse` / `Message.tool` continuation
  path, and the continuation response is normalized into the same stream.
- Local tool-call argument validation now uses the shared Rust `jsonschema`
  validator with external resolvers disabled. Invalid or externally-referenced
  schemas fail closed; nested required fields, enums, ranges, arrays, and
  `additionalProperties` constraints are enforced before any safe tool executes.
- Validation failures for malformed JSON or schema-mismatched tool arguments
  are now repairable twice. The host returns a typed error `ToolResult` to the
  active LiteRT-LM conversation and observes the continuation stream for a
  corrected tool call. Trust, confirmation, destructive, and open-world failures
  are not repairable and remain terminal.
- Physical-device local tool certification now passes on the attached Samsung:
  LiteRT-LM emitted `tool_calls=1`, called
  `knowme_diagnostics__local_runtime_status`, received the host tool result, and
  produced a final continuation response.
- Physical-device approval-required local tool certification now passes on the
  attached Samsung for `knowme_diagnostics__approval_required_echo`: LiteRT-LM
  emitted the tool call, Flutter rendered the approval-required policy card,
  a visible `Approve and run` tap executed the host-stored pending call by
  approval ID, the UI rendered the resulting `ToolUse` and `ToolResult` blocks,
  and expanded details contained `Approved diagnostic echo`,
  `approved_pending_tool_call`, and the approved phrase.
- Expired or stale approval recovery is now explicit. If a persisted approval
  card survives a process restart/reinstall but its host-owned pending call is
  gone, approval returns a run ID and publishes a typed
  `tool_approval_not_found` error instead of leaking an opaque FRB
  `CoreErrorImpl` object into chat.
- Android's LiteRT-LM tool-result continuation now has a bounded fallback: the
  bridge sends the normal `Message.tool(...)` continuation first and only sends
  one host-managed follow-up instruction when that continuation is empty and has
  no additional tool calls.
- A clean active approval-required run has been physically certified with
  phrase `clean777`: the visible approval action executed the host-stored
  pending call, the direct `Message.tool(...)` continuation produced final text,
  and the UI rendered `Approved diagnostic echo: clean777`.
- The core stream contract now has a typed `SkillActivation` event that maps to
  the existing `ContentBlock::Skill` block. Skill activation chunks must be
  surfaced and persisted alongside thinking, citation, tool, memory, lifecycle,
  and artifact chunks instead of being demoted to opaque raw/provider artifacts.

This proves the CPU smoke lane and code-level tool-call candidate preservation.
It also proves one complete safe local tool loop and one explicit
approval-required local tool loop on a physical Android device. True
token-level streaming callbacks, cancellation certification, richer prompt
certification, empty-continuation fallback certification, stale-card cleanup
affordances, and broader device coverage remain future gates.

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
- [x] Parse model tool-call candidates through fail-closed validation.
- [x] Preserve MCP tool annotations in local inference tool definitions.
- [x] Register an embedded trusted-local diagnostics MCP server during mobile
  boot for offline-safe tool-loop certification.
- [x] Execute trusted-local read-only/non-destructive/closed-world tool calls and
  reject all other local tool calls with typed tool-result errors.
- [x] Feed safe tool results back into LiteRT-LM and continue generation.
- [x] Replace shallow JSON validation with the shared full schema validator used
  by executable tool loops.
- [x] Allow at most two bounded repair attempts for malformed JSON or schema
  mismatches, and observe corrected tool calls emitted during continuation.
- [x] Certify one physical-device tool loop where LiteRT-LM chooses a safe local
  tool, receives the host result, and continues generation.
- [x] Render mutating, destructive, or open-world tool candidates as explicit
  approval-required/not-executed policy cards instead of raw JSON errors.
- [x] Add a secure pending-tool-call resume contract for user-approved execution
  of mutating, destructive, or open-world tools. The host stores the exact
  validated pending call by approval ID, Flutter approval calls a dedicated FFI
  method, Rust executes the stored call, and the result resumes the active
  LiteRT-LM conversation.
- [x] Certify one physical-device approval-required local tool flow where the
  visible Flutter policy card executes only the host-stored pending call by
  approval ID and renders the resulting tool-use/tool-result blocks.
- [x] Convert expired/stale approval IDs into typed persisted run errors instead
  of pre-subscription FFI failures or opaque `CoreErrorImpl` UI bubbles.
- [x] Certify a clean active approval-required tool flow where LiteRT-LM's
  direct post-tool continuation produces a useful final assistant answer.
- [ ] Force and certify the bounded empty-continuation fallback path separately.
- [x] Persist tool-call and tool-result events with the same AG-UI/A2UI content-block
  contract used by other providers.
- [x] Persist skill activation events with the same AG-UI/A2UI content-block
  contract. A skill activation is a first-class semantic chunk, not assistant
  answer Markdown and not a raw artifact fallback.
- Require explicit confirmation before any mutating tool can execute.

### R3.1 — Turn-region UI semantics

- [x] Render `SkillActivation` as `ContentBlock::Skill` and keep it outside the
  answer bubble, parallel to thinking, citation, tool, memory, lifecycle, and
  artifact chunks.
- [x] Keep long tool and skill-like operational labels on a single ellipsized
  header line on narrow mobile screens.
- [x] Show process-local approval guidance on approval-required cards so stale
  restored cards are not mistaken for active host approvals.
- [ ] Add branch-local cleanup for stale persisted approval cards if users need
  to remove only the stale policy card without deleting the whole assistant
  branch.

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
  library or any other startup-critical APK native library.
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

## Relationship to the llama.cpp six-gate standard

The Android six-gate postmortem remains valuable, but it does not change this
release train back to llama.cpp, Vulkan, or OpenCL. Its rules apply in two
places:

- the static and physical verification gates for the current LiteRT-LM APK; and
- any quarantined future native-backend experiment.

Future llama.cpp/Vulkan/OpenCL lanes must ship as optional runtime-probed
modules with no startup-critical vendor `DT_NEEDED` entries, no bundled vendor
driver libraries, explicit physical-device smoke tests, and measured CPU
fallback. They are not part of the production Android recovery baseline.

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
- Local reference projects for implementation comparison live under
  `/Users/gqadonis/Projects/references`, including the LiteRT-LM and Google AI
  Edge Gallery codebases. Use those references when validating API usage,
  message/tool continuation behavior, cancellation hooks, and Android lifecycle
  integration.

References:

- [LiteRT-LM Android API](https://developers.google.com/edge/litert-lm/android)
- [LiteRT-LM C++ API](https://developers.google.com/edge/litert-lm/cpp)
- [LiteRT-LM API overview](https://developers.google.com/edge/litert-lm/api_overview)
