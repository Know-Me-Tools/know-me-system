# Plugin Strategy — A Capability-Centric Open Model

**Scope:** the plugin system as a **wide-open capability vocabulary**: what a
capability is, the recognized set we publish, how plugins declare (or invent)
capabilities, A2UI component libraries as the cross-platform UI plane, headless
plugins, discovery/matching/routing via embedded capability cards, and the decisions
we lock in now so a marketplace can exist later.

**Status:** v1.0 strategy, 2026-07-18. Supersedes the "plugin kinds" framing in
`docs/knowme-builder-extensibility-spec.md` §5 (which becomes the *transport* layer for
this model) and completes `docs/strategies/agents/README.md` §4: **a native agent is a
plugin carrying the `agent` capability — nothing more.** Companions:
`docs/strategies/model-configuration/README.md` (resolver used by callbacks),
`docs/knowme-functional-spec.html` §08 (trust/distribution base).

---

## 1. The inversion

The previous framing enumerated extension kinds (skill, UI surface, settings, agent).
This document inverts it:

> **We publish a vocabulary of capabilities. A plugin is a signed WASM package that
> declares which capabilities it provides — one, many, or ones we've never heard of.**

Consequences, in order of importance:

1. **Agents are not special.** "Native agent" = a plugin declaring
   `knowme:agent`. Same manifest, same signing, same sandbox, same installer, same
   audit. (The Hands wizard remains the *product* surface that binds an
   agent-capability plugin into a Hand — see agents doc §4.3.)
2. **The system is open at the vocabulary level.** Third parties can invent
   capabilities we haven't defined. Recognized capabilities get first-class treatment;
   unrecognized ones are carried, sandboxed, and discoverable — their ecosystems give
   them meaning (§5).
3. **Anything is expressible**: an agent, a skill pack, a CRM connector, a theme, a
   settings-only integration, a streaming filter, a UI component library, a data-type
   provider — or combinations, in one package.

## 2. The capability declaration ("capability card")

Every capability — recognized or invented — is declared with the same card shape.
Recognized ones live in the **signed capability registry** (published alongside the
model catalog, same Ed25519 discipline); invented ones ship only in the plugin's
manifest.

```jsonc
{
  "id": "knowme:agent",                 // namespaced; reverse-DNS for third-party
  "version": "0.1.0",                   // 0.x semver: minor = breaking
  "name": "Agent",
  "description": "Autonomous task executor with A2A-style lifecycle …",
  //   ^ name+description+tags are the EMBEDDABLE text — vectorized locally into the
  //     capability index for search, matching, and routing (§7)
  "tags": ["agent", "automation", "a2a"],

  "settings_schema": { "$ref": "…" },   // JSON Schema 2020-12; rendered by the generic
                                        // settings engine; scopes per extensibility spec §6
  "actions": [                          // A2A-style tasks this capability exposes
    { "id": "start-task", "input_schema": {…}, "output_schema": {…},
      "streaming": true, "description": "…" }
  ],
  "events": {                           // bus topics, namespaced by capability id
    "emits":    [ { "topic": "task-status", "payload_schema": {…} } ],
    "consumes": [ { "topic": "knowme:context/assembled", "optional": true } ]
  },
  "chunks": [                           // stream-chunk types handled during hook
    "text-delta", "tool-call", "thinking", "a2ui-patch",
    { "id": "com.acme.chart-frame", "schema": {…} }        // custom chunk kinds allowed
  ],
  "ui": {                               // OPTIONAL — §4; absence = headless
    "a2ui_components": [ { "id": "task-board", "schema": {…},
                           "actions": ["start-task"], "events": ["task-status"] } ]
  },
  "requires":  [ "knowme:kv@0.1" ],     // capability dependencies (host must provide)
  "enhances":  [ "knowme:agent@0.1" ]   // optional composition: adds to another
                                        // capability's surface when both are present
}
```

`requires`/`enhances` are cheap now and load-bearing later: they are how a plugin
extends *another plugin* without a new mechanism.

## 3. The recognized set (v0.1)

Published, schema'd, first-class in UI and routing. Deliberately short; grown by
evidence, per the same restraint the WIT contract already practices:

| Capability | What declaring it means |
|---|---|
| `knowme:agent` | A2A-shaped task lifecycle (agents doc §4); bindable into Hands; carries its own identity (§9) |
| `knowme:tool` | A single MCP-compatible callable: name, description, JSON-Schema input/output, annotations — the atomic spec-first unit (see `strategies/skill-support`) |
| `knowme:skill` | A packaged practice: one or more tools + instructions + policy defaults + settings, with a scope (`global` / `agent-private` / `agent-exposed`) — see `strategies/skill-support` |
| `knowme:hook` | Declared participation in named pipeline stages (agents doc §3) |
| `knowme:provider` | An LLM/embedding provider kind + settings schema (model-config doc §2.2) |
| `knowme:ui-library` | A2UI component library (§4), no behavior of its own |
| `knowme:entity-schema` | Namespaced tables + sync classes (extensibility spec §5.4) |
| `knowme:settings` | Settings-only integration (config that other capabilities read) |
| `knowme:theme` | Design-token pack within the brand system's constraint envelope |

A plugin declares any subset. A "full extension" in the old framing is just
`skill + ui-library + settings + entity-schema` in one package.

## 4. A2UI — the one UI plane

Plugins never ship React or Flutter code. UI is **A2UI components as data**: declared
in the capability card, rendered by the shared renderers — Flutter via the official
GenUI SDK (`gen_ui_widgets`' `A2uiSurfaceView`, Google A2UI v0.9), React via UAR's
`a2ui-*` packages — over the UAR production profile `uar.a2ui/1` (Google A2UI
v0.9.1, catalog `urn:uar:a2ui:catalog:1`). The transport underneath is the AG-UI
event stream (`docs/strategies/ag-ui/README.md`); **AG-UI is the event vocabulary,
A2UI the validated rendering contract** — the two are never the same channel.
Component definition, registry, storage, selection, and renderer conformance are
specified in **`docs/strategies/a2ui/README.md`**, which governs wherever this
section sketches. Wiring is declarative: a component binds to
the capability's **actions** (user gesture → A2A task) and **events** (task status →
component state), so interactivity round-trips through the same audited bus as
everything else. A2UI is the normalization story: **one component library per plugin,
identical on every platform.** Chunk type `a2ui-patch` is how streaming operations
update live components mid-generation.

### 4.1 Why this is structurally correct (not just convenient)

The WASM sandbox already forbids platform code, so UI must cross the boundary as
data; A2UI makes that constraint a feature. Compared to the iframe/webview model
(Figma plugins, VS Code webviews): no per-plugin web runtime, no style divergence
(plugins inherit design tokens and accessibility from the renderer and **cannot go
off-brand — the theme envelope is enforced by construction, not by review**), and the
security model collapses to "the component vocabulary *is* the sandbox" — no DOM
access, no script injection surface. The deeper win: agent-generated UI and plugin UI
are **one pipeline** — one renderer pair, three producers (app, agents, plugins) —
which no webview approach can offer.

### 4.2 The four disciplines this stands on

A2UI succeeds or fails on execution of exactly four things:

1. **Vocabulary governance with an honest ceiling.** The component vocabulary is
   versioned registry data, like capabilities: additions ship as schema + renderer
   updates on evidence of demand, never ad hoc. The expressiveness wall is planned
   for, not denied: a rich standard library first; *declarative* escape hatches next
   (e.g. a chart component that takes a spec, not a drawing surface); and the
   webview/HTMX path retained **only** as an audited exception (§4.3). Unknown
   components degrade gracefully: renderers below a plugin's declared vocabulary
   minimum render a labeled fallback, never a crash or a blank.
2. **Renderer parity as a tested contract.** One schema source of truth generates
   both renderers' types; a conformance suite of golden renderings runs on *both*
   platforms per vocabulary version (the `flutter-golden-ui` harness skill covers
   half of this today); divergence is a bug class with an owner, not a known
   quirk. If parity is an intention rather than a gate, Flutter becomes the
   second-class citizen within a year.
3. **Local interaction semantics.** Components are deliberately *stateful* where
   interaction demands it: inputs with local echo, declarative validation,
   optimistic toggles. The bus sees debounced **semantic** events ("field
   committed", "form submitted"), never raw keystrokes. This is the difference
   between SDUI that feels native and SDUI that feels like a survey iframe — and it
   also keeps the audited bus signal-dense instead of noise-flooded.
4. **Authoring experience, or no ecosystem.** Ships *with* the vocabulary, not
   after it: a hot-reloading A2UI playground fed by a running plugin, side-by-side
   React/Flutter preview, and a `pxt-ui-surface` Builder skill that scaffolds
   components and screenshots both renderers into the plugin's docs. Nobody builds
   for a UI system they can't see.

### 4.3 The HTMX/webview path is an exception, not an alternative

Demoted explicitly: two first-class UI systems would fork the ecosystem and double
the conformance budget. A capability may declare a webview/HTMX surface only with a
`ui-escape-hatch` permission — amber at install, listed in the posture audit, and
carrying its own sandbox terms (isolated webview, no host bridge beyond the standard
`invoke`/`subscribe`). The expectation named in the card's review UI: escape-hatch
surfaces are web/desktop-only and second-class on mobile. Every escape-hatch grant is
a data point for what the A2UI vocabulary is missing — reviewed when considering
vocabulary additions (§4.2.1).

## 5. Headless plugins and the open vocabulary

- **No UI required, ever.** A capability + events is a complete plugin (a silent CRM
  sync, a log enricher, a webhook bridge). The installer and library UI treat headless
  as normal, not as "missing UI."
- **Unrecognized capabilities** are carried intact: permissions still declared and
  enforced, sandbox identical, events namespaced under the capability id, settings
  schema still rendered generically (it's just JSON Schema). What they *don't* get:
  structured pipeline stages (hooks require recognized `knowme:hook` declarations),
  first-class product surfaces, or our schema validation of their action payloads.
  UI: a neutral `unrecognized` pill (distinct from trust pills — a *signed* plugin can
  carry unrecognized capabilities; the two axes are independent).
- **Name integrity**: reverse-DNS ids + publisher `did:key:` signatures prevent
  squatting; the registry will refuse recognized-namespace (`knowme:*`) declarations
  from unsigned or non-first-party publishers.
- **Promotion path**: an invented capability that gains ecosystem traction can be
  adopted into the recognized registry (with its incumbent implementations
  grandfathered) — the vocabulary is versioned data, so recognition is a registry
  publish, not an app release.

## 6. Manifest v2 (`.pxt`)

The extensibility spec's manifest gains `capabilities: [CapabilityCard]` and retires
per-kind fields. Unchanged: content addressing (IPFS/OCI), SHA-256 per file, Ed25519 /
`did:key:` signing, permission review at install, SBOM. Added now — dormant until the
marketplace (§8): `publisher` (stable DID), `license` (SPDX or commercial ref), and an
optional `entitlement` block (verification endpoint + public key) that v1 hosts ignore.

## 7. Discovery, matching, routing

Capability cards are text, so they index like everything else:

- **Local capability index** — on install, the device embeds each capability's
  name/description/tags + action descriptions with the *device's own index binding*
  (model-config doc §6 rules apply: the index stamps its embedding model+dim;
  registry ships canonical text, never vectors — precomputed vectors would break the
  moment a user changes embedding models).
- **Routing** — the `route` pipeline stage queries this index: "user intent looks like
  `com.acme.crm-sync`'s `import-contacts` action" arrives as a scored routing
  contribution, same provenance/budget discipline as any hook contribution.
  Unrecognized capabilities participate in this *semantic* matching fully; only
  structured stage participation is gated on recognized declarations.
- **Marketplace search later = the same index server-side** over the registry's
  published cards. One mechanism, two deployments.

## 8. Marketplace-later (what we lock in now)

Business model explicitly deferred — critical mass first; distribution stays open,
free, and sideload-friendly. But three things are cheap now and expensive to retrofit,
so they land in manifest v2 immediately:

1. **Stable publisher identity** — the signing `did:key:` *is* the merchant identity
   later; key rotation gets a documented procedure now.
2. **License + entitlement fields** — schema'd, ignored by v1 hosts. **The
   entitlement standard is W3C Verifiable Credentials** (Data Model 2.0, SD-JWT/JOSE
   securing): the trust plane is already DID-native on both ends — publishers sign
   manifests with `did:key` (issuer), users hold `did:key` identity keys (holder) —
   so a paid entitlement is a VC: issuer = author DID, subject = user DID, claims =
   `{plugin_id, tier, version_range, type: perpetual|subscription|trial|seat,
   expiry}`. Properties this buys:
   - **Offline, sovereign verification** — pure signature-chain checking in the
     container; no license server, works air-gapped (stronger than the license-key →
     presigned-URL sketch in local-models §5.2, which remains the *download*-gating
     transport; the VC is the *rights* object).
   - **Wallet sync for free** — a VC is a small signed document; it syncs over OFP as
     a user-scoped entity, so every instance's identity keystore is the wallet and
     rights are presentable on any device without re-authentication.
   - **Enforcement at install/update/activation, never runtime** — Cedar evaluates
     the entitlement claim in the activation policy; a lapsed subscription blocks
     updates, never bricks installed software.
   - **Revocation by lifetime, not lists** — perpetual = long-lived VC, no
     revocation (sold stays sold); subscription = short-lived VC (30–90 d) reissued
     on renewal; Bitstring Status List available as an issuer-optional online
     freshness check. Transfer/resale = reissuance under author policy, never bearer
     passing. Teams = issue to org DID; members present membership + org entitlement
     as a delegation chain through the existing org/personal Cedar layering.
   - **Honesty clause** — client-side verification is friction-for-honesty, not DRM;
     the paid value is signed authenticity, updates, support, and convenience. No
     arms race.
   - **One credential shape for everything** — Pro-tier licensed models, team
     memberships, and future feature entitlements are the same VC with different
     claim types: one wallet, one verifier, one Cedar integration.
3. **Review-ready metadata** — SBOM, permission declarations, and capability cards
   are exactly what a curated storefront needs to render listings and diff permission
   changes across versions. No extra work later; it's the same artifact.

Revenue split, curation policy, refunds: out of scope until traction.

## 9. Governance and identity

Governance is **part of the specification**, not an afterthought bolted on at install:

- **Every capability card may carry a `governance` block**: Cedar policy templates
  (the plugin's proposed defaults, rendered and reviewed at install exactly like
  permissions), resource budgets (tokens, invocations/hour, network destinations),
  and the data classes touched. The host merges these *proposals* with the user's
  policies; the plugin never grants itself anything.
- **Agents are principals.** Each agent *instance* — a deployed Hand, whether
  declarative or component-backed — is minted its own `did:key:` at creation and
  registered with the container (and, when server-connected, with the identity plane:
  Ory/flint-gate). Identity is **per-instance, never per-definition**: two
  installations of the same shared agent are two principals.
- **Dual-identity resolution.** Every governed action carries *user as subject* and
  *agent as actor* — the RFC 8693 token-exchange pattern. Server-side, flint-gate
  issues short-lived JWTs with `sub` = user, `act` = agent DID, plus claims:
  capability set, hand id, placement, policy version. Because `flint_auth` (Anvil)
  already injects JWT claims into Postgres GUCs, **RLS can natively distinguish "the
  user directly" from "the user via agent X"** — no new database machinery. Locally,
  Cedar evaluates with the agent as principal and the user in context; policies may
  require either or both ("Inbox Triage may read mail *for Travis*; nothing may send
  mail").
- **Audit keys on the pair.** Every audit-trail entry records `(user, agent, capability,
  action)` — the question "what did this agent do on my behalf" becomes a query, and
  revoking one agent's access never touches the user's own.

## 10. Sharing

Plugins, agents, Hands, and skills are **sharable artifacts** — they are signed text
and WASM, so sharing is distribution, not replication of authority:

- **Channels**: OFP peers (personal mesh, team instances), signed export bundles, or
  published `.pxt` CIDs — same trust pipeline regardless of channel.
- **Definitions travel; identity and grants do not.** A received agent is installed
  like any plugin: fresh agent DID, permission review, first-use consent, its
  proposed Cedar policy shown as a *proposal*. A shared artifact can never carry its
  author's authority into your system.
- **Team semantics** ride the team instance: an org publishes governed agents/skills
  to its BossFang; members install with org Cedar policies layered over personal ones
  (org policy can forbid what personal policy would allow, never the reverse).

## 11. The settings substrate

Everything in this ecosystem has settings — plugins, capabilities, agents, skills,
tools, indexes — and **all of it stores in one metadata-driven substrate** (aligned
with the prometheus-entity-management metadata base-schema approach): two tables,
JSON columns, schema-as-data.

```sql
-- identical shape in SurrealDB (client) and Postgres/flint-forge (server)
settings_definition(
  id, owner_kind,            -- capability | plugin | agent | index | container
  owner_id, key,
  schema JSONB,              -- JSON Schema 2020-12
  scope,                     -- device | user | tenant | agent
  sensitivity,               -- normal | secret (secret ⇒ keychain/Vault ref, never value)
  sync_class, version, ui_hints JSONB )

settings_value(
  definition_id,
  scope_key,                 -- e.g. user:did:… | device:… | tenant:… | agent:did:…
  value JSONB,               -- validated against schema at write
  clock JSONB,               -- CRDT/Lamport metadata for synced scopes
  updated_at, updated_by )   -- updated_by is a (user, agent) pair — §9 audit rule
```

Consequences: **one generic settings engine** validates, stores, syncs (per scope and
sync class), and audits every setting in the system regardless of complexity; **A2UI
settings components are generated from `schema` + `ui_hints`** — every plugin gets a
settings page on every platform for free, and a plugin may *optionally* ship custom
A2UI components for richer settings UX, bound to the same definitions; the **`agent`
scope** is the fourth scope — agent instances have their own settings resolved through
the same tables; migrations are additive schema-version bumps on the definition row,
with old values migrated lazily by the engine. Secrets are references into
keychain/Flint Vault — the substrate never holds secret material, so the tables sync
freely under their declared sync classes.

## 12. Open questions (defaults taken, flagged for decision)

1. **Card embedding ownership** — default: registry ships text, devices embed locally
   (taken in §7). Alternative rejected: registry-shipped vectors.
2. **Unrecognized capabilities in routing** — default: semantic matching yes,
   structured stages no (taken in §7). Revisit if it proves too conservative.
3. **`enhances` composition semantics** — v0.1 limits it to *additive* surface
   (extra actions/components on the target capability); no interception of the target's
   traffic. Interception, if ever, needs its own threat model.
4. **Chunk-type governance** — custom chunk kinds are namespaced and schema'd, but who
   renders an unknown chunk? Default: hosts drop unknown chunks silently *except* into
   consumers that declared them. Confirm.
5. **`knowme:theme` envelope** — *resolved (v1.1)*: the envelope is enforced by
   construction (§4.1) — plugin UI renders only through the token-bound A2UI
   renderers, so a theme capability can only rebind tokens within the design
   system's constraint schema (single-accent restraint encoded as a validator on the
   token set); it can never inject styling past the renderer.

6. **Agent JWT lifetime & refresh** — default: short-lived (≤15 min) actor tokens
   issued per run by the placement's gateway; long-running tasks refresh via the
   placement, never hold long-lived credentials. Confirm against flint-gate's token
   model.
7. **Org-over-personal policy layering** (§10) — restrictive-only merge is the
   default; confirm no legitimate case requires an org grant exceeding personal policy.
8. **Entitlement-VC issuance topology** — author-issued directly, marketplace as
   co-issuer/broker (revenue split needs the marketplace in the flow), or both?
   Default: schema supports either issuer; topology decided with the marketplace.
   Related: author key rotation/compromise procedure for the issuer DID must be
   documented before any paid issuance begins.

## 13. Implementation map (this repo)

- `rust/crates/knowme_plugin_host`: `CapabilityCard` types + manifest v2 parsing;
  capability registry fetch/verify; per-capability WIT world binding (agent world per
  agents doc §4.1, skill/hook worlds per extensibility spec §5.1).
- `gen_ui_agent`: routing-stage query against the capability index; event bus topic
  namespacing by capability id.
- `gen_ui_db_graph`: capability index tables with `{model_ref, dim}` stamps.
- gen_ui renderers (React + Flutter): A2UI component registration from `ui-library`
  cards; `a2ui-patch` chunk handling in the stream renderer.
- Registry publishing job: capability registry as a second signed document beside the
  model catalog, same origin (local-models doc §5.2).
- Identity: agent-DID minting in `knowme_hands` (creation flow); actor-token issuance
  in the sidecar/BossFang gateway (flint-gate RFC 8693 exchange); `flint_auth` claim
  mapping for the `act` claim; Cedar entity schema gains the agent principal type.
- Settings substrate: `settings_definition`/`settings_value` repositories in
  `gen_ui_db` (SurrealDB) and a matching flint-forge migration; generic validation +
  scope resolution in one Rust module shared via FFI/WASM; A2UI settings-form
  generator in the gen_ui renderers.
- Docs cross-updates: agents doc §4.3 notes agents-are-plugins (done); skills/tools
  split defined in `docs/strategies/skill-support/README.md`; extensibility
  spec §5 re-scoped as transport under this vocabulary at next revision.
