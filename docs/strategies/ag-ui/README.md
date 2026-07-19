# AG-UI Event Transport Strategy — Profile, Normalization, and Completion Path

**Scope:** AG-UI as the event vocabulary between the Universal Agent Runtime (UAR)
and every KnowMe surface — the wire profile, the normalization seam into KnowMe's
provider-neutral contracts, what exists today in this repo, the verified gaps, and
the releases that close them.

**Status:** v1.1 strategy, 2026-07-19 (v1.1 reconciles with the service facade that
landed the same day). Grounded in the vendored UAR submodule
(`rust/vendor/universal-agent-runtime`, v1.0.0) and the existing seam in
`rust/crates/gen_ui_protocol`. The governing integration documents are
`docs/strategies/uar-knowme-runtime-integration.md` (ownership, deployment modes,
phases) and `docs/strategies/uar-future-release-roadmap.md` (releases UAR-1…5);
this doc owns the **event contract** those releases speak. Companions:
`docs/strategies/agents/README.md`
(placements, UAR grounding), `docs/strategies/a2ui/README.md` (the rendering
contract these events carry), `docs/strategies/local-models/android-agentskills-process-future-roadmap.md`
(mobile execution releases), PAGS-SPEC-KM-UIX-001 (the UI that consumes the stream).

---

## 1. Terminology — one sentence, never confused again

Per the UAR README: **AG-UI is the event transport vocabulary; A2UI is the
validated declarative rendering contract.** AG-UI answers "what happened in the
run"; A2UI answers "draw this surface." KnowMe's `ContentBlock` projection sits
*downstream* of AG-UI and must not be mistaken for either.

## 2. The wire profile (UAR, verified in the submodule)

UAR emits a profiled AG-UI event stream over SSE:

- **Profile id:** `uar.agui/1`; JSON Schema at
  `uar/schemas/uar-agui-v1.schema.json`
  (`$id: https://prometheus-ags.dev/schemas/uar-agui-v1.schema.json`).
- **Event families:** `RUN_STARTED` / `RUN_FINISHED` / `RUN_ERROR`;
  `STEP_STARTED` / `STEP_FINISHED`; `TEXT_MESSAGE_START` / `CONTENT` / `END`;
  `REASONING_*` (incl. `REASONING_MESSAGE_CONTENT`); `TOOL_CALL_START` / `ARGS` /
  `END` / `RESULT`; `STATE_SNAPSHOT`; `STATE_DELTA`; `MESSAGES_SNAPSHOT`; `RAW`;
  `CUSTOM` (e.g. `uar.skill.activated`).
- **Envelope fields:** `type`, `profile`, `eventId`, `sequence`, `threadId`,
  `runId`, `messageId`, `toolCallId`.
- **Compatibility:** UAR's compatibility policy keeps *normalized event profiles*
  Stable during 1.x, while explicitly **not** guaranteeing the Rust library API.
  Consequence for KnowMe: integrate at the **event-profile boundary**, never by
  reaching into UAR internals — this is the same discipline as the sidecar rule
  in the agents doc (§2.1 there) and it is what makes UAR upgrades reviewable.
- Custom agent/skill signals arrive as `CUSTOM` events with a `name` (e.g.
  `uar.skill.activated`) and a `value` payload; the profile is deliberately
  extensible there and only there.

## 3. What exists in this repo today (verified 2026-07-19)

The projection seam is already built and unit-tested; the live connection is not.

| Piece | Where | State |
|---|---|---|
| Provider-neutral internal contract `StreamEvent` (text/thinking deltas, tool-call lifecycle, skill activation, custom, done/error) | `rust/crates/gen_ui_types` | Working |
| UI envelope `A2uiEvent` (run_started, run_phase_changed, block, run_finished, run_cancelled, run_error, context_telemetry) | `gen_ui_types`; Dart mirror `mobile/lib/bridge/a2ui/a2ui_event.dart` | Working |
| **UAR ingress seam** `normalize_uar_agui_event` — maps `uar.agui/1` → `StreamEvent`, incl. `REASONING_MESSAGE_CONTENT` → `ThinkingDelta`, full `TOOL_CALL_*` lifecycle, `CUSTOM uar.skill.activated` → `SkillActivation`, unknown → `StreamEvent::Custom` | `rust/crates/gen_ui_protocol/src/lib.rs` | Working, unit-tested offline |
| `A2uiAdapter` — `StreamEvent` → `A2uiEvent`; cross-token `<think>` state machine (boundary-tested at every split point), tool-argument fragment accumulation, unclosed-reasoning protocol error, unknown events → `ArtifactBlock("provider_event")` | `gen_ui_protocol` | Working, unit-tested |
| `AguiAdapter` — `A2uiEvent` → `AguiEvent` (bidirectional seam) | `gen_ui_protocol` | Partial: `RunStarted`, `TextMessageContent`, `RunFinished` only |
| **Service-mode facade** `UarServiceRuntime` — creates UAR runs over the UAR HTTP SDK, consumes SSE, projects through the shared adapter (`UarEventProjector`) preserving order; submits A2UI artifact responses back to `/api/uar/runs/{run_id}/artifact-response`; `UarRuntimeStatus` exposed over FFI/Tauri (`embedded`/`service`/`cloud` mode + integration state + submodule commit) | `rust/crates/gen_ui_agent/src/uar.rs` | Working at the Rust seam (non-mobile builds); the desktop/web `uar` lane can route chat and A2UI action responses through a configured service, but live-service runtime certification remains open |
| FFI event stream to Flutter (JSON envelopes) + `ChatNotifier.streamBlock` + PEM persistence of messages/blocks | `gen_ui_ffi`, `mobile/lib/features/chat` | Working |
| Certified end-to-end proof on physical Android (embedded `knowme_diagnostics` MCP namespace → visible ordered blocks, persisted, restored) | see `docs/strategies/local-models/android-litert-lm-six-gates-roadmap.md` | Working for the MCP path |

## 4. The gaps (what "AG-UI implementation" still means)

1. **No production certification yet; embedded mode missing.** The submodule is
   excluded from the workspace build (`rust/Cargo.toml`) and linked through the
   HTTP SDK only. The *service-mode* facade exists at the Rust seam
   (`UarServiceRuntime`, `UarEventProjector` — `gen_ui_agent::uar`), and
   desktop/web can select the `uar` lane when `uar_service_url` plus a compiled
   `uar_agent_artifact` are configured. The projection is proven against
   fixtures and the SDK stream shape; it is not yet certified against a running
   UAR service. What remains: the **embedded/mobile kernel** (UAR-2 in the
   future-release roadmap; Phase 2 of the integration doc), production
   defaulting, and live service-mode certification.
2. **Envelope identity is dropped.** `eventId`, `sequence`, and `threadId` do
   not survive normalization; `StreamEvent`/`A2uiEvent` carry `run_id` only.
   Dedupe, ordering, replay, and the inspector's timeline (UIX-001 §10.6) all
   need them.
3. **Unmapped families.** `STEP_STARTED/FINISHED` (→ lifecycle named phases),
   `STATE_SNAPSHOT`/`STATE_DELTA` and `MESSAGES_SNAPSHOT` (→ inspector events /
   `ArtifactBlock` `agui_*` renderers, which already exist in
   `gen_ui_widgets`), `TEXT_MESSAGE_START/END` (block boundaries), and `RAW`
   (pass-through with provenance) have no mapping today.
4. **Bidirectional path incomplete.** Client→runtime traffic (user actions,
   confirmations, cancellations) needs the rest of `AguiAdapter`, or an
   explicit command channel — one of the two, documented (§5.4).
5. **Append-only event record.** The design standard (§7.5) requires the raw
   stream persisted for replay/audit; PEM stores materialized blocks today, not
   the raw envelopes.

## 5. Decisions and completion plan

### 5.1 Adopt `uar.agui/1` as the wire profile; keep three-layer normalization

`uar.agui/1` (wire) → `StreamEvent` (internal, provider-neutral) → `A2uiEvent`
(UI envelope) → `ContentBlock` projection. No layer is skipped and no layer
leaks upward: Flutter never sees AG-UI; UAR never sees `ContentBlock`. Extend
`normalize_uar_agui_event` in place — it is the canonical seam, with fixture
tests per event family. Other providers (llama.cpp direct, LiteRT-LM, cloud)
keep emitting `StreamEvent` natively; AG-UI is the *UAR* wire, not a
lowest-common-denominator rewrite of everything else.

### 5.2 Envelope preservation (release with the facade)

Add a `RunEnvelope { event_id, sequence, thread_id, run_id, message_id? }`
alongside `StreamEvent` in `gen_ui_types`, populated at ingress, carried to the
FFI envelope, and persisted with the raw record. Rules: `eventId` dedupes
(exactly-once projection), `sequence` orders (gap detection = protocol error,
never silent reorder), `threadId` scopes the run to a conversation.

### 5.3 Family mappings to land

| UAR family | KnowMe mapping |
|---|---|
| `STEP_STARTED/FINISHED` | Lifecycle named phase on `A2uiEvent::RunPhaseChanged` (the UIX-001 §6.4 strip) |
| `TEXT_MESSAGE_START/END` | Block boundary markers for the coalescing driver (semantic checkpoint triggers) |
| `STATE_SNAPSHOT` / `STATE_DELTA` / `MESSAGES_SNAPSHOT` | Inspector event records + existing `agui_*` ArtifactBlock renderers; never raw JSON into the answer bubble |
| `RAW` | `ArtifactBlock("provider_event")` with source/name/payload (existing path) |
| `CUSTOM` | Named registry of known customs (`uar.skill.activated`, …); unknown names → `ArtifactBlock("provider_event")`, never dropped |

### 5.4 Bidirectional: complete the command channel, not the adapter

Client→runtime intents (send message, cancel run, resolve confirmation, submit
A2UI action) are **commands**, not events. Keep `AguiAdapter` for the
event-shaped residue (thread state echoes), but route intents through typed
FFI/HTTP calls into the UAR facade — mirroring the sidecar rule: the same
command surface works embedded (mobile) and over HTTP (desktop/web).

### 5.5 Append-only event record

Persist raw envelopes (PEM `agui_events`: eventId, sequence, runId, threadId,
family, payload, timestamp) beside the materialized message blocks. Replay
rebuilds the thread from the record without contacting the model (UIX-001
§10.6). Checkpoints stay at semantic boundaries (`TEXT_MESSAGE_END`, tool
result, phase change) plus the existing debounce.

### 5.6 Test gates

- Fixture tests per family against the UAR JSON Schema (including malformed
  envelopes → typed protocol error, never a panic).
- Property test: arbitrary interleavings of reasoning/text/tool fragments never
  leak reasoning into answer blocks and never drop unknown events (extends the
  existing `gen_ui_protocol` test suite).
- Dedupe/order tests: replayed, duplicated, and gap'd envelopes.
- Physical certification rides the agent-skills roadmap's Release C (real
  installed AgentSkill emitting the full ordered block sequence).

## 6. Invariants (do not regress)

- The 11-variant `ContentBlock` union stays closed; new AG-UI families map to
  envelope/lifecycle/artifact channels, never to new block variants.
- Unknown payloads are preserved as `ArtifactBlock` records with provider,
  event name, and original payload — the protocol never silently drops.
- Reasoning quarantine: `REASONING_*` becomes `Thinking` blocks, displayed and
  persisted but excluded from future prompts, summarization, and fact
  extraction.
- Envelope fields are observability data, not user prose; the inspector
  renders them, the answer bubble never does.
