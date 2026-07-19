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

Static packaging checks start with
`scripts/android/verify-native-inference-gates.sh`. Physical launch checks start
with `scripts/android/verify-device-runtime-gates.sh`, which installs the APK,
launches the app, captures logcat, proves the process stayed alive, proves
`libgen_ui_ffi.so` loaded through `System.loadLibrary`, and proves Rust reached
the mobile migrations-ready state. Minimum physical model/prompt smoke checks
start with `scripts/android/certify-local-model-prompt.sh`, which drives the
Flutter chat UI with a fresh token prompt and requires both LiteRT-LM generation
logs and visible UI text evidence.

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
- The Android fallback marker is preserved across the JNI/Rust boundary as a
  canonical custom lifecycle event with source `litert_lm` and event name
  `tool_result_fallback_applied`, so fallback use is visible in the normalized
  stream instead of being a hidden bridge behavior.
- A clean active approval-required run has been physically certified with
  phrase `clean777`: the visible approval action executed the host-stored
  pending call, the direct `Message.tool(...)` continuation produced final text,
  and the UI rendered `Approved diagnostic echo: clean777`.
- The core stream contract now has a typed `SkillActivation` event that maps to
  the existing `ContentBlock::Skill` block. Skill activation chunks must be
  surfaced and persisted alongside thinking, citation, tool, memory, lifecycle,
  and artifact chunks instead of being demoted to opaque raw/provider artifacts.
- A minimum physical prompt smoke now passes through the Flutter UI on the
  attached Samsung: the script submitted `Reply exactly KMOK042738.`, LiteRT-LM
  started generation, produced text, and the UI hierarchy contained
  `KMOK042738`.
- The richer prompt-suite wrapper now exists at
  `scripts/android/certify-local-prompt-suite.sh`. The `exact` case physically
  passed with `KMEXACT043202`, and the `mermaid` case physically passed by
  rendering a visible Mermaid diagram for a fully specified Start-to-End graph.
- The same wrapper also physically passed the `office`, `code`, and `json`
  cases on the attached Samsung. Those runs proved visible UI evidence for a
  three-bullet project-sync agenda with marker `KMOFFICE044503`, a Dart `greet`
  function with marker `KMCODE044503`, and constrained JSON containing `status`
  with marker `KMJSON044503`.
- Restart recovery now has a physical gate:
  `scripts/android/certify-local-restart-recovery.sh` seeds a local-model
  response, force-stops the app, relaunches it, and requires the marker to
  reappear from restored UI state. It passed on the attached Samsung with
  marker `KMRESTART045015`.
- Multi-turn history now has a physical gate:
  `scripts/android/certify-local-multiturn-history.sh` seeds a local-model
  response with a marker, sends a second prompt that does not contain the full
  marker, and requires the second response to recover the marker from the
  immediately previous assistant message. It passed on the attached Samsung
  with marker `KMMULTI051210`.
- Cancellation now has a physical gate:
  `scripts/android/certify-local-cancellation.sh` submits a long-running prompt,
  taps the visible Stop control after LiteRT-LM generation starts, and requires
  the cancelled state to render in the UI. It passed on the attached Samsung
  with visible `Cancelled by user` evidence.
- Explicit safe tool calling now has a physical gate:
  `scripts/android/certify-local-tool-call.sh` submits a prompt that requests
  `knowme_diagnostics__local_runtime_status`, requires visible tool-use,
  tool-result, and final continuation text, and asserts logcat contains the
  produced tool call plus tool-result continuation. It passed on the attached
  Samsung with visible `Tool use knowme_diagnostics__local_runtime_status`,
  `Tool completed`, and `TOOLSTATUSOK` evidence.
- Empty-continuation fallback now has a deterministic debug-build harness:
  `scripts/android/certify-local-tool-result-fallback.sh` writes a scoped
  sentinel into the app/LiteRT runtime cache, uses a fresh `KMFALLBACK...`
  marker, requires a current diagnostic tool call in logcat, and rejects stale
  UI evidence. It passed on the attached Samsung with marker
  `KMFALLBACK055421`; logcat proved the diagnostic tool call, forced fallback
  acknowledgement, and canonical `tool_result_fallback_applied` event, while
  UIAutomator proved the fallback event artifact rendered.
- Skill activation now has physical Android proof:
  `scripts/android/certify-local-skill-activation.sh` runs a clean local-tool
  turn, requires current `knowme_diagnostics__local_runtime_status` tool-call
  evidence, and verifies `Skill knowme_diagnostics, running` and
  `Skill knowme_diagnostics, complete` render as their own ordered
  `ContentBlock::Skill` regions outside the answer content. It passed on the
  attached Samsung with marker `KMSKILL060942`.
- Clean-session certification is now shared instead of wrapper-specific:
  `scripts/android/lib-chat-state.sh` backs up and restores only the
  app-private conversation databases while preserving the LiteRT-LM model
  cache. The prompt gate, prompt suite, explicit tool-call, cancellation,
  restart-recovery, and multi-turn wrappers expose `CERT_CLEAN_CHAT_STATE=1`.
  Restart recovery and multi-turn history own that clean session across their
  full lifecycle so the seeded message is not restored away before the
  assertion. A clean explicit tool-call run passed on the attached Samsung with
  visible `Tool use knowme_diagnostics__local_runtime_status`,
  `Tool completed`, and `TOOLSTATUSOK` evidence, and cleanup left no
  certification backup/sentinel artifacts.
- Branch action certification now has both notifier-level and physical Android
  proof. Resubmit now sends the selected user message with its original branch
  prefix history while creating a new branch; Delete removes only the selected
  assistant response plus later messages on that branch; Regenerate deletes the
  old assistant and reruns the saved parent user. The physical gate
  `scripts/android/certify-local-branch-actions.sh` passed on the attached
  Samsung by tapping Resubmit, Delete, and Regenerate in the Flutter UI and
  validating persisted `chat_message` branch records in
  `entity-records.sqlite3`.

This proves the CPU smoke lane and code-level tool-call candidate preservation.
It also proves one complete safe local tool loop and one explicit
approval-required local tool loop on a physical Android device. True
token-level streaming callbacks, stale-card cleanup affordances, broader clean
session/branch isolation for certification, and broader device coverage remain
future gates.

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
- [x] Preserve the Android bridge's empty-continuation fallback marker as a
  canonical custom lifecycle event so fallback use is observable and persisted.
- [x] Add a deterministic debug-build harness for the bounded
  empty-continuation fallback path that uses a fresh marker and rejects stale UI
  evidence.
- [x] Force and certify the bounded empty-continuation fallback path separately.
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
- [x] Certify one physical UI prompt smoke where a fresh expected token crosses
  Flutter → Rust FFI → LiteRT-LM → normalized `ContentBlock` rendering.
- [x] Add a repeatable physical prompt-suite wrapper.
- [x] Certify the Mermaid prompt-suite case on the attached Samsung.
- [x] Certify office/personal task, code, and constrained JSON prompt-suite
  cases on the attached Samsung.
- [x] Certify restart recovery by force-stopping/relaunching the app after a
  generated local-model response and requiring persisted UI restoration.
- [x] Certify multi-turn history with a second prompt that does not include the
  full marker and must recover it from the previous assistant message.
- [x] Certify cancellation through the visible Stop control and persisted
  cancelled bubble.
- [x] Certify explicit safe local tool-calling with visible tool-use/tool-result
  blocks and final continuation text.
- [x] Certify empty-continuation fallback cases.
- [x] Add reversible clean chat-state isolation to the fallback certification
  wrapper so stale restored messages cannot contaminate fallback proof.
- [x] Add physical UI certification for skill activation chunks so a real
  skill activation is visible as its own ordered `ContentBlock::Skill` region,
  parallel to thinking and citation chunks.
- [ ] Generalize clean session/branch isolation across the remaining physical
  prompt wrappers.
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
- Android native-library failures must be decoded by gate, not chased by log
  string alone: missing link directives, bundled vendor libraries, manifest
  visibility, classloader namespace, symbol binding, and backend capability are
  different failures and require different fixes.

References:

- [LiteRT-LM Android API](https://developers.google.com/edge/litert-lm/android)
- [LiteRT-LM C++ API](https://developers.google.com/edge/litert-lm/cpp)
- [LiteRT-LM API overview](https://developers.google.com/edge/litert-lm/api_overview)
