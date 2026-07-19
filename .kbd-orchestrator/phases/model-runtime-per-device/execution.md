# EXECUTION: model-runtime-per-device

Project: KnowMe
Date: 2026-07-18
Selected backend: `openspec` via `/kbd-apply`
Dispatched to: SELF (Codex)
Backend rationale: OpenSpec is present and the phase changes already have traceable proposals/tasks. `/kbd-apply` is required so each task updates KBD progress and waypoint state.
Backend entrypoint: `/kbd-apply` task-by-task driver
OpenSpec available: YES
Source plan: `.kbd-orchestrator/phases/model-runtime-per-device/plan.md`

## EXECUTION SCOPE

- `android-litertlm-runtime`: Android LiteRT-LM Gemma 4 E2B catalog, verified downloads, lifecycle, and diagnostics
- `litertlm-native-adapter`: official LiteRT-LM Android SDK behind the Rust `InferenceProvider`
- `litertlm-local-tools`: bounded structured tool-call continuation
- `litertlm-device-proof`: physical Android build/deployment evidence and gate report

## DISPATCH CONTRACTS

- `android-litertlm-runtime` → `/kbd-apply android-litertlm-runtime`
  - Model class: frontier
  - Concrete model: `claude-sonnet-4-6` (per project model policy)
  - Model rationale: high-complexity cross-layer native runtime change
  - Progress file: `.kbd-orchestrator/phases/model-runtime-per-device/progress.json`
  - Handoff: complete each OpenSpec task through `kbd-apply`; run focused Rust checks before marking implementation complete

- `litertlm-native-adapter` → `/kbd-apply litertlm-native-adapter`
  - Model class: frontier
  - Concrete model: `claude-sonnet-4-6`
  - Model rationale: high-complexity JNI/FFI and native multimodal change
  - Progress file: `.kbd-orchestrator/phases/model-runtime-per-device/progress.json`

- `litertlm-local-tools` → `/kbd-apply litertlm-local-tools`
  - Model class: frontier
  - Concrete model: `claude-sonnet-4-6`
  - Model rationale: high-complexity security-sensitive agent/tool loop
  - Progress file: `.kbd-orchestrator/phases/model-runtime-per-device/progress.json`

- `litertlm-device-proof` → manual physical-device verification
  - Model class: frontier
  - Concrete model: `claude-sonnet-4-6`
  - Model rationale: external devices and deployment state require direct verification
  - Progress file: `.kbd-orchestrator/phases/model-runtime-per-device/progress.json`

## APPROVAL GATES

- Never activate an artifact before revision, byte length, and SHA-256 verification succeeds.
- Never execute a local tool call without registry and permission validation.
- Never mark Android local inference ready without a LiteRT-LM runtime gate report and physical-device self-test evidence.
- Never select GPU/NPU or device-specific LiteRT-LM artifacts until correctness, thermal safety, and measurable improvement are proven.

## FALLBACK CONDITIONS

- If OpenSpec task state cannot be synchronized, stop and restore the KBD apply path before editing further.
- If a model checksum or architecture check fails, discard the temporary artifact.
- If LiteRT-LM native adapter/delegate initialization or smoke output fails, use the documented CPU baseline or report a typed unavailable/error state with diagnostics.

## VERIFICATION REQUIREMENTS

- `cd rust && cargo check --workspace`
- Focused Rust tests for catalog, resume/checksum, LiteRT-LM diagnostics, backend selection, and inference contracts
- `cd mobile && flutter analyze` and affected Flutter tests
- Physical-device proof for the attached Android phone in the final Android change; iOS re-certification remains separate

## PROGRESS LEDGER

- [COMPLETED] `android-litertlm-runtime` — Codex via `/kbd-apply`
- [COMPLETED] `litertlm-native-adapter` — Codex via `/kbd-apply`
- [PENDING] `litertlm-local-tools` — Codex via `/kbd-apply`
- [COMPLETED] `litertlm-device-proof` — Manual physical-device verification

## OUTPUTS

- OpenSpec task state and verification records
- Runtime catalog/diagnostic implementation
- Physical-device evidence attached to the final change

## DEVICE EVIDENCE — ANDROID LITERT-LM CPU SMOKE

Date: 2026-07-19
Device: Samsung `SM S936U`, serial `R5GYB4AZD7A`, Android 16/API 36
Package: `tools.knowme.knowme`
Runtime: LiteRT-LM Android SDK, generic `gemma-4-E2B-it.litertlm`, CPU baseline

Verified:

- `flutter build apk --debug --target-platform android-arm64` succeeded.
- `scripts/android/verify-native-inference-gates.sh mobile/build/app/outputs/flutter-apk/app-debug.apk` passed G1/G2:
  - `libgen_ui_ffi.so` has no vendor GPU/OpenCL `DT_NEEDED`.
  - APK is arm64-only and contains `libgen_ui_ffi.so` and `liblitertlm_jni.so`.
  - APK does not bundle vendor GPU/OpenCL libraries.
- Fresh install/launch succeeded with process alive.
- `MainActivity` loads `libgen_ui_ffi.so` via `System.loadLibrary`, causing `JNI_OnLoad` to capture the JavaVM.
- The model artifact downloaded to app-private cache, matched byte length `2,588,147,712`, passed SHA-256 verification, and was published.
- LiteRT-LM loaded the cached artifact and initialized the Android SDK native runtime.
- Prompt smoke produced a visible assistant response: user prompt `Say%20only%20OK`, assistant response `OK`.

Key logcat evidence:

```text
LiteRT-LM model download progress ... bytes_received=2588147712 total_bytes=2588147712
LiteRT-LM model verification starting ... gemma-4-E2B-it.litertlm.part
LiteRT-LM model verified and published ... gemma-4-E2B-it.litertlm
LiteRT-LM model loading model=gemma-4-e2b-it-litertlm
LiteRT-LM native adapter loaded model=gemma-4-e2b-it-litertlm
LiteRT-LM generation starting
LiteRT-LM generation produced text chars=2
```

Known remaining gaps:

- The adapter currently returns one complete message instead of true token-level streaming callbacks.
- LiteRT-LM receives Rust-shortlisted tool declarations with automatic execution disabled.
- Returned LiteRT-LM tool-call candidates are preserved as canonical stream events and covered by Rust unit tests.
- Host-side tool-call validation, execution, result continuation, and bounded repair remain pending.
- Cancellation has not been certified during LiteRT-LM generation/load.
- The visible conversation still contains old error bubbles from earlier failed attempts; those are persisted historical errors, not current-run failures.

## TOOL-CALL PROPAGATION CHECKPOINT

Date: 2026-07-19

Implemented:

- Android `KnowMeLiteRtLmBridge` converts structured Rust `InferenceTool`
  entries into LiteRT-LM `OpenApiTool` providers.
- `automaticToolCalling=false` is preserved, so mutating or external tools are
  never executed inside the Android SDK.
- Rust parses LiteRT-LM `toolCalls` from the Android SDK response and emits
  canonical `ToolCallStarted`, `ToolCallDelta`, and `ToolCallComplete` events.
- `gen_ui_protocol` already converts those events into `ContentBlock::ToolUse`.

Verification:

- `cargo test -p gen_ui_inference --features local-litert-lm`
- `cargo test -p gen_ui_protocol`
- `flutter build apk --debug --target-platform android-arm64`
- `scripts/android/verify-native-inference-gates.sh mobile/build/app/outputs/flutter-apk/app-debug.apk`
- `flutter analyze lib`
- Physical-device no-regression smoke on `SM S936U`: cached model loaded and prompt `Say%20only%20OK` generated `OK`; logcat reported `tool_calls=0` for that prompt.

## LOCAL TOOL RESULT / VALIDATION CHECKPOINT

Date: 2026-07-19

Implemented:

- Added canonical `StreamEvent::ToolResult` so local, cloud, AG-UI, and A2UI
  paths have a typed host-result half for manual tool calling.
- `gen_ui_protocol::A2uiAdapter` maps `ToolResult` into the existing
  exhaustive `ContentBlock::ToolResult` variant.
- Android LiteRT-LM local generation now observes completed tool-call
  candidates, validates them against the turn's advertised tool shortlist,
  parses argument JSON, checks required fields, and checks shallow JSON-schema
  property types.
- Valid local tool calls fail closed with a typed `ToolResult` error until
  permission and mutability policy metadata exists. This deliberately prevents
  automatic MCP execution from a local model without confirmation policy.

## MCP TOOL POLICY METADATA CHECKPOINT

Date: 2026-07-19

Implemented:

- Extended `gen_ui_mcp::McpTool` to preserve MCP `title`, `annotations`,
  `execution`, and `outputSchema` fields from `tools/list`.
- Added shared `InferenceToolAnnotations` carrying `title`, `readOnlyHint`,
  `destructiveHint`, `idempotentHint`, and `openWorldHint`.
- Local inference tool definitions are now built directly from the MCP registry
  cache so those annotations survive into the LiteRT-LM/request boundary.
- Kept execution fail-closed. Per the MCP spec, annotations are hints and must
  not be treated as trustworthy by themselves; KnowMe still needs registry
  trust and confirmation policy before local automatic execution.

Source basis:

- MCP 2025-11-25 Tools spec: clients should keep a human in the loop, show tool
  invocations, and confirm sensitive operations.
- MCP 2025-11-25 Schema Reference: `ToolAnnotations` includes `readOnlyHint`,
  `destructiveHint`, `idempotentHint`, and `openWorldHint`; annotations are
  advisory and not a faithful safety guarantee for untrusted servers.

Verification:

- `cargo test -p gen_ui_protocol`
- `cargo test -p gen_ui_agent local_tool_gate -- --nocapture`
- `cargo test -p gen_ui_agent mcp_tool_annotations_survive_local_inference_mapping -- --nocapture`
- `cargo test -p gen_ui_mcp`
- `cargo test -p gen_ui_types structured_messages_preserve_roles_and_tools`
- `cargo check -p gen_ui_agent`
- `cargo test -p gen_ui_inference --features local-litert-lm`
- `flutter analyze lib`
- `flutter build apk --debug --target-platform android-arm64`
- `scripts/android/verify-native-inference-gates.sh mobile/build/app/outputs/flutter-apk/app-debug.apk`
- Physical-device no-regression smoke on `SM S936U`: installed the debug APK,
  launched `tools.knowme.knowme`, submitted `Say only OK`, and logcat reported
  `LiteRT-LM generation produced text chars=2 tool_calls=0 context="initial"`.

## LOCAL READ-ONLY TOOL EXECUTION CHECKPOINT

Date: 2026-07-19

Implemented:

- Added explicit MCP server trust metadata. Servers default to untrusted for
  local automatic execution.
- Registered KnowMe's own Forge/A2UI MCP registry as `TrustedLocal`; this trust
  is necessary but not sufficient for execution.
- The local LiteRT-LM host tool gate now executes only tool calls that satisfy
  all of:
  - server is trusted-local;
  - `readOnlyHint == true`;
  - `destructiveHint == false`;
  - `openWorldHint == false`;
  - model arguments pass the current shortlist/schema validation.
- Tool calls that are untrusted, mutating/destructive, open-world, malformed, or
  outside the advertised shortlist are rejected with typed `ToolResult` errors.
- Successful safe tool calls execute through the existing MCP `tools/call`
  bridge and emit `ToolResult` with the MCP result payload.

Verification:

- `cargo test -p gen_ui_mcp`
- `cargo test -p gen_ui_agent local_tool_gate -- --nocapture`
- `cargo test -p gen_ui_agent mcp_tool_annotations_survive_local_inference_mapping -- --nocapture`
- `cargo check -p gen_ui_agent`
- `cargo test -p gen_ui_types structured_messages_preserve_roles_and_tools`
- `cargo test -p gen_ui_inference --features local-litert-lm`
- `flutter analyze lib`
- `flutter build apk --debug --target-platform android-arm64`
- `scripts/android/verify-native-inference-gates.sh mobile/build/app/outputs/flutter-apk/app-debug.apk`

## TOOL-RESULT CONTINUATION CHECKPOINT

Date: 2026-07-19

Implemented:

- Added shared `InferenceToolResult` records for provider-neutral manual
  tool-result continuation.
- Added an optional `InferenceProvider::continue_with_tool_results` hook with a
  visible default error for providers that cannot continue a stateful local
  conversation.
- Implemented Android LiteRT-LM continuation through the active
  `Conversation` using `Content.ToolResponse` inside `Message.tool`.
- After a safe local MCP tool executes, `gen_ui_agent` publishes the
  `ToolResult`, sends the result back to LiteRT-LM, and publishes the final
  continuation stream into the same run.
- Rejected/unsafe tool calls do not continue the model; they remain typed error
  `ToolResult` blocks until confirmation UX/policy exists.

Verification:

- `cargo test -p gen_ui_agent local_tool_gate -- --nocapture`
- `cargo test -p gen_ui_types structured_messages_preserve_roles_and_tools`
- `cargo test -p gen_ui_inference --features local-litert-lm`
- `cargo check -p gen_ui_agent`
- `flutter analyze lib`
- `flutter build apk --debug --target-platform android-arm64`
- `scripts/android/verify-native-inference-gates.sh mobile/build/app/outputs/flutter-apk/app-debug.apk`

## FULL TOOL-SCHEMA VALIDATION CHECKPOINT

Date: 2026-07-19

Implemented:

- Added `jsonschema 0.48.1` as the shared Rust validator dependency with
  default external file/HTTP resolvers disabled.
- Replaced the shallow local tool argument validator with compiled JSON Schema
  validation.
- Invalid tool schemas and schemas requiring unsupported external resolution
  fail closed before local execution.
- Tests now cover nested required fields, enums, numeric constraints,
  `additionalProperties: false`, invalid schemas, and basic type mismatches.

Verification:

- `cargo test -p gen_ui_agent local_tool -- --nocapture`
- `cargo test -p gen_ui_types structured_messages_preserve_roles_and_tools`
- `cargo check -p gen_ui_agent`
- `cargo test -p gen_ui_inference --features local-litert-lm`
- `flutter analyze lib`
- `flutter build apk --debug --target-platform android-arm64`
- `scripts/android/verify-native-inference-gates.sh mobile/build/app/outputs/flutter-apk/app-debug.apk`
- Physical-device no-regression smoke on `SM S936U`: installed the debug APK,
  launched `tools.knowme.knowme`, submitted `Say only OK`, and logcat reported
  `LiteRT-LM generation produced text chars=2 tool_calls=0 context="initial"`.

## BOUNDED TOOL-REPAIR CHECKPOINT

Date: 2026-07-19

Implemented:

- Added a two-attempt repair budget for local LiteRT-LM tool calls whose
  arguments are malformed JSON or fail the advertised JSON Schema.
- Validation failures now emit a typed error `ToolResult` and are also sent back
  to the active LiteRT-LM conversation as an error `InferenceToolResult`, giving
  the model a bounded chance to correct the call.
- The continuation stream is now observed for additional tool-call events, so a
  corrected tool call produced after a validation error can be validated,
  executed if safe, and continued again with the tool output.
- The repair path is intentionally narrow. Unknown tools, untrusted servers,
  confirmation-required tools, destructive tools, and open-world tools do not
  trigger repair continuation.

Verification:

- `cargo test -p gen_ui_agent local_tool -- --nocapture`
- `cargo test -p gen_ui_types structured_messages_preserve_roles_and_tools`
- `cargo check -p gen_ui_agent`
- `cargo test -p gen_ui_inference --features local-litert-lm`
- `flutter analyze lib`
- `flutter build apk --debug --target-platform android-arm64`
- `scripts/android/verify-native-inference-gates.sh mobile/build/app/outputs/flutter-apk/app-debug.apk`
- Physical-device no-regression smoke on `SM S936U`: installed the debug APK
  over the current app data, launched `tools.knowme.knowme`, submitted a minimal
  prompt through adb input, and observed a visible local `OK` response in the
  chat UI. The adb input command inserted `%20` literally into the prompt, so
  this remains a runtime smoke test, not a clean prompt-certification artifact.

## EMBEDDED LOCAL TOOL CERTIFICATION CHECKPOINT

Date: 2026-07-19

Implemented:

- Added an embedded `knowme_diagnostics` MCP server during mobile boot.
- Registered it as trusted-local in the same `gen_ui_agent::state::mcp()`
  registry used by prompt assembly and local tool execution.
- Exposed one safe certification tool:
  `knowme_diagnostics__local_runtime_status`.
- The tool is argument-free, read-only, non-destructive, idempotent, and
  closed-world, so it is eligible for automatic execution under the existing
  host gate.
- The tool returns structured runtime status including runtime kind, platform,
  default local model, and safe-local-tool availability.

Verification:

- `cargo check -p gen_ui_ffi -p gen_ui_agent`
- `cargo test -p gen_ui_agent local_tool -- --nocapture`
- `flutter analyze lib`
- `flutter build apk --debug --target-platform android-arm64`
- `scripts/android/verify-native-inference-gates.sh mobile/build/app/outputs/flutter-apk/app-debug.apk`
- Physical-device boot on `SM S936U` logged
  `mobile diagnostics MCP tools registered server=knowme_diagnostics`.
- Physical-device tool-loop prompt on `SM S936U` produced the following logcat
  evidence:
  - `LiteRT-LM generation returned empty text tool_calls=1 context="initial"`;
  - `LiteRT-LM generation produced tool call id=litert-tool-0 name=knowme_diagnostics__local_runtime_status context="initial"`;
  - `LiteRT-LM tool-result continuation starting tool_results=1`;
  - `LiteRT-LM generation produced text chars=9 tool_calls=0 context="tool_result"`.

## FUTURE RELEASE ROADMAP CHECKPOINT

Date: 2026-07-19

Documented:

- Added
  `docs/strategies/local-models/android-native-inference-future-release-roadmap.md`.
- Captured the Android six-gate postmortem's durable process changes: automatic
  device-side proof, AgentSkills routing, fleet certification records, and
  richer local-agent prompt certification.
- Preserved the current LiteRT-LM-first direction. Vulkan/OpenCL remain explicit
  non-goals for the Android production baseline and may only return later as
  quarantined optimization experiments behind physical proof gates.

## TOOL CONFIRMATION VISIBILITY CHECKPOINT

Date: 2026-07-19

Implemented:

- Local tool policy results now include stable confirmation metadata for
  mutating, destructive, or open-world tool candidates:
  `requiresConfirmation`, `executionStatus`, `reasons`, and a nested
  `confirmation` policy object.
- Confirmation-required tool calls remain not executed. They do not trigger
  repair continuation and cannot be approved through ordinary model prose.
- Flutter now renders `local_tool_requires_confirmation` `ToolResultBlock`
  payloads as explicit approval-required/not-executed policy cards with
  human-readable reason chips and a copy action.
- The existing `submitInteraction` path is not used for approval execution,
  because it starts a normal chat turn. A future secure pending-tool-call resume
  contract is required before user-approved mutating/open-world execution can be
  enabled.

Verification:

- `cargo test -p gen_ui_agent local_tool -- --nocapture`
- `flutter analyze lib ../flutter_packages/gen_ui_widgets/lib test/features/chat/theme_accessibility_test.dart`
- `flutter test test/features/chat/theme_accessibility_test.dart`
- `cargo check -p gen_ui_agent -p gen_ui_ffi`
- `cargo test -p gen_ui_types structured_messages_preserve_roles_and_tools`
- `cargo test -p gen_ui_inference --features local-litert-lm`
- `flutter build apk --debug --target-platform android-arm64`
- `scripts/android/verify-native-inference-gates.sh mobile/build/app/outputs/flutter-apk/app-debug.apk`
- Physical-device launch on `SM S936U`: installed the APK, launched
  `tools.knowme.knowme`, observed `Loaded gen_ui_ffi through
  System.loadLibrary`, `mobile diagnostics MCP tools registered
  server=knowme_diagnostics`, and `mobile migrations ready`.

Remaining for `litertlm-local-tools`:

- Expand prompt certification beyond the deterministic diagnostics tool.

## SECURE TOOL APPROVAL / RESUME CHECKPOINT

Date: 2026-07-19

Implemented:

- Confirmation-required local tool results now include a host-generated
  `approvalId`.
- Rust stores the exact validated pending tool call in process state under that
  approval ID before returning the not-executed `ToolResult` to the model/UI.
- Added `gen_ui_agent::chat::approve_tool_confirmation` and FRB
  `chat_approve_tool_confirmation` so approval is a privileged host action, not
  a normal `chatAction`/model-prose interaction.
- Approval executes the stored MCP call arguments, publishes canonical
  `ToolUse`/`ToolResult` events for the continuation run, and resumes the active
  LiteRT-LM conversation through `InferenceProvider::continue_with_tool_results`.
- Flutter renders an `Approve and run` action only when the persisted policy card
  includes an approval ID. The chat screen routes `local_tool_approve` to the
  dedicated notifier/repository/FFI path and leaves other interactions on the
  generic A2UI path.
- The six-gate Android native-inference postmortem was folded into roadmap
  guidance as a future-backend verification standard. It does not change the
  production Android baseline back to llama.cpp, Vulkan, or OpenCL.

Verification:

- `cargo test -p gen_ui_agent local_tool -- --nocapture`
- `cargo check -p gen_ui_agent -p gen_ui_ffi`
- `cargo test -p gen_ui_types structured_messages_preserve_roles_and_tools`
- `cargo test -p gen_ui_inference --features local-litert-lm`
- `flutter analyze lib ../flutter_packages/gen_ui_widgets/lib test/features/chat/theme_accessibility_test.dart`
- `flutter test test/features/chat/theme_accessibility_test.dart`
- `flutter build apk --debug --target-platform android-arm64`
- `scripts/android/verify-native-inference-gates.sh mobile/build/app/outputs/flutter-apk/app-debug.apk`
- Physical-device launch on `SM S936U`: installed the APK, launched
  `tools.knowme.knowme`, observed `Loaded gen_ui_ffi through
  System.loadLibrary`, `mobile diagnostics MCP tools registered
  server=knowme_diagnostics`, and `mobile migrations ready`. No
  `AndroidRuntime`, `FATAL EXCEPTION`, `UnsatisfiedLinkError`, or KnowMe FFI
  `dlopen` failure was present in the launch smoke logs.

Remaining for `litertlm-local-tools`:

- Expand prompt certification beyond the deterministic diagnostics tool. The
  approval-required hardware certification is recorded in the next checkpoint.

## PHYSICAL TOOL APPROVAL / SKILL CHUNK CHECKPOINT

Date: 2026-07-19

Implemented:

- Added a second embedded trusted-local diagnostics MCP tool,
  `approval_required_echo`. It is read-only, non-destructive, and idempotent,
  but advertises `openWorldHint: true` so the existing local tool policy forces
  explicit approval instead of automatic execution.
- Fixed Flutter `ContentBlock.fromJson` decoding for Rust snake_case wire
  fields on tool-use, tool-result, and image blocks. This prevents
  `ToolUse.input_json`, `ToolResult.tool_use_id`, `ToolResult.output_json`,
  `ToolResult.is_error`, and `Image.data_base64` from failing in the UI stream.
- Added a typed `StreamEvent::SkillActivation` contract and mapped it through
  `A2uiAdapter` to the existing `ContentBlock::Skill` variant. Skill activation
  chunks are now part of the canonical semantic event plan, parallel to
  thinking, citation, tool, memory, lifecycle, and artifact chunks. They must be
  rendered outside assistant answer Markdown and persisted as ordered semantic
  blocks.

Verification:

- `cargo test -p gen_ui_ffi mobile_diagnostics_exposes_safe_and_approval_required_tools -- --nocapture`
- `cargo test -p gen_ui_protocol skill_activations_emit_skill_blocks -- --nocapture`
- `cargo test -p gen_ui_agent local_tool -- --nocapture`
- `cargo check -p gen_ui_agent -p gen_ui_ffi`
- `cargo check -p gen_ui_protocol -p gen_ui_types`
- `flutter analyze lib ../flutter_packages/gen_ui_widgets/lib test/features/chat/theme_accessibility_test.dart`
- `flutter test test/features/chat/theme_accessibility_test.dart`
- `flutter build apk --debug --target-platform android-arm64`
- `../scripts/android/verify-native-inference-gates.sh build/app/outputs/flutter-apk/app-debug.apk`
- Fresh install/launch of the rebuilt APK on `SM S936U`: logcat contained
  `mobile diagnostics MCP tools registered server=knowme_diagnostics` and
  `mobile migrations ready`, with no `AndroidRuntime`, `FATAL EXCEPTION`,
  `UnsatisfiedLinkError`, or `dlopen failed`.

Physical-device evidence:

- Device: Samsung `SM S936U`, serial `R5GYB4AZD7A`, Android 16/API 36.
- Installed and launched `tools.knowme.knowme/.MainActivity`.
- Prompted LiteRT-LM to call
  `knowme_diagnostics__approval_required_echo` with phrase `approval-two`.
- The native logs showed:
  - `LiteRT-LM generation produced tool call id=litert-tool-0 name=knowme_diagnostics__approval_required_echo context="initial"`
  - `LiteRT-LM tool-result continuation starting tool_results=1`
  - `LiteRT-LM generation returned empty text tool_calls=0 context="tool_result"`
- The Flutter UI rendered an approval-required policy card with an
  `Approve and run` action. After an explicit tap, the UI rendered
  `Tool use knowme_diagnostics__approval_required_echo` and `Tool completed`
  blocks. Expanded details contained `Approved diagnostic echo`, `approval-two`,
  and `approved_pending_tool_call`, proving that approval executed the
  host-stored pending call by approval ID and surfaced the tool output.

Remaining for `litertlm-local-tools`:

- Expand prompt certification beyond deterministic diagnostics tools.
- Expand post-tool final-answer certification beyond the diagnostics echo tool.
- Polish long tool-name layout in Flutter cards; the certified physical UI
  wraps long names awkwardly on narrow screens.

## TOOL CONTINUATION / EXPIRED APPROVAL ERROR CHECKPOINT

Date: 2026-07-19

Implemented:

- Android LiteRT-LM still sends the official `Message.tool(...)` continuation
  first. If that continuation returns neither text nor additional tool calls,
  the Android bridge now sends one bounded host-managed follow-up instruction in
  the same conversation asking the model to answer from the just-provided tool
  result. This addresses the observed silent post-approval finish without
  changing prompt ownership or allowing Flutter to assemble prompts.
- Expired or stale tool approvals no longer fail across FFI before the UI can
  subscribe. `approve_tool_confirmation` now returns a run ID and publishes a
  typed `RunError` with code `tool_approval_not_found` when the approval ID is
  no longer present in host state.
- Flutter sanitizes opaque persisted FRB `CoreErrorImpl` messages into a
  user-safe explanation that the local operation failed before details could be
  decoded and, for tool approvals, the approval may have expired after restart
  or reinstall.

Verification:

- `cargo test -p gen_ui_agent expired_tool_approval_publishes_typed_run_error -- --nocapture`
- `cargo test -p gen_ui_agent local_tool -- --nocapture`
- `cargo check -p gen_ui_agent -p gen_ui_ffi -p gen_ui_protocol -p gen_ui_types`
- `flutter analyze lib ../flutter_packages/gen_ui_widgets/lib test/features/chat/theme_accessibility_test.dart`
- `flutter test test/features/chat/theme_accessibility_test.dart`
- `flutter build apk --debug --target-platform android-arm64`
- `../scripts/android/verify-native-inference-gates.sh build/app/outputs/flutter-apk/app-debug.apk`

Physical-device evidence:

- Installed and launched the rebuilt APK on Samsung `SM S936U`.
- Navigated to Chat and verified persisted opaque approval-start failures no
  longer render `CoreErrorImpl`; the UI dump contained `approval may have
  expired` and `Retry the prompt`.
- Re-ran a clean active approval-required prompt in the same process:
  `Use approval_required_echo with phrase clean777. After approval answer clean777.`
- LiteRT-LM produced the active tool call:
  `LiteRT-LM generation produced tool call id=litert-tool-0 name=knowme_diagnostics__approval_required_echo context="initial"`.
- After tapping the visible `Approve and run` button, logcat showed
  `LiteRT-LM tool-result continuation starting tool_results=1` followed by
  `LiteRT-LM generation produced text chars=34 tool_calls=0 context="tool_result"`.
- The Flutter UI showed `Tool completed` and final assistant text
  `Approved diagnostic echo: clean777`. The UI dump contained `clean777`,
  `Approved diagnostic echo`, `Tool completed`, and
  `Tool use knowme_diagnostics__approval_required_echo`, with no
  `tool_approval_not_found` and no `CoreErrorImpl`.
- Launch/logcat smoke showed no `FATAL EXCEPTION`,
  `tools.knowme.knowme E AndroidRuntime`, `UnsatisfiedLinkError`, or
  `dlopen failed`.

Remaining for `litertlm-local-tools`:

- Force/certify the bounded empty-continuation fallback branch separately. The
  clean active approval run above did not trigger it because LiteRT-LM's direct
  `Message.tool(...)` continuation produced final text.
- Add a destructive cleanup affordance specifically for stale persisted approval
  cards if users should be able to remove only the stale policy card without
  deleting the whole assistant branch.

## TOOL CARD UI POLISH CHECKPOINT

Date: 2026-07-19

Implemented:

- Long tool names now render on a single ellipsized header line in Flutter tool
  cards instead of wrapping awkwardly across the card.
- Approval-required cards now include explicit process-local guidance:
  approvals can expire after app restart or reinstall, and the user should
  resubmit the prompt to create a fresh approval request.

Verification:

- `flutter analyze lib ../flutter_packages/gen_ui_widgets/lib test/features/chat/theme_accessibility_test.dart`
- `flutter test test/features/chat/theme_accessibility_test.dart`
- `flutter build apk --debug --target-platform android-arm64`
- `../scripts/android/verify-native-inference-gates.sh build/app/outputs/flutter-apk/app-debug.apk`

Physical-device evidence:

- Installed/launched the rebuilt APK on Samsung `SM S936U`.
- The rendered tool request card showed the long tool name as an ellipsized
  single-line `knowme_diagnostic...` label instead of the prior multi-line
  broken wrapping.
- Launch/logcat smoke showed no `FATAL EXCEPTION`,
  `tools.knowme.knowme E AndroidRuntime`, `UnsatisfiedLinkError`, or
  `dlopen failed`.

## TOOL RESULT FALLBACK OBSERVABILITY CHECKPOINT

Date: 2026-07-19

Implemented:

- The Kotlin LiteRT-LM bridge already applies a bounded fallback when
  `Message.tool(...)` returns no visible text and no additional tool calls: it
  sends one host-managed follow-up asking the model to answer from the just
  provided tool result.
- Rust now preserves the bridge's `toolResultFallbackApplied` flag across the
  JNI boundary and emits a canonical `StreamEvent::Custom` lifecycle record
  with source `litert_lm` and event name `tool_result_fallback_applied` before
  any fallback answer text.
- This keeps fallback behavior observable and persistable through the existing
  AG-UI/A2UI unknown/custom event path rather than hiding it inside Android
  bridge internals.

Verification:

- `cargo test --manifest-path rust/Cargo.toml -p gen_ui_inference --features local-litert-lm litert_lm -- --nocapture`
- `cargo test --manifest-path rust/Cargo.toml -p gen_ui_protocol -- --nocapture`
- `flutter build apk --debug --target-platform android-arm64`
- `../scripts/android/verify-native-inference-gates.sh build/app/outputs/flutter-apk/app-debug.apk`

Physical-device evidence:

- Installed/launched the rebuilt APK on Samsung `SM S936U`.
- Logcat showed `libgen_ui_ffi.so` loaded successfully through the app
  classloader and `Loaded gen_ui_ffi through System.loadLibrary`.
- Logcat showed `mobile migrations ready`, and `pidof tools.knowme.knowme`
  confirmed the app process remained alive after launch.
- No `FATAL EXCEPTION`, `tools.knowme.knowme E AndroidRuntime`, or
  `UnsatisfiedLinkError` was present. The launch log did contain Samsung's
  unrelated `libpenguin.so` graphics lookup failure, so this checkpoint does
  not claim a globally clean `dlopen failed` log.

Remaining:

- Force a physical-device run that actually triggers
  `tool_result_fallback_applied`; the clean `clean777` approval proof did not
  trigger it because LiteRT-LM's direct `Message.tool(...)` continuation already
  produced final text.

## ANDROID DEVICE RUNTIME GATE WRAPPER CHECKPOINT

Date: 2026-07-19

Implemented:

- Added `scripts/android/verify-device-runtime-gates.sh` as the physical-device
  counterpart to the existing static APK gate.
- The script installs the APK on the attached Android device, clears logcat,
  launches KnowMe, captures a bounded logcat file, verifies the app process is
  still alive, verifies `libgen_ui_ffi.so` loaded through
  `System.loadLibrary`, verifies Rust reached `mobile migrations ready`, and
  fails on scoped fatal/native-load errors.
- The script intentionally does not claim model prompt certification. G5/G6
  LiteRT-LM load/generation/tool certification remains a separate physical
  workflow layered on top of this launch gate.

Verification:

- `bash -n scripts/android/verify-device-runtime-gates.sh`
- `scripts/android/verify-native-inference-gates.sh mobile/build/app/outputs/flutter-apk/app-debug.apk`
- `scripts/android/verify-device-runtime-gates.sh mobile/build/app/outputs/flutter-apk/app-debug.apk`

Physical-device evidence:

- Device: Samsung `SM-S936U`, Android 16/API 36, ABI `arm64-v8a`, SoC `SM8750`.
- Gate output:
  - `G3 PASS: APK installed on attached Android device`
  - `G4 PASS: app launched and process stayed alive pid=12814`
  - `G4 PASS: libgen_ui_ffi loaded through System.loadLibrary`
  - `G4 PASS: Rust mobile boot reached migrations-ready state`
- Logcat artifact: `/tmp/knowme-device-runtime-gates.log`.

## ANDROID LOCAL MODEL PROMPT CERTIFICATION CHECKPOINT

Date: 2026-07-19

Implemented:

- Added `scripts/android/certify-local-model-prompt.sh` as the first physical
  model/prompt certification wrapper on top of the launch gate.
- The script runs `verify-device-runtime-gates.sh`, navigates to Chat with
  UIAutomator semantics plus bounded coordinate fallbacks, submits a fresh
  expected-token prompt through the Flutter composer, keeps the transcript
  pinned to `Latest`, captures logcat/UI XML, and fails unless LiteRT-LM starts
  generation, produces text, and the expected token appears in the visible UI.
- The script intentionally proves one minimum UI → Rust FFI → LiteRT-LM →
  visible ContentBlock path. It does not replace richer certification for
  Mermaid, code, tool calling, cancellation, restart recovery, or multi-device
  coverage.

Verification:

- `bash -n scripts/android/certify-local-model-prompt.sh`
- `scripts/android/certify-local-model-prompt.sh mobile/build/app/outputs/flutter-apk/app-debug.apk`

Physical-device evidence:

- Device: Samsung `SM-S936U`, Android 16/API 36, ABI `arm64-v8a`, SoC `SM8750`.
- Prompt: `Reply exactly KMOK042738.`
- Gate output:
  - `G5 PASS: LiteRT-LM generation started`
  - `G6 PASS: LiteRT-LM produced text`
  - `G6 PASS: expected visible UI text found: KMOK042738`
- Logcat artifact: `/tmp/knowme-local-model-certification.log`.
- UIAutomator artifact: `/tmp/knowme-local-model-certification.xml`.

## ANDROID LOCAL PROMPT SUITE CHECKPOINT

Date: 2026-07-19

Implemented:

- Added `scripts/android/certify-local-prompt-suite.sh` as a case-selectable
  physical prompt-suite wrapper around `certify-local-model-prompt.sh`.
- The suite currently defines `exact`, `office`, `mermaid`, `code`, and `json`
  cases. Each case generates a fresh marker, submits a UI prompt, and requires
  LiteRT-LM generation logs plus case-specific visible UI evidence.
- Adjusted the single-prompt gate to support multiple required visible
  substrings through `EXPECTED_SUBSTRINGS`, allowing rendered-block cases to
  assert structural UI evidence such as `Mermaid diagram` instead of only a raw
  answer token.
- Corrected non-tool prompt wording from `token` to `marker` so prompt
  certification cases do not accidentally trigger the approval-required
  diagnostics echo tool.

Verification:

- `bash -n scripts/android/certify-local-model-prompt.sh`
- `bash -n scripts/android/certify-local-prompt-suite.sh`
- `CASES=exact TIMEOUT_SECONDS=300 scripts/android/certify-local-prompt-suite.sh mobile/build/app/outputs/flutter-apk/app-debug.apk`
- `CASES=mermaid TIMEOUT_SECONDS=300 scripts/android/certify-local-prompt-suite.sh mobile/build/app/outputs/flutter-apk/app-debug.apk`
- `CASES=office,code,json TIMEOUT_SECONDS=300 scripts/android/certify-local-prompt-suite.sh mobile/build/app/outputs/flutter-apk/app-debug.apk`

Physical-device evidence:

- Exact case:
  - Prompt: `Reply exactly KMEXACT043202.`
  - Output: `G5 PASS: LiteRT-LM generation started`,
    `G6 PASS: LiteRT-LM produced text`, and
    `G6 PASS: expected visible UI text found: KMEXACT043202`.
  - Logcat artifact: `/tmp/knowme-local-model-certification-exact.log`.
  - UIAutomator artifact: `/tmp/knowme-local-model-certification-exact.xml`.
- Mermaid case:
  - Prompt: `Create a Mermaid graph TD from Start to End. Do not ask questions.
    Include marker KMMERMAID043921.`
  - Output: `G5 PASS: LiteRT-LM generation started`,
    `G6 PASS: LiteRT-LM produced text`, and
    `G6 PASS: expected visible UI text found: Mermaid diagram`.
  - Logcat artifact: `/tmp/knowme-local-model-certification-mermaid.log`.
  - UIAutomator artifact: `/tmp/knowme-local-model-certification-mermaid.xml`.
- Office case:
  - Prompt: `Write three agenda bullets for a project sync and include marker
    KMOFFICE044503.`
  - Output: `G5 PASS: LiteRT-LM generation started`,
    `G6 PASS: LiteRT-LM produced text`, and
    `G6 PASS: expected visible UI text found: KMOFFICE044503|agenda`.
  - Logcat artifact: `/tmp/knowme-local-model-certification-office.log`.
  - UIAutomator artifact: `/tmp/knowme-local-model-certification-office.xml`.
- Code case:
  - Prompt: `Write a Dart function called greet and include marker
    KMCODE044503 in a comment.`
  - Output: `G5 PASS: LiteRT-LM generation started`,
    `G6 PASS: LiteRT-LM produced text`, and
    `G6 PASS: expected visible UI text found: KMCODE044503|greet`.
  - Logcat artifact: `/tmp/knowme-local-model-certification-code.log`.
  - UIAutomator artifact: `/tmp/knowme-local-model-certification-code.xml`.
- JSON case:
  - Prompt: `Return JSON with fields status and marker. Include marker
    KMJSON044503.`
  - Output: `G5 PASS: LiteRT-LM generation started`,
    `G6 PASS: LiteRT-LM produced text`, and
    `G6 PASS: expected visible UI text found: KMJSON044503|status`.
  - Logcat artifact: `/tmp/knowme-local-model-certification-json.log`.
  - UIAutomator artifact: `/tmp/knowme-local-model-certification-json.xml`.

## ANDROID RESTART RECOVERY CHECKPOINT

Date: 2026-07-19

Implemented:

- Added `scripts/android/certify-local-restart-recovery.sh`.
- The script first delegates to the physical prompt gate to create a real
  LiteRT-LM response, then force-stops the app, relaunches it, and requires the
  generated marker to be visible from restored UI state.

Verification:

- `bash -n scripts/android/certify-local-restart-recovery.sh`
- `TIMEOUT_SECONDS=300 scripts/android/certify-local-restart-recovery.sh mobile/build/app/outputs/flutter-apk/app-debug.apk`

Physical-device evidence:

- Device: Samsung `SM S936U`, serial `R5GYB4AZD7A`, Android 16/API 36.
- Seed prompt: `Reply exactly KMRESTART045015.`
- Seed output: `G5 PASS: LiteRT-LM generation started`,
  `G6 PASS: LiteRT-LM produced text`, and
  `G6 PASS: expected visible UI text found: KMRESTART045015`.
- Relaunch output: `RESTART PASS: app relaunched and persisted marker is
  visible: KMRESTART045015`.
- Seed logcat artifact: `/tmp/knowme-local-restart-seed.log`.
- Seed UIAutomator artifact: `/tmp/knowme-local-restart-seed.xml`.
- Relaunch logcat artifact: `/tmp/knowme-local-restart-recovery.log`.
- Relaunch UIAutomator artifact: `/tmp/knowme-local-restart-recovery.xml`.

## ANDROID MULTI-TURN HISTORY CHECKPOINT

Date: 2026-07-19

Implemented:

- Added `scripts/android/certify-local-multiturn-history.sh`.
- The script delegates to the physical prompt gate to seed a real LiteRT-LM
  response with a marker, then submits a second prompt that does not include
  the full marker and requires the model/UI to recover it from the immediately
  previous assistant message.
- Tightened Android certification harness failure detection: `uiautomator dump`
  can crash with `UiAutomationService already registered` on this Samsung when
  dumps are invoked rapidly. The cert scripts now fail on a missing KnowMe
  process or KnowMe/native-load failures instead of treating arbitrary
  `uiautomator` `FATAL EXCEPTION` records as app crashes.
- Added local-run context diagnostics that log history count, roles, and text
  lengths before context assembly. The passing multi-turn run showed history
  reached Rust; no message text is logged.

Verification:

- `bash -n scripts/android/certify-local-model-prompt.sh`
- `bash -n scripts/android/certify-local-restart-recovery.sh`
- `bash -n scripts/android/certify-local-multiturn-history.sh`
- `cargo check --manifest-path rust/Cargo.toml -p gen_ui_agent`
- `cd mobile && flutter build apk --debug --target-platform android-arm64`
- `scripts/android/verify-native-inference-gates.sh mobile/build/app/outputs/flutter-apk/app-debug.apk`
- `TIMEOUT_SECONDS=300 scripts/android/certify-local-multiturn-history.sh mobile/build/app/outputs/flutter-apk/app-debug.apk`

Physical-device evidence:

- Device: Samsung `SM S936U`, serial `R5GYB4AZD7A`, Android 16/API 36.
- Seed prompt: `Reply exactly KMMULTI051210.`
- Seed output: `G5 PASS: LiteRT-LM generation started`,
  `G6 PASS: LiteRT-LM produced text`, and
  `G6 PASS: expected visible UI text found: KMMULTI051210`.
- Second prompt: `Read the immediately previous assistant message It contains
  one all caps codeword beginning with KMMULTI Reply with exactly that codeword
  then HISTORYOK Do not explain`.
- Second output: `MULTITURN PASS: second turn recovered prior marker from chat
  history`.
- Logcat artifact: `/tmp/knowme-local-multiturn-history.log`.
- UIAutomator artifact: `/tmp/knowme-local-multiturn-history.xml`.
- Seed logcat artifact: `/tmp/knowme-local-multiturn-seed.log`.
- Seed UIAutomator artifact: `/tmp/knowme-local-multiturn-seed.xml`.

Remaining:

- Add separate wrappers for cancellation, explicit tool-calling suites, and
  skill activation UI proof. Skill activations already map to
  `ContentBlock::Skill`; the remaining gate is physical evidence that a real
  activation renders and persists as its own ordered block beside thinking and
  citation chunks.

## BLOCKERS

- NONE at dispatch time

## REFLECTION HANDOFF

Reflect must consume per-change implementation, evidence, certification, and publication status, including Android LiteRT-LM gate reports, CPU/delegate selection, and iOS memory fallback observed during separate iOS re-certification.

## EXECUTION READY
