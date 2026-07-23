# Design: plugin-sandbox-extraction

## Context

`plugin-wit-convergence` delivered `knowme:plugin@1.0.0` (seven worlds, nine-case `host-error`, capability-granular host interfaces) but no runtime. The candidate implementation pattern is librefang's `sandbox_component.rs` (923 lines) + `host_functions.rs` (2,222 lines). Close reading of those files shows the structure to preserve and the coupling to break:

**Worth preserving (the pattern):**
- `PluginHostState` trio: deny-by-default `WasiCtx` (empty-but-valid responses for guest language runtimes) + always-on deny-by-default `WasiHttpCtx` (StarlingMonkey's preview2-shim auto-imports `wasi:http/types` even when unused — omitting it breaks JS guests at link time) + `ResourceTable` (sandbox_component.rs:101-141).
- `bindgen!` with `imports: { default: async | trappable }` and a `with`-style error-type unification so error classification lives in one place (sandbox_component.rs:58-87).
- One-liner host trait impls delegating to a single dispatch point (sandbox_component.rs:181-277) — capability checks never scatter across impls.
- Limits: `Store::set_fuel` with the 0 = unlimited convention (sandbox_component.rs:497-514), `ResourceLimiter` memory ceiling (sandbox.rs:133+), `timeout_secs` → epoch interruption with a preserved error string contract.
- `.cwasm` cache with wasmtime-version mismatch invalidation (UAR `plugin_loader.rs`).

**librefang-specific coupling to break (the extraction risk):**
- `GuestState` — carries librefang's `Capability` enum, agent id, and `KernelHandle` (a handle into the librefang kernel's agent/registry subsystems).
- `host_functions::dispatch` — a 2,222-line JSON-RPC dispatcher whose side effects are librefang kernel operations (`agent_send`, `agent_spawn`, kernel-scoped KV).
- `SandboxConfig.capabilities: Vec<Capability>` — librefang's capability vocabulary, not KnowMe's manifest capability cards.
- Single world (`plugin`) — KnowMe has seven; `bindgen!` generates one world per macro invocation.

Stakeholders: `plugin-lifecycle-registry` (will own instantiation policy), `plugin-hook-pipeline` and `plugin-app-shell` (invoke paths), `plugin-conformance-harness` (fixtures), mobile (must run under the same trait on iOS/Android).

## Goals / Non-Goals

**Goals:**
- A `PluginHost` trait as the crate's sole public runtime surface; a wasmtime-46 implementation (`WasmtimePluginHost`) instantiating `knowme:plugin@1.0.0` components.
- Capability subset wiring: undeclared import → structured `MissingCapability` at instantiation, never a first-use trap.
- A typed `HostDispatch` trait (one method per WIT host interface) decoupling host-trait impls from backends; in-memory reference backend exercising the full error taxonomy.
- Fuel, memory, and wall-clock limits with stable, testable error mapping into the WIT nine-case `host-error`.
- `.cwasm` precompiled cache keyed by content hash + engine version.
- Proof for `hook` and `contributor` worlds end-to-end with fixture components (Rust guest, wasm32-wasip2).

**Non-Goals:**
- Production backends (real KV/store, SSRF-guarded HTTP client, Cedar-gated `uar-invoke`, AG-UI fabric, inference, memory, A2UI surface push) — each arrives with its owning later change and plugs into `HostDispatch` without touching the sandbox.
- `agent`/`provider`/`service`/`workflow` world bindings beyond compile-checking (their WIT is `unstable`-annotated).
- Lifecycle, manifest v2, installation, signing (later changes).
- Vendored-tree modifications of any kind.
- Migrating librefang itself onto the extracted host.

## Decisions

**D1 — Extract the pattern, not the code: new `knowme_plugin_host::sandbox` with KnowMe types, no librefang dependency.**
`GuestState`/`KernelHandle`/`dispatch` are librefang kernel internals; depending on them would drag the kernel into KnowMe's plugin host and violate "no librefang-specific coupling in the extracted API." Instead: `PluginHostState { guest: GuestContext, table, wasi, wasi_http }` where `GuestContext { plugin_id, capabilities: HashSet<HostCapability>, dispatch: Arc<dyn HostDispatch>, limits }`. `HostCapability` is a crate-local enum mirroring the WIT interface names — later mapped from manifest capability cards by `plugin-lifecycle-registry`. *Alternative considered:* depend on `librefang-runtime` as a library — rejected: it couples KnowMe's host to a vendored fork's kernel handle and its JSON-RPC dispatch semantics, and fails the assessment's falsification test by construction.

**D2 — `HostDispatch` typed trait replaces JSON-RPC dispatch.**
librefang's dispatch is JSON in/out because it predates the typed WIT layer (its WIT is "a typed shim over the existing dispatch"). KnowMe has no legacy dispatch to preserve, so the seam is typed from the start: `trait HostDispatch: Send + Sync { async fn kv_get(&self, cx: &GuestContext, ns: &str, key: &str) -> Result<Option<Vec<u8>>, HostError>; async fn net_fetch(...) -> Result<HttpResponse, HostError>; ... }`. Capability checks live in the sandbox's host-trait impls (the single choke point), not inside backends: the impl checks `cx.capabilities.contains(...)` → `capability-denied` before calling dispatch. *Alternative:* checks inside each backend — rejected: duplicates librefang's single-source-of-truth lesson; the sandbox must be able to audit every denial in one place.

**D3 — One `bindgen!` per world, behind a world-dispatch layer.**
`bindgen!` emits one Rust module per world; seven worlds = seven macro invocations in `sandbox/bindings/`. `WasmtimePluginHost::instantiate` takes a `WorldKind` (mirrors the manifest `worlds` vocabulary from change #1), selects the linker wiring and typed invoke path, and returns an opaque `InstanceHandle`. Invoke methods are typed per world (`invoke_hook`, `invoke_contributor`, `invoke_shell_action`); unstable worlds return `SandboxError::UnsupportedWorld` until their WIT stabilizes. *Alternative:* runtime-introspected component calls without bindgen — rejected: loses compile-time ABI checking, the main reason for the component model.

**D4 — Always-on deny-by-default `WasiCtx` + `WasiHttpCtx`, exactly per librefang's evidence.**
Copy the rationale, not the code: guest language runtimes (CPython, StarlingMonkey, Go) call `wasi:cli/environment` during init and must get empty-but-valid responses; JS guests auto-import `wasi:http/types` regardless of usage, so the HTTP context must always bind — with no outbound handler registered unless a `net` capability is granted (handler arrives with the production net backend). This is the single most expensive lesson to relearn by trial; it is adopted verbatim.

**D5 — Limits: fuel + memory ceiling + epoch wall-clock, with librefang's conventions.**
`fuel_limit` (0 = unlimited → seed `u64::MAX/2` to avoid wasmtime's max-value edge), `ResourceLimiter` at 64 MiB default (from `SandboxLimits` in lib.rs:97-109), `timeout_secs` default 30 via epoch interruption. Error mapping is spec-pinned: out-of-fuel → `budget-exceeded`; memory ceiling → `budget-exceeded` (with detail string); epoch timeout → `timeout`; trap → `internal` with the trap message preserved (librefang pins the display string in a kernel regression test — we adopt the same discipline with a snapshot test).

**D6 — `.cwasm` cache keyed by (sha256 of component bytes, wasmtime engine version).**
Adopts UAR `plugin_loader.rs`'s version-mismatch invalidation. Cache dir injected via `SandboxConfig` (platform dirs resolved by the caller — lifecycle change — keeping the sandbox platform-agnostic and unit-testable with tempdirs).

**D7 — Reference backend is in-memory and deliberately small, but the denial paths are real.**
`InMemoryDispatch` implements every `HostDispatch` method: KV/log/env/time work for real (HashMap + capture sink); `net_fetch` returns `capability-denied` unless the guest holds `net` *and* the URL passes a local SSRF predicate (loopback/link-local/RFC1918/metadata) — the predicate ships now because the conformance negative fixtures need it; `uar_invoke`/`infer`/`memory_recall`/`a2ui_push` return `capability-denied` with "backend not installed" detail. This keeps the sandbox honest: guests can prove isolation without waiting for production backends.

## Risks / Trade-offs

- [Per-world bindgen multiplies generated code and linker wiring surface] → contained in `sandbox/bindings/` with a shared `wire_host_interfaces!` macro; worlds beyond the first three are compile-only until stabilized.
- [The extraction may hit librefang assumptions invisible in source reading (e.g., dispatch's exact JSON shapes that wit_host depends on)] → D2 sidesteps this by not porting dispatch at all; the risk transfers to "did we get the host-trait impl boilerplate right," which the fixture tests answer directly. If fixtures reveal a fundamental mismatch, fallback is clean-room (same WIT, no pattern reuse) — scope-contained to this change.
- [Async bindgen (`async | trappable`) forces a tokio runtime into `gen_ui_ffi`'s call graph later] → `PluginHost` methods are async; FFI bridging is the lifecycle change's problem (frb handles async); the sandbox itself stays runtime-agnostic by taking an ambient tokio handle.
- [Epoch interruption requires a ticking engine thread — battery cost on mobile] → engine shared per-process (one `Engine` in `WasmtimePluginHost`), epoch tick only while at least one instance is executing; mobile tuning lands in `plugin-mobile-parity-hardening`.
- [In-memory backend could leak into production by accident] → it is feature-gated (`#[cfg(any(test, feature = "in-memory-dispatch"))]`); the default build has no backend until lifecycle wires one.

## Migration Plan

No consumers exist yet — nothing to migrate. Rollout order: sandbox + trait land → `plugin-conformance-harness` promotes fixtures → `plugin-lifecycle-registry` becomes the first real consumer. Rollback: delete the new modules; the crate returns to types-only with no external impact.

## Open Questions

- Should `InstanceHandle` support pause/resume (serialize store state) for mobile app suspension? Deferred to `plugin-mobile-parity-hardening`; the trait leaves room (`unload` is explicit, state is opaque).
- Epoch tick granularity (10 ms vs 100 ms) — needs on-device measurement; default 50 ms with a config knob.
- Whether `contributor`'s 200 ms epoch budget is enforced by the sandbox (fuel) or the pipeline (wall-clock): proposal — sandbox enforces hard fuel/timeout, pipeline enforces the soft 200 ms contribution budget. Confirm in `plugin-hook-pipeline` design.
