# Agent Execution Strategy — Placements, Hooks, and WASM Agents

**Scope:** how agents execute in every KnowMe environment (web, mobile, desktop, peers);
the hook mechanism that lets plugins participate in prompt processing and context
assembly; and native WASM agents with an A2A-shaped WIT contract and host inference
callbacks.

**Status:** v1.1 strategy, 2026-07-19 (v1.1 adds §7: grounding in the vendored UAR
submodule — where this doc's "UAR (`gen_ui_agent`)" references land concretely).
Companion to
`docs/knowme-builder-extensibility-spec.md` (§4 agents, §5 component model),
`docs/strategies/model-configuration/README.md` (§4 resolution, used by every callback
here), and `docs/knowme-functional-spec.html` (§06 Hands, §08 plugins, §09 privacy).

---

## 1. The organizing principle: one contract, many placements

Agent definitions are **text**. A Hand — identity, configuration, schedule, skill
grants, Cedar policy (`rust/crates/knowme_hands::HandDefinition`) — carries no code, so
*where* it executes is a runtime decision, not an authoring decision. We exploit that
fully: there is **one AgentService contract** (the FRF Agent service proto /
`frf-agentproto` ContentBlocks, with OFP as its personal-mesh profile) and several
**placements** that implement it:

| Placement | What it is | Where available |
|---|---|---|
| `embedded` | UAR (vendored submodule — §7) linked into the app process, reached via FFI/IPC | Everywhere — the universal baseline; the only option on mobile |
| `sidecar` | UAR in its own OS process on the same machine, speaking AgentService over localhost | Desktop (macOS/Windows/Linux) |
| `peer` | A BossFang instance (home server, personal cloud, team) | Anywhere reachable |
| `kiln` | Flint Kiln edge function (server-side WASM) | flint-forge deployments |

The Hands wizard's Step 3 (Deployment) picks among these; the §4.3 model resolver runs
*at the placement*, against that placement's providers. Nothing else about the Hand
changes. This is the whole strategy in one sentence: **placement is configuration.**

## 2. Client-side agents per environment

### 2.1 Desktop — embedded baseline, sidecar promotion

The POC already embeds UAR in the Tauri core; that remains the default. The **sidecar**
is an opt-in promotion with three concrete wins:

- **Lifecycle independence** — Hands keep running when the window closes; "run at
  login" makes the machine an always-on Hands node without the UI open.
- **Crash + resource isolation** — a wedged agent run cannot take down the UI;
  OS-level memory/CPU limits apply to the sidecar process, and the sidecar can host
  its own inference engine instance so agent inference doesn't contend with the
  interactive Engine Card session.
- **Structural honesty** — a sidecar speaking AgentService on localhost *is* a minimal
  local BossFang. It appears in Settings → Instances as "This device" with a
  `sidecar` pill; promoting it to a full always-on peer later (OFP-visible to your
  other devices) is a toggle, not a migration.

Implementation: Tauri sidecar API for the app-scoped mode; launchd agent / systemd
user unit / Windows service for run-at-login. The app talks to it exactly as it talks
to a remote peer — same client code path, `localhost` transport. **Rule: the sidecar
never gets a private API.** If the app needs something from the sidecar, a remote peer
needs it too; that discipline is what keeps the placement table true.

### 2.2 Mobile — embedded only, honest about background limits

iOS forbids child processes and long-running background execution; Android is looser
but WorkManager-shaped. So: UAR embedded via FRB/FFI, and Hands on mobile run in one
of three modes, surfaced honestly in the wizard:

- **Foreground/opportunistic** — runs when the app is open or in short
  `BGTaskScheduler`/WorkManager windows (best-effort cadence; the wizard says
  "runs about daily, when iOS allows" instead of promising 9:00 AM);
- **Delegated** — the default recommendation when any peer exists: the Hand deploys to
  a peer, the phone gets results via sync (this is the moodboard's "On: Home Mac"
  pattern);
- **Triggered** — event-driven runs on capture/share-sheet actions, which iOS does
  permit.

The anti-pattern we refuse: pretending a phone can host cron. The wizard's Step 3
shows real semantics per placement.

### 2.3 Web — session agents, delegated Hands

The browser runs UAR-in-WASM (`gen_ui_wasm` + `frf-wasm::agent`): full chat-agent and
tool-call capability inside the session, skills limited to browser-safe surfaces
(web-fetch via host, memory, no shell/fs). Scheduled Hands don't exist in a tab —
service workers are not a scheduler — so the web tier authors and monitors Hands but
always deploys them to a peer. A PWA with a reachable peer is a full KnowMe; a PWA
alone is a chat client with session agents, and says so.

## 3. The hook pipeline (context factory extensibility)

### 3.1 The pipeline

Every agent turn — chat or Hand — flows through named stages in UAR's orchestration
loop:

```
ingress → intent-classify → route → context-assemble → prompt-build
        → [inference] → tool-gate → tool-execute → post-process → egress
```

Plugins (`.pxt` components) may hook stages, but under two structural rules that keep
the pipeline sane:

**Declared, not discovered.** The manifest announces every hook: stage, the *data
classes* it reads (`intent`, `prompt-text`, `retrieval-query`, `tool-results`, …), and
what it contributes. Install-time review renders this as permissions — a hook reading
`prompt-text` is an amber, explicitly-consented grant, same visual grammar as every
other trust surface. No runtime hook registration; what a plugin can see is knowable
before it runs.

**Contribute, don't mutate.** Hooks return **typed, provenance-tagged contributions**,
not modified pipelines:

- `intent-classify`: additional intent labels with confidence (the router may use
  them; base classification is never overwritten);
- `route`: routing *criteria* (e.g. "this looks like a `com.acme.crm` task —
  prefer the CRM plugin's toolset");
- `context-assemble`: context fragments with a declared **token budget request**,
  source attribution, and relevance score — the context factory allocates the real
  budget across competing contributions deterministically (score × grant, then
  truncate); a plugin can enrich context, it cannot crowd out the user's memory;
- `tool-gate`: advisory annotations only — Cedar remains the sole authority on
  allow/deny;
- `post-process`: renderable artifacts (AG-UI events / HTMX fragments per the
  extensibility spec), never silent rewrites of model output.

Ordering is deterministic (manifest priority, then plugin id); every hook runs in the
plugin's own WASM sandbox with a per-stage epoch budget (default 200 ms; a slow hook is
skipped and logged, never blocks the turn); every invocation lands in the audit trail
with its contribution. Hooks compose with skills rather than replacing them: a hook
that wants data fetched still calls `uar-invoke` under its allow-list.

### 3.2 Why this shape

Free-form middleware ("here's the prompt, return a new prompt") is the classic
extensibility trap: order-dependent, un-reviewable, and two plugins deep it becomes
un-debuggable. Contributions with budgets and provenance keep the context factory
**auditable** — the audit log can show exactly which plugin added what to any prompt —
which is the same property the FUNC-SPEC demands of skills and sync. The hook system is
KnowMe's prompt-processing story *and* its prompt-injection containment story: a
malicious contribution is visible, attributed, budget-capped, and revocable.

## 4. Native WASM agents

### 4.1 The `agent` world

A third capability in the `prometheus:component/*` family (extensibility spec §5.1) —
sketch:

```wit
package prometheus:component@0.1.0;

interface agent-host {
  record model-info { ref: string, capabilities: list<string>,
                      ctx-len: u32, placement: string }
  /// Capability- and Cedar-filtered view of the container's registry —
  /// resolved by the model-configuration resolver; never exposes keys.
  list-models: func(required-capabilities: list<string>) -> list<model-info>;

  record infer-request { model-ref: option<string>,   // none = resolve for me
                         messages-json: string, params-json: string,
                         tools-json: option<string> }
  /// Host executes against container providers under the agent's Cedar policy
  /// and token budget. Streaming: chunk polling in 0.1 (WASI 0.2 has no async
  /// exports); the shape is chosen so WASI 0.3 futures make it native later.
  infer-start: func(req: infer-request) -> result<string, string>;   // handle
  infer-poll:  func(handle: string) -> result<option<string>, string>; // chunk
  infer-cancel: func(handle: string);
}

interface agent {
  /// A2A-shaped lifecycle: tasks with messages in, status + artifacts out.
  record task-request { task-id: string, messages-json: string,
                        metadata-json: string }
  start-task: func(req: task-request) -> result<_, string>;
  poll-task:  func(task-id: string) -> result<string, string>;  // status+artifacts
  cancel-task: func(task-id: string);
}
```

plus the core imports every component gets (log, kv, emit-event, uar-invoke).

### 4.2 The three deliberate properties

- **Inference by callback, never by key.** The agent asks *for inference*, not *for a
  provider*; the host resolves via the model-configuration resolution order (§4.3
  there), enforces Cedar ("this agent may use local models and `home-mac`, not cloud"),
  meters tokens against the agent's budget, and records the call in the audit trail.
  Provider credentials never cross the sandbox boundary. `list-models` returns the
  filtered view so an agent can *prefer* ("give me something with `tools` and 32k
  ctx") without ever holding secrets.
- **A2A-shaped on purpose.** Using the [A2A protocol](https://github.com/google/A2A)
  vocabulary (tasks, messages, artifacts, status; an AgentCard derived from the `.pxt`
  manifest) means a WASM agent can be *exposed* as a real A2A endpoint by BossFang or
  Kiln with a thin adapter, and remote A2A agents slot into the placement table as
  just another executor. Interop both directions for free.
- **Same lifecycle as Hands.** A WASM agent is not a fourth kind of thing: it is the
  `agent_impl` of a Hand. The wizard, Cedar policy, consent prompts, run history,
  audit trail, pause/resume, and placement choice are identical; the only difference
  is that Step 1 references a component instead of a prompt template.

### 4.3 Two agent kinds, one lifecycle (the synthesis)

> **Framing update (see `docs/strategies/plugins/README.md`):** a native agent is not
> a distinct package type — it is a plugin declaring the `knowme:agent` **capability**.
> Everything below about the component kind is the agent-capability view of that
> capability-centric model; the Hands wizard is the product surface that binds an
> agent-capability plugin into a Hand.

```
Hand ─┬─ declarative (text): prompt/skill/schedule metadata — the 90% case,
      │    portable to every placement including web
      └─ component (WASM):  exports prometheus:component/agent — logic-bearing
           agents, portable to every placement with a Wasmtime host
           (embedded desktop, sidecar, peer, Kiln; web pending a
           JS-host shim — declarative-only there in v0.1)
```

Everything else in this document — placements (§1–2), hooks (§3), model callbacks
(§4.1) — applies to both kinds identically. That identity is the design: no matter how
an agent is implemented or where it runs, it is created, governed, audited, and
observed the same way.

## 5. Risks and open questions

1. **Sidecar ops** — packaging/signing a helper binary per OS, run-at-login UX,
   single-instance locking, port/socket auth (localhost token). Solved problems, but
   real work; budget it (roadmap M1.5).
2. **Hook privacy optics** — `prompt-text` hooks are powerful; the permission UI must
   make them feel as weighty as they are. Consider a per-conversation "no plugin
   hooks" toggle (mirrors on-device-only mode's spirit).
3. **WASM streaming ergonomics** — chunk polling is workable but clunky; revisit the
   WIT when WASI 0.3 async lands (extensibility spec §9.6 tracks this).
4. **Budget arbitration** — the context factory's token-budget allocator across
   plugin contributions needs a deterministic, documented policy from day one; it is
   the piece users will ask "why did it include that?" about.
5. **A2A drift** — track the A2A spec's evolution; keep our vocabulary an adapter
   away, not a fork.

## 6. Implementation map (this repo)

- `rust/crates/gen_ui_agent`: name the pipeline stages (§3.1) as an explicit typed
  enum; contribution types + budget allocator; hook dispatch to the plugin host.
- `rust/crates/knowme_plugin_host`: `agent-host`/`agent` worlds added to the WIT
  family; hook declarations parsed from the `.pxt` manifest (extends the existing
  `PluginManifest`).
- `rust/crates/knowme_hands`: `agent_impl: Declarative | Component{ref}` on
  `HandDefinition`; placement enum gains `Sidecar`.
- New `knowme-sidecar` binary crate: thin main() around UAR + AgentService server +
  localhost auth; Tauri sidecar config in `desktop/src-tauri`.
- `mobile/`: BGTaskScheduler/WorkManager adapters reporting into the same run-history
  tables; wizard copy for the three mobile modes (§2.2).
- Audit: hook invocations and infer-callback records join the existing per-Hand audit
  log (FUNC-SPEC §06).

## 7. UAR grounding (2026-07-19): the submodule is real — map to it

The Universal Agent Runtime is now vendored at **`rust/vendor/universal-agent-runtime`**
(v1.0.0; a single root crate `universal-agent-runtime` with `[lib]` plus the
`universal-agent-runtime` and `uar-sidecar` binaries; excluded from the host
workspace build). Every "UAR" reference above now has a concrete target, and three
corrections fall out:

1. **Placements map to UAR consumption modes, not to new code.**
   - `embedded` (mobile, the only phone option) = UAR **as a Rust library**, behind a
     facade in `gen_ui_agent` (Release A of
     `docs/strategies/local-models/android-agentskills-process-future-roadmap.md`).
     UAR's compatibility policy explicitly does **not** guarantee its Rust API during
     1.x — the supported surface is sidecar HTTP/SSE. Mobile has no sidecar option,
     so: pin the submodule SHA, keep the facade the *only* touchpoint, and treat
     every UAR bump as a reviewed migration.
   - `sidecar` (desktop) = the **`uar-sidecar` binary** — the lifecycle-isolation
     story of §2.1 is a shipped binary, not a new crate to write.
   - `peer` = BossFang serving the UAR **`server-full`** bundle (HTTP/SSE — the
     integration surface UAR actually supports; see the BossFang doc).
   - `kiln` = unchanged (server-side WASM).
2. **Agent definition is compiled, and there are two artifacts with one owner.**
   `HandDefinition` stays KnowMe's authoring/governance surface (identity, schedule,
   grants, Cedar). The *execution* artifact is UAR's **UAR-AGENT-MD → compiler →
   `AgentArtifact`** (`src/uar/domain/artifact.rs`, `src/uar/compiler/`, with
   ed25519 signing and a conformance suite); A2A exposure uses UAR's
   `AgentCard`/`AgentSkill` (`src/uar/api/a2a/`). The mapping rule: a declarative
   Hand *compiles to* a UAR-AGENT-MD artifact; a component (WASM) Hand binds to one
   — §4.3's "two agent kinds, one lifecycle" survives with UAR's artifact as the
   shared execution form. Execution itself is UAR's `RunManager`
   (`src/uar/runtime/manager.rs`) over `ractor` agent actors and `AgentGraph`.
3. **The "one AgentService contract" is realized as protocols, not a proto.** The
   frf-agentproto framing of §1 is superseded by UAR's actual surface: the
   **AG-UI event profile `uar.agui/1`** (see `docs/strategies/ag-ui/README.md`),
   the **A2A task vocabulary**, and OpenAI-compatible endpoints in server mode.
   "Placement is configuration" still holds — the client speaks the same AG-UI +
   task contract to an embedded runtime (in-proc) or a peer (HTTP/SSE).

Skill definition/execution grounds the same way (UAR `src/uar/domain/skills.rs` +
the `prometheus-skill-system` content repo): see `docs/strategies/skill-support/README.md`
§8. A2UI surfaces declared by agents (UAR-AGENT-MD §06) resolve through
`docs/strategies/a2ui/README.md`.
