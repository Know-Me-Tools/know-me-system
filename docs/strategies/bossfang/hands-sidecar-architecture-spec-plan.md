# Hands on BossFang — Architecture, Functional Specification & kbd Project Plan

**Status:** Draft v1.0 · 2026-07-21
**Owner:** KnowMe LLC
**Supersedes:** the UAR-as-scheduler assumption in `docs/implementation-roadmap.md` (M1) for sidecar/peer placements
**Inputs:**
- `docs/strategies/bossfang/README.md` v1.0 (2026-07-18) — BossFang roles, placement-as-configuration, security model
- `docs/strategies/agents/README.md` — AgentService contract, four placements
- `docs/strategies/uar-knowme-runtime-integration.md` (2026-07-19) — the integration precedent this document mirrors
- `rust/vendor/librefang` @ `0493da782` ("BossFang — Libre Agent Operating System", v2026.7.11, 29 workspace crates)
- `rust/crates/knowme_hands/src/lib.rs` — Hands domain model (real types, stubbed execution)
- `docs/research/os-june-comparison-report.md` (2026-07-21) — earned-autonomy, trigger, and sandbox recommendations
- `.kbd-orchestrator/project.json` — kbd model policy, execution-agent registry, constraints

---

## 1. Purpose & Scope

KnowMe's **Hands** are autonomous, scheduled agents ("work while you sleep"). Per the owner's
directive, Hands execution is **not** built into the KnowMe Rust core: it is delegated to **BossFang**
(the `librefang` fork vendored at `rust/vendor/librefang`), which owns **scheduling, workflow,
orchestration, configuration, and the Hands data stores**. KnowMe owns the shell, the wizard UX, the
policy gate, and the projection of runs/conversations into the KnowMe entity graph.

This document defines:

1. **Architecture** — how BossFang runs as a supervised **sidecar** of the KnowMe desktop app, how it
   is reached from mobile, and how Hands data flows between BossFang stores and KnowMe stores.
2. **Binary supply chain** — how the BossFang sidecar binary and skill/plugin binaries are **built,
   signed, stored, and obtained**, based on current best practices (Tauri `externalBin`, cosign/Sigstore,
   SLSA provenance, OCI artifacts, digest pinning).
3. **Functional specification** — numbered, testable requirements.
4. **Project plan** — phased delivery driven by **agentic AI code generation under kbd**, expressed as
   kbd phases + OpenSpec changes with model-class assignments and exit criteria.

**Non-goals:** re-implementing a scheduler in `gen_ui_agent`; channel-gateway UX (covered by
`knowme:channel` in the strategy doc); OFP sync internals (M2, `knowme_ofp`).

---

## 2. Key Facts Established by Research

### 2.1 The vendored BossFang is a shipping system, not scaffolding

`rust/vendor/librefang` (v2026.7.11) provides, out of the box:

| Need | Crate | Evidence |
|---|---|---|
| Cron scheduling, persistence, tick loop | `librefang-kernel` (`src/cron.rs`, `cron_delivery.rs`) | DashMap + JSON persistence, delivery queue |
| Approvals, config store, agent identity | `librefang-kernel` (`approval.rs`, `config.rs`, `agent_identity_registry.rs`) | — |
| Curated autonomous capability packages ("hands" marketplace) | `librefang-hands` (`registry.rs`, `hands_hub.rs`) | install/activate flows + hub client |
| HTTP/WS daemon API | `librefang-api` | Axum server, ACP pipes, approval endpoints |
| Agent loop, artifact store, browser tools, A2A | `librefang-runtime` (+`-audit`, `-media`, `-mcp`, `-sandbox-docker`) | — |
| Channel sidecars (out-of-process) | `librefang-channels` (`src/sidecar.rs`) | supervisor reusable for other sidecars |
| Skills/plugins + credential vault + OAuth2 PKCE | `librefang-skills`, `librefang-extensions` | OpenClaw-compatible loader |
| Single daemon binary | `librefang-cli` → `[[bin]] name = "librefang"` | `bossfang start` serves dashboard on :4545 |
| UAR as an LLM backend | `librefang-llm-drivers` (`drivers/uar.rs`, feature `uar-driver`) | reuses KnowMe's UAR investment |

Critically, the submodule already contains a **frozen sidecar protocol v1** (NDJSON-over-stdio wire
spec, `docs/architecture/sidecar-protocol.md`) with a conformance corpus (`conformance/sidecar/corpus/`),
and an internal assessment (`docs/architecture/uar-sidecar-assessment.md`, 2026-06-17) recommending
UAR run as a supervised sidecar — with the `[uar.sidecar]` config surface (`enabled`, `command`,
`endpoint`, `restart`, backoff) already landed (`b5e9813bc`). **KnowMe's sidecar design should reuse
BossFang's own supervisor and protocol rather than inventing a parallel one.**

### 2.2 KnowMe's current Hands state

`knowme_hands` has the full domain model — `HandTemplate` (6 archetypes with default cron/event
schedules), `HandSchedule::{Cron,Event,Manual}`, `DeploymentTarget::{Local, BossFang{instance_id}}`,
the 3-step wizard (`HandIdentity → HandConfiguration → HandDeployment` carrying a `cedar_policy`
string), and `HandRun`/`SkillInvocation` audit records with `CedarDecision`. Execution returns
`NotImplemented`. The KnowMe workspace does **not** yet depend on, spawn, or configure librefang;
desktop `tauri.conf.json` has **no** `externalBin` today.

### 2.3 External best practices (web research, 2026-07-21)

- **Tauri sidecars** are first-class: `bundle.externalBin` + `$TARGET_TRIPLE`-suffixed binaries +
  `tauri_plugin_shell` spawn + capability-scoped permissions in `capabilities/*.json`
  ([Tauri v2 sidecar docs](https://v2.tauri.app/develop/sidecar/)). For **long-running** sidecars the
  community standard is spawn-at-app-start with stdio supervision, escalating to TCP/localhost for
  rich IPC ([Evil Martians, 2025](https://evilmartians.com/chronicles/making-desktop-apps-with-revved-up-potential-rust-tauri-sidecar)).
- **Checksums alone are insufficient**: a compromised release can swap binary + `.sha256` atomically.
  The current baseline is **Sigstore cosign keyless signing** (OIDC identity, Rekor transparency log),
  plus **SLSA provenance** and **SBOMs** (syft) ([nanobrew security audit issue](https://github.com/justrach/nanobrew/issues/118);
  [GoReleaser supply-chain guide](https://goreleaser.com/blog/supply-chain-security/);
  [OneUptime code-signing guide](https://oneuptime.com/blog/post/2026-01-07-go-code-signing/view)).
- **WASM/plugin artifacts**: OCI registries (via `oras`, media type
  `application/vnd.wasm.content.layer.v1+wasm`/…`+component`) are the pragmatic 2026 standard —
  content-addressable, cosign-compatible, pull-by-digest in production
  ([Systems Hardening, 2026-05](https://www.systemshardening.com/articles/wasm/wasm-module-registry-security/);
  [wasmCloud packaging docs](https://wasmcloud.com/docs/v1/concepts/packaging/)).
  **warg** is the purpose-built Component Model registry protocol (append-only transparency log) but
  still early; **IPFS** is content-addressed but pinning is unreliable and is "not practical as a
  primary distribution mechanism today" ([ironclaw RFC #101](https://github.com/nearai/ironclaw/issues/101)).
- **Pin by digest/revision, never by mutable tag.** KnowMe already does this for model artifacts
  (`gen_ui_inference/src/catalog.rs`: revision-pinned HF URLs + SHA-256 from LFS etags, enforced by
  test). The same catalog pattern generalizes to sidecar and skill binaries.

---

## 3. Architecture

### 3.1 Ownership matrix

| Concern | Owner | Notes |
|---|---|---|
| Hand scheduling (cron/event tick, catch-up, delivery queue) | **BossFang** (`librefang-kernel`) | KnowMe never ticks a schedule |
| Workflow / orchestration / agent loop | **BossFang** (`librefang-runtime`) | UAR available as LLM driver (`uar-driver` feature) |
| Hands configuration + run history + scheduler state stores | **BossFang** (its own data dir) | per-instance; definitions sync via OFP later (M2) |
| Hand definition UX (3-step wizard), template gallery | **KnowMe** (`knowme_hands` + desktop/mobile UI) | definitions translated into BossFang agent+cron config |
| Policy enforcement at the tool gate | **KnowMe** (Cedar, dual identity) | BossFang calls skills through the KnowMe policy bridge |
| Conversation/run projection into entity graph + SurrealDB | **KnowMe** (`gen_ui_agent`, `gen_ui_db_graph`) | same projection rule as chat: all ingress lands in the graph |
| Skill execution sandbox | **KnowMe** (Cedar + WASM host M3; loopback-proxy write pattern — see os-june report §5.3) | BossFang-side docker sandbox optional for peer deployments |
| Trust ladder (earned autonomy) | **KnowMe domain model**, persisted via BossFang run records | see FR-7 |

This preserves the strategy doc's invariant: **KnowMe-specific behavior lands as adapters and
upstreamable patches around the librefang crates; the submodule tracks upstream.**

### 3.2 Placement model (unchanged from strategy; Hands-specific behavior defined)

```
                 ┌──────────────────────── KnowMe app ────────────────────────┐
                 │  Shell UI (wizard, gallery, run history, approvals)        │
                 │  knowme_hands (domain)  ──►  HandsBridge (adapter)         │
                 └───────────────┬───────────────────────┬────────────────────┘
                                 │ AgentService           │ FFI (mobile)
                                 │ localhost HTTP/SSE     │
        ┌────────────────────────▼─────────┐   ┌──────────▼───────────┐
        │  BossFang sidecar (desktop)      │   │  Embedded UAR        │
        │  librefang daemon, own data dir  │   │  (mobile: no BossFang│
        │  cron + runtime + hands registry │   │  possible on-device) │
        └───────────────┬──────────────────┘   └──────────────────────┘
                        │ OFP (M2)
        ┌───────────────▼──────────────────┐
        │  BossFang peer (always-on box /  │
        │  personal cloud — "BossFang      │
        │  instance" deployment target)    │
        └──────────────────────────────────┘
```

- **`embedded`** (mobile, and desktop local-lane fallback): UAR library in-process; Hands run only in
  a reduced "foreground reminder" mode. *No BossFang on mobile* — mirrors the UAR doc's explicit
  constraint.
- **`sidecar`** (desktop default): the `librefang` binary as a Tauri-bundled, app-scoped child process
  speaking AgentService on localhost. **"A sidecar speaking AgentService on localhost *is* a minimal
  local BossFang"** — and per the strategy rule, the sidecar never gets a private API.
- **`service`** (desktop, opt-in "run at login"): same binary installed as launchd agent / systemd
  user unit / Windows Service so Hands run while the app is closed. The app connects to the running
  daemon instead of spawning it.
- **`peer`**: a user-owned always-on BossFang instance (`DeploymentTarget::BossFang{instance_id}`),
  reached over OFP with `did:key:` identity (M2 dependency).

### 3.3 Sidecar process & supervision architecture

**Acquisition:** the `librefang` binary is bundled with the desktop app as a Tauri sidecar
(`bundle.externalBin`), one artifact per target triple (§4.1). Downloaded-at-runtime is *rejected*
for the sidecar itself (a sovereignty product must work air-gapped out of the box); runtime download
is reserved for optional skill binaries (§4.3).

**Lifecycle state machine** (reuses the model-download lifecycle shape already used in
`gen_ui_inference`): `notInstalled → verifying → starting → ready ⇄ degraded → stopped`, with
`failed{reason}` from any state. Supervision responsibilities (in a new
`tauri-plugin-gen-ui` module, delegating to a Rust `SidecarSupervisor` in a thin
`knowme_sidecar` crate so the logic is testable outside Tauri):

1. **Verify before first spawn** — SHA-256 against the pinned digest baked into the build; on macOS
   also check codesign validity (the binary is signed & notarized as part of the app bundle).
2. **Spawn** via `tauri_plugin_shell` with capability-scoped permissions
   (`shell:allow-execute`, `sidecar: true`, args allowlist — never `args: true`)
   ([Tauri sidecar docs](https://v2.tauri.app/develop/sidecar/)).
3. **Port & auth**: bind `127.0.0.1` only, port 0 (OS-assigned), handed to the daemon via arg;
   per-boot bearer token passed via env, required on every request. No mTLS needed on loopback;
   token + loopback + Cedar at the tool gate is the defense-in-depth stack.
4. **Health & readiness**: poll `/healthz` until ready (budget 10 s, exponential backoff); expose
   `degraded` on repeated health failure while the process lives.
5. **Restart policy**: restart on unexpected exit with capped exponential backoff (align with
   librefang's own `[uar.sidecar]` restart/backoff surface); after N=5 failures, surface
   `failed` to Settings with logs and a "reset sidecar state" action.
6. **Graceful shutdown**: on app exit, send the daemon's shutdown RPC, wait 5 s, then kill. Orphaned
   daemons from crashes are reaped on next boot by lockfile + PID check.
7. **Data dir**: `<app_data_dir>/bossfang/` (config, cron JSON, run records, artifact store),
   excluded from backups the user hasn't opted into; deletion from Settings wipes it (verified
   deletion is a sovereignty feature).

**IPC:** AgentService over localhost HTTP + SSE for runs/approvals/events (the same contract used
for the UAR service placement, so `gen_ui_agent` needs *one* client). BossFang-internal sidecars
(channel, harness CLIs) use librefang's frozen NDJSON-over-stdio protocol v1 — KnowMe never speaks
that protocol directly.

**Event mapping:** BossFang run/approval/log events map onto KnowMe content blocks via the existing
AG-UI/A2UI adapter path (`gen_ui_protocol`), following the UAR doc's rule: **no event silently
dropped**; unknown events surface as `unsupported` (os-june report §5.9 pattern).

### 3.4 Data ownership & flows

- **Definitions**: `HandDefinition` (KnowMe domain) is translated by `HandsBridge` into BossFang
  agent config + cron entry + skill grants. BossFang stores its native form; KnowMe keeps the domain
  form as an entity so the wizard round-trips and OFP can sync definitions later.
- **Runs**: BossFang owns the authoritative run ledger. KnowMe subscribes (SSE) and projects each
  `HandRun` + `SkillInvocation` (with `CedarDecision`) into SurrealDB for the Runs UI and the audit
  export (M7). Projection is idempotent on BossFang's run IDs.
- **Approvals**: pending approvals are pushed to the shell as native notifications; the decision
  round-trips through the same endpoint BossFang's own API exposes (`librefang-api` approval
  endpoints), wrapped by KnowMe's Cedar dual-identity check.
- **Memory writes**: a Hand may write to the KnowMe knowledge graph only through
  `gen_ui_agent::memory` behind Cedar — BossFang gets a skill surface, not a DB handle.

### 3.5 Security model (delta over strategy doc §2)

- Cedar evaluated at the **skill invocation boundary inside KnowMe's bridge**, keyed
  `(user, hand, skill, action)`, hash-chained audit — BossFang proposes, KnowMe disposes.
- **Trust ladder (earned autonomy)**, adopted from the os-june comparison (P0 recommendation #1):
  `HandTrustMode::{ReadOnly, Approval, Autonomous}` with promotion to `Autonomous` only after 3
  consecutive clean `Approval` runs; demotion on any denied/errored run. Stored on `HandDefinition`,
  enforced by the bridge, displayed in the wizard step 3.
- The sidecar binary is a **signed, pinned, bundled artifact** (§4); skill WASM executes only in the
  M3 `knowme_plugin_host` sandbox (64 MiB / 30 s epoch / no direct network).

---

## 4. Binary Supply Chain — Build, Sign, Store, Obtain

### 4.1 BossFang sidecar binary

**Build.** CI builds `librefang` (bin `librefang`, feature set `knowme-sidecar` = kernel + runtime +
api + hands + uar-driver; channels/extensions off for the app-scoped sidecar) for:

| Target triple | Notes |
|---|---|
| `aarch64-apple-darwin` | signed + notarized with the app |
| `x86_64-apple-darwin` | per platform-support policy |
| `x86_64-pc-windows-msvc` | signed (Authenticode) |
| `x86_64-unknown-linux-gnu` | `.deb`/AppImage bundling |

Rust flags for reproducibility: `--locked`, `trim-paths`, stripped, pinned toolchain (librefang pins
1.94.1 via rust-toolchain). The KnowMe release pins a **librefang version + commit SHA** in
`versions.toml` (single source of truth), with a June-style compatibility matrix page recording which
KnowMe features gate on which BossFang version.

**Naming & bundling (Tauri contract):** artifacts are renamed to
`desktop/src-tauri/binaries/bossfang-<TARGET_TRIPLE>[.exe]` and listed as
`bundle.externalBin: ["binaries/bossfang"]`
([Tauri sidecar docs](https://v2.tauri.app/develop/sidecar/)). Spawn permissions declared in
`desktop/src-tauri/capabilities/` with an explicit args allowlist (`serve`, `--port`, `--data-dir`,
`--auth-token` validators).

**Sign & attest (release pipeline):**
1. `sha256sum` per artifact → `checksums.txt` (necessary, not sufficient).
2. **cosign keyless `sign-blob`** per artifact from the release workflow's OIDC identity →
   `.sigstore.json` bundles published alongside; identity pinned to the release workflow path.
3. **SLSA provenance** via `slsa-github-generator`; **SBOM** via `syft` (SPDX) per artifact.
4. Record the **source commit in OCI labels** for any container/peer image of BossFang
   (`org.opencontainers.image.revision`) and pin deploys as signed git tags — the June `/verify`
   playbook (os-june report §5.6).

**Obtain (client side):** nothing to obtain at runtime — the binary is in the bundle. First-boot
verification is local (digest + codesign). The Settings → Hands panel shows sidecar version,
librefang commit, and verification status (trust surfacing, not theater).

### 4.2 Where binaries live (storage)

| Artifact | Store of record | Addressing |
|---|---|---|
| Sidecar binaries + bundles + SBOMs | GitHub Releases on the KnowMe repo (immutable tag) + GHCR OCI mirror | tag + digest; `versions.toml` pins both |
| Peer/container images | GHCR `ghcr.io/<org>/bossfang` | pull **by digest** in compose/k8s overlays under `deploy/` |
| Skill WASM components (M3) | OCI registry (GHCR) as `…+component` artifacts via `oras`; IPFS as optional mirror, not primary | content digest; Ed25519 `ManifestSignature` over `did:key:` per `knowme_plugin_host` |
| Skill metadata/catalog | signed JSON catalog (cosign-signed), fetched like the model catalog | revision + SHA-256 |

Rationale: OCI gives content-addressing, existing auth, cosign compatibility, and caching today;
warg becomes the primary registry for Component Model skills once its tooling stabilizes (tracked as
a P2 decision point); IPFS pinning is too unreliable to be the primary path
([ironclaw RFC #101](https://github.com/nearai/ironclaw/issues/101)) — keep the `WasmRef{ipfs_cid,
sha256}` field but treat CID as a mirror reference.

### 4.3 Skill binaries at runtime (download path)

For optional, post-install skill artifacts (WASM components, harness CLI shims), reuse the
**model-catalog pattern** from `gen_ui_inference`:

```text
SkillArtifact { id, revision, oci_digest | url, sha256, byte_len, min_host_version, signature }
```

with lifecycle `notInstalled → downloading → verifying → loading → ready` (+cancelling/failed),
resumable Range downloads, engine-scoped cache dir (`<app_data_dir>/skills/`), and a test asserting
every catalog entry is revision/digest-pinned with a 64-hex digest — the exact discipline already
enforced in `catalog.rs:133-144`. Loading a skill binary that fails signature/digest verification is
**fail-closed**.

### 4.4 Update strategy

- Sidecar updates ship **with app updates** (auto-update channel) — never self-updating in place.
- Skill artifacts update via catalog revision bump + user consent in the Skills UI; rollbacks keep
  the previous verified artifact until the new one reaches `ready`.
- A `bossfang doctor` posture command (strategy §6) surfaces version skew, unsigned artifacts, and
  audit-chain gaps in Settings.

---

## 5. Functional Specification

Requirements are testable; IDs are stable for OpenSpec/kbd traceability. MoSCoW: **M**ust (v1),
**S**hould (v1.1), **C**ould (later).

### 5.1 Sidecar lifecycle

- **FR-1 (M)** On desktop launch with Hands enabled, the app verifies (SHA-256 + codesign on macOS)
  and spawns the bundled BossFang sidecar bound to `127.0.0.1` on an OS-assigned port with a per-boot
  auth token. *Acceptance:* sidecar reaches `ready` ≤ 10 s on a clean machine; UI shows status.
- **FR-2 (M)** Unexpected sidecar exit triggers capped exponential-backoff restart (≤ 5 attempts);
  exhaustion surfaces `failed` with logs and a state-reset action. *Acceptance:* kill -9 simulation
  recovers; 6th failure yields the failed UI state.
- **FR-3 (M)** App quit shuts the sidecar down gracefully (RPC, 5 s, then kill); orphaned processes
  from crashes are reaped on next boot. *Acceptance:* no orphan after forced app kill + relaunch.
- **FR-4 (S)** "Run at login" installs/removes the same binary as a launchd agent / systemd user
  unit / Windows Service; the app then attaches instead of spawning. *Acceptance:* Hand fires with
  the app closed.
- **FR-5 (M)** Wiping `<app_data_dir>/bossfang/` from Settings verifiably deletes all Hand configs,
  run history, and cached artifacts. *Acceptance:* directory absent post-wipe; restart re-initializes.

### 5.2 Hands definition & deployment

- **FR-6 (M)** The 3-step wizard (`HandIdentity → HandConfiguration → HandDeployment`) creates a
  `HandDefinition` and translates it into BossFang agent config + schedule + skill grants.
  *Acceptance:* a Researcher Hand created in the wizard appears in `librefang` config and fires on
  its cron in a time-travel test.
- **FR-7 (M)** Trust ladder: new Hands start at `Approval` (or `ReadOnly`); promotion to
  `Autonomous` requires 3 consecutive clean approval runs; any denied/errored run demotes.
  *Acceptance:* unit + integration tests on the ladder transitions; wizard step 3 displays the mode.
- **FR-8 (M)** `HandSchedule::{Cron, Event, Manual}` all supported. Event triggers persist
  per-source cursors (e.g., Gmail history id, calendar scan watermark) in BossFang config.
  *Acceptance:* synthetic event fires the Hand exactly once per cursor advance.
- **FR-9 (M)** `DeploymentTarget::{Local, BossFang{instance_id}}`: Local = sidecar/service; BossFang =
  peer over OFP (M2-gated; UI hidden until then). *Acceptance:* peer target round-trips through the
  domain model and fails gracefully pre-M2.

### 5.3 Execution, policy, audit

- **FR-10 (M)** Every skill invocation proposed by a running Hand is authorized by Cedar
  (`(user, hand, skill, action)`) inside the KnowMe bridge before dispatch; decisions recorded as
  `SkillInvocation` with `CedarDecision`. *Acceptance:* policy deny blocks dispatch and appears in
  the run ledger.
- **FR-11 (M)** Pending approvals reach the shell as notifications; the user's decision round-trips
  to BossFang. *Acceptance:* approval latency < 2 s local; timeout policy (deny) documented.
- **FR-12 (M)** Run ledger projection: every BossFang run event projects idempotently into SurrealDB
  (`HandRun`, `SkillInvocation`); unknown events surface as `unsupported`, never dropped.
  *Acceptance:* duplicate delivery does not double-record; conformance corpus replay passes.
- **FR-13 (S)** Hand writes to the knowledge graph pass through `gen_ui_agent::memory` with Cedar;
  tombstoned deletions propagate (os-june P0 #4). *Acceptance:* deleted memory does not reappear
  after re-sync/re-run.

### 5.4 Supply chain

- **FR-14 (M)** Release pipeline produces target-triple sidecar artifacts with SHA-256 checksums,
  cosign keyless signatures, SLSA provenance, and SBOMs; `versions.toml` pins librefang version +
  commit + artifact digests. *Acceptance:* `make verify`-style CI gate fails on unsigned artifact.
- **FR-15 (M)** First-boot verification fails closed: mismatched digest or invalid signature ⇒
  sidecar never spawns; user sees a repair/reinstall flow. *Acceptance:* bit-flipped binary is
  rejected.
- **FR-16 (S)** Skill catalog entries are digest-pinned and signed; runtime download is resumable and
  verify-before-load. *Acceptance:* interrupted download resumes; tampered artifact rejected.

### 5.5 Mobile

- **FR-17 (M)** Mobile shows Hands as "runs on your desktop/peer" with read-only run history
  projected via entity sync; no local scheduling. *Acceptance:* Flutter Hands screen renders ledger
  from entities without any BossFang dependency.

---

## 6. Project Plan — Agentic Code Generation under kbd

### 6.1 How kbd drives this program

Per `.kbd-orchestrator/project.json`, work is organized as **phases** (each with `assessment.md`,
`plan.md`, `execution.md`, `goals.md`, `progress.json`, handoffs) containing ordered **OpenSpec
changes** applied via `/opsx:apply <change>`, with a **model policy** by complexity
(`opsx-apply-low` → small, `-medium` → medium, `-high` → frontier; assess/plan/reflect → frontier)
and a registry of execution agents (codex, cline, kimi, roo-code…). This program adds one new kbd
phase tree:

```
.kbd-orchestrator/phases/hands-on-bossfang/
  assessment.md        ← initial deep-dive (this document + submodule analysis as input)
  goals.md             ← FR IDs as the goal contract
  plan.md              ← the change list below, with dependencies/agents/model classes
  execution.md         ← running log
  children/            ← per-change working dirs
  handoffs/            ← agent-to-agent handoff envelopes
```

Rules of engagement for the generating agents (from `constraints.md` + base rules): gen-ui-core
boundary respected (all networking/policy/persistence in Rust); React layering
(Component → Hook → Store → invoke); no private sidecar API; build-health green before certify;
every OpenSpec change carries `proposal.md`, `design.md`, `tasks.md`, `specs/`.

### 6.2 Phase plan

**Phase HB-0 — Foundation & verification harness** *(prereq for everything)*

| Change | Scope | Model class | Suggested agent | Exit criteria |
|---|---|---|---|---|
| `bossfang-sidecar-build` | CI build matrix for `librefang` `knowme-sidecar` feature set; target-triple renaming; `externalBin` wiring; capabilities file | medium | codex | FR-14 CI green; sidecar spawns from dev bundle |
| `bossfang-sidecar-supervisor` | `knowme_sidecar` crate + Tauri module: verify→spawn→health→restart→shutdown; state machine; data-dir management | high | cline | FR-1,2,3,5,15 passing |
| `hands-bridge-agentservice` | `HandsBridge` adapter in `gen_ui_agent`: AgentService client shared with UAR-service path; auth token; SSE subscribe | high | codex | bridge round-trip vs `bossfang start` in dev |

**Phase HB-1 — Hands v1 (wizard → schedule → approve → ledger)** *(depends on HB-0)*

| Change | Scope | Model class | Suggested agent | Exit criteria |
|---|---|---|---|---|
| `hands-definition-translation` | `knowme_hands` → BossFang config compiler (agent, cron, skill grants); entity persistence of definitions | medium | kimi | FR-6, FR-8(cron/manual) |
| `hands-cedar-gate` | Cedar dual-identity check at skill boundary; `SkillInvocation` records; hash-chained audit | high | cline | FR-10 |
| `hands-approvals-ux` | desktop notifications + approval inbox; decision round-trip; timeout=deny | medium | roo-code | FR-11 |
| `hands-ledger-projection` | SSE → SurrealDB idempotent projection; `unsupported` surfacing; Runs UI (React feature + mobile read view) | medium | codex | FR-12, FR-17 |
| `hands-trust-ladder` | `HandTrustMode` + promotion/demotion rules + wizard step-3 surfacing | low | kimi | FR-7 |

**Phase HB-2 — Event triggers & run-at-login** *(depends on HB-1; parallelizable)*

| Change | Scope | Model class | Exit criteria |
|---|---|---|---|
| `hands-event-triggers` | connector event sources + cursor persistence (Gmail delta, calendar) reusing librefang trigger surfaces | high | FR-8(event) |
| `hands-service-placement` | launchd/systemd/Windows-service install + attach mode | medium | FR-4 |

**Phase HB-3 — Skill artifact supply chain** *(depends on HB-0; feeds M3)*

| Change | Scope | Model class | Exit criteria |
|---|---|---|---|
| `skill-catalog-oci` | signed skill catalog; OCI pull-by-digest; resumable verify-before-load cache | medium | FR-16 |
| `plugin-host-wasm-gate` | `knowme_plugin_host` execution behind Cedar with catalog-sourced artifacts | high | M3 alignment; sandbox limits enforced |

**Phase HB-4 — Peer placement & sync hooks** *(M2-dependent)*

| Change | Scope | Model class | Exit criteria |
|---|---|---|---|
| `hands-peer-target` | `DeploymentTarget::BossFang` over OFP; definitions sync, run history per-instance | high | FR-9 |

### 6.3 Sequencing & effort

```text
HB-0 (2–3 wks agent-time)  ──► HB-1 (3–4 wks) ──► HB-2 (2 wks)
                    └──────► HB-3 (2–3 wks, parallel) ──► HB-4 (post-M2)
```

kbd executes each change as: assess (frontier) → plan (frontier) → `/opsx:apply` (per model class) →
build-health gate → reflect/certify (frontier) → archive. Human sign-off required at each phase
boundary (the established waypoint pattern: `implementation_ready` → applied → certified).

### 6.4 Risk register

| Risk | Likelihood | Mitigation |
|---|---|---|
| Upstream openfang drift breaks the fork | M | pin by commit in `versions.toml`; compatibility matrix; upstreamable-patches-only rule; conformance corpus in CI |
| Sidecar binary size/notarization friction | M | minimal `knowme-sidecar` feature set; sign in release CI; measure cold-start ≤ 10 s budget |
| Two sources of truth for schedules (BossFang cron vs KnowMe definitions) | M | KnowMe domain is authoritative in the wizard; compiler is one-way; reconciliation check in `bossfang doctor` |
| Agent-generated Cedar policies too permissive | H | policy snapshots reviewed by frontier model + human; deny-by-default templates per archetype |
| Mobile users expect local autonomy | M | explicit UX copy ("Hands run on your desktop/peer"); revisit if LiteRT-LM + UAR embedded matures |
| GHCR/OCI reliance conflicts with air-gap promise | L | all required artifacts bundled; network only for optional skills; offline path documented |

---

## 7. Sources

**Repository (read 2026-07-21):** `rust/vendor/librefang` (`Cargo.toml`, `README.md`,
`docs/architecture/sidecar-protocol.md`, `sidecar-channels.md`, `uar-sidecar-assessment.md`,
`conformance/sidecar/corpus/`), `docs/strategies/bossfang/README.md`, `docs/strategies/agents/README.md`,
`docs/strategies/uar-knowme-runtime-integration.md`, `rust/crates/knowme_hands/src/lib.rs`,
`rust/crates/knowme_plugin_host/src/lib.rs`, `rust/crates/gen_ui_inference/src/catalog.rs`,
`.kbd-orchestrator/project.json`, `.kbd-orchestrator/constraints.md`, `openspec/changes/`.

**Web (retrieved 2026-07-21):**
- Tauri v2 sidecar documentation — https://v2.tauri.app/develop/sidecar/
- Long-running Tauri sidecars (Evil Martians) — https://evilmartians.com/chronicles/making-desktop-apps-with-revved-up-potential-rust-tauri-sidecar
- Checksum insufficiency + cosign recommendation — https://github.com/justrach/nanobrew/issues/118
- GoReleaser supply-chain security — https://goreleaser.com/blog/supply-chain-security/
- Go binary signing pipeline (cosign/SLSA/SBOM) — https://oneuptime.com/blog/post/2026-01-07-go-code-signing/view
- WASM registry security (warg vs OCI) — https://www.systemshardening.com/articles/wasm/wasm-module-registry-security/
- WASM plugins via OCI — https://oneuptime.com/blog/post/2026-02-24-distribute-wasm-plugins-oci-registry-istio/view
- wasmCloud OCI packaging — https://wasmcloud.com/docs/v1/concepts/packaging/
- Plugin registry trade-offs incl. IPFS — https://github.com/nearai/ironclaw/issues/101
