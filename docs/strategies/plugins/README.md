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
| `knowme:agent` | A2A-shaped task lifecycle (agents doc §4); bindable into Hands |
| `knowme:skill` | UAR-invocable tool(s); appears in the Skills tab and allow-lists |
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
in the capability card, rendered by the existing gen_ui renderers (React on
desktop/web, Flutter on mobile — the POC's `gen_ui` protocol crates and the AG-UI
event stream are exactly this machinery). Wiring is declarative: a component binds to
the capability's **actions** (user gesture → A2A task) and **events** (task status →
component state), so interactivity round-trips through the same audited bus as
everything else. The HTMX-fragment path from the extensibility spec remains as the
escape hatch for web-only surfaces, but A2UI is the normalization story: **one
component library per plugin, identical on every platform.** Chunk type `a2ui-patch`
is how streaming operations update live components mid-generation.

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
2. **License + entitlement fields** — schema'd, ignored by v1 hosts. When commerce
   arrives, entitlement verification reuses the license-key → short-lived-URL
   machinery already specced for Pro models (local-models doc §5.2) — keys gate
   *download/activation*, never runtime, preserving the sovereignty guarantee.
3. **Review-ready metadata** — SBOM, permission declarations, and capability cards
   are exactly what a curated storefront needs to render listings and diff permission
   changes across versions. No extra work later; it's the same artifact.

Revenue split, curation policy, refunds: out of scope until traction.

## 9. Open questions (defaults taken, flagged for decision)

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
5. **`knowme:theme` envelope** — how much may a theme deviate from the brand system?
   Needs a constraint schema (single-accent restraint as a validator, not a vibe).

## 10. Implementation map (this repo)

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
- Docs cross-updates: agents doc §4.3 notes agents-are-plugins (done); extensibility
  spec §5 re-scoped as transport under this vocabulary at next revision.
