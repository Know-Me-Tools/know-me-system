# KnowMe ↔ Universal Agent Runtime Integration Strategy

Date: 2026-07-19

This supersedes any plan that implied KnowMe mobile would implement real agent
definition/execution, skill definition/storage, A2UI registry management, or
general LLM access independently. Those capabilities belong to Universal Agent
Runtime (UAR). KnowMe owns the product UI, mobile shell, local-device model
provider bridge, persistence projection, and platform-specific certification
evidence.

## Runtime ownership

| Capability | Owner | KnowMe responsibility |
|---|---|---|
| Agent definitions and execution | UAR | Render, select, configure, and invoke through the Rust boundary |
| Skill definitions, storage, routing, and eval fixtures | UAR | Surface activations as `ContentBlock::Skill` and persist projected blocks |
| A2UI execution loop, component catalog, and registry | UAR | Render approved surfaces, preserve source-run provenance, and return typed user actions |
| General LLM/provider access | UAR | Offer local-device providers and expose user-safe backend status |
| Tool governance and approvals | UAR policy/runtime | Preserve visible approval/cancellation/error semantics in chat |
| Mobile local inference | KnowMe local provider, exposed to UAR | LiteRT-LM/Gemma 4 E2B on Android; iOS local lane separately certified |
| Conversation UI and on-device projection | KnowMe | AG-UI/A2UI/content-block rendering, branch actions, local cache, UX |

## Deployment modes

### Flutter mobile

Mobile cannot depend on BossFang/LibreFang as a local sidecar. UAR must be used
as a library behind KnowMe's Rust FFI boundary.

Target shape:

```text
Flutter UI
  → Riverpod/repository
  → gen_ui_ffi
  → KnowMe Rust runtime facade
  → embedded UAR kernel
  → KnowMe local provider adapter
  → LiteRT-LM / platform local model
```

The UAR Rust SDK already exposes an `embedded` feature, but it currently links
the full AGPL runtime. Before mobile production, extract or stabilize a narrow
dependency-light embedded kernel profile with:

- no HTTP server startup requirement;
- no local sidecar process;
- mobile-safe persistence/config hooks;
- callbacks or traits for local LLM providers;
- typed UAR stream events available in-process;
- cancellation and checkpoint hooks compatible with app lifecycle.

### Desktop

Desktop may use UAR either as:

- a supervised sidecar/service over HTTP/SSE; or
- a linked library after the same embedded-kernel boundary is stable.

BossFang/LibreFang may supervise or call UAR out-of-process on desktop, but
KnowMe should still consume UAR through typed Rust contracts rather than letting
React own provider/tool/prompt orchestration.

### Web

Browser clients use UAR through HTTP/service access. They do not embed UAR.
Browser/PGlite state remains a local projection/cache of UAR events and
KnowMe-visible conversation entities.

### Cloud

Cloud UAR is the authoritative service for hosted agents, skills, tools, A2UI
catalog management, provider credentials, and governance. Mobile uses cloud UAR
when remote execution is selected or when a capability cannot run locally.

## Local model bridge into UAR

The Android LiteRT-LM work is not a competing agent runtime. It becomes a UAR
provider/backend:

```text
UAR agent/run manager
  → provider routing
  → KnowMeLocalProvider
  → gen_ui_inference::LiteRtLmProvider
  → Android LiteRT-LM SDK/JNI
```

Provider bridge requirements:

- Accept UAR structured messages, tool declarations, tool results, structured
  output constraints, sampling settings, cancellation tokens, and optional
  attachments/extracted text.
- Return UAR-compatible stream events for text, reasoning, tool calls,
  lifecycle, errors, cancellation, usage, and raw provider payloads.
- Preserve the KnowMe local-model lifecycle states:
  `notInstalled`, `downloading`, `verifying`, `loading`, `ready`,
  `cancelling`, `cancelled`, `failed`.
- Keep Android CPU LiteRT-LM as the universal correctness floor; GPU/NPU remain
  optional device-specific UAR provider variants only after certification.
- Expose local backend diagnostics to UAR so routing can decide whether a run
  should stay local, fall back to cloud, or ask the user.

Implementation status: `gen_ui_protocol` now contains a dependency-free
bidirectional local-provider bridge. It converts KnowMe `InferenceRequest`
values into UAR-compatible driver payloads, converts UAR `LlmRequest`-shaped
payloads back into structured KnowMe `InferenceRequest` values, and projects
local `StreamEvent`s into both AG-UI events and UAR `NormalizedEvent`-shaped
payloads. Tool-call streaming uses a stateful outbound adapter so final UAR tool
completion events preserve the exact tool name and accumulated JSON arguments.
`gen_ui_agent::uar::UarLocalInferenceDriverBridge` now wraps that protocol seam
around a real `InferenceProvider` trait object: it loads the selected local
model spec, calls the local provider with the structured request, and streams
UAR `NormalizedEvent`-shaped payloads back out. This does not wire embedded
mobile UAR yet; it makes the future UAR `ExternalLlmDriver` provider
registration a closure-level adapter instead of a prompt-flattening rewrite.
UAR itself now also exposes the matching opt-in runtime hook:
`Orchestrator::from_driver(...)` and `RunManager::with_llm_driver(...)`. The
default service path still constructs `LiterLlmDriver`; embedded/mobile hosts
can inject an `ExternalLlmDriver` without changing provider routing for cloud
or desktop service deployments.

## Event and block mapping

KnowMe's content-block work remains valuable as the cross-platform projection
layer for UAR events.

| UAR / AG-UI event | KnowMe projection |
|---|---|
| reasoning events | `ThinkingBlock` outside answer bubble |
| text deltas | Markdown/text answer blocks |
| tool call lifecycle/result | `ToolUseBlock` / `ToolResultBlock` |
| `uar.skill.activated` and process lifecycle | `SkillBlock` outside the answer bubble, parallel to thinking/citation chunks |
| memory/retrieval/citation events | `MemoryBlock` / `CitationBlock`; RAG citation arrays expand into ordered citation blocks and memory recall items remain inspectable as recall blocks |
| A2UI surface/state events | `ArtifactBlock(type: a2ui)` rendered through A2UI |
| unknown/custom provider payloads | `ArtifactBlock` with source/event/payload |
| cancellation/error | terminal run status bubbles, not fake assistant prose |

The rule stays strict: no UAR event is silently dropped. Unknown payloads become
inspectable artifacts.

## AG-UI/A2UI execution ownership

UAR owns the complete live artifact loop, not only the component registry. For
every A2UI-producing turn UAR must:

- create and persist the source run;
- emit the official AG-UI run, step, text, reasoning, tool, state, artifact,
  error, cancellation, and completion events;
- validate and publish A2UI surface messages and state patches;
- retain surface replay data for reconnect/restart recovery;
- accept typed A2UI actions against the durable source run;
- merge the submitted client data model and action context under UAR policy;
- create and stream the continuation run through the same AG-UI protocol; and
- own component-library promotion, versioning, and governance.

React and Flutter are deliberately thin clients in this loop. They render the
approved catalog, maintain only transient form state, persist the projected
conversation blocks, and send a typed action envelope containing
`sourceRunId`, `surfaceId`, `sourceComponentId`, action name/context, timestamp,
and the client data model. They do not synthesize continuation prompts, execute
agent logic, or guess which UAR run owns a restored surface.

The acceptance gate is an actual prompt → UAR run → A2UI surface → client
restart/reload → restored surface → typed action → UAR continuation → AG-UI
completion sequence. A display-only fixture or client-local callback does not
qualify.

## A2UI catalog integration

UAR owns the catalog registry and supported component profiles. KnowMe owns
rendering conformance:

- Flutter must implement the approved UAR/A2UI catalog components needed by the
  mobile experience.
- Unknown catalog ids or component names fail closed.
- Component registry updates come from UAR, but mobile can cache approved
  profiles for offline rendering.
- A component action always targets the persisted UAR source run; an ephemeral
  client run map may be used only as a same-session fallback.
- Renderer tests should use UAR golden fixtures where available, not
  hand-authored one-off examples only.

## Persistence policy

UAR is the source of truth for agent/skill/run definitions. KnowMe persists a
local projection for UX and recovery:

- conversation messages and ordered content blocks;
- run id, parent message id, branch id;
- terminal run state/error/cancellation;
- model/backend diagnostics;
- selected UAR agent/skill ids and event provenance;
- local-only model installation state.

When online sync is enabled, UAR run/checkpoint state reconciles with KnowMe's
local projection. Reasoning and skill process notes remain visible/persisted as
conversation artifacts but are excluded from future prompt context unless UAR
policy explicitly includes them.

## Phased implementation

### Phase 1 — Contract alignment

- Add the UAR submodule as a pinned dependency reference.
- Define a KnowMe `UarRuntimeFacade` trait with two implementations:
  - embedded/mobile;
  - HTTP/service. **Service facade exists at the Rust seam level through
    `UarServiceRuntime`; embedded/mobile remains future work.**
- Expose typed desktop/web service configuration and the non-default `uar` chat
  lane. **Done:** the Tauri command surface can save a UAR service URL,
  keychain-backed API key reference, compiled agent artifact, and optional
  session id; `gen_ui_agent::chat` routes that lane through `UarServiceRuntime`
  and submits A2UI action responses back through UAR's artifact-response
  endpoint. The Tauri UAR runtime-status command now reports `service_runtime`
  once the effective service config can load. The React desktop Settings screen
  now saves this config, while the Chat lane switcher and Models screen expose
  the `uar` lane only as a visible, guarded option until the service is runnable.
  Settings can also run a non-destructive `/health` probe against the saved UAR
  service config and show the result before a chat run is attempted.
- Define event mapping tests from UAR stream payloads to KnowMe `ContentBlock`
  variants.
- Keep the existing local LiteRT-LM path active while the facade is introduced.

### Phase 2 — Mobile embedded kernel

- Build or extract the UAR embedded kernel profile needed by Flutter mobile.
- Wire KnowMe's local LiteRT-LM provider as a UAR provider.
- Route one local Android chat turn through UAR while preserving the already
  certified LiteRT-LM prompt/tool/skill/cancellation semantics.

### Phase 3 — Real skills and A2UI registry

- Register a real UAR skill fixture in mobile.
- Certify visible `SkillBlock` lifecycle from UAR, not only MCP namespace
  derivation.
- Render a UAR-managed A2UI surface from the approved catalog.
- Persist and restore the UAR skill/A2UI blocks from `entity-records.sqlite3`.
- Keep UAR skill activation chunks first-class: selectable/copyable, ordered in
  the turn timeline, persisted separately from answer Markdown, and excluded
  from future prompt context by default unless UAR policy opts in.

### Phase 4 — Cross-platform convergence

- Desktop/web use HTTP/service UAR.
- Mobile uses embedded UAR with cloud fallback.
- All platforms share the same UAR event mapping and KnowMe projection tests.
- Fleet certification reports include UAR runtime mode:
  `embedded`, `service`, `cloud`, or `hybrid`.

## Immediate next gate

Before replacing the current Android local-chat path, prove the seam in a
non-invasive way:

1. Commit the UAR submodule pin. **Done:** KnowMe pins UAR under
   `rust/vendor/universal-agent-runtime`.
2. Add a compile-time/offline contract test fixture that maps representative
   UAR stream events into KnowMe content blocks. **Done:** `gen_ui_protocol`
   normalizes UAR `uar.agui/1` reasoning, skill activation, tool lifecycle,
   direct citations, RAG citation arrays, memory updates, memory recall, and
   unknown custom events into KnowMe stream/content blocks.
   `gen_ui_agent::uar` also projects UAR SDK SSE stream events, including direct
   `agui.rag_citations`, `agui.custom` envelopes, and native normalized SSE
   events, through the same adapter while preserving order.
3. Add the inverse local-provider driver seam. **Done:** UAR `LlmRequest`-shaped
   message/tool/settings payloads convert into KnowMe `InferenceRequest`, and
   local provider `StreamEvent`s convert back into UAR `NormalizedEvent`-shaped
   payloads with stateful tool-call argument preservation.
4. Add the host-side local provider bridge. **Done:**
   `UarLocalInferenceDriverBridge` can call any platform-selected
   `InferenceProvider`, including Android LiteRT-LM, from a UAR payload and
   return UAR normalized-event payloads.
5. Add the UAR injected-driver run hook. **Done:** UAR supports
   `Orchestrator::from_driver(...)` and `RunManager::with_llm_driver(...)`, so
   a host-provided `ExternalLlmDriver` can become the primary model stream for
   a run.
6. Exercise the desktop/web service facade against a deterministic
   UAR-compatible HTTP/SSE surface. **Done:** the local harness creates a run,
   streams events through the UAR SDK, projects them into KnowMe A2UI events,
   submits an artifact response to the UAR artifact-response endpoint, and
   verifies the UAR cancel endpoint.
7. Add a React desktop artifact-response bridge. **Done minimally:** A2UI/input
   artifacts can collect a JSON object response and submit it through the chat
   hook/store layer to the typed Tauri interaction command. Full UAR
   component-registry rendering remains future work owned by the UAR integration.
8. Add Android certification report fields for `uarRuntimeMode` and
   `uarSubmoduleCommit`. **Done:** the local certification report records UAR
   authority, target modes, current integration state, and submodule metadata.
9. Keep physical local-model gates green.

This preserves current Android progress while moving the architecture toward
UAR as the real agent runtime authority.

The next implementation checkpoint is the future-release roadmap in
`docs/strategies/uar-future-release-roadmap.md`: certify the desktop/web `uar`
lane against a live UAR service from the React UI, starting with the saved
service health probe, then lane switching, streaming, cancellation, and A2UI
artifact responses. The deterministic local HTTP/SSE harness has already proven
the KnowMe protocol path, including cancellation and the minimal React artifact
response bridge, so the remaining work is external service/runtime
certification, more UAR golden fixtures, the final UAR component-registry
renderer, and then the mobile embedded UAR kernel as the owner of real
chat/agent execution.
