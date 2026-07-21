# Embedded Universal Agent Runtime (UAR) — Flutter Mobile Assessment

**Date:** 2026-07-21
**Scope:** Read-only code assessment of the Flutter mobile app (`mobile/`) and its Rust core (`rust/`). No code was changed.
**Claims assessed:**

1. The universal agent runtime is fully connected and operational across multiple turns, embedded in-process (library-only, no external/network connection on Flutter mobile).
2. The user can switch between the local direct model and UAR runtime agents within a conversation, back and forth, and have it work.

**Verification methods:** full source trace (Dart → FFI → `gen_ui_agent` → vendored UAR), feature-flag and stub audit, spec/task cross-check (`openspec/changes/embed-uar-mobile-offline/`, `.kbd-orchestrator/`), and a live `cargo check -p gen_ui_ffi -p gen_ui_agent` which **completed with exit 0** (including the vendored `universal-agent-runtime` and SDK with `embedded-mobile` features).

---

## Executive summary

| Criterion | Verdict |
|---|---|
| UAR embedded as an in-process Rust **library** (no server/socket/sidecar on mobile) | ✅ **Confirmed** |
| UAR **wired end-to-end** (Dart → FFI → agent → embedded UAR → on-device inference → event stream) | ✅ **Wired and compiling** — no stubs on the happy path |
| UAR **offline-capable** (no network required for the runtime itself) | ✅ **Confirmed** (excluding one-time model download and opt-in cloud/service modes) |
| UAR **operational across multiple turns** | 🟡 **Partial** — works within a process; agent context is lost on app restart (in-memory persistence) |
| **Fully operational / certified** | 🟡 **Not yet** — compile-verified only; device & network-disabled certification tasks unchecked |
| Switch between local direct model and UAR agents **within a conversation, back and forth** | 🟡 **Partially met** — switching works and is mechanically safe, but it is global (not per-conversation), history hand-off is one-directional, and the per-conversation agent/policy selection is non-functional on embedded mobile |

**Bottom line:** The embedded-kernel architecture is genuinely delivered — UAR runs in-process via flutter_rust_bridge with the on-device model as its credential-free default LLM, offline, with one kernel reused across turns. The honest gaps are: (1) UAR conversation state is in-memory only, so agent context does not survive app restart; (2) the local↔UAR switch is global and history flows coherently only from UAR → local, not local → UAR; (3) the per-conversation policy/agent picker is decorative on mobile because the control plane is HTTP-only; (4) none of this is yet certified on a physical device with networking disabled, per the project's own openspec tasks.

---

## 1. Embedded library integration (Claim 1, part a)

**Verdict: Confirmed.** UAR is instantiated in-process as a Rust library over flutter_rust_bridge. No listener, sidecar, loopback socket, or network discovery exists on the mobile path.

### Full instantiation trace (all verified in source)

1. **Dart boot** — `mobile/lib/features/startup/presentation/providers/startup_notifier.dart:33-37`: `bridge.runMigrations(dataDir: …)` → `attachSyncShapes()` → ready.
2. **FFI binding** — `mobile/lib/bridge/api/boot.dart:18` (generated, frb 2.12.0) → `rust/crates/gen_ui_ffi/src/api/boot.rs:165` `run_migrations(data_dir)`.
3. **Runtime construction** — `boot.rs:275-284` builds the platform `InferenceProvider` (`LiteRtLmEngine` on Android, `LlamaCppEngine` elsewhere); `boot.rs:286-311` calls `gen_ui_agent::uar::UarEmbeddedRuntime::build_local(inference, LocalModelSpec, provider_config)`. Failure is a **fatal boot error** (`boot.rs:297`) — there is no silent fallback to a network lane. Stored via `gen_ui_agent::state::init_with_inference_and_uar(...)` (`boot.rs:315-320`).
4. **Embedded kernel** — `rust/crates/gen_ui_agent/src/uar.rs:607-645`: `UarEmbeddedRuntime { runtime: universal_agent_runtime_sdk::Runtime }`, built with `.local_provider(driver, provider_config).persistence(InMemoryProvider).seed_defaults(true)`. Its doc comment (uar.rs:601-606) states: *"No listener, sidecar, loopback socket, or network discovery is used."*
5. **SDK** — `rust/vendor/universal-agent-runtime/sdks/rust/src/runtime.rs:36-39`: `Runtime { inner: Arc<EmbeddedRuntime> }`. The HTTP server exists only behind a separate `server` cargo feature (`runtime.rs:196-205`), which is **not enabled** in the mobile graph (`rust/crates/gen_ui_agent/Cargo.toml:54,58` enables only `http-client`, `embedded-mobile`, `in-memory-backend`).
6. **Kernel** — `rust/vendor/universal-agent-runtime/src/embedded.rs:1-7`: *"deliberately does not construct AppState, bind a socket, load a sidecar, or start transport/background services."* `EmbeddedRuntimeBuilder::build` (embedded.rs:247-360) wires the `Orchestrator` and `RunManager` directly to the host LLM driver (embedded.rs:321-344).
7. **Chat turn** — `pem_chat_repository.dart:31` → `bridge.chatSend` → `gen_ui_ffi/src/api/chat.rs:32 chat_send` → `gen_ui_agent::chat::send_with_attachments` (chat.rs:231) → lane dispatch (chat.rs:194-196) → `send_uar` (chat.rs:1297) → `selected_uar_runtime()` (chat.rs:1252) → `UarEmbeddedRuntime::start_run` (uar.rs:691-712) → SDK `start_agent_run("orchestrator-agent", ...)` → events stream back via `state::publish` → `chat_events` sink (`gen_ui_ffi/src/api/streams.rs:36-78`, backlog + live broadcast).

The HTTP-based `UarServiceRuntime` (uar.rs:1010-1130) is constructed only for `HttpService`/`CloudService` deployment modes (chat.rs:1274-1294); the default on iOS/Android is `EmbeddedLibrary` (chat.rs:1245-1247), and `UarRuntimeStatus.mobile_sidecar_allowed` is hard-coded `false` (uar.rs:563).

## 2. Offline capability (Claim 1, part b)

**Verdict: Confirmed** for the runtime/agent loop itself.

- The agent loop's LLM is `UarLocalInferenceDriverBridge` (uar.rs:200-283): UAR `LlmRequest` → KnowMe `InferenceRequest` → the platform on-device `InferenceProvider` (LiteRT-LM / llama.cpp) → normalized event stream.
- Registered provider configs are credential-free with **empty `base_url` and no `api_key`** (Android: uar.rs:127-150; iOS: uar.rs:167-190).
- `EmbeddedRuntimeBuilder::build` never constructs an HTTP/external driver. Hardcoded URLs in `llm/registry.rs:534-549` are defaults for named remote providers and are not touched by the embedded path.
- Default lane: `default_lane()` returns `LANE_UAR` on mobile whenever the embedded kernel is registered (chat.rs:1172-1182) — on-device by default, no keys.

Network-using paths exist but are **off the default path**: one-time model artifact download (`prepare_local_model`, chat.rs:1694), fastembed model cache (boot.rs:185-187), demo-corpus seeding, the explicit BYOK cloud lane, and the opt-in UAR service mode (`save_uar_service_config`, chat.rs:1046-1113).

## 3. Multi-turn operation (Claim 1, part c)

**Verdict: Partial.**

**Within a process: works.** `send_uar` passes `session_id: config.session_id.or(Some(conversation_id))` (chat.rs:1352); UAR's `RunManager` resolves the session via `sessions.get_or_create(&id)` (`rust/vendor/universal-agent-runtime/src/uar/runtime/manager.rs:853-857`), appends via `session.add_user_message(&input)` (manager.rs:868), and the LLM is called with accumulated history (`orchestrator.chat_with_history(messages)`, manager.rs:1711). Same `conversation_id` (Dart `threadId`) → shared agent context across turns. The kernel is built **once per process** during startup and stored in a process-lifetime `OnceCell<AgentState>` (`state.rs:68`, `state.rs:104-130`); all turns reuse it via `state::embedded_uar()` (state.rs:263-268); it is never recreated per turn.

**Across app restart: lost.** UAR's `SessionStore` is an in-memory `RwLock<HashMap<String, Session>>` (`src/session/thread.rs:284-306`) and the persistence layer injected at uar.rs:632-636 is `InMemoryProvider` — the code itself flags this: *"the next persistence milestone swaps it for the mobile durable adapter"* (uar.rs:622-625). `openspec/changes/embed-uar-mobile-offline/design.md:45-47` explicitly requires durable persistence for production, and task 3.1 covering it is **unchecked**.

**Nuance:** the UI transcript persists (entity store `entity-records.sqlite3`, boot.rs:181-183) and semantic memory/graph-RAG persists (embedded SurrealDB rocksdb, boot.rs:189-197). KnowMe-level chat history *is* passed to `chat_send` — but the UAR lane **ignores the `messages` argument** (chat.rs:194-196 passes only `user_message` to `send_uar`). After restart the screen shows old messages but the agent has no conversational context.

## 4. Completeness vs. stubs

**Implemented and compiling:**

- Zero `TODO`/`todo!()`/`unimplemented!()` in `gen_ui_agent/src/uar.rs`, `gen_ui_ffi/src/api/boot.rs`, `local_provider.rs`, or the vendored UAR src (stubs appear only in frb-generated scaffolding and docs).
- `cargo check -p gen_ui_ffi -p gen_ui_agent` → **exit 0** this session.
- Generated Dart bindings match the FFI surface one-to-one (`chat.dart` ↔ `api/chat.rs`; `boot.dart` ↔ `api/boot.rs`; `streams.dart` ↔ `api/streams.rs`).
- Real implementations: event projection (`embedded_stream_payload`, uar.rs:858-1006), tool-call adaptation (`UarNativeEventAdapter`, uar.rs:327-549), A2UI continuation/promotion (uar.rs:718-801), cancellation (chat.rs:1382-1387), tool-approval flow (chat.rs:756).
- openspec `tasks.md` §1 (Repair embedded library) and §2 (Connect local inference): all 8 tasks checked — matches the code.

**Unchecked / incomplete (14 tasks):**

- §3 Offline data & control plane: 3.1 durable persistence (the gap above), 3.2 direct FFI admin ops (JSON FFI functions exist at `api/chat.rs:145-272` but the task is unverified), 3.3 offline-availability advertising, 3.4 seeded-agent verification.
- §4 Client integration: Flutter wiring verification (4.1), event parity in mobile chat (4.2), diagnostics UX (4.3), desktop alignment (4.4).
- §5 Build & certification: all unchecked — host tests (5.1), mobile cross-compiles (5.2), and the critical **network-disabled physical-device certifications (5.3–5.5)**.
- Orchestrator state agrees: `.kbd-orchestrator/phases/model-runtime-per-device/children/embedded-uar-offline-agents/progress.json` shows `execute_complete: false`; its `assessment.md` is stale (pre-implementation; its "MISSING/STUB" claims no longer match the code).

---

## 5. Local direct ↔ UAR switching (Claim 2)

**Verdict: Partially met.**

### 5.1 Switching surfaces

- **Settings — global lane selector.** `mobile/lib/features/settings/presentation/screens/settings_screen.dart:395-410`: `SegmentedButton` with `UAR` / `Local` segments bound to `snapshot.activeLane`; `settings_notifier.dart:63-67` → FFI `setActiveLane`.
- **Per-conversation toolbar.** `chat_screen.dart:95` mounts `ChatPolicyToolbar`; `chat_policy_toolbar.dart:94-113` shows `Orchestrator` (`chat_mode='uar'`), `Agent` (`chat_mode='agent'` + agent picker), `Local` (`chat_mode='local'`). However, `chat_control_notifier.dart:71-81 setMode()` flips the **global** lane plus a per-conversation policy patch that (see 5.4) cannot persist on embedded mobile.
- **No per-message selector exists.** The lane is read once per send from the global setting.

### 5.2 Routing path

- Dart `chat_notifier.dart:291-336 sendMessage()` builds full history (`_historyForRust`, :733) → FFI `chatSend` → `api/chat.rs:32-50` → `send_with_attachments`.
- Rust reads the **global** `active_lane` setting on every send (`chat.rs:177-195`, `:245-249`). `LANE_LOCAL` → `send_local` **with** history; `LANE_UAR` → `send_uar(user_message, conversation_id, run_id)` — **history is dropped** (`:195`).
- `send_local` (`chat.rs:1722-1920`) is real: governed context assembly from passed history (:1803-1811), model load, streaming via `InferenceProvider` with a local tool-call gate. Real engines exist (`gen_ui_inference/src/litert_lm.rs` ~958 lines, `llama.rs` ~962 lines).
- `send_uar` (`chat.rs:1297-1408`) is real: embedded runtime selected via `selected_uar_runtime()`; `start_run` with `session_id = conversation_id`, **`agent_id: None`, `run_policy: None`** (:1349-1355) → embedded default `"orchestrator-agent"` (uar.rs:700-708).
- Caveat: on mobile both lanes use the *same on-device model* — "local direct" vs "UAR" is direct-prompt vs agent-loop, not on-device vs remote.

### 5.3 Back-and-forth switching & history coherence

- **Mechanically safe.** Both runtimes are built once at boot and never disposed (`boot.rs:276-320`; the local engine and embedded UAR share the same provider Arc). `set_active_lane` validates then persists (`chat.rs:1190-1211`). Nothing breaks on repeat switches **within a process**.
- **UAR → local: coherent.** Dart always sends full history (`chat_notifier.dart:299, 332`); `send_local` consumes it; UAR-produced assistant messages persist in the Flutter entity store, so the local model sees them.
- **local → UAR: lossy.** `send_uar` ignores Dart history; the embedded UAR session only contains turns that ran on the UAR lane (manager.rs:868). Local-lane turns are invisible to the agent on switch-back.
- **Across restarts: UAR loses everything** (in-memory `InMemoryProvider`, uar.rs:632-636). The local lane survives because history comes from Dart's persistent store each turn.

### 5.4 Broken / placeholder wiring in the switch UX

- **Per-conversation policy is unreachable on embedded mobile.** `control_plane.rs` implements `conversation_policy` / `save_conversation_policy` / `catalog` / `effective_config` exclusively via `chat::uar_service_client()` (`chat.rs:1461-1470`) — an **HTTP** client requiring `uar_service_url`. On embedded mobile there is no URL, so the toolbar's `load()`/`patch()` fail (`chat_control_notifier.dart:64-66, 100-102` → error icon, `chat_policy_toolbar.dart:209-216`). The global-lane flip still succeeds, so the *switch* works — but the toolbar's optimistic `chat_mode` display is never persisted.
- **The 'Agent' mode is effectively decorative.** `agent_id` never reaches a run (`agent_id: None` hardcoded, chat.rs:1353); every UAR run uses the embedded default `orchestrator-agent`.
- **The lane is global despite the per-conversation UI.** `chat.rs:177-181` consults only the global setting; tapping "Local" in conversation A silently reroutes conversation B.
- Minor: stale docstring on `active_lane()` says `"cloud" or "local"` (`chat.rs:985`) though it now returns three lanes; `mobile/lib/features/models/` is an empty README stub (real model UI lives in Settings).

### 5.5 Claim-2 verdict detail

The user *can* flip between "local direct model" and "UAR orchestrator" mid-conversation (Settings or toolbar) and back, and both paths execute real on-device inference without breaking on repeat switches. It does not fully "work as stated" because:

1. The switch is **global, not per-conversation** (and never per-message), despite a per-conversation-looking toolbar.
2. **History coherence is one-directional** (UAR → local only); local-lane turns are dropped when switching to UAR.
3. **UAR session state is in-memory only** — lost on every app restart.
4. **Per-agent selection does not function** on embedded mobile (HTTP-only control plane; hardcoded `agent_id: None`).

These gaps correspond exactly to the unchecked openspec tasks 3.1–3.4 and 4.1–4.4 — the spec itself records this work as incomplete.

---

## 6. Gap summary and remediation map

| # | Gap | Impact | Spec task | Key locations |
|---|---|---|---|---|
| 1 | UAR persistence is `InMemoryProvider`; session store is in-memory | Agent context lost on restart; violates design.md:45-47 | 3.1 | uar.rs:622-636; `src/session/thread.rs:284-306` |
| 2 | UAR lane ignores passed chat history | local → UAR switches lose local-lane context | 4.2 (parity) | chat.rs:194-196, 1297-1355 |
| 3 | Control plane is HTTP-only (`uar_service_client`) | Per-conversation policy/agent picker fails on mobile | 3.2 | chat.rs:1461-1470; control_plane.rs |
| 4 | `agent_id: None` hardcoded on UAR runs | 'Agent' toolbar mode decorative; only orchestrator reachable | 3.2/4.1 | chat.rs:1353; uar.rs:700-708 |
| 5 | Lane is global, not per-conversation | Switching in one conversation reroutes all | 4.1 | chat.rs:177-195; settings_screen.dart:395-410 |
| 6 | No device/offline certification | "Operational" is compile-verified only | 5.1–5.5 | openspec tasks.md §5 |
| 7 | `models` feature is an empty stub | Roadmap M6 item; model UI split into Settings | — | mobile/lib/features/models/ |

## 7. Final verdicts against the stated claims

**Claim 1 — "UAR is fully connected and operational across multiple turns, embedded as a library, no network on Flutter mobile":**
Mostly true at the architecture and code level: embedded-library ✅, offline ✅, wired end-to-end ✅ (compiles clean, no stubs on the happy path), multi-turn within a session ✅. Not fully true as stated: multi-turn state does not survive restart (in-memory persistence), and "operational" has not been certified on-device with networking disabled (openspec §5 unchecked).

**Claim 2 — "User can switch between local direct model and UAR agents within a conversation, back and forth, and have it work":**
Partially true. Back-and-forth switching between local-direct and the UAR orchestrator works and is safe to repeat within a process. Not fully true as stated: the switch is global rather than per-conversation, context hand-off works only from UAR → local, per-agent selection is non-functional (everything runs as `orchestrator-agent`), and UAR context evaporates on restart.
