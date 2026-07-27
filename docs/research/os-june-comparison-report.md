# KnowMe × os-june — Comparative Analysis & Recommendations

**Date:** 2026-07-21
**Author:** KnowMe research agent (deep-research pass)
**Reference codebase:** `/Users/gqadonis/Projects/references/os-june` (fork of `open-software-network/os-june`)
**KnowMe codebase:** `/Users/gqadonis/Projects/know-me/know-me-system`

> **Method note.** The `deep-research` skill was not present in this environment's skill index, so the
> deep-research pass was executed manually: full static analysis of both repositories by exploration
> agents, plus web research against primary sources (product site, GitHub, Venice/Phala/Nous
> documentation) and third-party coverage. All externally sourced claims carry citations inline.

---

## 1. Executive Summary

**June** (os-june) is an open-source, MIT-licensed "private AI on your Mac" app from Open Software:
chat, voice dictation, bot-free meeting notes, and a sandboxed local agent in one Tauri 2 desktop
workspace, backed by a TEE-attested Rust API (`june-api`) that routes to zero-retention models by
default ([github.com/open-software-network/os-june](https://github.com/open-software-network/os-june);
[opensoftware.co/june](https://opensoftware.co/june)). It is pre-1.0 (v0.0.35) but shipping near-daily
releases at very high velocity (1,648 commits; ~870 in the last 30 days).

**KnowMe** is a sovereign, cross-platform personal-intelligence platform: shared Rust core, Flutter
mobile + Tauri/React desktop shells, on-device inference lanes, and an embedded SurrealDB graph-RAG
memory layer, with autonomous agents ("Hands"), peer sync (OFP), and a WASM plugin system as typed
skeletons against an ambitious functional spec.

**The two products are complements, not clones.** June is a *private work assistant* — it deliberately
has **no vector DB, no embeddings, no knowledge graph, no journaling/life-logging**. KnowMe's mission
(personal intelligence that knows you, structurally sovereign, on every device) is a superset of June's
ambition. June therefore cannot teach KnowMe much about *personal modeling* — but it is arguably the
best open-source reference implementation available today for **verifiable privacy architecture,
shipped autonomous-routine UX, connector custody, sandboxed agent execution, and audio capture
engineering** — all areas where KnowMe has spec'd but unshipped subsystems.

**Top 5 recommendations** (details in §6):

1. **Adopt June's "earned autonomy" trust ladder and routine template gallery for Hands** (M1).
2. **Add privacy-tier labels and authoritative capability metadata to the KnowMe model catalog.**
3. **Steal the sandbox write-pattern: OS-level write-jail + token-gated loopback proxy for agent tool writes.**
4. **Build Audio Scribe on June's meeting-capture architecture: source-separated WAVs, energy-based turn detection (not diarization), durable idempotent transcription jobs.**
5. **Adopt June's verifiability playbook for the optional cloud lanes: OCI source-commit labels, signed deploy tags, and a self-serve `/verify` attestation walkthrough.**

---

## 2. The Two Systems at a Glance

| Dimension | **KnowMe** | **June (os-june)** |
|---|---|---|
| Mission | Sovereign personal intelligence, on every device | Private AI workspace for Mac (Windows partial) |
| Platforms | macOS/Windows/Linux/iOS/Android + web server | macOS 14+ primary; Windows partial |
| License / openness | Not open source (per repo docs) | MIT, "privacy claims can be checked instead of believed" |
| Stack | Rust core (22 crates), Flutter mobile, Tauri 2 + React 19 desktop, axum server | Tauri 2 Rust + React 18/TypeScript, Rust axum `june-api` |
| Local inference | **Yes** — llama.cpp (desktop/iOS), LiteRT-LM (Android), WebLLM (web); Qwen/Gemma GGUF catalog | Via user-configured local endpoint only; cloud-first via Venice |
| Cloud inference | BYOK via liter-llm (143 providers) | June API → Venice zero-retention default; anonymized third-party opt-in; TEE fallback via Phala |
| Memory | **Graph-RAG**: SurrealDB embedded, 384-dim on-device embeddings, HNSW + BM25 + RRF + graph expansion | Plain agent-curated fact table in SQLite (`memories` ≤4000 chars), SQL LIKE search. No vectors, no graph |
| Autonomous agents | Hands: 6 templates, typed domain model, **scheduler stubbed** | Routines: cron + **event triggers** (Gmail deltas, calendar), template gallery, **earned autonomy**, shipped |
| Audio | Whisper-tiny engine real; Scribe UI absent | Dictation into any app + bot-free meeting notes w/ system audio; shipped, polished |
| Connectors | Hand skills referenced (`email-imap`, `web-fetch`…), unwired | Gmail, Calendar, Notion, GitHub shipped; Linear in progress; tokens in OS Keychain |
| Sandbox | Cedar policy spec'd; execution sandbox not built | macOS Seatbelt write-jail default; loopback proxy for writes |
| Verifiability | "Sovereignty is structural" claim; AES-256-GCM at rest spec'd | TEE attestation (Intel TDX/Phala), OCI commit labels, signed deploy tags, public `/verify` walkthrough |
| Sync | OFP gossip + CRDT (skeleton), Electric-SQL-shape sync engine (partial) | E2E-encrypted note sharing only (AES-256-GCM content keys, per-invite envelopes) |
| Telemetry | Not specified | Opt-in Brave-style P3A counters w/ explicit never-collect list |
| Pricing | Free/$20/$200 planned | Free/$20/$100 live; "privacy is never a paywall" |
| Maturity | Core loop real (chat, local inference, graph-RAG); differentiators scaffolded | v0.0.35, feature-complete for v1 scope, 169 release tags |

---

## 3. What June Actually Is (research findings)

### 3.1 Product and positioning

June's pitch is that AI apps are "a remote company's window into your work"; its answer is local-first
architecture plus *verifiable* cloud inference rather than promises
([README](https://github.com/open-software-network/os-june)). The marketed feature set: chat with a
local Hermes-based agent, push-to-talk dictation pasted into any app, bot-free meeting notes with
mic + system-audio capture, scheduled/event-triggered "routines," image generation, and per-model
privacy-tier labels ([opensoftware.co/june](https://opensoftware.co/june)).

### 3.2 The privacy stack (June's strongest asset)

- **Local custody:** all content on-device; OAuth/provider tokens in the OS Keychain; connector calls
  go direct from the device so Open Software "structurally cannot read user mail" (repo: ADR-0016,
  `docs/private-connectors-threat-model.md`).
- **Zero-retention routing:** default models are Venice *private* models — contract-enforced
  zero data retention. Venice exposes four privacy modes (Anonymous / Private / TEE / E2EE) per model
  via its `/models` API ([Venice privacy docs](https://docs.venice.ai/overview/privacy);
  [venice.ai/privacy](https://venice.ai/privacy)).
- **TEE-attested backend:** `june-api` runs in an Intel TDX confidential VM on Phala Cloud; the
  production image records its source commit in the OCI `org.opencontainers.image.revision` label;
  deploys are pinned as signed `deploy/<env>/<sha>` git tags; Phala Trust Center publishes attestation
  evidence; every deployment serves a self-serve `/verify` walkthrough (repo README, ADR-0024).
  This mirrors Venice's own March 2026 move from trust-based to hardware-verifiable privacy via NEAR
  AI Cloud and Phala ([Venice blog, 2026-03-18](https://venice.ai/blog/venice-launches-end-to-end-encrypted-ai);
  [CryptoBriefing coverage](https://cryptobriefing.com/encrypted-ai-inference-venice-release/)).
- **Independent assessment caveat:** Venice's default Private mode is still *contractual*, not
  cryptographic, and no completed third-party audit had been published as of April 2026 — TEE/E2EE
  modes are the cryptographically strong tiers
  ([Perspective AI review, 2026-06](https://perspectiveai.xyz/is-venice-ai-safe-privacy-review-2026/)).
  June inherits this caveat for its default lane.

### 3.3 The agent runtime

June embeds Nous Research's open-source **Hermes Agent** runtime, pinned by commit/SHA with a sealed
compatibility-patch script and a compatibility matrix gating features. Hermes itself is a persistent,
skill-learning agent runtime (MIT) that crossed 140k GitHub stars in under three months, with a
five-stage learning loop (execute → evaluate → extract skill → refine → retrieve), sandbox backends,
and 200+ model support ([hermes-agent.org](https://hermes-agent.org/);
[AlphaSignal, 2026-06-16](https://alphasignal.ai/news/nous-research-s-hermes-agent-can-now-spend-real-money-autonomously);
[OpenHosst guide](https://openhosst.com/blog/hermes-agent)). June's contribution on top is a
**total event classifier** (every Hermes wire frame maps to exactly one typed event; unknown frames
surface as `unsupported` instead of vanishing), a write-jailed bridge, and identity override via an
injected `SOUL.md`.

### 3.4 Market context

June sits in a crowded, fast-growing "private Mac AI" segment — Granola-style bot-free meeting notes,
WisprFlow-style dictation — with open-source entrants like MacParakeet (free, GPL-3.0, on-device
Parakeet ASR) undercutting paid tools on price
([macparakeet.com](https://macparakeet.com/); [Muesli](https://github.com/Muesli-HQ/muesli)).
June's differentiation within that crowd is the *verifiable* backend and the agent/routines layer.
For KnowMe, this validates both the demand for privacy-first personal AI *and* the pricing bands
KnowMe already plans ($20/$200 vs June's $20/$100).

---

## 4. Detailed Comparison Against KnowMe's Mission

KnowMe's mission pillars (FUNC-SPEC v1.0): structural sovereignty, eight thinking tools, a knowledge
graph that grows with use, autonomous Hands, optional encrypted peer sync, and a plugin ecosystem.

### 4.1 Where KnowMe is genuinely ahead — and should not regress

1. **Personal knowledge modeling.** June's "memory" is a single agent-curated SQLite table with LIKE
   search. KnowMe's `gen_ui_db_graph` (HNSW + BM25 + native RRF + RELATE-edge expansion) is a real
   graph-RAG system. June's unimplemented "Biography" concept (auto-built user profile from
   notes+mail+calendar, defined in CONTEXT.md but with zero code) is exactly the territory KnowMe's
   knowledge-driven-development philosophy already occupies. **Do not simplify memory toward June's
   model.** The lesson flows the other way.
2. **On-device inference as the default lane.** June treats local models as a power-user endpoint
   option; KnowMe treats them as the zero-config default across four platforms, with device/thermal
   budgets in `gen_ui_context`. This is the stronger sovereignty posture.
3. **Cross-platform reach.** June is Mac-first with a partial Windows shell and no mobile story (an
   iOS companion is only listed as "not yet done"). KnowMe's Flutter shell + FFI surface (~50
   functions) is a durable structural advantage — personal intelligence is mostly *not* at a desk.
4. **Sync and ecosystem vision.** OFP gossip, CRDT merge, WASM Component Model plugins with IPFS
   distribution and Ed25519 signing — June has nothing comparable (its E2E note-sharing is the
   closest analog, see §5.7).

### 4.2 Where June is ahead — the learning targets

1. **Shipped autonomy vs. typed skeleton.** KnowMe's `knowme_hands` is 199 honest lines with a stubbed
   scheduler. June's Routines are in users' hands: cron schedules, **event triggers** (Gmail history
   deltas polled ~90s, calendar `event_upcoming` scans), a template gallery (morning brief, weekly
   review, inbox triage, meeting prep), and trust modes with **earned autonomy** — a routine must
   complete 3 clean approval-gated runs before it may run autonomously.
2. **Verifiability.** KnowMe asserts sovereignty; June *proves* its backend. OCI labels, signed deploy
   tags, Phala attestation, and a `/verify` page turn a marketing claim into a checkable artifact.
3. **Sandboxed execution.** June runs its agent under a macOS Seatbelt write-jail by default, and
   sandboxed MCP servers *write* through token-gated loopback proxies so enforcement stays in Rust
   without weakening the sandbox (recorder, memory, connectors all use the pattern). KnowMe's Cedar
   policy is spec'd but no execution sandbox exists for Hand skills like `shell-exec` or `filesystem`.
4. **Audio product engineering.** June's meeting pipeline is the reference design: detect meetings in
   ~27 apps without a bot, capture mic + system audio as **source-separated WAVs**, order turns by
   energy detection (deliberately *not* diarization), reject echo/bleed with evidence tiers, keep
   audio so failed steps retry without re-recording, and run transcription as **durable, fingerprinted,
   idempotent jobs** (ADR-0026). KnowMe has a whisper engine and FFI stubs but no capture pipeline,
   no job durability, and no Scribe UI.
5. **Memory UX honesty.** June's memory is user-inspectable, editable, project-scoped, hard-deletable
   with **tombstones** (`memory_tombstones`) for future sync-proofing, and can be globally disabled
   fail-closed via a `disabled_toolsets` deny propagated through every agent construction path
   (ADR-0027 + addendum). KnowMe's memory promote/delete FFI exists, but there is no tombstone
   concept for the planned sync engine, and no single fail-closed switch.
6. **Model catalog discipline.** June's catalog stores only *authoritative* capabilities
   (`supportsFunctionCalling`, `supportsVision` — never marketing traits, ADR-0007) and labels every
   model with a privacy tier in the picker. KnowMe's catalog (`gen_ui_inference/src/catalog.rs`)
   tracks artifacts and quantizations but has no privacy-tier dimension.
7. **Process artifacts.** 36 append-only ADRs with dated addenda; a 671-line `CONTEXT.md` binding
   glossary with `_Avoid_` lists enforced across code, UI copy, and agent prompts; an enforceable
   `spec/` directory whose violations fail review; per-version API contract fixtures gating CI
   (ADR-0021); pnpm `minimumReleaseAge` 7-day supply-chain cooldown. KnowMe's process (40 base rules,
   OpenSpec, versions.toml) is strong but lacks the ADR log and the glossary.

---

## 5. Deep Dives on Transferable Patterns

### 5.1 Earned autonomy (→ Hands, milestone M1)

June encodes trust as data, not vibes: `read_only → approval → autonomous`, with promotion gated on
three consecutive clean approval runs. This solves the real product problem Hands will face: users
will not hand an overnight agent `shell-exec` on day one, and a binary "allow everything" toggle
teaches them to either live in fear or click through. **Recommendation:** add `trust_mode` and
`consecutive_clean_runs` to the `knowme_hands` domain model now — it is far cheaper to add before the
scheduler lands than after.

### 5.2 Event triggers (→ Hands)

June's proactive value comes less from cron than from *events*: Gmail history-id deltas and upcoming
calendar events fire routines (`src-tauri/src/connectors/triggers.rs`). KnowMe's Hand templates
(Inbox Triage, Market Pulse, Daily Journal) are all cron-shaped today. **Recommendation:** define a
`Trigger` enum in `knowme_hands` (`Cron | Event { source, cursor }`) with per-connector cursor
persistence, matching the shape June proved out.

### 5.3 Sandbox write pattern (→ Hand execution + plugin host)

The loopback-proxy pattern deserves direct adoption: sandboxed processes get **read** access directly
but must **write** through a token-gated loopback HTTP proxy implemented in Rust, so every mutation
passes policy enforcement without granting the sandbox DB or filesystem write access. This maps
cleanly onto KnowMe's planned Cedar policy gate for `knowme_plugin_host` and Hand skills — the policy
enforcement point lives in trusted Rust, the untrusted WASM/skill code never holds write capability.

### 5.4 Audio capture architecture (→ Audio Scribe)

Three decisions to copy wholesale:

1. **Source-separated capture, energy-based turn ordering** — sidesteps diarization's complexity and
   failure modes while still producing readable "conversation turns."
2. **Keep the audio; make every downstream step a durable, idempotent, fingerprinted job** — failed
   transcription or note generation retries without re-recording (June: `note_transcription_jobs`,
   `recording_checkpoints`).
3. **Ephemeral live preview, durable final artifact** — live transcript is explicitly transient, which
   lowers the privacy surface of the riskiest moment (during capture).

For desktop, June's out-of-process `.app` helper for CoreAudio process taps with `status.json` file
IPC (ADR-0004) is a pragmatic template for KnowMe's Tauri shell.

### 5.5 Privacy-tier model catalog (→ Models tile + context assembly)

Add a `privacy_tier` enum to the model catalog: `Local | ZeroRetention | Anonymized | TeeAttested`.
Surface it in the Models UI exactly like June's picker. This is cheap (metadata + UI badge) and
converts KnowMe's three-lane architecture into a *legible* privacy story — today a user cannot tell
at selection time what happens to their prompt in the cloud lane. Venice's four-mode framework
(Anonymous/Private/TEE/E2EE) is the reference vocabulary
([Venice docs](https://docs.venice.ai/overview/privacy)).

### 5.6 Verifiability playbook (→ optional cloud/BossFang lanes)

KnowMe's sovereignty is strongest on-device, but the moment a prompt crosses to Flint, BossFang, or a
BYOK cloud lane, KnowMe is asking for the same trust June refused to ask for. The playbook:

1. Record the source commit in OCI image labels for every deployed service.
2. Pin deploys as signed git tags (`deploy/<env>/<sha>`).
3. Where a confidential-compute backend is ever offered, publish attestation evidence (Phala Trust
   Center model) and a `/verify` walkthrough.
4. Publish a **threat-model document** per connector/integration, as June did
   (`docs/private-connectors-threat-model.md`).

Items 1–2 are nearly free and pay off immediately in credibility with the privacy-conscious personas
KnowMe targets (attorneys, therapists).

### 5.7 E2E sharing mechanics (→ OFP sync)

June's private note sharing is a compact, proven design: AES-256-GCM content keys, per-invite key
envelopes, PBKDF2 passcodes, and the key carried in the URL fragment so the server only ever sees
ciphertext (`src/lib/share-crypto.ts`). KnowMe's OFP sync is gossip+CRDT shaped, but the *envelope*
pattern for granting a peer access to specific content is directly reusable, as is the discipline of
`memory_tombstones` so deletions propagate and stay deleted across sync.

### 5.8 Process artifacts (→ repo governance)

- **ADRs, append-only, addenda allowed.** KnowMe's decisions currently live in strategy docs that get
  edited; June's ADR log (36 entries) preserves *why*, including reversals. Given KnowMe's recent
  pivot history (mistral.rs → LiteRT-LM), an ADR trail would pay for itself quickly.
- **CONTEXT.md glossary with `_Avoid_` lists.** KnowMe already struggles with term drift across
  gen_ui/knowme/UAR/Flint vocabularies; a binding glossary enforced in review is a proven fix.
- **Total event classifier for external runtimes.** KnowMe integrates UAR (and possibly other agent
  runtimes later). June's rule — every wire frame maps to exactly one typed event, unknown frames
  surface as `unsupported` — made upstream runtime upgrades detectable instead of silently breaking.
  Strongly recommended for the UAR SSE/event adapters in `gen_ui_agent::uar`.
- **Pinned + patched upstream with a compatibility matrix.** If KnowMe ever pins UAR/librefang
  versions for release (it vendors both as submodules), June's pattern — SHA-pinned runtime, sealed
  patch script, feature-gating compatibility matrix, upgrade checklist — is the way to do it safely.

### 5.9 Telemetry with a never-collect list

June's P3A-style telemetry (opt-in, coarse counters, weekly buckets, explicit never-collect list
including "hashes of content, embeddings, excerpts" — `docs/telemetry.md`) is a ready-made policy
template for KnowMe, which currently has no telemetry story. For a sovereignty product, publishing
the never-collect list *before* shipping any telemetry is itself a trust feature.

---

## 6. Consolidated Recommendations

Priority-ordered against KnowMe's existing roadmap (M1–M7). **P0** = adopt in the current milestone;
**P1** = next two milestones; **P2** = strategic/backlog.

| # | Pri | Recommendation | Maps to | Effort | June evidence |
|---|---|---|---|---|---|
| 1 | P0 | **Earned-autonomy trust ladder** (`read_only → approval → autonomous`, N clean runs to promote) in the Hands domain model | M1 Hands | S | Routines trust modes, ADR trail |
| 2 | P0 | **`Trigger` enum with event sources + persisted cursors** (email delta, calendar upcoming) for Hands | M1 Hands | M | `connectors/triggers.rs` |
| 3 | P0 | **Privacy-tier metadata on the model catalog** (`Local / ZeroRetention / Anonymized / TeeAttested`) + picker badges | Models tile | S | ADR-0007, model picker UI |
| 4 | P0 | **Memory tombstones + fail-closed global memory disable** before the sync engine is wired | M2 sync | S | `015_memories.sql`, ADR-0027 addendum |
| 5 | P1 | **Loopback-proxy write pattern**: Hand skills/plugins read directly, write via token-gated Rust proxy behind Cedar | M1/M4 plugins | M | June MCP loopback proxies |
| 6 | P1 | **OS write-jail for agent execution** (Seatbelt macOS; document Windows/Linux gap honestly, as June does) | M1 Hands | M | ADR-0006 |
| 7 | P1 | **Audio Scribe pipeline**: source-separated capture, energy-based turns, durable idempotent transcription jobs, ephemeral live preview | Scribe tool | L | ADR-0002/0004/0026 |
| 8 | P1 | **OCI source-commit labels + signed deploy tags** for any KnowMe-operated service; publish per-integration threat models | Flint/BossFang | S | ADR-0024, `/verify` |
| 9 | P1 | **Total event classifier** for UAR wire events; unknown frames → `unsupported` | UAR integration | S | `hermes-control-plane/` |
| 10 | P2 | **ADR log (append-only) + CONTEXT.md binding glossary** | repo governance | S | `docs/adr/`, CONTEXT.md |
| 11 | P2 | **E2E envelope-sharing pattern** (AES-256-GCM content keys, per-invite envelopes) for OFP content grants | M5 OFP | M | `share-crypto.ts` |
| 12 | P2 | **P3A-style telemetry policy with published never-collect list** (even if telemetry itself stays unshipped) | trust posture | S | `docs/telemetry.md` |
| 13 | P2 | **Routine/Hand template gallery UX** modeled on June's (morning brief, weekly review, meeting prep) | M1 Hands | S | `routine-templates.ts` |
| 14 | P2 | **Supply-chain cooldowns** (`minimumReleaseAge`, SHA-pinned external runtimes, sealed patch scripts for vendored UAR/librefang) | build/release | S | `pnpm-workspace.yaml`, Hermes pin |

**Explicit non-recommendations:**

- *Do not* adopt June's flat agent-curated memory table as KnowMe's memory model — KnowMe's graph-RAG
  is the stronger bet and the core of its mission. Adopt only its UX properties (inspectable,
  editable, scoped, tombstoned, fail-closed).
- *Do not* chase Venice-style cloud-routing as a default lane; KnowMe's local-first default is the
  differentiator June can't match on mobile.
- *Do not* treat TEE attestation as meaningful for KnowMe until a KnowMe-operated server component
  actually handles user content — today it would be theater. Items 1–2 of the verifiability playbook
  (commit labels, signed deploys) are the honest starting point.

---

## 7. Strategic Takeaway

June validates KnowMe's thesis from the adjacent lane: privacy-first personal AI is a real, funded,
fast-moving market, and buyers respond to *proof* over *promises*. June is winning the desktop
private-work-assistant niche by shipping verifiable privacy and dependable autonomy; it has
deliberately not built personal knowledge modeling, on-device inference, or mobile.

KnowMe's path is therefore not to imitate June's scope but to **import its discipline**: make Hands
earn autonomy the way June's routines do, make every model's privacy tier visible the way June's
picker does, make agent writes pass through an enforcement point the way June's loopback proxies do,
make the audio pipeline durable the way June's transcription jobs do, and make every server-side
claim checkable the way June's `/verify` does — while pressing the advantages June cannot follow:
the knowledge graph, on-device inference, and the phone in the user's pocket.

---

## 8. Sources

**Codebases (analyzed locally, 2026-07-21):**
- KnowMe: `README.md`, `docs/knowme-functional-spec.html`, `docs/implementation-roadmap.md`,
  `rust/crates/gen_ui_db_graph/`, `rust/crates/gen_ui_agent/`, `rust/crates/gen_ui_inference/src/catalog.rs`,
  `rust/crates/knowme_hands/src/lib.rs`, `rust/crates/gen_ui_ffi/src/api/`,
  `docs/diagnostics/mobile-uar-local-inference-2026-07-21.md`, `docs/assessment-2026-07-16.md`
- os-june: `README.md`, `CONTEXT.md`, `docs/adr/` (ADR-0002/0004/0006/0007/0012/0014/0016/0017/0018/0021/0022/0024/0026/0027/0028/0030/0031),
  `docs/private-connectors-threat-model.md`, `docs/telemetry.md`, `src-tauri/src/connectors/triggers.rs`,
  `src-tauri/migrations/015_memories.sql`, `src/lib/share-crypto.ts`,
  `src/components/routines/routine-templates.ts`, `src-tauri/src/hermes_bridge.rs`, `june-api/`

**Web (retrieved 2026-07-21):**
- June product site — https://opensoftware.co/june
- os-june repository — https://github.com/open-software-network/os-june
- Venice privacy architecture — https://docs.venice.ai/overview/privacy and https://venice.ai/privacy
- Venice E2EE/TEE launch — https://venice.ai/blog/venice-launches-end-to-end-encrypted-ai;
  coverage: https://cryptobriefing.com/encrypted-ai-inference-venice-release/
- Independent Venice privacy assessment — https://perspectiveai.xyz/is-venice-ai-safe-privacy-review-2026/
- Hermes Agent (Nous Research) — https://hermes-agent.org/;
  https://alphasignal.ai/news/nous-research-s-hermes-agent-can-now-spend-real-money-autonomously;
  https://openhosst.com/blog/hermes-agent
- Competitive landscape — https://macparakeet.com/; https://github.com/Muesli-HQ/muesli
