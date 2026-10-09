# Prometheus Skills in KnowMe — integration guide

> **Status:** design, 2026-07-30. Written from
> `prometheus-skill-pack` phase `ideation-and-decision-tools`.
> KnowMe is the reference container for this skill set: the skills described here are
> intended to ship **inside** this app, on web, Tauri desktop, and Flutter/Rust-FFI
> mobile.

## Why this document exists

The Prometheus skill pack has 145 skills. **107 of them (74%) are prompt-only** and
will run in KnowMe today with no runtime work at all. The other 38 ship shell scripts,
and iOS and Android cannot spawn processes.

This guide says which skills land where, what already exists in this repo to host
them, and what has to be true before the rest can follow. It is deliberately explicit
about what is *not* built yet.

## The three execution tiers

This mirrors the `ui-surface` pattern the skill pack already uses for rendering
(Tier 0 text → Tier 1 native prompt → Tier 2 MCP App): **the contract is portable,
the substrate differs per platform.**

| Tier | Substrate | What it serves | KnowMe status |
|---|---|---|---|
| **E0 — prompt-only** | Any harness. No execution. | 107 skills, unchanged, on every platform | **Works today.** Needs no runtime. |
| **E1 — WASM component** | `knowme_plugin_host` (wasmtime 46, component model, precompiled `.cwasm`) | Deterministic script logic recompiled as components | **Host exists, ships nowhere.** See §Gap 1. |
| **E2 — remote invoke** | `host-uar.invoke` → paired desktop or cloud node | Toolchain skills that can never run on a phone | **Seam declared, transport unbuilt.** See §Gap 2. |

The important consequence: **E0 and E2 need no new runtime.**

**E1 is a research bet, not a delivery path.** The WASM host being pre-built here
makes E1 look like the obvious win. Current evidence says otherwise:

- iOS forbids JIT, so wasmtime runs an interpreter (~**10× slowdown**) or a
  precompiled `.cwasm`. There is **no certification of WASIp2-on-Pulley-on-iOS** —
  only inference from feature-parity language.
- **No documented WASI-preopen-on-iOS integration** for any runtime.
- **App Store guideline 4.7.2** — whether it permits a WASM plugin host exposing
  native APIs has **no precedent in either direction**. Highest-risk unknown; needs
  Apple contact before committing.

E1 is still the right *eventual* on-device answer. **E2 preserves every script at
zero porting cost today**, which is why it is the tier to build first.

### What lands in which tier

| Category | Count | Tier | Note |
|---|---|---|---|
| Prompt-only skills | 107 | E0 | Ship as-is |
| Shell-only scripts | 13 | E1 | Deterministic; good component candidates |
| Network + python3 | ~6 | E1 or E2 | Includes the adversarial-review family |
| Toolchain (cargo/npm/python) | 15 | **E2 only** | Building code is desktop work. There is no mobile answer and there should not be one. |
| git | 3 | E2 | |
| Service (launchctl/docker) | 1 | E2 | |

**The ideation skills are the portable ones.** All five `adversarial-review` scripts
need only **python3 + HTTP**:

- `git` is used only in *diff mode* (`build-review-packet.sh:176–181`)
- `cargo` is already opt-in and degrades gracefully (`:362–367`)
- The `--mode skill|agent` paths need neither

So the cross-model judging capability — the one that makes an ideation verdict
trustworthy — is E1-viable, not stuck on the desktop.

## What KnowMe already has

Verified in this repository:

### `knowme_plugin_host` — the E1 host, already built

`rust/crates/knowme_plugin_host/Cargo.toml` pulls **wasmtime 46** with
`component-model`, `async`, and `runtime` — deliberately **without `cranelift`**,
which is the shape a no-JIT platform needs. `openspec/specs/plugin-sandbox-host/spec.md`
ratifies ten requirements, including the ones that matter for skills:

- **Capability-gated instantiation** — a component importing an interface outside its
  capability set fails with `MissingCapability` *before any guest code executes*
- **Deny-by-default WASI** — no preopens, no env, no stdio, no sockets
- **Resource limits** — fuel, 64 MiB memory ceiling, 30 s epoch timeout
- **Typed `HostDispatch` seam** — every denial auditable at one choke point
- **SSRF-guarded `net_fetch`** — loopback, link-local, RFC1918, cloud metadata denied
- **Precompiled component cache** — `.cwasm` keyed by content hash + engine version

That last one is what makes iOS plausible: compile off-device, load precompiled.

### `host-uar.invoke` — the E2 seam, already declared

`rust/crates/knowme_plugin_host/src/dispatch.rs:73` declares `uar_invoke`. This is
the interface a guest calls to say *"run this on the runtime's behalf"* — and it does
not care whether the runtime is in-process, on a paired desktop, or in the cloud.
That indifference is the whole design.

### `gen_ui_mcp` — the precedent for platform-split transports

`rust/crates/gen_ui_mcp/Cargo.toml:22–23`: *"stdio transport spawns a child process —
native-only, opt-in."* Remote MCP transports are already the mobile default here.
The skill layer should split exactly the same way, for exactly the same reason.

### The extensibility spec

`docs/knowme-builder-extensibility-spec.md` §2 already establishes the per-target
data plane and states the iOS constraint plainly — *"iOS cannot run Postgres: no fork,
no JIT"* — then draws the right conclusion: the **entity layer, not SQL, is the
portability boundary**. Skills need the same treatment: portable contract, per-tier
substrate.

Its `prometheus:component/*` WIT family (§5.1) and `.pxt` package format (§7.2) are
where a packaged skill-as-component belongs. A Prometheus skill compiled to E1 **is**
a `prometheus:component/agent@0.1.0` component; it should not invent a second format.

## The two gaps

### Gap 1 — `knowme_plugin_host` ships nowhere

No crate depends on it. `gen_ui_ffi` — the mobile FFI crate — lists
`gen_ui_types`, `runtime`, `protocol`, `agent`, `mcp`, `client`, `db`, `db_graph`,
`inference`, `audio`. **Not `knowme_plugin_host`.** The host is written, specified,
and orphaned.

Before it can join the mobile build:

1. Confirm wasmtime 46 builds for `aarch64-apple-ios` and Android ABIs with
   `runtime`-only features
2. Confirm App Store acceptance of an interpreter-only WASM runtime loading
   precompiled `.cwasm` — this is a policy question, not just a technical one
3. Measure binary size delta against the mobile budget
4. Add it to `gen_ui_ffi` behind a feature flag so desktop is unaffected if mobile
   proves infeasible

### Gap 2 — no P2P, and the remote path is fail-open

KnowMe has **no iroh, no WebRTC, no Loro** in any crate. It reaches services over
plain `reqwest` + SSE. Meanwhile `prometheus-skill-pack/substrate/sovereign-sync`
already runs iroh 1.0 + iroh-gossip 0.101 + Loro 1.13 with a REST + AG-UI SSE surface.

**Build relay-first, not P2P-first.** This is the opposite of the intuitive design,
and the evidence is one-sided:

- **Hole-punching fails specifically where phones are.** >90% of cellular ASes use
  CGNAT and ~40% of that is symmetric (Richter et al., IMC 2016); when both peers sit
  behind endpoint-dependent NATs, probing succeeds ~0.01% of the time. iroh's own
  docs say it: *"Where it fails (some corporate firewalls or cellular networks) iroh
  automatically falls back to the relay."* The familiar 70–90% success figures are
  desktop/server populations.
- **The NAT binding and the battery are the same problem.** Cellular CGNAT mapping
  timeout is ~65 s, so holding it needs keepalives every 15–30 s — and RFC 8085 warns
  that keepalive frequency becomes the dominant power cost. Apple DTS: *"If you need
  to keep your network connections alive indefinitely, that's not possible given the
  iOS multitasking architecture."* The mechanism is **suspension**, and the Xcode
  debugger prevents it — so this bug looks fixed in development.
- **Three vendors converged on cloud relay within four months** — Claude Code Remote
  Control (*"outbound HTTPS requests only and never opens inbound ports"*), OpenAI
  Codex mobile, and Cursor. None use P2P.

**The differentiator is already in the substrate.** iroh QUIC is end-to-end encrypted
between node IDs, so the relay forwards ciphertext it cannot read — `sovereign-sync`
is a **blind relay**. The incumbents are not: Anthropic stores transcripts server-side
and structurally excludes zero-data-retention orgs. Keep the relay; stop calling it
"no intermediate server" and start calling it *a relay that cannot read your data*.

**Design for episodic sessions:** durable state + fast idempotent resume + push as
doorbell + foreground service only while the user is present.

> ⚠️ FCM deprioritises high-priority messages when it *"detects a pattern in which
> messages don't result in user-facing notifications"*, evaluated per-install over 7
> days. A silent push-to-wake channel is exactly that pattern and degrades **silently,
> for a subset of users**. Post a genuine user-visible notification on wake.

**Security must land before transport.** `mcp_client_pool.rs:120` —
`allowed_tools` **empty → all tools allowed**. Fine for a loopback daemon;
unacceptable the moment a phone can trigger execution on a desktop. Deny-by-default
allow-lists, capability tokens, pairing, and replay protection come **first**. The
larger risk is **composition** across 145 skills invoking each other through a remote
seam — a different threat model than any single sandboxed skill.

**Start cheaper than either.** `SURREAL_MEMORY_URL` already points the memory client
at a remote host. Remote *services over HTTPS* works today and needs no relay work at
all.

## How to use the skills to build this

The skill pack is meant to be used *on* this work, not merely shipped in it.

| Task | Skill |
|---|---|
| Vet the E1-vs-E2 decision before building | `/adversarial-review --mode artifact` — a different model reviews the design |
| Author a skill-as-component | `librefang-wasm-skill` (capability-checked WASM ABI) → adapt to `prometheus:component/*` |
| Generate a new skill with a cross-model gate | `pmpo-skill-creator` — reflect phase now dispatches `--mode skill` review |
| Stand up a native agent | `native-agent` — generation is gated by `--mode agent` review before the ready banner |
| Run the KBD lifecycle on this phase | `/kbd-assess` → `/kbd-analyze` → `/kbd-plan` → `/kbd-apply` |
| Prove a gate discriminates | Copy `adversarial-review/tests/run-fixture-suite.sh` — flawed *and* clean fixtures, inversion fails the suite |

**Non-negotiables inherited from this repo.** Any skill integration must respect:

- **`gen_ui_core` invariant** — all networking, LLM, and persistence in Rust. A skill
  may not open a socket from Dart or React.
- **Layering (Rule 16, enforced by `audit.sh`)** — Flutter: Widget → provider →
  Repository → FFI. React: Component → Hook → Store → `invoke()`. Stores are the
  *only* `invoke()` layer.
- **Install-time capability negotiation** — refuse a component whose imports exceed
  the host surface, before any code runs.
- **`.pxt` packaging** — signed manifest, SBOM, SHA-256 per file, DID + cosign.

## What is not decided

Carried as open questions in the skill pack's phase assessment; each blocks part of
the above:

1. **Does App Store guideline 4.7.2 permit a WASM plugin host exposing native APIs?**
   No precedent either direction. Blocks E1 on iOS entirely; needs Apple contact.
2. Real arm64 binary-size delta for wasmtime 46 in the mobile bundle — published
   figures are desktop-only.
3. Auth/crypto for the blind relay: capability tokens, pairing, replay protection.
4. Rewrite the adversarial-review scripts as a WASM component, or call them via E2?
   Leaning E2 — doing both is waste.

**Answered, so not open:** a P2P connection **cannot** survive iOS suspension, and
relay is the **common case** on cellular rather than the exception.

**Staleness:** App Review Guidelines change without notice — re-verify 2.5.2 / 4.7.2
before submission. The cellular CGNAT data is 2016 with no modern replication. WASI
Preview 3 is landing in Wasmtime 46+ and may move the component-model story
mid-build. Android Platform root cert rotates **February 2026** if direct Key
Attestation is used.

Until (1) is answered, **E0 + E2 is the shippable configuration**: 107 skills work
everywhere unchanged, and the rest work anywhere the user has one paired machine.
That is already the product promise — the same runtime and skills wherever you go —
without betting on an unproven on-device runtime.
