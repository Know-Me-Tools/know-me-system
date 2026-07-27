# ASSESSMENT: embedded-uar-offline-agents

Project: KnowMe
Date: 2026-07-20
Codebase baseline: KnowMe has a certified Android LiteRT-LM provider and UAR protocol/service adapters, but the mobile UAR lane is HTTP-only and the SDK's advertised embedded `Runtime` does not construct runtime state.
Cross-tool progress: none in this new child phase; four parent model-runtime changes are recorded complete and `litertlm-local-tools` remains the parent active change.

## IMPLEMENTATION STATUS

- Mobile UAR runtime facade: **MISSING** — `gen_ui_agent::chat` routes every `uar` lane request to `send_uar_service`, and `ensure_uar_lane_supported` only accepts a configured UAR service URL.
- UAR SDK embedded runtime: **STUB** — `RuntimeBuilder::build` stores `AppConfig` but sets `state` to `None`, so `orchestrator()`, `run_manager()`, `vector_matcher()`, and `persistence()` all return `None`; `start()` launches the HTTP server rather than an in-process kernel.
- Host local-model driver seam: **PARTIAL** — `UarLocalInferenceDriverBridge` converts UAR payloads into KnowMe `InferenceRequest` streams, and UAR supports `Orchestrator::from_driver` plus `RunManager::with_llm_driver`, but no embedded runtime constructs a `RunManager` with that driver.
- Local provider/model registration: **MISSING** — the installed `gemma-4-e2b-it-litertlm` model is not represented in UAR `ProviderRegistry`/`ModelRouter` and cannot be selected by an offline UAR agent.
- Offline agent execution: **MISSING at the application boundary** — UAR contains `RunManager`, skill, tool, policy, AG-UI, and A2UI implementations, but Android never instantiates them and therefore cannot use them without a network UAR service.
- Mobile-safe persistence/configuration: **PARTIAL** — UAR has SurrealKV and in-memory backends, while KnowMe already embeds SurrealDB, but `AppState` construction is coupled to server startup and there is no host-storage/mobile configuration builder.
- Skills and tools: **PARTIAL** — UAR registries and native skills exist, and KnowMe has local MCP gating, but the mobile process does not initialize UAR registries or run the UAR-owned tool/skill loop.
- AG-UI/A2UI loop: **DONE for the service runtime and projection layer, PARTIAL for mobile** — canonical events, action continuation, replay, and component promotion exist, but only through HTTP/SSE on Android.
- Optional BYOK cloud access: **PARTIAL** — cloud/service configuration is optional for the local inference lane, but selecting the UAR lane makes service configuration mandatory instead of falling back to an embedded authority.
- Desktop offline agent execution: **PARTIAL** — desktop has a certified UAR service flow, but no in-process UAR kernel using the KnowMe local provider.
- Mobile dependency profile: **RISK** — the UAR minimal graph still includes server/web dependencies and resolves OpenSSL-related crates for `aarch64-linux-android`; the current SDK `embedded` feature links the entire runtime rather than a mobile-specific feature profile.
- Licensing: **REQUIRES EXPLICIT HANDLING** — the SDK documents that enabling `embedded` links the AGPL runtime. This is not a technical blocker, but distribution obligations must be preserved and documented.

## CROSS-TOOL PROGRESS

- `android-litertlm-runtime`: **COMPLETE** in parent phase (codex).
- `litertlm-native-adapter`: **COMPLETE** in parent phase (codex).
- `litertlm-device-proof`: **COMPLETE** in parent phase with physical SM-S936U evidence.
- `document-uar-karpathy-knowledge-strategy`: **COMPLETE** in parent phase.
- `litertlm-local-tools`: parent ledger remains **IN_PROGRESS** even though several checkpoints are implemented.
- `embedded-uar-offline-agents`: no prior implementation work recorded.

## SPEC GAP SUMMARY

- `openspec/specs/mobile-model-runtime` specifies the Android local model but does not require that UAR consume it as an offline provider.
- `docs/strategies/uar-knowme-runtime-integration.md` correctly requires a mobile embedded library, but its implementation-status section explicitly leaves that work future.
- The SDK calls its `embedded` feature embeddable, but the current `state: None` implementation cannot execute an in-process run. The name overstates current behavior.
- No canonical mobile `EmbeddedRuntime` API defines initialization, provider registration, run streaming, cancellation, settings/agent/skill access, A2UI actions, persistence, or shutdown.
- No offline acceptance fixture proves airplane-mode prompt → UAR agent → local LiteRT-LM → tool/skill/A2UI events → persisted restore/continuation.

## BUILD HEALTH

- KnowMe desktop/React/Flutter checks: **PASS** at the immediately preceding certification gate.
- Rebased UAR check: **FAIL** — `rmcp 2.2.0` calls `SseStream::from_bytes_stream` while the resolved `sse-stream 0.2.3` exposes `from_byte_stream`. This upstream dependency mismatch must be fixed before the new UAR commit is pushed.
- Android embedded-UAR build: **UNKNOWN/NOT PRESENT** — the application does not enable the SDK `embedded` feature, so existing APK success does not test an embedded UAR.
- Dependency graph inspection: **RISK** — `aarch64-linux-android` minimal UAR resolution includes Axum, notify, SurrealDB, and OpenSSL crates.
- Test coverage: **PARTIAL** — service protocol, local LiteRT-LM, policy, and A2UI tests exist; embedded-runtime and offline-UAR integration tests do not.

## CONSTRAINT CHECK

- AGENTS.md violations: **CURRENT ARCHITECTURAL VIOLATION** — mobile's UAR-owned agent logic is reached through HTTP instead of the required embedded Rust-library boundary.
- `constraints.md` violations: no UI-layer inversion found, but offline cross-platform parity is not met.
- Child scope: assessment writes are permitted; implementation will require deliberately widening scope to UAR, Rust core, Flutter, tests, specs, and certification evidence.

## GOAL PROGRESS

- Embed mobile-safe UAR kernel behind Rust FFI with no network: **NOT MET**.
- Register the downloaded LiteRT-LM model as UAR provider/model: **NOT MET**.
- Run agents, skills, tools, memory, policy, AG-UI, and A2UI offline: **NOT MET**.
- Keep remote/cloud access optional and BYOK-only: **PARTIAL**.
- Certify desktop and physical Android offline execution: **NOT MET**.
- Persist decision path to KBD/Prometheus/Karpathy loops: **PARTIAL** — the child phase and first evidence-backed progress record now exist.

## SYCOPHANCY REVIEW

- Score: `0.01785714365541935`.
- One low-severity S-07 length advisory was reported; no correction was mandatory because each section supplies downstream assessment evidence.

## ASSESSMENT COMPLETE

---

# ADDENDUM (2026-07-27): re-assessment against current code

The July 20 baseline above is preserved, not rewritten. Most of it is now
**obsolete** — the mobile embedded-UAR work landed between then and now
(tasks M5a–M7, M5b.1–M5b.3, FIX.1–FIX.2, MS1–MS3). Every claim below was
re-checked against the code today rather than inherited.

## Claims from 2026-07-20 that are NO LONGER TRUE

| Claim (07-20) | Verified today |
|---|---|
| "`chat` routes every `uar` lane request to `send_uar_service`" | **FALSE.** `send_uar_service` no longer appears in `chat.rs`. `state::embedded_uar()` is wired at three call sites (`chat.rs:1085`, `:1225`, `:1419`). |
| "`ensure_uar_lane_supported` only accepts a configured UAR service URL" | **FALSE.** It is now a one-line delegate to `selected_uar_runtime()` (`chat.rs:1361`); no URL requirement. |
| "SDK `RuntimeBuilder::build` sets `state` to `None`, so `orchestrator()`, `run_manager()`, `persistence()` return `None`" | **FALSE.** Those accessors return `Arc<T>` directly (`runtime.rs:77`, `:83`, `:95`). The stub was replaced by a real in-process runtime. |
| "Local provider/model registration: MISSING" | **FALSE.** `uar.rs` builds the runtime with `.local_provider(driver, provider_config)` behind a `UarRoutingDriver`, and cloud providers are registered post-build. |

Anyone planning from the 07-20 text alone would have re-done work that is
already shipped and device-certified.

## The ONE gap that remains — and it is exactly task 3.1

`rust/crates/gen_ui_agent/src/uar.rs:880` constructs the embedded runtime with:

```rust
let persistence: Arc<dyn PersistenceLayer> =
    Arc::new(providers::memory::InMemoryProvider::new());
```

Everything the embedded runtime persists — agents, skills, policies, settings,
memory, knowledge, conversations, run state, A2UI — lives in RAM and is lost on
process death. That is the whole of task 3.1, and its premise is **current and
correct**, unlike the rest of the 07-20 assessment.

UAR ships three persistence providers: `memory.rs`, `postgres.rs`,
`surreal.rs`. `SurrealDbProvider::new` is **async**, which matters: the swap is
not a one-line constructor change, because the surrounding builder call is
already async but the provider needs a storage path and connection.

## What this changes about scope

The phase's remaining work is far smaller than `tasks.md` suggests. Of its 14
open items, several describe work now visibly done (embedded runtime, FFI
operations, local provider registration, Flutter repository wiring). The file
was never updated — the same drift that left `qwen35-local-tools` fully
unticked while device-certified.

**Recommendation for plan:** re-verify each of the 14 tasks against code before
planning any of them. Do not plan from `tasks.md` as written.

## Open questions for plan

1. **Where does the mobile SurrealDB file live**, and is it the KnowMe instance
   already embedded for memory/graph-RAG, or a second store? Two SurrealDB
   instances in one mobile process is a resource decision, not a detail.
2. **What happens to existing in-memory state on upgrade?** There is no
   migration path from RAM to disk; first launch after the swap starts empty.
   That needs to be stated, not discovered.
3. **Is desktop in scope for the same swap?** Desktop shares the embedded
   runtime path, so leaving it on `InMemoryProvider` would split behaviour
   across surfaces.

## Not re-verified in this pass

The airplane-mode certification items (5.4, 5.5) and the dependency-profile /
licensing risks from 07-20 were **not** re-checked. They are not contradicted
by anything found today, but they carry the 07-20 date, not this one.
