# A2UI Strategy — Component Registry, Definition, Storage, and Selection

**Scope:** A2UI as KnowMe's single declarative UI plane — the component catalog
(vocabulary governance), where component/surface definitions live (definition),
how they persist and roam (storage), how a run picks and renders one
(selection), and the renderer conformance gate. This is the completion plan for
the "A2UI registry and component definition/storage/selection" gap.

**Status:** v1.1 strategy, 2026-07-19 (v1.1 aligns ownership with the integration
doc: UAR owns the catalog; KnowMe owns rendering conformance). Grounded in the vendored UAR submodule
(`rust/vendor/universal-agent-runtime` v1.0.0, `src/uar/a2ui/`) and the existing
Flutter renderer (`flutter_packages/gen_ui_widgets`). The governing ownership split
is `docs/strategies/uar-knowme-runtime-integration.md` ("A2UI catalog
integration"). Companions:
`docs/strategies/ag-ui/README.md` (the event transport that carries A2UI
traffic), `docs/strategies/plugins/README.md` §4 (`knowme:ui-library`
capability cards), `docs/strategies/agents/README.md` (agent UI surfaces),
PAGS-SPEC-KM-UIX-001 (turn-level rendering).

---

## 1. Terminology and the standing rule

**AG-UI is the event transport vocabulary; A2UI is the validated declarative
rendering contract** (UAR README). The standing security rule, from both UAR
and the plugins doc: **A2UI artifacts map to an approved component catalog and
never execute model-provided or plugin-provided HTML/JavaScript.** The
component vocabulary *is* the sandbox.

## 2. What exists today (verified 2026-07-19)

### In UAR (`src/uar/a2ui/`)

- `protocol.rs` — the validated production profile: **`uar.a2ui/1` over Google
  A2UI `v0.9.1`**, catalog id **`urn:uar:a2ui:catalog:1`**. Serde DTOs with
  `deny_unknown_fields`: `A2uiMessage` (`Create` / `Components` / `Data` /
  `Delete`) and a `Component` enum (`Text`, `Button`, `TextField`, `CheckBox`,
  `ChoicePicker`, …) — the enum *is* the approved catalog, enforced at
  deserialization.
- `schema.rs` — `ArtifactSchema`: the definition of one A2UI surface type.
- `registry.rs` — `A2uiRegistry`: resolves `a2ui_schema_id` → schema.
  **In-memory only.**
- `routes.rs` — REST: schema listing + user-response ingestion.
- Runtime flow: an agent declares UI surfaces in its UAR-AGENT-MD (§06); the
  runtime emits `agui.artifact_input_request` SSE events; the frontend renders
  form/confirm/select widgets; the user's response re-enters the paused run as
  a tool result.
- Reference renderers (React): `uar/frontend/packages/a2ui-core` (vendored
  Google `@a2ui/web_core`, version-pinned v0.8/v0.9), `a2ui-react`, `-lit`,
  `-svelte`, `-inspector`, `-uar`.

### In KnowMe

- **Flutter renderer exists:** `gen_ui_widgets`' `A2uiSurfaceView` renders
  Google A2UI v0.9 JSONL through Flutter's official GenUI SDK (`package:genui`)
  — protocol state machine, surface lifecycle, schema validation, reactive data
  binding included — and emits a typed `A2uiAction` (`surfaceId`, `name`,
  `sourceComponentId`, `timestamp`, `context`, optional `clientDataModel`). It
  is wired into the thread as the `artifact` block kinds `a2ui` /
  `application/a2ui+json`, with actions forwarded through
  Riverpod → Repository → FFI (layering intact).
- **No KnowMe-side registry, storage, or selection logic yet** — the renderer
  consumes whatever a block hands it; nothing owns the catalog, validates a
  surface against a registered schema, or records which components exist.

## 3. The four open problems

### 3.1 Definition — one catalog of record, three producers

Components and surfaces come from three producers: the **built-in catalog**
(first-party vocabulary, base = UAR's `urn:uar:a2ui:catalog:1` at the pinned
submodule version), **agent artifacts** (UAR-AGENT-MD §06 surface
declarations), and **plugin `ui-library` cards** (plugins doc §4). One schema
format governs all three: the UAR `ArtifactSchema` shape, extended with a
namespaced `id` (`knowme:a2ui:*` first-party; reverse-DNS for plugins, same
rule as capability ids) and a semver `version`. Authoring new first-party
components means extending the catalog URN with a version bump — vocabulary
governance per plugins doc §4.2.1, never ad-hoc per-producer shapes.

### 3.2 Storage — UAR owns the catalog; KnowMe persists a cache plus contributions

The ownership line (integration doc): **UAR owns the catalog registry and the
supported component profiles; KnowMe owns rendering conformance.** Storage falls
into two classes accordingly:

| Class | Contents | Store | Lifecycle |
|---|---|---|---|
| **Catalog cache** | The approved UAR catalog profiles (`urn:uar:a2ui:catalog:1` at the pinned submodule version, plus later UAR-issued updates) | Reproducible from the pinned artifact at boot; optionally persisted as an **offline-rendering cache** on mobile (the integration doc's "cache approved profiles for offline rendering") | Refreshed only with a submodule/catalog update; version surfaced in diagnostics |
| **Contributed schemas** | Plugin `ui-library` components, installed agent surfaces | **SurrealDB `a2ui_schema` records**: `{id, version, schema_json, source (plugin/agent id), content_hash, installed_at}` | Persist across restart; sync with the plugin/agent definition they arrived with (definitions travel, grants don't — plugins doc §10); removed on uninstall |

KnowMe's registry module (behind FFI, not a Flutter construct) resolves
`a2ui_schema_id` across both classes, catalog first, then contributed, with
exact-version then compatible-minor semantics. It is a **conformance cache and
contribution store, never a competing catalog authority**: components whose ids
are not in the UAR catalog or an installed contribution are rejected (fail
closed, §3.3). The embedded UAR instance keeps its own in-memory registry for
its own surfaces; the two are synchronized at agent/plugin install time, never
merged ad hoc.

### 3.3 Selection — deterministic resolve-then-negotiate

When a run wants to show a surface:

1. **Declare** — the agent artifact or skill carries `a2ui_schema_id` +
   minimum vocabulary version (UAR-AGENT-MD §06 / capability card `ui`).
2. **Resolve** — the registry returns the schema (§3.2) or a typed
   `schema_not_found` (which renders as an honest fallback panel, never a
   crash).
3. **Negotiate** — the renderer on *this* platform checks the schema's
   component set against its supported vocabulary: full support → render;
   partial → render supported components + labeled fallback for the rest
   (plugins doc §4.2.1: "a labeled fallback, never a crash or a blank");
   below minimum vocabulary → labeled fallback with an upgrade hint.
   **Fail closed on the unknown:** a component id outside the UAR catalog and
   all installed contributions is never executed or rendered as an interactive
   element — the labeled fallback *is* the closed failure surface, and the
   rejection is logged with the run's provenance (integration doc: "unknown
   catalog ids or component names fail closed").
4. **Interact** — user gestures produce typed `A2uiAction`s (Flutter: the
   existing `A2uiSurfaceView` callback) routed Riverpod → Repository → FFI →
   the paused run as its tool result (UAR's `routes.rs` ingestion in server
   placements; the facade's equivalent when embedded). Local interaction stays
   local: inputs echo and validate on-device; the bus sees semantic events
   ("form submitted"), never keystrokes (plugins doc §4.2.3).

### 3.4 Conformance — parity is a gate, not an intention

Per plugins doc §4.2.2: one schema source generates both platforms' types; a
conformance suite of golden renderings runs on **both** renderers per catalog
version. Concretely: fixture surfaces exercising every catalog component ×
light/dark × phone/wide, golden-tested in Flutter (`flutter-golden-ui`
harness, Alchemist) and screenshot-tested in React; divergence is a bug with
an owner. The Flutter renderer's GenUI SDK pins Google A2UI v0.9 — track the
UAR profile's `v0.9.1` patchline deliberately; a profile bump is a reviewed
submodule + catalog update, not a background drift.

## 4. Settings surfaces are the first consumer

The generic settings engine (plugins doc §11) generates A2UI forms from
`settings_definition.schema` + `ui_hints` — every plugin, agent, and provider
gets a settings page on every platform for free, rendered through exactly this
registry/selection path. That makes settings the proving ground for §3 before
agent-authored surfaces arrive: same schema format, same negotiation, same
fallback rules, zero bespoke settings UI.

## 5. Completion plan (releases)

- **A2UI-R1 — Registry + storage.** `a2ui_schema` SurrealDB table; KnowMe
  registry module behind FFI with the two-class resolution; built-in catalog
  load at boot; `schema_not_found` fallback panel (both renderers).
- **A2UI-R2 — Selection + negotiation.** Vocabulary-version capability check
  per platform; labeled partial/fallback rendering; `A2uiAction` → run-resume
  wired through the UAR facade (embedded) and `routes.rs` (server placements).
- **A2UI-R3 — Settings-form generator.** `schema` + `ui_hints` → A2UI form
  surfaces for plugin/agent/provider settings (consumes R1+R2).
- **A2UI-R4 — Conformance gate.** Shared fixture set; Flutter goldens + React
  screenshots per catalog version in CI; divergence blocking.
- **A2UI-R5 — Agent/plugin surfaces.** `ui-library` card installation into the
  contributed store; UAR-AGENT-MD §06 surfaces flowing through embedded UAR;
  `agui.artifact_input_request` end-to-end on mobile (rides the agent-skills
  roadmap's Release C certification).

## 6. Invariants (do not regress)

- No model- or plugin-provided HTML/JS ever executes; unknown components
  degrade to labeled fallbacks.
- A2UI traffic rides the AG-UI stream and the ContentBlock projection
  (`artifact` blocks) — no parallel UI channel.
- Renderers inherit design tokens and Flat 2.0 from the host theme; plugins
  cannot go off-brand by construction (plugins doc §4.1).
- The component vocabulary is versioned registry data; renderer parity per
  version is tested, not aspirational.
- Actions crossing the bus are semantic, typed, and audited like any tool
  result — including the confirmation surfaces for mutating tools (UIX-001
  §6.6), which are A2UI forms with `AlwaysConfirm` policy, not a special case.
