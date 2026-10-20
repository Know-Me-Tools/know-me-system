# KnowMe — Implementation Roadmap (spec → module map)

**Repo:** `know-me-system` · **Source spec:** `docs/knowme-functional-spec.html` (FUNC-SPEC v1.0)
and `docs/knowme-moodboard-user-journeys.html` (UX-MOOD v1.0).
This document maps every spec subsystem to where it lives (or will live) in the
monorepo, and orders the gap-closure work. The starter code is the
`knowme-poc` reference application from `hybrid-mobile-architecture-src`,
migrated here with build-critical relative paths preserved.

## Current state: what the POC already gives us

| Spec subsystem | Status | Where |
|---|---|---|
| Rust core (7-layer arch, FUNC-SPEC §05) | **Working** | `rust/crates/gen_ui_*` — types, runtime, protocol, client, mcp, db, db_graph (SurrealDB), inference, audio (whisper-rs), agent (UAR/PMPO, Cedar-adjacent tools), host, ffi, tauri plugin, wasm, axum server, `knowme-web-server` |
| Desktop app (Tauri 2 · React 19 · Zustand 5 · Tailwind 4) | **Working** | `desktop/` — features: home, chat, hands, entities, memory, models, settings, startup |
| Mobile app (Flutter · Riverpod · FRB 2.12) | **Working** | `mobile/` — features: chat, memory, notes, overview, startup |
| Publishable packages | **Working** | `packages/` (gen-ui-react, gen-ui-wasm), `flutter_packages/` (gen_ui_flutter, gen_ui_widgets, prometheus_entity_management) |
| Infra / deploy | **Working** | `infra/`, `deploy/` (k8s, flint-gate, keto, postgres), `Dockerfile`, `docker-compose.yaml` |
| AI Chat tile (§04.1) | **Working** | `desktop/src/features/chat`, `mobile/lib/features/chat`, `rust/crates/gen_ui_agent` |
| Hands tile UI (§04.2) | **Partial** | `desktop/src/features/hands` exists; scheduler/policy execution not wired |
| Models tile (§04.7) | **Partial** | `desktop/src/features/models`; benchmark + remote BossFang linking open |
| Memory / knowledge graph | **Working** | `gen_ui_db_graph` (SurrealDB 3.x) + memory features |

## New skeletons added in this reorganization

| Spec subsystem | Skeleton | What is real / what is stubbed |
|---|---|---|
| Hands system (§06) | `rust/crates/knowme_hands` | Domain model (6 templates, 3-step flow, schedules, runs, Cedar decisions) real; UAR scheduler wiring stubbed |
| BossFang / OFP sync (§07) | `rust/crates/knowme_ofp` | Instance kinds, peer roles, per-entity sync defaults, Lamport clocks real; QUIC gossip + CRDT merge stubbed |
| Plugin system (§08) | `rust/crates/knowme_plugin_host` + `wit/knowme-plugin.wit` + `plugins/` | Manifest/permission model + v0.1 WIT contract real; Wasmtime host, IPFS fetcher, Ed25519 verify stubbed |
| Missing tiles (§04.3–.6) | `desktop/src/features/{ask-image,audio-scribe,prompt-lab,skills}` · `mobile/lib/features/{hands,ask_image,audio_scribe,prompt_lab,skills,models,settings}` | Per-feature blueprint READMEs; deliberately not wired into routers so all existing builds stay green |

## Milestones

### M1 — Hands become real (§06)
Wire `knowme_hands::HandDefinition` into the UAR scheduler in `gen_ui_agent`;
enforce Cedar per skill invocation (deny-by-default, first-use consent);
persist `HandRun` + audit log to SurrealDB; expose Tauri commands; connect the
existing `desktop/src/features/hands` UI; 3-step wizard per UX-MOOD Screen 04.

### M2 — OFP peer sync (§07)
QUIC transport + gossip rounds in `knowme_ofp`; entity envelopes over
`gen_ui_types::sync`; CRDT merge into `gen_ui_db_graph`; `did:key:` payload
encryption + mutual TLS; Settings → Instances/Sync panels (UX-MOOD 05–06)
driven by `PeerInstance` / `GossipRoundSummary`. Conservative defaults are
already encoded in `SyncEntityKind::default_enabled()`.

### M3 — Plugin host (§08)
`wasmtime::component::bindgen!()` against `knowme-plugin.wit`; sandbox limits
(64 MiB / 30 s epoch) from `SandboxLimits`; IPFS fetch with gateway fallback +
SHA-256 verify; Ed25519 manifest verification; installer dialog (UX-MOOD 07);
build `plugins/examples/hello-knowme` end-to-end as the acceptance test.

### M4 — Remaining tiles: Ask Image, Audio Scribe, Prompt Lab (§04.3–.5)
Vision path through `gen_ui_inference`; scribe UI over `gen_ui_audio`
(engine already compiles whisper-rs + cpal); Prompt Lab as ephemeral
inference reusing the Chat path. Wire routes into `app/navigation.ts` as each
screen lands.

### M5 — Skills tile + audit surface (§04.6, §09)
Registry/grants UI over `gen_ui_mcp` + `gen_ui_agent::tools`; hash-chained
append-only audit log; signed audit-bundle export.

### M6 — Mobile parity
Bring hands, ask_image, audio_scribe, prompt_lab, skills, models, settings to
Flutter; extend `rust/flutter_rust_bridge.yaml` `rust_input` as new FFI
surfaces land (knowme_hands, knowme_ofp types).

### M7 — Tiers & packaging (§10–§11)
Free/$20/$200 gating (no tier ever reduces local sovereignty), licensed
Qwen-35B path, air-gap ("on-device-only" Cedar lockdown) verification.

## Housekeeping notes

- New crates are workspace members; run `cargo hakari generate` if hakari
  checks complain about `workspace-hack` coverage.
- `plugins/examples/hello-knowme` is intentionally **outside** the rust/
  workspace (wasm32-wasip2 target, own profile).
- The full functional spec is also pinned at
  `https://ipfs.prometheusags.ai/ipfs/QmXFPBMvNw6jgbTmtt6TZszouJWxERbnDF7XpDfkkTUcFf`
  (verified identical in structure to `docs/knowme-functional-spec.html`).
