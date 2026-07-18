# KnowMe Builder — Local-First × Realtime × WASM Extensibility

**Status:** DRAFT v0.1 — pre-research thoughts, 2026-07-18.
This document seeds the spec that will be finalized once the commissioned deep-research
report lands. It captures the current inventory, the architectural position I believe the
evidence already supports, the decisions the report must confirm or overturn, and the
project plan skeleton. It is written against the actual code in:

- `hybrid-mobile-architecture-src` — the **KnowMe Builder** skill set (23 repo skills:
  `orchestrate-prometheus-application`, `deploy-hybrid-agentic-stack`,
  `hybrid-design-tokens`, `flutter-golden-ui`, `tauri-*`, `openspec-*`, `a11y-gate`,
  `karpathy-progress-memory`, …) plus the `hybrid-mobile-architecture` scaffolding skill
  and the knowme-poc reference app this monorepo was seeded from
- `prometheus/prometheus-skill-pack` — 35 top-level / 95+ total AI skills across 13
  categories (notably `react/prometheus-entity-skills`, `flutter/flutter-rust-ffi`,
  `flint/flint-sdk-{ts,dart,go,swift,kotlin,csharp}`, `htmx/htmx-alpine-lit`,
  `rust/librefang-wasm-skill`, `architecture/clean-architecture`), multi-harness install
  (Claude Code, Kimi, OpenCode, Codex, Cursor, …), surreal-memory MCP
- `prometheus/flint-realtime-fabric` (FRF) — the realtime spine: Apache Iggy durable
  event log; one multiplexed gateway exposing Spine/Signal/Sync/Agent/Entity/Authz
  services from frozen `proto-v1`; **Loro CRDT** (`frf-crdt`: `LoroDeltaApplier`,
  merge/apply/store); `frf-wasm` browser client (crdt + agent + pub/sub); Postgres CDC
  (`frf-postgres-cdc`); media via LiveKit + str0m WebRTC; Matrix/ATProto federation
  bridges; Cedar policy, Ory identity, Keto authz; uniffi FFI → Swift/Kotlin/Dart
- `prometheus/flint-forge` — the data & edge-compute plane on Postgres 18:
  **Quarry** (`fdb-*`, PostgREST-compatible REST + pg_graphql passthrough + GraphQL
  subscriptions over LISTEN/NOTIFY or FRF), **Anvil** (`ext-flint-*` pgrx extensions:
  auth GUC/RLS contract, hooks→webhooks, in-DB LLM "Ember", Vault secrets, meta/
  reflection), **Kiln** (`fke-*`, WASM Component Model edge functions on Wasmtime,
  `flint:host@0.1.0` WIT, `flint-skill` authoring SDK, stores **fs/ipfs/oci/s3**,
  signing **cosign + DID**)
- `prometheus/PGLITE-LOCAL-FIRST-ARCHITECTURE.md` — the PGlite/Electric field guide
  (VFS matrix, shape-factory tenant invariant, `entity-graph-react` `useLiveQuery`,
  additive migrations, retry-with-poison)
- `docs/pglite-oxide-tauri-hybrid.md` (this repo) — verified pglite-oxide constraints

Related in-repo anchors: `docs/knowme-functional-spec.html` (§07 OFP sync, §08 plugin
system), `rust/crates/knowme_plugin_host` (`knowme:plugin@0.1.0` WIT),
`rust/crates/knowme_ofp`, `docs/implementation-roadmap.md` (M1–M7).

---

## 1. The thesis

Everything needed for a world-class local-first + realtime platform already exists across
these repos — **as parts**. What is missing is one cohesive contract that makes the parts
compose: a single **per-target data-plane matrix**, a single **sync topology** with
clearly separated read/write/presence paths, one **extension component model** shared by
client and server (instead of two WIT dialects), one **settings/schema surface** that
extensions declare and containers store-and-sync, and one **packaging + distribution
standard** with signing. The KnowMe Builder skill set then becomes the *authoring
harness*: skills that let AI tools create, test, sign, publish, and install these
extensions against the standard.

The rest of this document specifies each of those five contracts.

## 2. Per-target data plane (the ground truth to build on)

Hard constraints already verified in `docs/pglite-oxide-tauri-hybrid.md` — the spec must
not fight them:

| Tier | Local store | How | Sync client |
|---|---|---|---|
| Web (all browsers) | **PGlite** `idb://` (+ `relaxedDurability`), OPFS-AHP where Chrome/Firefox-only is acceptable | WASM in a worker; `live` extension for reactivity | `frf-wasm` (pub/sub, CRDT, agent) + Electric shapes |
| Desktop (Tauri: macOS/Win/Linux) | **pglite-oxide** (PGlite WASI guest in a Rust host, real PG wire protocol) — SQLx/SeaORM unchanged | Rust core sidecar; falls back to PGlite-in-WebView if the sidecar is undesirable | `frf-sdk-rust` in the Tauri core |
| Mobile (iOS/Android) | **SQLite + sqlite-vec** (iOS cannot run Postgres: no fork, no JIT) | via `gen_ui_core` FFI (already the POC pattern) | uniffi FFI transport (Swift/Kotlin/Dart bindings exist in FRF) |
| Cloud | **Postgres 18 via flint-forge** | Quarry REST/GraphQL under RLS; Anvil in-DB compute | FRF CDC (`frf-postgres-cdc`) feeds the spine |

Consequences the spec embraces rather than papers over:

1. **"Same SQL everywhere" holds for web/desktop/cloud, not mobile.** The entity layer —
   `@prometheus-ags/prometheus-entity-management` (Zustand/React) and its Flutter/Riverpod
   counterpart — is therefore the *portability boundary*, not SQL. Repositories beneath
   the entity graph speak PGlite SQL on web/desktop and SQLite SQL on mobile; everything
   above (stores, hooks/providers, components) is dialect-free.
2. **PGlite has no RLS.** Tenancy is enforced at the shape factory (predicates ⊆ server
   RLS), exactly as `electricsql-tenant.ts` in prometheus-entity-management already does.
   That invariant becomes normative in this spec (§4.3) and extends to extensions: an
   extension can never attach a shape without a tenant predicate.
3. **Schema discipline:** server tables = client tables + `_local` columns; additive-only
   migrations, versioned, applied at boot; divergent local models go in a `client_*`
   namespace. Extensions get a namespaced schema each (§5.4).

## 3. Sync topology — three paths, not one

The single biggest architecture failure mode in local-first systems is forcing all data
through one sync mechanism. The evidence across the field guide and FRF's design says:
separate three paths with different consistency needs.

### 3.1 Read path — replicated queries (server-authoritative)

Postgres (flint-forge) → CDC → **shapes** → local store. Two interchangeable engines:

- **ElectricSQL shapes** into PGlite (web/desktop) — the field guide's recommended
  default; partial replication by predicate, TCC+ consistency.
- **FRF EntityService `watch`** as the sovereign alternative: Quarry's
  `FLINT_CHANGE_SOURCE=fabric` already anticipates this; the missing `WatchEntityType`
  RPC in FRF is a known gap (it currently fails closed) and is a P0 item in the plan
  (§8, W1). This gives one spine for entity replication that mobile (no Electric client)
  can also consume over FFI.

Rule: the read path is **rebuildable state**. Blowing away the local replica and
re-syncing must always be safe.

### 3.2 Write path — intent log (client-authoritative until acked)

Local writes are recorded as **idempotent, keyed actions** in the local DB (`_local`
write-tracking columns + an outbox table), pushed over FRF Spine (durable Iggy log),
applied server-side through Quarry/Anvil under RLS, and confirmed back through the read
path. Retry-with-poison + dead-letter for non-idempotent effects (field guide §6 policy,
adopted normatively). This is also exactly the shape of `knowme_ofp`'s Hand/run sync in
the KnowMe spec — OFP becomes a *profile* of this path (§3.5).

### 3.3 Collaboration/presence path — CRDT (peer-authoritative)

Document-style and presence state (cursors, co-editing, agent scratchpads, **settings**,
per §6) does not belong in either path above. It rides **Loro CRDTs** via `frf-crdt`
(`LoroDeltaApplier`) with deltas fanned out over:

- FRF Sync service (server-relayed, durable) — default;
- **WebRTC data channels** (str0m signaling is already proven in FRF phase 18) for
  P2P/offline-LAN merge;
- **lora-rs** as a constrained transport profile for off-grid deltas — must be treated as
  a *delta budget* problem (LoRa duty-cycle + payload limits ⇒ compressed, batched,
  priority-tiered deltas; presence never rides LoRa). This is a research question for the
  incoming report: recommended framing is "CRDT delta scheduling under transport budget",
  not "sync over LoRa".

### 3.4 Why this triad is the recommendation

Server-authoritative replication for queryable data (fast, tenant-safe, rebuildable);
logged intents for writes (auditable, offline-safe, idempotent); CRDTs only where
multi-writer merge is the actual requirement. Zero/LiveStore-style "CRDT everything"
systems buy simplicity at the cost of RLS/tenancy and Postgres compatibility — the wrong
trade given flint-forge's Anvil/RLS investment. PowerSync/RxDB only matter if Postgres
were not the backend; it is.

### 3.5 OFP alignment

KnowMe's OFP (FUNC-SPEC §07, `rust/crates/knowme_ofp`) is this topology restricted to a
personal mesh: EntityService-style replication for Hands/conversations, intent log for
runs, Loro for memory-graph merge, `did:key:` payload encryption, mTLS peers. The spec
should declare OFP an **FRF profile** (same proto, personal-peer deployment) rather than
a parallel protocol — one codebase, two deployment shapes (BossFang peer vs. flint
gateway).

## 4. Agents: client-side and cloud-hosted, one contract

- **Client-side agents** — UAR/`gen_ui_agent` (desktop/mobile via FFI; browser via
  `frf-wasm::agent`) run against local models and the *local* data plane; skill calls
  Cedar-governed; outputs written as ordinary intents (§3.2) so they sync like any write.
- **Cloud agents** — Kiln edge functions (`fke-runtime`) and BossFang/librefang
  instances run against Quarry + Anvil (including in-DB LLM via Flint Ember); they
  publish results onto the Spine, arriving at clients through the read path.
- **The contract:** `frf-agentproto` `ContentBlock` is already the wire shape for agent
  traffic; adopt it as the single agent-event vocabulary and map **AG-UI events onto it**
  (§5.3). An agent is then *location-transparent*: the same Hand definition
  (`knowme_hands::HandDefinition`) deploys to a device, a BossFang peer, or a Kiln
  function — deployment target changes latency and capability, never the protocol.

## 5. The extension component model (the heart of this spec)

### 5.1 Unify the two WIT worlds

Today there are two adjacent contracts: `knowme:plugin@0.1.0` (client container,
`rust/crates/knowme_plugin_host/wit/`) and `flint:host@0.1.0` (Kiln edge functions,
`flint-forge/wit/` + `flint-skill` SDK). Same substrate (Wasmtime + Component Model),
overlapping host surfaces (log, storage/kv, invoke, events vs. db, llm, kv, identity,
secrets). **Proposal:** factor a layered family under one package namespace:

```
prometheus:component/core@0.1.0      log, kv-storage, emit-event, clock        (every host)
prometheus:component/agent@0.1.0     uar-invoke (Cedar-gated skills)           (client + Kiln)
prometheus:component/data@0.1.0      sql-query (scoped schema), entity-graph   (per §5.4)
prometheus:component/llm@0.1.0       complete, embed (local model or Ember)    (capability-gated)
prometheus:component/ui@0.1.0        render surfaces (per §5.3)                (client containers)
prometheus:component/settings@0.1.0  schema declaration + change notification  (per §6)
```

A **native skill** is a component exporting the skill world (today's Kiln/UAR shape). A
**full extension** is a component exporting skill + ui + settings worlds. Hosts advertise
which interfaces they provide; the installer refuses a component whose imports exceed the
host's surface — capability negotiation is structural, at install time, before any code
runs (same posture as KnowMe FUNC-SPEC §08).

### 5.2 What a full extension can add

Per the requirement: (a) native skills for agent harnesses; (b) UI/UX surfaces; (c) a
settings schema the container stores and syncs; (d) entity types with sync policies;
(e) Hands templates; (f) scheduled/event handlers. All declared in the manifest (§7.2),
all reviewed at install.

### 5.3 UI: two render targets, one event bus

- **HTMX/server-rendered fragments** — the extension component exports
  `render(surface, props-json) → html` and the container injects the fragment into a
  sandboxed webview/iframe with the `plugin-bridge` (`window.knowme.invoke/subscribe` —
  the deferred v0.2 surface in FUNC-SPEC §08; this spec is the concrete driver for it).
  The `htmx-alpine-lit` skill in the skill pack is the authoring guide; forge's
  `htmx-alpine-animations-research.md` already explores the idiom.
- **A2UI components over AG-UI** — for the Flutter tier and rich React surfaces the
  extension emits **AG-UI events** (the POC's `gen_ui` protocol crates and the KnowMe
  UI/UX standard already speak AG-UI); the container's gen_ui renderer materializes
  A2UI component trees. Extension UI is therefore *data*, not code, on this path — which
  is what makes the same extension render on web, desktop, and mobile.

Both paths ride one bus: extension → `emit-event` → container event router → (HTMX swap |
AG-UI stream). Agent-generated UI and extension UI are the same mechanism.

### 5.4 Extension data: scoped schemas with sync policies

Each extension gets a namespaced schema (`ext_<id>` on Postgres; same-named tables in
PGlite/SQLite). The manifest declares each table's **sync class**: `replicated`
(read-path shape, tenant predicate mandatory — shape-factory invariant applies),
`intent` (outbox), `crdt` (Loro doc), or `local-only`. Migrations ship in the package,
additive-only, applied by the container at install/upgrade — client-side by the container
runtime, server-side through a Quarry admin surface. Anvil's `flint_meta` reflection
makes the server side introspectable; the same metadata drives client codegen.

## 6. The common settings model

Settings are the first proving ground for the whole stack, because they exercise every
layer at small scale:

- Extension declares a **JSON-Schema (2020-12) settings document** in its manifest
  (`settings.schema.json`) plus per-field metadata: scope (`device` | `user` | `tenant`),
  sync class (default: `crdt` for user scope, `local-only` for device scope, `replicated`
  for tenant scope), sensitivity (secret fields route to keychain/Vault, never CRDT).
- Container renders the settings UI **generically from schema** (React and Flutter
  renderers; A2UI form components) — extensions get a settings page for free; both
  `desktop/src/features/settings` and `mobile/lib/features/settings` grow one dynamic
  panel, not N.
- Values live in the container store (Zustand slice / Riverpod provider) backed by a
  **Loro map per (extension, scope)**; synced per §3.3 (FRF Sync, WebRTC, or OFP mesh);
  server persistence lands in `ext_<id>._settings` via the intent path when tenant-scoped.
- Change notification enters the component through `settings@0.1.0`
  (`on-change(key, value-json)`), so extensions react live on every device.

This model is exactly KnowMe FUNC-SPEC §07's "Settings: default per-device" made precise:
scope decides sync, schema decides rendering, sensitivity decides storage.

## 7. Packaging & distribution

### 7.1 Position

Do **not** invent a new decentralized VCS. Compose three proven layers, all already
present or trivially reachable from flint-forge:

1. **Content addressing & transport:** IPFS (KnowMe FUNC-SPEC §08 already commits to CID
   install + gateway fallback; `fke-store-ipfs` exists) **and** OCI registries
   (`fke-store-oci`) as the enterprise mirror — the same artifact, two transports.
2. **Signing & identity:** dual-path exactly as Kiln already implements —
   `fke-sign-did` (Ed25519 over canonical manifest, publisher `did:key:` — the sovereign
   path, matches KnowMe's plugin trust model verbatim) and `fke-sign-cosign`
   (Sigstore/cosign — the enterprise/supply-chain path, works with OCI + Rekor
   transparency log).
3. **Versioning & discovery ("the git-like part"):** a signed, append-only **release log
   per extension** (name → ordered list of (version, CID, manifest hash, signature)),
   itself a CRDT/merkle log that syncs over the same fabric — publishable to IPNS/DNSLink,
   an OCI tag list, or a plain git repo of manifests. Evaluate **Radicle** (Rust,
   sound-theory decentralized git) in the research report as the collaboration/source
   layer; but the *distribution* unit is the signed component artifact, not the repo.

### 7.2 The package format (`.pxt` — prometheus extension)

An IPFS/OCI directory: `manifest.json` (extends `knowme_plugin_host::PluginManifest`:
id, version, WIT worlds imported/exported, permissions, uar_skill_allowlist, settings
schema ref, ui surfaces, entity schemas + sync classes, migrations, signature block),
`component.wasm`, `migrations/`, `ui/` (optional HTMX/A2UI assets), `settings.schema.json`,
`SBOM.spdx.json`. SHA-256 of every file in the manifest; manifest signed per §7.1.2.
Version semantics: 0.x minor = breaking (as `knowme:plugin` already states); hosts pin
world versions.

## 8. What to add, where (project plan skeleton)

Workstreams, orderable after the research report confirms §3–§7:

- **W1 — FRF gap:** implement `WatchEntityType` RPC (unblocks Quarry
  `FLINT_CHANGE_SOURCE=fabric` and the sovereign read path). Small, unlocks the most.
- **W2 — WIT family:** factor `prometheus:component/*` worlds from `flint:host@0.1.0` +
  `knowme:plugin@0.1.0`; adapters so existing Kiln skills and KnowMe plugins keep working;
  publish `flint-skill`-style authoring SDKs (Rust first, TS/Go via componentize).
- **W3 — Container runtime (this repo):** extend `knowme_plugin_host` M3 scope to the
  full extension model — settings engine (schema→UI renderers for React + Flutter),
  extension event router (HTMX + AG-UI paths), scoped-schema migrator over PGlite/SQLite.
- **W4 — Data plane adapters:** pglite-oxide provider in `gen_ui_db` for Tauri; PGlite
  worker + entity-graph wiring for web; sqlite-vec parity layer for mobile; outbox/intent
  tables + retry-with-poison runtime as a shared crate (`prometheus-sync-core`), used by
  both knowme_ofp and web/desktop clients.
- **W5 — CRDT settings + collaboration:** Loro-backed settings maps end-to-end (FRF Sync
  → WebRTC fallback); the LoRa delta-budget spike (research-gated).
- **W6 — Packaging:** `.pxt` format + `forge-cli`/new `pxt` CLI verbs (init, build, sign
  [did|cosign], pin [ipfs|oci], publish-log, install, verify); registry log format.
- **W7 — KnowMe Builder skills:** new skills in the two skill repos that drive W2–W6 with
  AI harnesses: `pxt-new-extension`, `pxt-settings-schema`, `pxt-ui-surface` (HTMX and
  A2UI variants), `pxt-entity-schema`, `pxt-test-harness` (golden tests against a
  containerized host), `pxt-sign-and-publish`, `pxt-install-verify`; wire them into
  `orchestrate-prometheus-application` and the openspec-* change workflow so extension
  development is itself spec-driven.
- **W8 — Conformance:** a host-conformance suite (does a container correctly enforce
  permissions, shape predicates, settings scopes?) — this is what makes "conform to our
  standard" testable, and what the AI skills run in CI.

## 9. Questions the deep-research report must settle

1. Electric shapes vs. FRF EntityService as the *primary* read path (or Electric on
   web-only, FRF everywhere else)? Cost of running Electric alongside the fabric?
2. Loro vs. Automerge/Yjs stance is settled by `frf-crdt` (Loro) — but confirm Loro's
   Dart/FFI story for Flutter parity, and snapshot/compaction strategy for years-long
   settings histories.
3. TanStack DB's role vs. entity-graph-react at scale (field guide decision #7).
4. LoRa: realistic delta budgets (SF/duty cycle), and whether meshtastic-style store-and-
   forward is worth a transport adapter, or LoRa should be deferred entirely.
5. Radicle vs. plain signed-manifest git repos vs. IPNS for the release log; petname/
   naming layer for extension discovery.
6. WASI 0.2/0.3 timeline effects on the component worlds (async exports would simplify
   `handle-event` streaming).
7. Precedents worth mining for the extension UI sandbox: Figma plugin iframe model, Zed
   extensions, Obsidian, VS Code webviews — what do they get wrong that the A2UI-as-data
   path avoids?
8. Settings-as-CRDT prior art (e.g., Ditto, Jazz, Instant) — anything to steal for scope
   semantics and secret-field handling?

## 10. Recommendation (pre-report)

Adopt the triad sync topology (§3) with flint-forge as the sole server datastore and FRF
as the sole spine; unify the WIT surfaces into `prometheus:component/*` (§5.1) so one
signed component runs as a Kiln edge function, a KnowMe client extension, or both; make
settings the first end-to-end vertical (§6); package as `.pxt` over IPFS+OCI with
DID+cosign dual signing (§7); and grow the KnowMe Builder skill set into the authoring
harness (W7) so every one of these artifacts is producible, testable, and publishable by
an AI agent under the openspec workflow. Nothing here requires new infrastructure
categories — every layer extends a crate or skill that already exists in these repos.
