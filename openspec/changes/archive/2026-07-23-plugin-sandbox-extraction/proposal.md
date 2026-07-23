# Proposal: plugin-sandbox-extraction

## Why

Change #1 (`plugin-wit-convergence`) delivered the contract — `knowme:plugin@1.0.0` with seven worlds — but there is still **no host that can instantiate a component**. The strategy plan (D3) bets that librefang's `sandbox_component.rs` (the most complete Component Model host in the repo: deny-by-default `WasiCtx`, capability subset at instantiation, typed `host-error`, fuel + memory limits) generalizes into `knowme_plugin_host` without librefang-specific coupling. The 2026-07-22 assessment flagged exactly this as an **unverified assumption**. This change proves or falsifies it before lifecycle, manifest, and hook-pipeline work commit to the pattern.

## What Changes

- New module **`knowme_plugin_host::sandbox`** implementing a real wasmtime-46 Component Model host against `knowme:plugin@1.0.0`:
  - `bindgen!` against the new WIT package (per-world bindings for `hook`, `contributor`, `app-shell`; unstable worlds bound as their signatures stabilize).
  - `PluginHostState` — deny-by-default `WasiCtx` + `WasiHttpCtx` (no preopens/env/stdio/network; no outbound HTTP handler), `ResourceTable`, per-guest capability set.
  - Capability subset wired at instantiation: a component importing an interface its grant set doesn't include fails with a structured **`MissingCapability`** error before guest code runs.
  - Resource limits: `Store::set_fuel` (configurable budget, 0 = unlimited convention), `ResourceLimiter` memory ceiling (default 64 MiB per `SandboxLimits` in lib.rs), wall-clock epoch interruption (default 30 s).
  - `.cwasm` precompiled cache with engine-version invalidation (pattern from UAR's `plugin_loader.rs`).
- New **`PluginHost` trait** (crate-root public API): `load`, `instantiate`, `invoke_hook`/`invoke_contributor`/`invoke_shell_action` typed per world, `unload` — the only surface later changes (lifecycle, hook pipeline, app-shell) are allowed to touch.
- New **`HostDispatch` abstraction** replacing librefang's `host_functions::dispatch` JSON-RPC: a typed trait with one method per host interface, backed in this change by a **minimal in-memory reference backend** (KV map, log sink, stub deny-all net, stub event bus) so capability gating and error taxonomy are fully exercised. Production backends (real KV store, SSRF-guarded HTTP, Cedar-gated `uar-invoke`, AG-UI fabric, inference, memory) plug in via the same trait in later changes.
- Error taxonomy: runtime `SandboxError` (compilation/instantiation/missing-capability/execution/out-of-fuel/memory-exceeded/timeout) + faithful lift into the WIT `host-error` nine-case variant.
- Tests: for each of `hook` and `contributor` worlds — instantiate a fixture component, exercise granted capability, verify undeclared capability fails at instantiation, verify fuel exhaustion and memory ceiling trap.
- **No changes to vendored trees.** librefang keeps its own host; this is an extraction of the *pattern* into KnowMe types, not a dependency on librefang crates.

## Capabilities

### New Capabilities
- `plugin-sandbox-host`: The runtime host that instantiates and invokes `knowme:plugin@1.0.0` components — capability-gated instantiation, resource limits, host-dispatch abstraction, error taxonomy, precompiled cache, and the `PluginHost` public trait surface.

### Modified Capabilities
- (none)

## Impact

- **New:** `rust/crates/knowme_plugin_host/src/sandbox/` (host, state, dispatch, limits, cache), `src/host.rs` (`PluginHost` trait), `src/dispatch.rs` (`HostDispatch` + in-memory backend), test fixture components under `rust/crates/knowme_plugin_host/tests/fixtures/`.
- **Modified:** `rust/crates/knowme_plugin_host/Cargo.toml` (async wasmtime features, `wasmtime-wasi-http`), `src/lib.rs` (re-exports).
- **Unblocks:** `plugin-conformance-harness` (fixtures promote into it), `plugin-lifecycle-registry`, `plugin-hook-pipeline`, `plugin-app-shell` — all program against `PluginHost`.
- **Risk retired:** if librefang's pattern does *not* generalize (e.g., its `GuestState`/`KernelHandle` coupling proves load-bearing), this change surfaces it here — the fallback is a clean-room host on the same WIT, and only this change's scope is affected.
- **Verification:** `cargo test -p knowme_plugin_host` green including negative fixtures; `cargo check -p knowme_plugin_host` clean; no edits inside `rust/vendor/`.
