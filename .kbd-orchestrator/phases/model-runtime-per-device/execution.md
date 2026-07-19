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

Remaining for `litertlm-local-tools`:

- Add user confirmation UX/policy for mutating, destructive, or open-world
  tools.
- Expand prompt certification beyond the deterministic diagnostics tool.

## BLOCKERS

- NONE at dispatch time

## REFLECTION HANDOFF

Reflect must consume per-change implementation, evidence, certification, and publication status, including Android LiteRT-LM gate reports, CPU/delegate selection, and iOS memory fallback observed during separate iOS re-certification.

## EXECUTION READY
