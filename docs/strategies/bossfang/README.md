# BossFang Strategy — The Governed Claw and the Universal Harness

**Scope:** BossFang/librefang's expanded role: an always-on personal-assistant gateway
("claw"-class functionality) *and* the container that unifies agent execution across
UAR, external AI harnesses (Claude Code, Codex, OpenCode, …), and workflow
orchestration — under one identity, governance, and audit plane.

**Status:** v1.0 strategy, 2026-07-18. Builds on
`docs/strategies/agents/README.md` (placements; sidecar = minimal local BossFang),
`docs/strategies/plugins/README.md` (capabilities, governance/identity §9, settings
substrate §11), `docs/strategies/skill-support/README.md` (outbound MCP), and the
flint stack (FRF bridges/media, flint-gate, Flint Vault).

---

## 1. What BossFang already is, and what it becomes

Today (FUNC-SPEC §07 + FRF/forge reality): OFP sync peer · remote model server · UAR
agent container · the deployment target for always-on Hands. This document adds two
roles that complete it:

1. **Channel gateway** — messaging/voice ingress (the "claw" role): the assistant is
   reachable from WhatsApp/Telegram/Matrix/etc. and works while every screen is off.
2. **Harness orchestrator** — external agent runtimes (Claude Code, Codex CLI,
   OpenCode, Gemini CLI, …) execute *inside* BossFang's governance envelope, unified
   with native UAR agents.

Both arrive as **capability plugins** (`knowme:channel`, `knowme:harness`) — no new
subsystem kinds, the §1-inversion of the plugins doc doing its job.

### 1.1 The vendored base (2026-07-18)

The [`GQAdonis/librefang`](https://github.com/GQAdonis/librefang) fork (BossFang —
"Libre Agent Operating System", community fork of openfang; 24 crates, 2,100+ tests)
is vendored as a submodule at **`rust/vendor/librefang`**. It is not a blank canvas —
large parts of this document already exist there and this strategy becomes a
*customization and integration* plan rather than a build plan:

| Strategy concern | librefang crate(s) already providing it |
|---|---|
| Channel gateway (§3) | `librefang-channels` — "pluggable messaging integrations" |
| Hands (§1, agents doc) | `librefang-hands` — "curated autonomous capability packages" |
| Skills + marketplace + claw compat (skill-support doc) | `librefang-skills` — "registry, loader, marketplace, and OpenClaw compatibility" |
| UAR agent metadata | `librefang-uar-spec` — UAR-AGENT-MD parser + AgentManifest translator |
| Harness/IDE embedding | `librefang-acp` — Agent Client Protocol adapter (Zed/VSCode/JetBrains over stdio JSON-RPC); `librefang-subprocess` + `librefang-runtime-sandbox-docker` for driving external CLIs sandboxed |
| Extensions + credentials | `librefang-extensions` — one-click MCP setup, credential vault, OAuth2 PKCE |
| Memory/karpathy wiki | `librefang-memory`, `librefang-memory-wiki` — durable markdown vault with provenance frontmatter |
| Governance/audit hooks | `librefang-runtime-audit`, `librefang-kernel-metering`, `librefang-kernel-router` |
| Model drivers | `librefang-llm-driver(s)` |

Integration posture: KnowMe-specific behavior (Cedar dual-identity per plugins doc §9,
OFP profile, the capability-card vocabulary, settings substrate) lands as **adapters
and upstreamable patches around these crates**, keeping the submodule tracking its
upstream; the ACP adapter also opens a *fourth* harness direction — BossFang agents
embedded into Zed/VSCode/JetBrains, not just external harnesses embedded into
BossFang. The gap analysis between `librefang-skills`' OpenClaw-compat model and this
stack's structural-boundary claims (§2) is the first integration task.

## 2. Positioning: the claw with a real boundary

The claw category ([OpenClaw](https://github.com/openclaw/openclaw)) proved enormous
demand for an always-on personal agent gateway. Its own
[security documentation](https://docs.openclaw.ai/gateway/security) states the limits
of its model: *"not a hostile multi-tenant security boundary"*; one trusted operator
per gateway; if multiple people can message the bot *"they share its delegated tool
authority"*; prompt injection *"not solved by system prompt guardrails alone"*;
plugins run in-process as trusted code. Mitigations are configuration hygiene
(allowlists, tokens, file permissions, a `security audit` command).

BossFang's claim is categorical, not incremental: **the properties OpenClaw documents
as out of scope are the properties this stack already enforces structurally.**

| Threat (OpenClaw's own list) | Their mitigation | BossFang's structure |
|---|---|---|
| Prompt injection from channels | Model choice + guardrails | **Cedar at the tool gate** — an injected instruction cannot exceed the agent's policy regardless of what the model believes (FUNC-SPEC §06) |
| Shared delegated authority | DM pairing/allowlist config | **Untrusted principals** (§3.2): senders are identities with scoped grants, not co-owners of the bot's tools |
| Tool blast radius | Deny-lists, sandbox flags | Per-agent, per-skill grants; budgets in governance blocks; deny-by-default |
| Plugin supply chain | "Treat plugin folders as trusted code" | WASM sandbox + reviewed capability/permission cards + signatures |
| Exposed gateway | Loopback + token hygiene | flint-gate auth, mTLS peers, `did:key:` payload encryption (OFP) |
| "What did it do?" | Log review | Hash-chained audit keyed on `(user, agent, capability, action)` |

We *adopt* their genuinely good UX-level inventions as schema'd defaults (§3.3) and
their `security audit` idea as a first-class posture tool (§6). We do not adopt their
trust model.

## 3. The channel gateway (`knowme:channel`)

### 3.1 Transport is already built

FRF ships Matrix inbound (live `/sync` streaming), ATProto outbound (writes to a
PDS), and media via LiveKit + str0m (negotiation proven) — channels are adapters over
existing bridge machinery, not new infrastructure. Each channel is a capability
plugin: settings schema (account/pairing config; secrets → Vault), events
(`message-received`, `delivery-status`), actions (`send`, `react`), and chunk support
for streaming replies into chat-shaped channels.

### 3.2 Ingress principals — the structural upgrade

Every external sender resolves to a principal: paired senders map to known contacts
(with explicit, scoped grants: "may converse", "may trigger the Reading-List Hand",
never blanket); unknown senders are `untrusted` by construction. Cedar context on
every action carries `channel`, `sender`, and `trust-tier`, so policy can say things
config files can't: *"content originating from channel senders may never parameterize
`shell-exec` for any agent"*. Channel content enters the pipeline tagged
(provenance discipline from agents doc §3) — it can *trigger*, it cannot *authorize*.

### 3.3 Adopted UX patterns (as settings schemas, defaults conservative)

Pairing-code first contact; allowlist mode; group mention-gating; per-sender session
isolation (each channel+sender pair is its own conversation context); reply-context
visibility limits. All are fields on the channel capability's settings schema —
auditable, syncable, org-overridable — not hand-edited config.

### 3.4 Reach

Where the user is: DM the assistant from the phone's messenger when the KnowMe app
isn't installed (or is out of reach); voice in/out via the media plane later. Every
conversation ingested through a channel is a normal KnowMe conversation — synced,
searchable, memory-connected — because it lands in the same entity graph.

## 4. The harness orchestrator (`knowme:harness`)

### 4.1 The unification claim

Codex Desktop orchestrates OpenAI's runtime; Claude's apps orchestrate Claude. KnowMe
+ BossFang orchestrates **all of them, plus local models, under one plane**: one
identity model (each harness task runs under an agent DID; RFC 8693 `act` claims),
one governance model (Cedar wraps every harness's tool activity at the boundary), one
audit trail, one model-routing layer (liter-llm + local engines), one UI (Hands,
conversations, A2UI). The prometheus-skill-pack already targets ten-plus harnesses
for *authoring*; this is the identical neutrality applied to *execution*.

### 4.2 The adapter contract

A harness adapter is a capability plugin declaring `knowme:harness`:

- **Execution**: spawns the harness CLI in a sandbox (container/jail; workspace
  mounts declared and reviewed like any permission; no host-direct default);
- **Translation**: harness I/O ⇄ AgentService/A2A tasks with `frf-agentproto`
  ContentBlocks — which is precisely the crate's design center; streaming progress
  becomes ordinary chunk traffic (A2UI-renderable);
- **Credentials**: harness API keys live in Flint Vault / keychain, injected into the
  sandbox at spawn, never stored in workspaces (the classic harness foot-gun);
- **Settings schema**: per-harness model prefs, workspace roots, tool allowances,
  concurrency caps;
- **Metering**: tokens/cost per harness run feed the same local usage stats that
  drive suggestions (model-config doc §7) — "this workflow ran 3× cheaper via local
  Qwen3-Coder than via harness X" is a card we can actually show.

A Hand's Step 2 can therefore select executor = UAR (native), a WASM component agent,
*or* a harness adapter — Step 3's placement unchanged. Same wizard, same audit, same
pause/resume.

### 4.3 Workflow orchestration

BossFang's workflow layer composes across executors: A2A tasks between native Hands,
agent-exposed skills (skill-support doc §2), and harness tasks — fan-out, gates,
sequencing — with the kbd-orchestrator waypoint pattern (phased plans, assessments,
sign-offs) as the proven shape for long multi-phase work. Workflows are themselves
declarative artifacts: shareable, governable, auditable like everything else.

### 4.4 Honest constraints

Vendor CLIs change and their ToS govern automation — adapters pin versions and stay
within each vendor's supported headless modes; a harness is only as controllable as
its flags allow (some tool-permission surfaces are coarse; Cedar then gates at the
sandbox boundary: filesystem scope, network egress, secret access); harness runs are
the one place where prompt-level governance is *advisory* — which is exactly why the
sandbox + Vault + audit boundary is the enforcement line.

## 5. Placement continuity

The agents-doc placement table needs no change: **sidecar = minimal local BossFang.**
Claw features are configuration on any placement — a laptop sidecar can run channels
while the lid is open; the M4-mini "always-on Hands node" (local-models doc §10) is
the natural home; a VPS personal-cloud instance serves the road. One contract, and
Settings → Instances already renders the topology. Channels default **off** on
non-always-on placements to avoid the "assistant that answers only when my laptop is
awake" trap — the wizard says which instance will answer.

## 6. Posture audit (adopt the idea, exceed the implementation)

`bossfang audit` (forge-cli verb + Settings surface): walks channel policies × granted
tools × agent policies and reports *reachable authority* — "an unknown Telegram
sender can reach: nothing. A paired sender can reach: conversation + Reading-List
trigger." Because grants, principals, and policies are data (not config-file
convention), the audit is a query with proofs, not a lint. Ship it in the same
release as the first channel plugin; it is the trust story's demo.

## 7. Open questions

1. **Channel plugin placement** — channels only on `sidecar`/`peer` placements
   (default), or allow phone-local channel bridges (iMessage requires a Mac anyway)?
2. **Sender identity portability** — pairing a sender on one instance: does the
   grant sync across the user's mesh (user-scoped) or stay per-instance
   (device-scoped)? Default: user-scoped identity, per-instance *reachability*.
3. **Harness sandbox tier on macOS** — containers vs. dedicated OS user vs.
   seatbelt profiles; pick per-OS defaults and document the guarantees honestly.
4. **Voice loop scope** — str0m negotiation is proven but full sovereign SFU is
   deferred in FRF; sequence voice channels after text channels ship.
5. **Naming** — "BossFang" (product) vs "librefang" (OSS core) boundary for the
   channel/harness features: which land in the open core?

## 8. Implementation map

- FRF: channel adapters over `frf-bridge-matrix`/`frf-bridge-atproto` patterns; a
  generic channel-adapter trait in `frf-librefang` (bus + tenant-actor registry
  already exist); sender-principal mapping into the identity plane.
- flint-gate: actor-token issuance for harness runs; pairing-code flows.
- `knowme_plugin_host`: `knowme:channel` + `knowme:harness` cards in the recognized
  registry (plugins doc §3 table grows by two).
- Adapters (first wave): Claude Code, Codex CLI, OpenCode — one shared
  sandbox/translation core, thin per-harness shims.
- KnowMe app: Instances panel shows channel/harness capabilities per instance;
  Hands wizard executor selection; audit views for channel-originated runs.
- `bossfang audit` in forge-cli; posture panel in Settings → Privacy.

## 9. References

- [OpenClaw](https://github.com/openclaw/openclaw) · [security model](https://docs.openclaw.ai/gateway/security) ·
  [docs](https://docs.openclaw.ai/) · [assistant setup](https://docs.openclaw.ai/start/openclaw) ·
  [freeCodeCamp: build & secure a personal agent with OpenClaw](https://www.freecodecamp.org/news/how-to-build-and-secure-a-personal-ai-agent-with-openclaw/)
- In-repo: `docs/strategies/agents/README.md` · `docs/strategies/plugins/README.md` ·
  `docs/strategies/skill-support/README.md` · FRF `README.md` (phase 18: Matrix
  inbound, ATProto outbound, str0m negotiation) · `frf-agentproto`, `frf-librefang`
