# UAR Future Release Roadmap for KnowMe

Date: 2026-07-19

This roadmap captures the future-release work created by the decision that
Universal Agent Runtime (UAR) owns real agent execution, skills, A2UI registry
management, tool governance, and general LLM/provider access. KnowMe's local
model work stays valuable, but it becomes a local provider/backend that UAR can
route to.

## Current baseline

- UAR is pinned as a source submodule at `rust/vendor/universal-agent-runtime`.
- Desktop Tauri can build and launch for demo use from the debug app bundle.
- iOS Simulator can build and launch the Flutter shell for demo use when the
  demo-only mobile boot bypass is enabled with
  `KNOWME_DEMO_SKIP_MOBILE_BOOT=true`.
- The demo iOS path required Flutter Rust Bridge version alignment back to
  `2.12.0` across Dart, Rust, and generated bindings. Do not upgrade FRB again
  without regenerating bindings and validating every mobile target.
- The demo iOS boot bypass is not product architecture. It exists only to keep
  the UI shell available while production mobile boot/runtime certification is
  moved into a future release gate.
- KnowMe can normalize representative UAR `uar.agui/1`, AG-UI SSE, and native
  normalized SSE event payloads into its provider-neutral `StreamEvent`
  contract.
- The same projection path then renders UAR reasoning, skill activation, tool
  lifecycle, text, RAG citation arrays, memory recall, error, completion, and
  unknown custom payloads as KnowMe A2UI/content blocks.
- KnowMe can now adapt in both directions at the local-provider seam:
  - KnowMe structured `InferenceRequest` → UAR `LlmRequest`-compatible payload;
  - UAR `LlmRequest`-shaped payload → KnowMe structured `InferenceRequest`;
  - KnowMe local `StreamEvent` → UAR `NormalizedEvent`-shaped payload.
- `gen_ui_agent::uar::UarLocalInferenceDriverBridge` wraps the adapter around a
  real platform-selected `InferenceProvider`, including Android LiteRT-LM, so a
  future embedded UAR `ExternalLlmDriver` can call local inference through a
  narrow host callback.
- The local-to-UAR normalized-event adapter is stateful for tool calls, so the
  eventual UAR `ExternalLlmDriver` stream can receive complete tool names and
  accumulated JSON arguments rather than lossy completion markers.
- UAR's low-level LLM-driver `NormalizedEvent` contract now has native
  `skill.activation` and `custom` variants. The UAR run manager maps those into
  existing domain `SkillActivated` and provider-event `Artifact` events, so the
  external local-provider path does not need to hide semantic chunks inside
  unrelated runtime-step payloads.
- UAR now has an opt-in injected-driver run path:
  `Orchestrator::from_driver(...)` and `RunManager::with_llm_driver(...)`.
  Service/default deployments still build `LiterLlmDriver`; embedded mobile can
  supply an `ExternalLlmDriver` backed by KnowMe's local provider bridge.
- `gen_ui_agent::uar::UarServiceRuntime` can create a UAR run through the UAR
  HTTP SDK, consume its SSE stream, and project events into KnowMe A2UI blocks.
- Desktop/web A2UI action responses can be submitted back to the active UAR run
  through the same `/api/uar/runs/{run_id}/artifact-response` endpoint UAR's
  own frontend uses.
- Flutter FFI and the Tauri plugin expose read-only UAR runtime status; desktop
  status now reports `service_runtime` once the effective UAR service
  configuration can load.
- Desktop/web can select the `uar` chat lane when a UAR service URL and compiled
  agent artifact are configured through the typed Tauri command surface.
- The React desktop shell now exposes that service path: Settings can save the
  UAR service URL, optional keychain-backed API key, compiled agent artifact
  JSON/path, and optional session id; the Chat lane switcher and Models screen
  show the `uar` lane but refuse to activate it until the service config is
  runnable.
- Desktop Settings can now probe the saved UAR service via its `/health`
  endpoint and surface HTTP/status feedback without creating a run or changing
  lanes.
- The Rust service facade is covered by a local UAR-compatible HTTP/SSE harness:
  it creates a run, consumes the service stream through the UAR SDK, projects
  streamed text into KnowMe A2UI events, submits an A2UI artifact response to
  `/api/uar/runs/{run_id}/artifact-response`, and calls the UAR cancellation
  endpoint at `/api/uar/runs/{run_id}/cancel`.
- The React desktop shell has a minimal A2UI artifact-response bridge: A2UI/input
  artifact data parts can collect a JSON object response, route it through the
  chat hook/store layer, invoke the typed Tauri interaction command, and preserve
  the resulting run/error lifecycle in the conversation. This is a bridge, not
  the final UAR component-registry renderer.
- Android local inference is still owned by the current LiteRT-LM path. It is
  not yet routed through UAR.

## Release DEMO-0 — Demo shell stabilization

Goal: preserve a reliable near-term demo surface without confusing demo
shortcuts with certified runtime architecture.

- Keep desktop Tauri demoable from the built debug `.app` and debug DMG.
- Keep iOS Simulator demoable from the built Flutter `Runner.app`.
- Keep FRB versions aligned with the currently generated mobile bindings:
  `flutter_rust_bridge = 2.12.0` and matching Rust crate/macros.
- Keep the simulator-only boot bypass behind
  `--dart-define=KNOWME_DEMO_SKIP_MOBILE_BOOT=true`.
- While that define is enabled, use the demo-only in-memory chat repository so
  the simulator shell can load conversations and render a canned streamed turn
  without touching the production Rust entity store or inference runtime.
- Do not ship the demo boot bypass in production mobile builds.
- Do not ship the in-memory demo repository as a product runtime path.
- Track the production fix separately:
  - mobile boot must initialize FFI, entity store, and runtime state without
    crashing;
  - startup must not require model download, embedding, Xberg, OCR, or
    inference;
  - native boot failures must surface as visible Flutter startup errors rather
    than aborting the process.
- Preserve the working evidence path:
  - desktop process running from the Tauri bundle;
  - simulator app installed with bundle id `tools.know-me.knowme`;
  - simulator screenshot showing the KnowMe shell after launch;
  - chat shell loads without an indefinite spinner in demo mode;
  - no `SIGABRT` or Flutter runtime version mismatch in the immediate launch
    log window.

Exit gate: the demo operator can launch desktop and iOS Simulator reliably from
the current workspace, with known caveats documented and no claim that mobile
runtime certification is complete.

## Release UAR-1 — Runtime facade and projection certification

Goal: make UAR visible as a first-class runtime boundary and add an explicitly
configured desktop/web service lane without changing mobile local execution yet.

- Keep `gen_ui_agent::uar` as the stable local boundary.
- Expose `UarRuntimeStatus` through FFI/Tauri so every platform can show:
  - runtime authority;
  - target access mode;
  - current integration state;
  - submodule commit;
  - whether mobile sidecars are allowed. **Done. Desktop status is
    configuration-aware and promotes to `service_runtime` when the UAR service
    config is complete.**
- Provide a service-mode facade for desktop/web/cloud UAR:
  - create run;
  - stream events;
  - cancel run;
  - submit A2UI artifact responses;
  - project UAR events into KnowMe A2UI. **Done at Rust seam level.**
- Provide typed desktop/web configuration for UAR service URL, keychain-backed
  API key, compiled agent artifact, and optional session id. **Done through the
  Tauri command surface and React desktop Settings UI. The saved config also has
  a non-destructive health probe for live-service readiness checks.**
- Add a non-default `uar` chat lane for desktop/web that uses that service
  facade and remaps projected UAR events back to the KnowMe run id. **Done at
  Rust seam level and exposed in the React desktop lane/model selectors.**
- Add golden fixtures from UAR's `uar.agui/1` profile:
  - reasoning;
  - text;
  - tool call lifecycle;
  - tool result;
  - `uar.skill.activated`;
  - A2UI/artifact availability;
  - citation/memory events, including RAG citation arrays and memory recall;
  - unknown future custom events.
- Verify every fixture becomes an ordered KnowMe content-block projection and
  unknown events are preserved as artifacts. **Done at protocol-test level for
  reasoning, text, tool lifecycle, skill activation, A2UI artifacts, direct
  citations, RAG citation arrays, memory updates, memory recall, lifecycle
  artifacts, cancellation, native normalized SSE events, and unknown events.**
- Verify UAR driver payloads convert into KnowMe local-provider requests without
  losing structured roles, tool declarations, sampling settings, grammar, or
  context-plan provenance. **Done at protocol-test level.**
- Verify local-provider stream events convert into UAR normalized-event payloads
  with complete tool-call arguments. **Done at protocol-test level.**
- Verify the desktop/web service facade against a UAR-compatible HTTP/SSE
  surface:
  - create run;
  - stream SSE events;
  - project service events into KnowMe content blocks;
  - submit artifact responses;
  - cancel run. **Done with a local protocol harness.**
- Expose a React desktop bridge for A2UI artifact responses without letting
  components invoke Tauri directly. **Done minimally:** artifacts marked as A2UI
  or input-capable can submit a JSON object through `useChat().submitInteraction`,
  the store creates a visible continuation/error run, and the Tauri command
  validates/reroutes the structured payload.

Exit gate: protocol/component tests prove UAR event projection across Flutter,
React/web, and Tauri. Service-mode execution must additionally be proven
against a real UAR service, including saved service config, health probe,
lane switching, streaming, cancellation, and A2UI artifact response submission,
before desktop/web can be called runtime-certified. The local harness proves
KnowMe's protocol path, including cancellation and the minimal React artifact
response bridge, not the external service deployment or the final UAR component
registry.

## Release UAR-2 — Mobile embedded kernel

Goal: run the UAR agent loop in-process on mobile while preserving the existing
Flutter → Rust FFI boundary.

- Extract or stabilize a mobile-safe UAR embedded kernel profile:
  - no HTTP server requirement;
  - no local sidecar process;
  - no BossFang/LibreFang sidecar dependency on phones;
  - cancellable run manager hooks;
  - mobile persistence/config hooks;
  - provider registration traits;
  - in-process UAR event stream.
- Register KnowMe's Android LiteRT-LM/Gemma local lane as a UAR provider.
  The provider should use UAR's `ExternalLlmDriver` seam with
  `gen_ui_agent::uar::UarLocalInferenceDriverBridge` as the host callback.
- Construct the embedded/mobile UAR run manager with
  `RunManager::with_llm_driver(...)` so UAR owns the run/tool/skill loop while
  KnowMe owns only the local model bridge and lifecycle diagnostics.
- Keep Android CPU LiteRT-LM as the universal correctness floor.
- Keep GPU/NPU delegates optional and selected only after physical-device
  certification.
- Route one Android chat turn through UAR while preserving current lifecycle
  messages, cancellation, persistence, tool validation, and visible error
  semantics.

Exit gate: attached Android device completes a UAR-routed local chat turn with
the same evidence level as the current LiteRT-LM certification.

## Release UAR-3 — Real skills and A2UI registry

Goal: replace diagnostic skill simulation with actual UAR-owned skill and A2UI
behavior.

- Load/register a real UAR skill fixture on mobile.
- Emit `uar.skill.activated` for selected/running/complete/failed lifecycle.
- Render skill activation chunks outside answer Markdown, parallel to thinking
  and citation chunks.
- Use UAR's A2UI registry as the catalog authority.
- Fail closed on unknown or unsafe A2UI component ids.
- Cache approved mobile renderer profiles for offline rendering.
- Persist skill/A2UI blocks in `entity-records.sqlite3` with run/message
  provenance.

Exit gate: physical Android proof shows skill selection/execution, A2UI
artifact rendering, persistence, restart recovery, and cancellation.

## Release UAR-4 — Web and desktop convergence

Goal: make desktop and browser consume UAR through the service path while mobile
uses the embedded path.

- Desktop uses UAR over HTTP/SSE as a supervised service or sidecar.
- Web uses cloud/service UAR over HTTP/SSE.
- Both routes feed the same `UarEventProjector` behavior as mobile.
- React moves remaining agent/tool/prompt ownership out of UI stores and into
  UAR-backed typed service calls.
- Browser/PGlite remains a local projection/cache, not the owner of runtime
  truth.

Exit gate: identical UAR AG-UI fixtures produce identical ordered content
blocks on Flutter, React/web, and Tauri.

## Release UAR-5 — Cloud fallback and provider routing

Goal: allow UAR to choose local or cloud execution without hiding the backend
from the user.

- UAR receives local runtime diagnostics from KnowMe providers.
- UAR can route to:
  - Android LiteRT-LM local provider;
  - iOS local provider;
  - desktop local provider;
  - cloud provider;
  - cloud BossFang/LibreFang-accessible service.
- Mobile uses cloud access for capabilities that require BossFang/LibreFang or
  other sidecar-only services.
- User-visible lifecycle states show local/cloud routing, fallback reason,
  cancellation, and errors.

Exit gate: routing/fallback is visible, persisted, cancellable, and never
silently swaps local execution for cloud execution.

## Non-goals

- Do not reintroduce Android llama.cpp/OpenCL/Vulkan as the default path.
- Do not make Flutter own agent orchestration, tool policy, prompt assembly, or
  A2UI catalog decisions.
- Do not claim UAR mobile runtime certification while only the projection seam
  is implemented.
- Do not run BossFang/LibreFang as a local mobile sidecar.
