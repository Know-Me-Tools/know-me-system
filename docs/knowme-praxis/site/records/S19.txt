# KnowMe Universal Plugin System — Recommendation, Architecture, Functional Specification & Project Plan

> **AMENDMENT — 2026-08-04 (v1.1, authoritative deltas).** This plan is amended by
> `uar-plugin-system-research-2026-08-04.md` (this directory; canonical copy in
> `universal-agent-runtime/docs/plugins/`). The following supersede the corresponding
> sections below wherever they conflict:
>
> 1. **Contract ownership moved to UAR.** The plugin *contract* (WIT family, host
>    interfaces, capability-card schema, manifest+signing, lifecycle, shared sandbox
>    host crate, runtime-generic recognized set) is owned by the Universal Agent
>    Runtime and published from the UAR repo. KnowMe retains the *product* layer:
>    `knowme:*` capabilities (theme, app-shell chrome, Hands binding), marketplace,
>    entitlement VC issuance, trust-pill/review UX, distribution channels, and the
>    container-inversion roadmap. `knowme_plugin_host` remains the product host
>    wrapping the UAR-owned shared host crate (D3 unchanged in structure).
> 2. **D1 target renamed:** `knowme:plugin@1.0.0` → **`uar:plugin@0.2.0`** (version
>    humility: 1.0.0 only after real plugins across ≥3 kinds and ≥2 hosts). The
>    convergence now has **four** inputs, not three: `knowme:plugin@0.1.0`,
>    `librefang:plugin@0.1.0`, `uar-plugin.wit`, **and `prometheus:component@0.1.0`**
>    (skill pack) — adjudicated in one UAR ADR (skill-task world).
> 3. **Host profiles introduced:** `headless` (base host + skill-task + hook —
>    librefang/BossFang nodes) and `full-ui` (+ events/a2ui/settings/app-shell/
>    provider — KnowMe). Cards declare `profile_required`; installers reject
>    mechanically at install, not at runtime.
> 4. **Performance claims corrected (§2.2/D2):** no "zero-cost"/"1–5 ms" blanket
>    claims. Honest budgets: Cranelift ≈1.45–1.6× native on real workloads; **iOS
>    runs Pulley interpretation (~order of magnitude slower; `.cwasm` AOT native
>    code cannot be loaded on iOS)**; JS components carry ~8 MB engine each;
>    browser host is jco-transpiled (not wasmtime) with a host-parity conformance
>    seam. Bundled-vs-synced distribution split for App Store 3.3.2.
> 5. **Settings substrate unified:** JSON Schema **2020-12** everywhere (the
>    draft-7 references in §2.9/§3.4 are superseded); one scope enum to be emitted
>    from the Rust types (docs-from-code) — the README/plan scope-vocabulary
>    divergence is a known defect.
> 6. **Twelve review holes have normative fixes** (research doc Part 3 SR-1…SR-12):
>    namespace proof (did:web/DNS-TXT or transparency log), signed advisory/
>    revocation blocklist with auto-quarantine, permission **combination** risk
>    surfacing at install, host-stamped event provenance + emit-allowlist + rate
>    caps, contributor reentrancy guard (no `infer` in pipeline stages), sidecar
>    OS-fence + UDS broker, content-addressed generations with atomic
>    payload+registry+catalog+settings swap and rollback, crash-loop circuit
>    breaker, contribution ordering/aggregate budgets, universal action I/O schema
>    validation, mobile execution reality (Pulley), and 0.2.0 versioning.
> 7. **app-shell / service / workflow worlds stay 0.x** until proven; skill-task +
>    hook ship first (evidence order).

| Field | Value |
|---|---|
| Version | 1.1 (v1.0 amended 2026-08-04 per banner above) |
| Date | 2026-07-22 (amended 2026-08-04) |
| Status | Proposed — extends `docs/strategies/plugins/README.md`; contract layer re-homed to UAR per amendment |
| Audience | AI build agents (primary), human reviewers |
| Execution model | This plan assumes an AI agent performs the implementation |

**Related documents (must be read before building):**
- `docs/strategies/plugins/README.md` v1.0 — capability-centric plugin model (base model this document finalizes)
- `docs/strategies/agents/README.md` v1.1 — placements, hook pipeline §3, WASM `agent` world sketch §4.1
- `docs/strategies/a2ui/README.md` v1.1 — A2UI component registry / catalog ownership
- `docs/strategies/ag-ui/README.md` v1.1 — `uar.agui/1` event profile and KnowMe seams
- `docs/strategies/skill-support/README.md` v1.1 — `knowme:tool` vs `knowme:skill`, scopes, MCP interop
- `docs/strategies/bossfang/README.md` v1.0 and `docs/strategies/bossfang/hands-sidecar-architecture-spec-plan.md` — BossFang roles, sidecar supply chain
- `docs/knowme-builder-extensibility-spec.md` v0.1 — older extension-kind framing (superseded as model; retained as transport reference)

---

# Part 1 — Recommendation

## 1.1 The thesis

KnowMe stops being "an app with features" and becomes **a container whose only job is to provide the resources every AI application needs**: identity, storage, settings, inference, memory, event transport (AG-UI), rendering contract (A2UI), tool interoperability (MCP), agent interoperability (A2A), policy (Cedar), and lifecycle supervision. Everything the user touches — chat, settings, memory, hands, every future feature — is a **plugin** against this container. Core features are simply *first-party, pre-installed, pre-trusted plugins*. The user must be unable to tell the difference.

This is the same inversion the industry has converged on: MCP standardizes agent→tool, A2A standardizes agent→agent, AG-UI standardizes agent→application event streams, and Google's A2UI (v0.9.1 current, v1.0 candidate) standardizes agent→rendered-UI. KnowMe's container requirement is exactly: *understand AG-UI and A2UI, host WASM components, and get out of the way.*

## 1.2 Decisions (each with rationale)

**D1 — One component model, one WIT family: `uar:plugin@0.2.0` *(amended 2026-08-04; was `knowme:plugin@1.0.0` — see banner: UAR-owned, four-dialect convergence incl. `prometheus:component`)*.**
Today there are three divergent WIT dialects in the repo: `knowme:plugin@0.1.0` (`rust/crates/knowme_plugin_host/wit/knowme-plugin.wit`), `librefang:plugin@0.1.0` (`rust/vendor/librefang/crates/librefang-skills/wit/`), and `uar-plugin.wit` (vendored UAR). We converge them into a single `knowme:plugin` WIT package with multiple *worlds* (one per plugin kind: `hook`, `service-contributor`, `agent`, `provider`, `app-shell`). librefang's host interface set (`fs`, `net` SSRF-guarded, `kv`, `agent`, `env`, `time`) is the most mature and is adopted as the substrate of `knowme:plugin/host`; UAR's `embedded-mobile` profile becomes the `mobile` world variant. The builder-extensibility spec already flagged this convergence ("one extension component model… instead of two WIT dialects") — this plan executes it.

**D2 — Wasmtime 46 everywhere; component model + WASI P2 only.**
`knowme_plugin_host/Cargo.toml` still references wasmtime 41 (commented out); both vendored hosts use wasmtime 46. Standardize on wasmtime 46, `wasmtime::component::{Component, Linker, ResourceTable}`, deny-by-default `WasiCtx`, and `.cwasm` precompiled caching (already in UAR's `plugin_loader.rs`). Rationale: component model gives language-agnostic plugins (Rust, TinyGo, Python via componentize-py, JS via ComponentizeJS/StarlingMonkey — the last already proven in librefang's `wasmtime-wasi-http` support), deny-by-default capability security, and 1–5 ms instantiation with precompiled cache. Core (non-component) modules are accepted only behind a compatibility shim for legacy skills.

**D3 — Reuse librefang's `sandbox_component.rs` as the reference host; build `knowme_plugin_host` as the product host that wraps it.**
`rust/vendor/librefang/crates/librefang-runtime/src/sandbox_component.rs` is the most complete component host in the repo (capability subset wired at instantiation, undeclared imports → `MissingCapability`, typed `host-error` variants, additive-only semver policy). Do not write a third host. Extract/generalize that pattern into `knowme_plugin_host` behind a `PluginHost` trait so BossFang, UAR, and KnowMe mobile/desktop share one sandbox implementation.

**D4 — Capability cards are the unit of advertisement; the manifest (`.pxt`, manifest v2) is the unit of distribution.**
Adopt the capability-card model from `docs/strategies/plugins/README.md` as-is: a plugin is a bundle that *advertises* capabilities (`knowme:agent`, `knowme:tool`, `knowme:skill`, `knowme:hook`, `knowme:provider`, `knowme:ui-library`, `knowme:entity-schema`, `knowme:settings`, `knowme:theme`) and is granted entitlements. Extend the recognized set v1.0 with three new capabilities required by the goals below: `knowme:app-shell` (full-application A2UI container), `knowme:service` (OpenAPI/HTTP service on an allocated port), and `knowme:workflow` (BossFang-orchestratable workflow).

**D5 — AG-UI is the event bus; A2UI is the rendering contract; A2A/MCP are the interop surfaces.**
Terminology rule (already enforced in the strategy docs, restated because everything depends on it):
- **AG-UI (`uar.agui/1` profile)** — the event transport vocabulary. Every plugin that participates in a run emits/consumes AG-UI events through the existing, unit-tested seam `gen_ui_protocol::{normalize_uar_agui_event, A2uiAdapter, AguiAdapter}`. Plugins never invent their own event protocol.
- **A2UI (Google A2UI v0.9.1, `uar.a2ui/1` profile)** — the rendering contract. Every piece of UI a plugin shows — a settings form, a confirmation, a full application shell — is an A2UI surface rendered by the host's catalog (`A2uiSurfaceView` on Flutter, `a2ui-{core,react,lit}` on web/desktop).
- **MCP** — tool exposure both directions (plugin tools appear in `gen_ui_mcp`'s registry with `McpServerTrust` tiers; a plugin may itself mount an MCP server).
- **A2A** — agent exposure (a plugin agent publishes an AgentCard and is reachable via the existing UAR A2A dispatcher/registry pattern).

**D6 — Prompt-processing hooks are a typed pipeline, not scattered callbacks.**
Implement the hook pipeline already specified in `docs/strategies/agents/README.md` §3.1: `ingress → intent-classify → route → context-assemble → prompt-build → [inference] → tool-gate → tool-execute → post-process → egress`, as an explicit typed enum in `gen_ui_agent`. Rules carried from that doc, unchanged: *declared not discovered* (manifest announces stage + data classes), *contribute don't mutate* (typed, provenance-tagged, budget-capped contributions; 200 ms per-stage epoch budget; Cedar is sole authority at tool-gate). This is what makes the "context builder pattern" (scenario 1) work identically across system core, UAR core, and BossFang core.

**D7 — Settings are a substrate, not per-plugin tables.**
Generalize UAR's existing `SettingsManager.register_extension()` (schema-as-data, JSON-Schema-validated writes) into a two-table `settings_definition` / `settings_value` substrate with scopes `system | device | user | agent`, exactly as specified in `docs/strategies/plugins/README.md` §11. Ownership levels: system core owns system scope, UAR core owns agent scope, BossFang core owns node scope; sensitivity `secret` ⇒ keychain/Vault reference, never inline value. A2UI settings forms are generated from `schema` + `ui_hints`, so a plugin's settings appear in the same Settings UI as core settings with zero per-plugin UI code.

**D8 — Services get supervised sidecars with allocated ports, not in-process servers.**
A plugin that is "a web application or service exposing an OpenAPI spec on a unique port" runs as a **supervised sidecar process** (or WASM component with `wasi:http`), managed by a new `ServiceBroker` in `knowme_plugin_host`: port allocation from a per-instance range, health checks, lifecycle tied to plugin activation, OpenAPI spec ingestion at registration (operations become `knowme:tool` entries in the MCP registry, trust-tiered). Supply chain per the BossFang sidecar plan: cosign/SLSA/OCI verification before first spawn. The sidecar never gets a private API — all traffic from the container goes through the broker.

**D9 — Plugins are first-class on mobile. No private APIs, no desktop-only features.**
The embedded UAR kernel (`uar.a2ui/1`, `EmbeddedLibrary` mode) and the Flutter `A2uiSurfaceView` renderer are already mobile-operational. WASM components run on mobile through the same wasmtime host (mobile world: reduced host imports, no `wasi:http` server sockets; sidecars disallowed on iOS, replaced by in-component service worlds). If a capability cannot be honored on a platform, the manifest says so and the UI degrades explicitly — the capability card carries a `platforms` field.

**D10 — Distribution is content-addressed and signed.**
Keep `knowme_plugin_host`'s existing model: `WasmRef { cid, sha256 }` (IPFS content addressing), `ManifestSignature { publisher_did, signature_b64 }` (Ed25519, `did:key:`), `SignatureStatus { Verified, Unsigned, Invalid }`, `SandboxLimits` (default 64 MiB / 30 s). Unsigned plugins install only into a `developer` trust tier with explicit user consent, mirroring `gen_ui_mcp`'s `McpServerTrust::{Untrusted, TrustedLocal}` pattern.

## 1.3 What this recommendation deliberately does NOT do

- Does not invent a new event protocol (AG-UI profile exists and is tested).
- Does not invent a new UI spec (A2UI is adopted, `uar.a2ui/1` profile already in code).
- Does not write a third WASM host (librefang's is generalized).
- Does not give plugins direct network access (network flows through Cedar-gated host functions — `net` import with SSRF guard — or through `uar-invoke("web-fetch", …)`).
- Does not allow plugins to render raw HTML/JS in the container (A2UI only; iframe embedding is available only inside an `app-shell` plugin's own surface and is catalog-mediated).

---

# Part 2 — Architecture

## 2.1 System context

```
┌──────────────────────────────────────────────────────────────────────┐
│  CONTAINERS (any of: Flutter mobile · Tauri desktop · web ·          │
│  BossFang node · UAR service)                                        │
│                                                                      │
│  ┌────────────────────────────────────────────────────────────────┐  │
│  │ SHELL — navigation, chrome, surface host                        │  │
│  │  renders A2UI surfaces from core and plugins identically        │  │
│  └──────────────┬─────────────────────────────────────────────────┘  │
│                 │ A2UI surfaces / A2uiAction events                   │
│  ┌──────────────▼─────────────────────────────────────────────────┐  │
│  │ AG-UI EVENT FABRIC (`uar.agui/1`)                               │  │
│  │  gen_ui_protocol::{A2uiAdapter, AguiAdapter,                    │  │
│  │   normalize_uar_agui_event}  ← existing, tested                 │  │
│  └──────┬───────────────┬──────────────────┬─────────────────────┘  │
│         │               │                  │                        │
│  ┌──────▼─────┐  ┌──────▼───────┐  ┌───────▼────────┐               │
│  │ RUNTIME    │  │ HOOK         │  │ SERVICE        │               │
│  │ (UAR       │  │ PIPELINE     │  │ BROKER         │               │
│  │  kernel,   │  │ (typed stages│  │ (ports, health,│               │
│  │  embedded) │  │  in gen_ui_  │  │  OpenAPI ingest│               │
│  │            │  │  agent)      │  │  sidecar sup.) │               │
│  └──────┬─────┘  └──────┬───────┘  └───────┬────────┘               │
│         │               │                  │                        │
│  ┌──────▼───────────────▼──────────────────▼─────────────────────┐  │
│  │ PLUGIN HOST (`knowme_plugin_host`)                             │  │
│  │  wasmtime 46 component sandbox (generalized librefang host)    │  │
│  │  manifest v2 (.pxt) · capability cards · entitlements (Cedar)  │  │
│  │  lifecycle: install → verify → activate → deactivate → remove  │  │
│  └──────┬───────────────┬──────────────────┬─────────────────────┘  │
│         │ WIT host imports (capability-subset per plugin)           │
│  ┌──────▼─────┐  ┌──────▼───────┐  ┌───────▼────────┐               │
│  │ SETTINGS   │  │ STORAGE      │  │ POLICY         │               │
│  │ SUBSTRATE  │  │ (gen_ui_db,  │  │ (Cedar; sole   │               │
│  │ (def/value │  │  SurrealDB,  │  │  authority at  │               │
│  │  + scopes) │  │  kv, secret) │  │  tool-gate)    │               │
│  └────────────┘  └──────────────┘  └────────────────┘               │
└──────────────────────────────────────────────────────────────────────┘
        ▲ MCP (gen_ui_mcp, trust tiers)   ▲ A2A (AgentCard, registry)
        ▲ OpenAPI (broker-ingested)       ▲ IPFS/OCI distribution
```

## 2.2 Plugin kinds — how each required scenario maps

All plugins share: manifest v2, capability cards, signed WASM component (or supervised sidecar), settings namespace, lifecycle. They differ in which WIT world they target and which capabilities they advertise.

| # | Scenario (from requirements) | Plugin kind | WIT world | Advertised capabilities | Execution model |
|---|---|---|---|---|---|
| 1 | System component: settings + network + reacts during prompt processing (system prompt, memory recall, RAG recall, skills activation) — the portable **context builder** | **Contributor** | `knowme:plugin/contributor` | `knowme:hook` (stage list), `knowme:settings`, `knowme:provider` (optional) | Host calls `contribute(stage, context-in) → contribution` at each declared pipeline stage; network only via `net` import (SSRF-guarded) or `uar-invoke` |
| 2 | Full application semantics: own A2UI component acts as an **application shell container**, composing other A2UI components/views across web/mobile/desktop | **App** | `knowme:plugin/app-shell` | `knowme:app-shell`, `knowme:ui-library`, `knowme:settings`, optionally `knowme:agent` | Registered as a navigation root; receives a root A2UI surface + catalog token; emits `surfaceUpdate`/`dataModelUpdate`; child surfaces resolve through catalog-first-then-contributed registry |
| 3 | Hooks only — called only on specific events | **Hook** | `knowme:plugin/hook` | `knowme:hook` | Event-subscription table in manifest (`on: [run.started, tool.confirmation.requested, …]`); invoked with epoch budget; no UI, no settings unless declared |
| 4 | Web app/service exposing OpenAPI on a unique port | **Service** | sidecar process *or* `knowme:plugin/service` (wasi:http component) | `knowme:service` (openapi URL, port request), `knowme:tool` (generated) | ServiceBroker allocates port, supervises process, ingests OpenAPI → MCP tools; health-checked; killed on deactivate |
| 5 | Image-inference provider (adds image support when the active model lacks it) | **Provider** | `knowme:plugin/provider` | `knowme:provider` with `modality: image-inference` | Registered in the inference router as a modality fallback; the local/UAR lane consults it when a request carries image input the primary engine rejects |
| 6 | Workflow referenceable by BossFang orchestration | **Workflow** | `knowme:plugin/workflow` | `knowme:workflow` (graph ref) | Declarative workflow document (nodes = capability invocations); executed by BossFang harness; progress streams as AG-UI events |

A single `.pxt` bundle may combine kinds (e.g., an App that is also a Workflow and exposes an MCP server).

## 2.3 The WIT family — `uar:plugin@0.2.0` *(amended 2026-08-04; authored in the UAR repo, consumed here)*

One package; host imports shared; worlds per kind. (Sketch — authoritative WIT lives in `rust/crates/knowme_plugin_host/wit/`.)

```wit
package uar:plugin@0.2.0;

// ---- shared host imports (capability subset granted at instantiation) ----
interface host-log     { log: func(level: level, msg: string); }
interface host-kv      { get: func(ns: string, key: string) -> option<list<u8>>;
                         set: func(ns: string, key: string, val: list<u8>) -> result<_, host-error>;
                         delete: func(ns: string, key: string) -> result<_, host-error>; }
interface host-settings{ get: func(key: string) -> option<json>;
                         set-own: func(key: string, val: json) -> result<_, host-error>; } // own namespace only
interface host-net     { fetch: func(req: http-request) -> result<http-response, host-error>; } // SSRF-guarded, Cedar-checked
interface host-events  { emit: func(evt: agui-event) -> result<_, host-error>;
                         subscribe: func(patterns: list<string>) -> result<subscription, host-error>; }
interface host-infer   { complete: func(req: inference-request) -> result<stream<inference-event>, host-error>; }
interface host-memory  { recall: func(query: string, k: u8) -> list<memory-hit>; } // Cedar-gated, provenance-tagged
interface host-a2ui    { push-surface: func(surface-id: string, msgs: list<a2ui-message>) -> result<_, host-error>; }
interface host-time / host-env / host-random  // as in librefang host.wit
interface host-error   { variant host-error { capability-denied(string), path-denied(string),
                         ssrf-denied(string), policy-denied(string), budget-exceeded, internal(string) } }

// ---- exports per world ----
world hook {
  include base-host;
  export handle-event: func(evt: agui-event) -> result<option<agui-event>, host-error>;
}
world contributor {
  include base-host;
  export contribute: func(stage: pipeline-stage, input: contribution-input)
                     -> result<contribution, host-error>; // typed, provenance-tagged, budget-capped
}
world agent {
  include base-host;
  export start-task: func(task: a2a-task) -> result<task-handle, host-error>;
  export poll-task / cancel-task;   // A2A-shaped, per agents doc §4.1
}
world provider {
  include base-host;
  export capabilities: func() -> provider-capabilities; // modalities, limits
  export infer: func(req: inference-request) -> result<stream<inference-event>, host-error>;
}
world app-shell {
  include base-host-with-a2ui;
  export init-shell: func(config: shell-config) -> result<_, host-error>;
  export on-action: func(action: a2ui-action) -> result<_, host-error>;
  export on-navigate / on-resume / on-suspend;
}
world service { // WASM-native service alternative to a sidecar
  include base-host;
  export handle: func(req: http-request) -> result<http-response, host-error>; // wasi:http aligned
}
world workflow {
  include base-host;
  export definition: func() -> workflow-document;
  export on-signal: func(signal: workflow-signal) -> result<_, host-error>;
}
```

Versioning policy (adopted from librefang): **additive-only within a major version**; undeclared imports fail at instantiation with `MissingCapability`; the host resolves compatibility ranges, not exact pins (per wasmCloud's `VersionReq` lesson).

## 2.4 Manifest v2 (`.pxt` bundle)

Extends `knowme_plugin_host`'s existing `PluginManifest` (lib.rs:64) — new fields in **bold**:

```json
{
  "id": "com.acme.travel-suite",
  "name": "Travel Suite",
  "version": "1.2.0",
  "description": "…",
  "wasm": { "cid": "bafy…", "sha256": "…" },
  "worlds": ["app-shell", "contributor", "service"],
  "capabilities": [
    { "kind": "knowme:app-shell", "navigation": { "icon": "globe", "title": "Travel", "route": "/travel" },
      "catalog": "urn:acme:a2ui:catalog:1", "platforms": ["ios","android","desktop","web"] },
    { "kind": "knowme:hook", "stages": ["context-assemble","prompt-build"],
      "data-classes": ["memory.recall","rag.recall"], "epoch-budget-ms": 150 },
    { "kind": "knowme:service", "openapi": "components/service/openapi.json",
      "port": { "request": "ephemeral" }, "health": "/healthz" },
    { "kind": "knowme:settings", "schema": "settings/schema.json", "ui-hints": "settings/ui.json",
      "scope": "user" }
  ],
  "permissions": ["storage", "net:api.acme.com", "memory:read"],
  "uar-skill-allowlist": ["web-fetch"],
  "entitlements": { "vc": "…optional verifiable credential…" },
  "signature": { "publisher-did": "did:key:z6Mk…", "signature-b64": "…" }
}
```

The bundle is a content-addressed archive: `manifest.json`, WASM component(s), OpenAPI specs, settings schemas, A2UI catalog contributions, workflow documents.

## 2.5 Lifecycle

```
install → verify (signature → cid/sha256 → WIT world check → capability grant vs policy)
        → register (capability cards → registry; settings schema → substrate;
                    OpenAPI → broker ingest; A2UI catalog → registry merge)
        → activate (instantiate component w/ capability subset; allocate ports;
                    subscribe hooks; mount navigation)
        → deactivate (drain runs; kill sidecars; unsubscribe; keep data)
        → upgrade (verify new bundle; migrate settings values via schema versioning; atomic swap)
        → remove (deactivate + delete namespace data after user confirmation; audit record)
```

Every transition emits an AG-UI `CUSTOM` lifecycle event (`knowme.plugin.*`) and writes a Cedar-audited record.

## 2.6 The hook pipeline (context-builder substrate)

Implementation target: `rust/crates/gen_ui_agent/src/pipeline.rs` (new). Stages as a typed enum per agents doc §3.1:

```rust
pub enum PipelineStage {
    Ingress, IntentClassify, Route, ContextAssemble, PromptBuild,
    Inference, ToolGate, ToolExecute, PostProcess, Egress,
}
```

- Contributors declare stages + data classes in the manifest; the pipeline **never** reflects over plugins to discover hooks.
- `contribute()` returns `Contribution { items: Vec<ContributionItem> }` where each item is typed (`SystemPromptFragment | MemoryHit | RagSlice | SkillRef | GuardrailRule`), provenance-tagged (`plugin-id`, `cid`), and budget-capped (tokens + 200 ms epoch; over-budget ⇒ contribution dropped, telemetry emitted, pipeline continues).
- The existing `gen_ui_context` factory (`BudgetPlan`, `SliceKind`, `AssembleRequest`) consumes contributions at `ContextAssemble`/`PromptBuild`; memory/RAG/skills activations become contribution *inputs* so a contributor can reshape recall across system core, UAR core, and BossFang core identically.
- Cedar remains sole authority at `ToolGate`; hooks may *propose* guardrail rules, never grant.

## 2.7 A2UI app-shell model

- An `app-shell` plugin receives a **root surface** and the host catalog token (`urn:uar:a2ui:catalog:1` base + its own contributed catalog, resolved **catalog-first-then-contributed**, fail-closed — per the A2UI strategy doc).
- The host shell mounts it as a navigation peer of Chat/Settings/Memory; the plugin composes views by emitting `surfaceUpdate` / `dataModelUpdate` / `beginRendering` messages (A2UI v0.9.1; adopt v1.0's `actionResponse` RPC when finalized).
- User interactions return as typed `A2uiAction`s (already implemented end-to-end: `A2uiSurfaceView` → Riverpod → Repository → FFI on Flutter; `a2ui-react` on web/desktop).
- Core features migrate to the same model (Part 4, P4) — this is what makes "user cannot tell core from plugin" true rather than aspirational.
- A2UI registries may be *served by* plugins (flint-forge already exposes an A2UI registry as MCP at `/mcp/v1/a2ui` — the pattern to generalize).

## 2.8 Service broker

New module `knowme_plugin_host::service_broker`:

- **Port allocation**: per-instance ephemeral range (default 43100–43199), leased on activate, released on deactivate; leases persisted so restarts are stable.
- **Supervision**: sidecar process spawn/monitor/restart (max N restarts, backoff) reusing librefang's `librefang-channels/src/sidecar.rs` supervisor pattern and frozen sidecar protocol v1 (NDJSON-over-stdio) for control; WASM-native services instead run in the component host with `wasi:http`.
- **OpenAPI ingestion**: at registration, fetch `capabilities[knowme:service].openapi`, validate (3.0/3.1), and generate `knowme:tool` entries into `gen_ui_mcp`'s registry with `McpServerTrust` derived from the plugin's signature status; operations inherit the plugin's Cedar policy.
- **No private APIs**: container↔service traffic goes through the broker's loopback proxy with per-plugin auth tokens injected by the broker; the service cannot be reached except through the broker.

## 2.9 Settings substrate

Two tables, identical in SurrealDB (embedded) and Postgres (server), generalized from UAR's `SettingsManager.register_extension()`:

```
settings_definition(slug, owner_plugin, scope: system|device|user|agent|node,
                    schema: jsonschema-draft7, ui_hints: json, sensitivity: plain|secret, version)
settings_value(slug, scope, scope_id, value | secret_ref, clock: crdt, updated_by, updated_at)
```

- Writes validated against the JSON Schema; `secret` ⇒ keychain (mobile/desktop) or Vault ref (server).
- Settings UI: host generates A2UI forms from `schema` + `ui_hints` and interleaves plugin sections into the existing Settings screen — plugin settings are indistinguishable from core settings.
- A contributor/plugin reads its own namespace via `host-settings.get`; cross-namespace reads require an explicit permission and Cedar grant.

## 2.10 Security & trust model

| Layer | Mechanism | Existing anchor |
|---|---|---|
| Distribution | IPFS CID + sha256 pinning; Ed25519 `did:key:` manifest signatures; cosign/SLSA for sidecar binaries | `WasmRef`, `ManifestSignature` in `knowme_plugin_host` |
| Trust tiers | `Verified` / `Unsigned(developer, consent-gated)` / `Invalid(refused)`; mapped to `McpServerTrust` | `gen_ui_mcp` registry.rs:71 |
| Sandbox | wasmtime component model; deny-by-default WASI; capability subset at instantiation; 64 MiB / 30 s defaults, per-plugin override | librefang `sandbox_component.rs`; `SandboxLimits` |
| Network | only via `host-net` (SSRF-guarded, domain allowlist in `permissions`) or Cedar-gated `uar-invoke` | librefang `net`; `PluginPermission` |
| Policy | Cedar sole authority at tool-gate; plugin entitlements as VC-backed Cedar policies | `knowme_hands` `cedar_policy` pattern |
| Audit | lifecycle + policy decisions + hook overruns as Cedar-audited records | `CedarDecision` in `knowme_hands` |
| UI | A2UI catalog-only rendering; no plugin HTML/JS execution in container | A2UI strategy doc |

## 2.11 Platform matrix

| Capability | Desktop (Tauri) | Mobile (Flutter) | Web | BossFang node |
|---|---|---|---|---|
| WASM component host | wasmtime 46 | wasmtime 46 (embedded, mobile world) | via `gen_ui_wasm` shim (limited worlds) | wasmtime 46 |
| Hook pipeline | full | full | via UAR service | full |
| App-shell (A2UI) | `a2ui-react` | `A2uiSurfaceView` (package:genui) | `a2ui-lit/react` | headless (no UI) |
| Service sidecar | yes | Android: component-only; iOS: component-only | remote services only | yes (primary) |
| Provider (image inference) | yes | yes | via service | yes |
| Workflow | orchestrate via BossFang | orchestrate via BossFang | via UAR service | executes |

---

# Part 3 — Functional Specification

## 3.1 Actors

| Actor | Description |
|---|---|
| **User** | Installs, configures, uses plugins; cannot distinguish core from plugin features |
| **Plugin author** | Writes components in any WASM-targeting language; publishes signed `.pxt` bundles |
| **Container** | KnowMe core (this system) — provides substrate services |
| **BossFang** | Orchestration node executing workflows and hands |
| **AI build agent** | Implements this plan (primary consumer of Parts 3–4) |
| **Reviewer** | Human approving entitlements, trust-tier promotions, milestone gates |

## 3.2 User stories (numbered US-x; acceptance criteria in 3.3)

- **US-1** As a user, I install a plugin from a `.pxt` bundle (or catalog URL) and see its features appear — navigation entries, settings sections, tools — without restarting the app.
- **US-2** As a user, I open Settings and see the plugin's settings rendered with the same components and behavior as core settings; secrets never display values.
- **US-3** As a user, I open a full plugin application (app-shell) and navigate its views; every view is native-feeling on mobile, desktop, and web.
- **US-4** As a user, I chat with an agent whose context is enriched by contributor plugins (memory, RAG, system-prompt fragments); I can inspect provenance of what was injected.
- **US-5** As a user, when the active model lacks image input, image messages are transparently routed to an installed image-inference provider plugin.
- **US-6** As a user, I trigger a workflow plugin (e.g., via a Hand) and watch its progress as live events; I can cancel it.
- **US-7** As a user, I install a service plugin; its capabilities appear as tools the agent can call, with clear trust labeling.
- **US-8** As a user, I deactivate or remove a plugin; its surfaces disappear, its sidecars stop, and (on remove) I choose whether to delete its data.
- **US-9** As a plugin author, I write a hook in Rust/TinyGo/Python/JS against the WIT, package it, and see it invoked only for the events I declared.
- **US-10** As a plugin author, unsigned bundles install only after explicit developer-mode consent, and the UI shows the trust tier.
- **US-11** As a reviewer, I can audit every lifecycle transition, policy decision, and budget overrun per plugin.
- **US-12** As a user, core features (chat, settings) behave identically before and after the plugin system ships — because they are rebuilt as plugins on the same APIs (dogfood proof).

## 3.3 Feature catalog with acceptance criteria

**F1 — Plugin host core** (`knowme_plugin_host`)
- F1.1 Wasmtime-46 component sandbox, deny-by-default WASI, capability subset at instantiation. *AC: component importing an undeclared capability fails with `MissingCapability`; memory/fuel limits enforced (test: fork-bomb & memory-bomb components are killed within limits).*
- F1.2 Manifest v2 parse/validate (`parse_manifest` extended). *AC: invalid schema, unsupported world, or unknown capability kind → structured `PluginHostError`; golden-file tests per fixture.*
- F1.3 Signature verification (Ed25519/did:key) + CID/sha256 pinning. *AC: tampered component → `Invalid`; unsigned → `developer` tier with consent record.*
- F1.4 Lifecycle state machine with persistence and audit. *AC: crash mid-activate resumes deterministically; every transition audited.*
- F1.5 `.cwasm` precompiled cache with version-mismatch invalidation. *AC: warm activation < 50 ms on desktop reference hardware.*

**F2 — Capability registry**
- F2.1 Capability cards persisted; queryable by kind, platform, trust. *AC: registry survives restart; query latency < 5 ms at 500 plugins.*
- F2.2 Recognized set v1.0 = strategy-doc v0.1 set + `app-shell`, `service`, `workflow`. *AC: unknown kinds rejected at manifest validation with migration note.*

**F3 — Hook pipeline** (`gen_ui_agent::pipeline`)
- F3.1 Typed stage enum + contribution types integrated with `gen_ui_context`. *AC: contributions appear in assembled context with provenance tags; ordering deterministic.*
- F3.2 Epoch budget enforcement (default 200 ms/stage). *AC: over-budget contributor dropped, `knowme.plugin.budget_exceeded` telemetry emitted, run continues.*
- F3.3 Event-subscription hooks. *AC: hook fires only for declared patterns; wildcard beyond declaration rejected.*

**F4 — Settings substrate**
- F4.1 `settings_definition`/`settings_value` in SurrealDB + Postgres; scope model; secret refs. *AC: schema-invalid write rejected; secret value never leaves keychain/Vault; CRDT clock merges on OFP sync.*
- F4.2 A2UI-generated settings forms in Flutter + React Settings screens. *AC: a plugin with settings renders a section indistinguishable from core; golden tests per platform.*

**F5 — A2UI app-shell**
- F5.1 Navigation mounting of app-shell plugins on all three UI targets. *AC: plugin root appears beside Chat; deep-link route works; no chrome mismatch (Flat 2.0 respected).*
- F5.2 Catalog merge (base + contributed, fail-closed). *AC: component not in merged catalog → render error surface, never raw content.*
- F5.3 Action round-trip (`A2uiAction` → plugin `on-action`). *AC: latency budget < 100 ms local; full integration test on mobile.*

**F6 — Service broker**
- F6.1 Port lease + supervision + health. *AC: killed sidecar restarts ≤ N with backoff; deactivate frees the port (verified).*
- F6.2 OpenAPI ingestion → MCP tools. *AC: a sample service's operations appear in `tools/list` within 2 s of activation; calls route through broker proxy; undeclared outbound hosts blocked.*
- F6.3 WASM-native service world (`wasi:http`-aligned). *AC: same OpenAPI ingestion works for component services.*

**F7 — Provider plugins**
- F7.1 Modality registration + inference-router fallback. *AC: with primary engine lacking image input and provider installed, an image message succeeds; without provider, a clear capability error surfaces.*
- F7.2 Streaming parity. *AC: provider streams through the same `InferenceEvent` path as built-in engines.*

**F8 — Workflow plugins**
- F8.1 Workflow document schema + BossFang execution binding. *AC: sample 3-node workflow executes on BossFang; progress emits AG-UI events; cancel propagates.*
- F8.2 `knowme_hands::DeploymentTarget::BossFang` integration. *AC: a Hand referencing a workflow plugin deploys and runs end-to-end (replaces current `NotImplemented`).*

**F9 — Distribution & catalog**
- F9.1 IPFS/OCI fetch with pinning. *AC: offline install from local bundle works; CID mismatch aborts.*
- F9.2 In-app plugin catalog UI (browse/install/trust badges). *AC: catalog is itself an A2UI app-shell (dogfood).*

**F10 — Interop surfaces**
- F10.1 Plugin MCP server mounting. *AC: plugin tools listed via `gen_ui_mcp` with correct trust tier and annotations.*
- F10.2 Plugin AgentCard + A2A reachability. *AC: external A2A client can `message/send` to a plugin agent via the container's dispatcher.*
- F10.3 AG-UI emission for all plugin runs. *AC: emitted events validate against `uar/schemas/uar-agui-v1.schema.json` (conformance test).*

**F11 — Migration dogfood**
- F11.1 Settings feature re-implemented as first-party plugin. *AC: zero user-visible regression (golden UI suite).*
- F11.2 Chat feature consumes hook pipeline for its context assembly. *AC: existing chat tests pass; provenance inspector shows core contributor.*

## 3.4 Data contracts

- **Manifest v2**: JSON Schema `knowme/schemas/plugin-manifest-v2.schema.json` (new artifact; generated from Rust types via `schemars`).
- **Capability card**: per `docs/strategies/plugins/README.md` §capability-cards, extended with `platforms: string[]` and `epoch-budget-ms`.
- **AG-UI profile**: `uar.agui/1` — unchanged; plugin lifecycle events as `CUSTOM` with `knowme.plugin.*` names.
- **A2UI profile**: `uar.a2ui/1` over Google A2UI v0.9.1; catalogs URN-namespaced per plugin (`urn:<publisher>:a2ui:catalog:<n>`).
- **Workflow document**: new `knowme.workflow/1` JSON schema (nodes = capability invocations with Cedar policy refs; edges = data/signal flow; signals = pause/resume/cancel).
- **Settings**: JSON Schema Draft 7 per definition; `ui_hints` maps to A2UI form components only (fail-closed).

## 3.5 Error handling & edge cases

| Case | Behavior |
|---|---|
| Plugin panics/traps | Run fails with structured error event; plugin marked `degraded`; 3 traps ⇒ auto-deactivate with notification |
| Over-budget hook | Contribution dropped, telemetry, pipeline continues (never blocks the run) |
| Sidecar port conflict | Broker retries next free port in range; range exhausted ⇒ activation fails with actionable error |
| Unsigned plugin | Installs only to developer tier after explicit consent; tools from it run with `Untrusted` gating (confirmation per call) |
| Settings schema upgrade | Values migrated per declared `version` migrations; un-migratable values preserved read-only with banner |
| Platform unsupported | Capability card mismatch ⇒ install allowed, capability hidden with explanation (never a crash) |
| Plugin requests undeclared import | Instantiation-time `MissingCapability` (never runtime surprise) |
| Network attempt without permission | `capability-denied`/`ssrf-denied`; audit record; no silent proxying |
| App-shell emits unknown component | Fail-closed error surface; incident logged with plugin id |
| Core/plugin API drift | WIT compatibility ranges resolved at load; incompatible ⇒ plugin `suspended` with upgrade prompt |

---

# Part 4 — Project Plan (AI-agent execution)

## 4.0 Execution model for the AI build agent

- **One phase = one openspec change** (`openspec/changes/<phase-name>/` with proposal/design/tasks), matching repo convention.
- **Task granularity**: each task ≤ 1 focused PR; every task lists *verification* (test command + expected result) in `tasks.md` — the agent may not check a box without running the verification.
- **Conformance-first**: before building features, build the conformance harness (P0) so every later phase has executable acceptance.
- **Dogfood gate**: no phase is "done" until its capability is consumed by a first-party plugin, not only by test fixtures.
- **Hard constraints** (from AGENTS.md/base rules): layering UI→Hooks→Stores→Services; Flat 2.0; entity state via `prometheus-entity-management`; all networking/LLM/persistence stays in Rust core; audit script must pass.

## 4.1 Phases

### P0 — Convergence & conformance harness (foundation)
**Goal:** one WIT family, one wasmtime, executable acceptance.
- P0.1 Author `uar:plugin@0.2.0` WIT package (in the UAR repo per the amendment banner; this repo consumes it) (worlds: hook, contributor, agent, provider, app-shell, service, workflow) in `rust/crates/knowme_plugin_host/wit/`, absorbing `librefang:plugin` host interfaces and `uar-plugin` needs. *Verify: `wasm-tools component wit` validates; mapping doc from both legacy dialects.*
- P0.2 Align workspace on wasmtime 46 (remove the commented 41 refs; unpin/liberate `knowme_plugin_host` deps). *Verify: `cargo tree | grep wasmtime` shows single major.*
- P0.3 Extract/generalize librefang `sandbox_component.rs` into `knowme_plugin_host::sandbox` behind `PluginHost` trait (capability subset, `MissingCapability`, typed `host-error`, additive semver check). *Verify: vendored conformance corpus (`rust/vendor/librefang/conformance/`) passes against the new host.*
- P0.4 Conformance harness: `plugins/conformance/` — fixture components per world (Rust + TinyGo minimum), golden manifests, negative fixtures (undeclared import, budget bomb, ssrf attempt). *Verify: CI job green.*
**Milestone M0:** harness proves the sandbox contract before any product code depends on it.

### P1 — Host core & lifecycle
- P1.1 Manifest v2 + JSON Schema + signature/CID verification. 
- P1.2 Lifecycle state machine + persistence (`gen_ui_db`) + Cedar-audited transitions.
- P1.3 Capability registry (cards, platform/trust queries).
- P1.4 Local bundle install path (file picker / drag-drop) — no network distribution yet.
- **Dogfood:** port `plugins/examples/hello-knowme` to manifest v2 + component world; install→activate→emit-event→deactivate end-to-end.
**Milestone M1:** `hello-knowme` runs as a real component plugin on desktop. (Absorbs roadmap M3.)

### P2 — Settings substrate + hook pipeline (context-builder story)
- P2.1 `settings_definition`/`settings_value` tables (SurrealDB + Postgres), scopes, secret refs, JSON-Schema validation; generalize UAR `SettingsManager.register_extension()` as the UAR-side adapter.
- P2.2 A2UI settings-form generator; render plugin sections in Flutter + React Settings.
- P2.3 `gen_ui_agent::pipeline` typed stages; contributor invocation with epoch budgets; `gen_ui_context` consumes typed contributions with provenance.
- P2.4 Reference contributor plugin: "memory-booster" — declares `context-assemble`, uses `host-memory.recall` + `host-net` (allowlisted domain) to enrich prompts; settings via substrate.
- P2.5 Provenance inspector (A2UI surface) showing injected contributions per run.
**Milestone M2:** the portable context-builder pattern works identically on desktop and mobile embedded UAR.

### P3 — A2UI app-shell + core dogfood
- P3.1 App-shell registration/navigation mounting (Flutter `A2uiSurfaceView` host page; React shell route; web lit renderer).
- P3.2 Catalog merge + fail-closed rendering; contributed-catalog persistence (SurrealDB `a2ui_schema` per A2UI strategy doc).
- P3.3 Action round-trip through FFI to `on-action`.
- P3.4 **Settings feature re-implemented as a first-party app-shell plugin** (F11.1) — the "user can't tell" proof.
- P3.5 Reference third-party-style app: "Notes+" full app plugin (list/detail/editor views from A2UI).
**Milestone M3:** two apps (one core, one plugin) live side-by-side, indistinguishable.

### P4 — Service broker + provider plugins
- P4.1 ServiceBroker: port leases, supervision (sidecar protocol v1 reuse), health, broker proxy auth.
- P4.2 OpenAPI ingestion → `gen_ui_mcp` tool generation with trust tiers.
- P4.3 WASM-native service world.
- P4.4 Provider world + inference-router modality fallback; reference image-inference provider (wraps an existing engine; mobile included).
- P4.5 Reference service plugin: a small REST service (e.g., a notes search API) exposed on a broker port with OpenAPI.
**Milestone M4:** scenarios 4 and 5 demonstrably work on desktop + mobile (component-only services on iOS/Android).

### P5 — Workflows + BossFang + distribution
- P5.1 `knowme.workflow/1` schema + validation.
- P5.2 BossFang execution binding via hands-sidecar plan; `DeploymentTarget::BossFang` becomes real.
- P5.3 Workflow progress/cancel over AG-UI; A2UI run-detail surface.
- P5.4 IPFS/OCI distribution + cosign/SLSA verification for sidecar bundles.
- P5.5 Plugin catalog UI (itself an app-shell plugin).
**Milestone M5:** full install→orchestrate→observe loop from the catalog.

### P6 — Mobile parity & hardening
- P6.1 Mobile world profile (reduced imports) + conformance on iOS/Android devices.
- P6.2 Offline certification rerun (extends `embed-uar-mobile-offline` §5 tasks) with plugins installed.
- P6.3 Performance budgets: activation < 50 ms warm; hook p95 within epoch; app-shell first paint parity with core screens.
- P6.4 Security review pass: SSRF matrix, Cedar policy corpus, audit completeness.
**Milestone M6:** release candidate.

## 4.2 Dependency graph (critical path)

```
P0 ──► P1 ──► P2 ──► P3 ──► P6
        │      │      │
        │      │      └────► P5 ──► P6
        │      └───────────► P4 ──► P6
        └──────────────────► P4 (broker needs lifecycle/registry)
```

P3 and P4 may run in parallel after P2; P5 depends on P3 (surfaces) and P4 (service tools in workflows). P6 gates release.

## 4.3 Risk register

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| WIT convergence breaks vendored UAR/librefang builds | Med | High | P0 mapping doc + compatibility shims; vendored hosts keep legacy worlds behind features until P6 |
| wasmtime on iOS build friction | Med | High | Reuse existing rust_builder/mobile toolchain; spike in P0 (compile trivial component for aarch64-apple-ios before P1) |
| A2UI spec drift (v1.0 candidate changes) | Med | Med | Pin v0.9.1 profile; isolate spec version in `gen_ui_protocol`; adopt v1.0 in a dedicated later change |
| Hook epoch budgets too tight for real contributors | Med | Med | Budgets configurable per capability card; telemetry from P2 reference plugin calibrates defaults |
| Sidecar supply-chain verification complexity | Med | High | Reuse BossFang sidecar plan verbatim (frozen protocol + cosign); no bespoke scheme |
| Componentize-py/JS maturity gaps for "any language" | High | Med | Guarantee Rust + TinyGo first-class at M1; Python/JS labeled beta with conformance fixtures in P0; documented matrix |
| Scope creep: rebuilding core features as plugins balloons | Med | High | Dogfood limited to Settings (P3) + chat context consumption (P2); chat-as-plugin full migration is post-M6 |
| Plugin settings sync conflicts (OFP/CRDT) | Low | Med | CRDT clock in schema from day one; reuse `gen_ui_db::sync` seams |

## 4.4 Effort & sequencing estimate (AI-agent build, human review gates)

Assuming one AI build agent with human review at milestones, calendar time dominated by build/test cycles rather than typing:

| Phase | Est. agent-effort | Review gate |
|---|---|---|
| P0 | 4–6 days | M0 conformance green |
| P1 | 5–8 days | M1 hello-knowme E2E |
| P2 | 6–9 days | M2 context-builder demo |
| P3 | 6–9 days | M3 core/plugin parity demo |
| P4 | 5–8 days | M4 service + provider demos |
| P5 | 5–8 days | M5 catalog→workflow loop |
| P6 | 4–7 days | RC sign-off |
| **Total** | **~5–8 weeks** with P3/P4 overlap | 7 human gates |

## 4.5 Definition of done (applies to every phase)

1. All `tasks.md` boxes checked *with* their verification commands run and logged.
2. Conformance corpus green (including new fixtures added by the phase).
3. `scripts/audit.sh` passes; layering rules respected.
4. Golden UI tests updated for any surface change; Flat 2.0 preserved.
5. AG-UI events from new paths validate against the `uar.agui/1` schema.
6. Docs updated: the relevant `docs/strategies/*/README.md` version bumped with change notes; this plan's phase table annotated with completion dates.

---

## Appendix A — Scenario → capability checklist (traceability)

| Requirement scenario | Delivered by |
|---|---|
| System component w/ settings, network, prompt-stage reactions (context builder) | contributor world + settings substrate + hook pipeline (F3, F4) |
| Full application shell composing A2UI views cross-platform | app-shell world + catalog merge + navigation mounting (F5) |
| Hooks-only plugins | hook world + event subscription (F3.3) |
| Web app/service with OpenAPI on unique port | service broker + OpenAPI→MCP (F6) |
| Image-inference provider | provider world + router fallback (F7) |
| Workflows for BossFang orchestration | workflow world + BossFang binding (F8) |
| Advertise capabilities/endpoints/A2UI registry/A2A/MCP | capability cards + F10 interop surfaces |
| Core vs plugin indistinguishable | F11 dogfood + P3.4 |

## Appendix B — Research anchors (external)

- AG-UI: open event-based agent↔UI protocol (~16 event types, transport-agnostic; SDKs in TS/Python/Kotlin/Java/Go) — docs.ag-ui.com; first-party support from Google, Microsoft, Amazon, Oracle ecosystem tooling.
- A2UI: Google-originated declarative generative-UI spec; v0.9.1 current (`application/a2ui+json`), v1.0 candidate adds `actionResponse` RPC; official renderers incl. Flutter GenUI SDK — a2ui.org.
- Protocol stack: MCP (agent→tool), A2A (agent→agent), AG-UI (agent→app transport), A2UI (rendering payload) — complementary layers; AG-UI explicitly carries A2UI payloads.
- WASM: wasmtime (Bytecode Alliance reference runtime) leads component-model/WASI-P2 support; capability-based deny-by-default security is the established plugin-sandbox pattern (Shopify Functions, Envoy filters); wasmCloud's provider/host-plugin model and `VersionReq` compatibility informed D3/WIT versioning; Extism's "host-controlled HTTP without WASI" informed the `host-net` design.
