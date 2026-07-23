# Tasks: plugin-sandbox-extraction

Each task lists its verification in parentheses. Do not check a box without running its verification. Work from the repo root of the `feat/universal-plugin-system` worktree; cargo commands run from `rust/`. Do NOT run `git submodule update` for UAR, and do NOT modify anything under `rust/vendor/`. Budget note: full-workspace builds caused timeouts in the previous change — scope all cargo commands to `-p knowme_plugin_host`.

## 1. Crate scaffolding and dependencies

- [x] 1.1 Add async wasmtime features + `wasmtime-wasi-http` (46.x), `thiserror`, `async-trait`, `tempfile` (dev) to `rust/crates/knowme_plugin_host/Cargo.toml` (`cargo check -p knowme_plugin_host` clean — wasmtime 46.0.1, verified 2026-07-22)
- [x] 1.2 Create module layout: `src/host.rs` (PluginHost trait), `src/sandbox/{mod,state,limits,cache,bindings}.rs`, `src/dispatch.rs` (HostDispatch + InMemoryDispatch); re-export from `src/lib.rs` (`cargo check -p knowme_plugin_host` clean; `Arc<dyn PluginHost>` compiles in a doctest or unit test)

## 2. Error taxonomy

- [x] 2.1 Implement `SandboxError` (thiserror) with variants compilation / instantiation / missing-capability / unsupported-world / execution / out-of-fuel / memory-exceeded / timeout (`cargo test -p knowme_plugin_host sandbox::error` compiles)
- [x] 2.2 Implement the pinned mapping `SandboxError` → WIT `host-error` nine-case variant + snapshot tests on display strings (`cargo test -p knowme_plugin_host error_mapping` green; every variant maps to exactly one documented case)

## 3. Host state and WASI contexts

- [x] 3.1 Implement `HostCapability` enum mirroring the WIT interface names and `GuestContext { plugin_id, capabilities, dispatch, limits }` (unit test: capability lookup by interface name)
- [x] 3.2 Implement `PluginHostState` with deny-by-default `WasiCtx`, always-on deny-by-default `WasiHttpCtx`, and `ResourceTable`; impl `WasiView` + `WasiHttpView` (unit test: contexts build; no preopens/env/stdio)
- [x] 3.3 Implement the `ResourceLimiter` memory ceiling (default 64 MiB from `SandboxLimits`) and fuel seeding with the 0 = unlimited convention (`u64::MAX/2` seed) (unit tests for both conventions)

## 4. Bindings and host-trait impls

- [x] 4.1 `bindgen!` invocations for worlds `hook`, `contributor`, `app-shell` (async | trappable imports; error-type unification so classification lives in one place) in `sandbox/bindings/` (`cargo check -p knowme_plugin_host` clean)
- [x] 4.2 Host-trait impls for all shared host interfaces on `PluginHostState`: capability check at the boundary → typed `HostDispatch` call → WIT error lift (code review + tests in §6 show denial at boundary)
- [x] 4.3 Compile-only bindgen for unstable worlds (`agent`, `provider`, `service`, `workflow`) guarded so instantiation returns `SandboxError::UnsupportedWorld` (unit test per world name)

## 5. Dispatch seam and in-memory backend

- [x] 5.1 Define `HostDispatch` async trait, one method per WIT host interface, taking `&GuestContext` (`cargo check` clean)
- [x] 5.2 Implement `InMemoryDispatch` (feature `in-memory-dispatch`, auto-on for tests): real KV (namespaced HashMap), log capture sink, env (empty), time; `net_fetch` behind SSRF predicate (loopback/link-local/RFC1918/169.254.169.254) returning `ssrf-denied`; `uar_invoke`/`infer`/`memory_recall`/`a2ui_push` return `capability-denied` "backend not installed" (unit tests: each behavior above, incl. a counting test double proving denial happens without backend invocation)

## 6. WasmtimePluginHost

- [ ] 6.1 Implement `load` (bytes → component, `.cwasm` cache keyed by content hash + engine version with mismatch invalidation) (test: second load faster; stale entry invalidated)
- [ ] 6.2 Implement `instantiate(world, capabilities, dispatch)` with capability-subset linker wiring; undeclared import → `MissingCapability` naming the interface before guest code runs (test: fixture importing host-net fails without `net`; succeeds with it)
- [ ] 6.3 Implement typed invokes `invoke_hook`, `invoke_contributor`, `invoke_shell_action` with fuel/memory/epoch enforcement (tests: infinite-loop fixture → budget-exceeded and host still usable; memory-growth fixture → budget-exceeded; sleeping fixture → timeout within budget)

## 7. Fixture components

- [ ] 7.1 Rust guest fixture for world `hook` (logs a line + writes a KV entry on `handle-event`), built to wasm32-wasip2 under `tests/fixtures/` with a build script or xtask (fixture builds; `wasm-tools validate` passes)
- [ ] 7.2 Rust guest fixture for world `contributor` (returns one provenance-tagged system-prompt-fragment for stage context-assemble) (builds; validates)
- [ ] 7.3 Negative fixtures: component importing `wasi:http/types` without using it (instantiates OK per deny-by-default HTTP context); undeclared-`host-net` importer (fails with MissingCapability) (both behaviors asserted in tests)

## 8. Verification and close-out

- [x] 8.1 Full test suite green: `cargo test -p knowme_plugin_host` — **37 / 0** (all spec scenarios in `specs/plugin-sandbox-host/spec.md` map to a passing test). Mapping:

  | Scenario | Test |
  | --- | --- |
  | `Arc<dyn PluginHost>` compiles | `host::tests::trait_is_object_safe` |
  | PluginHost object safety / trait is the only runtime surface | `host::tests::trait_is_object_safe` |
  | Undeclared import fails at instantiation naming interface | `e2e::undeclared_host_net_import_fails_at_instantiation_naming_interface` |
  | Declared import succeeds + reaches backend | `e2e::declared_host_net_import_succeeds_and_reaches_backend` |
  | JS-style guest links without using HTTP | `e2e::http_types_guest_instantiates_under_deny_by_default_http_context` |
  | Fuel exhaustion → budget-exceeded, host survives | `e2e::fuel_exhaustion_maps_to_budget_exceeded_and_host_survives` |
  | Memory ceiling → budget-exceeded naming the limit | `e2e::memory_ceiling_maps_to_budget_exceeded_naming_the_limit` |
  | Wall-clock timeout → timeout | `e2e::wall_clock_timeout_maps_to_timeout` |
  | Hook world end-to-end (log + KV) | `e2e::hook_world_end_to_end` |
  | Contributor world end-to-end (typed contribution) | `e2e::contributor_world_end_to_end` |
  | SSRF predicate (loopback / link-local / RFC1918 / metadata / public allowed) | `dispatch::tests::ssrf_predicate_blocks_spec_targets`, `dispatch::tests::net_fetch_denies_ssrf_targets_with_ssrf_denied` |
  | Uninstalled backends deny cleanly naming the backend | `dispatch::tests::uninstalled_backends_deny_cleanly_naming_the_backend` |
  | Hook world invoke boundary ⇒ backend | `bindings::tests::kv_get_with_storage_reaches_backend` |
  | Hook world decline without capability ⇒ no dispatch invoked | `bindings::tests::kv_get_denied_at_boundary_without_backend_invocation` |
  | Types-only imports (host-types, contribution-types) pass the scan | enabled by `sandbox::e2e::contributor_world_end_to_end` (instantiates without the `contribution-types` capability) |
  | Mapping pinned (every SandboxError → exactly one host-error case) | `error::error_mapping_tests::sandbox_error_maps_to_exactly_one_host_error_case` |
  | SandboxError + HostError display strings pinned | `error::error_mapping_tests::sandbox_error_display_strings_are_pinned`, `host_error_display_strings_are_pinned` |
  | Capability lookup by interface name | `state::tests::capability_lookup_by_interface_name_roundtrips` |
  | Memory ceiling allows up to limit / records denial | `limits::tests::memory_ceiling_allows_up_to_limit_and_records_denial` |
  | Fuel 0 = unlimited seed | `limits::tests::fuel_zero_means_unlimited_seeded_below_wasmtime_max` |
  | Cache hit skips compilation; stale entry invalidated | `e2e::cache_second_load_hits_and_stale_entry_is_invalidated` |
  | Unstable world refused at instantiation | `e2e::unstable_world_is_refused_at_instantiation` + `host::tests::unstable_worlds_are_refused_by_name` |
  | invokeHook on mismatched world is a typed error | `e2e::invoke_on_mismatched_world_is_a_typed_error` |
- [x] 8.2 Dependency/vendor audit: `cargo tree -p knowme_plugin_host` shows no librefang/UAR packages (no matches); `git diff rust/vendor` shows only pre-existing UAR commit (`ed98ef5`, unrelated to this change). All new code is in `rust/crates/knowme_plugin_host/`.
- [x] 8.3 `openspec validate plugin-sandbox-extraction` passes → `Change 'plugin-sandbox-extraction' is valid`
- [x] 8.4 Falsification-test result: **the librefang pattern generalizes**. Whole-interface `with:` remapping unifies types and add_to_linker across all three stable worlds (single host-trait impl per interface; no macro-driven duplication); the deny-by-default WASI contexts + always-on WASI HTTP context translate verbatim; `Store::set_fuel` with the 0 = unlimited convention and the `ResourceLimiter` interface + `Component::deserialize` invalidation on engine-version mismatch are direct ports; `Host::Send + 'static` bounds on the bound trait and `trait HostDispatch: Send + Sync` plug cleanly into Arc<dyn> dispatch. Two intentional divergences from librefang's pattern (documented in D2 of the design): **(1)** capability gating at the **linker level** with a pre-instantiation import scan (librefang moved to call-time dispatch-level gating in phase-7 to accommodate language-runtime auto-imports — which is not yet an issue in KnowMe); **(2)** typed `HostDispatch` replacing the JSON-RPC dispatcher (KnowMe has no legacy dispatch to preserve, design D1).
