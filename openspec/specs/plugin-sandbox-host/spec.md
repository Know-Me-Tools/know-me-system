# plugin-sandbox-host Specification

## Purpose
TBD - created by archiving change plugin-sandbox-extraction. Update Purpose after archive.
## Requirements
### Requirement: PluginHost public trait
The crate SHALL expose a `PluginHost` trait as its sole public runtime surface with methods to load a component, instantiate it for a declared world and capability set, invoke world-specific entry points (`hook`, `contributor`, `app-shell` at minimum), and unload instances. All methods MUST be async and the trait MUST be object-safe (`Arc<dyn PluginHost>` usable).

#### Scenario: Trait is the only runtime surface
- **WHEN** the crate's public API is inspected
- **THEN** no public function outside `PluginHost` and its data types performs instantiation or invocation, and `Arc<dyn PluginHost>` compiles

### Requirement: Capability-gated instantiation
Instantiation SHALL wire exactly the host interfaces implied by the caller-provided capability set. A component importing an interface outside that set MUST fail instantiation with a structured `MissingCapability` error naming the interface, before any guest code executes.

#### Scenario: Undeclared import fails at instantiation
- **WHEN** a fixture component importing `host-net` is instantiated with a capability set lacking `net`
- **THEN** instantiation returns `SandboxError::MissingCapability` naming `host-net` and the guest's `init` export never runs

#### Scenario: Declared import succeeds
- **WHEN** the same component is instantiated with `net` in the capability set
- **THEN** instantiation succeeds and the guest can call `host-net.fetch` (reaching the installed dispatch backend)

### Requirement: Deny-by-default WASI contexts
Every instance SHALL receive a `WasiCtx` with no preopens, no environment, no stdio inheritance, and no sockets, plus an always-bound `WasiHttpCtx` with no outbound request handler unless a network backend is installed. Guests whose language runtimes query environment or HTTP types at init MUST still instantiate successfully.

#### Scenario: JS-style guest links without using HTTP
- **WHEN** a fixture component that imports `wasi:http/types` (but never calls it) is instantiated
- **THEN** instantiation succeeds and any attempted outbound request fails

### Requirement: Resource limits
The host SHALL enforce per-invocation fuel (0 = unlimited convention), a linear-memory ceiling via `ResourceLimiter` (default 64 MiB), and a wall-clock timeout via epoch interruption (default 30 s). Limit violations MUST map to typed errors: out-of-fuel and memory-ceiling to `host-error.budget-exceeded`, wall-clock to `host-error.timeout`.

#### Scenario: Fuel exhaustion
- **WHEN** a fixture containing an infinite loop is invoked with a finite fuel budget
- **THEN** the invocation returns a budget-exceeded error and the host remains able to instantiate and invoke other components

#### Scenario: Memory ceiling
- **WHEN** a fixture that grows memory past 64 MiB is invoked with the default ceiling
- **THEN** the invocation returns a budget-exceeded error naming the memory limit

#### Scenario: Wall-clock timeout
- **WHEN** a fixture that sleeps past the configured `timeout_secs` is invoked
- **THEN** the invocation returns a timeout error within timeout_secs plus one epoch tick

### Requirement: Typed HostDispatch seam
Host-trait implementations SHALL perform capability checks at the sandbox boundary and delegate side effects to a `HostDispatch` trait with one method per WIT host interface. A denial MUST be producible at the boundary without invoking the backend, and every denial MUST be auditable at that single choke point.

#### Scenario: Denial without backend
- **WHEN** a guest calls `host-kv.get` without the `storage` capability and no backend is installed
- **THEN** the guest receives `host-error.capability-denied` and no dispatch method is invoked (verified by a counting test double)

### Requirement: In-memory reference backend
The crate SHALL provide an `InMemoryDispatch` (feature-gated for tests) implementing every `HostDispatch` method: real KV/log/env/time behavior, `net_fetch` guarded by an SSRF predicate (loopback, link-local, RFC1918, cloud metadata) returning `ssrf-denied`, and `capability-denied` with "backend not installed" for `uar_invoke`, `infer`, `memory_recall`, and `a2ui_push`.

#### Scenario: SSRF predicate
- **WHEN** a guest with the `net` capability calls `net_fetch` against `http://169.254.169.254/` or `http://127.0.0.1/` via the in-memory backend
- **THEN** the call returns `host-error.ssrf-denied`

#### Scenario: Uninstalled backends deny cleanly
- **WHEN** a guest calls `host-uar.invoke` with the capability granted but only the in-memory backend installed
- **THEN** the call returns `host-error.capability-denied` whose detail names the missing backend

### Requirement: Precompiled component cache
The host SHALL cache precompiled `.cwasm` artifacts keyed by component content hash and wasmtime engine version, invalidating on version mismatch. A second load of unchanged bytes MUST skip recompilation.

#### Scenario: Cache hit skips compilation
- **WHEN** the same component bytes are loaded twice with a cache directory configured
- **THEN** the second load completes from cache measurably faster and a stale cache entry from a different engine version is invalidated and recompiled

### Requirement: World-dispatch for stable worlds
Instantiation SHALL accept a world identifier matching the manifest `worlds` vocabulary and provide typed invoke paths for `hook` (`handle-event`), `contributor` (`contribute`), and `app-shell` (`init-shell`, `on-action`). Instantiating an `unstable`-annotated world (`agent`, `provider`, `service`, `workflow`) MUST return `SandboxError::UnsupportedWorld` until its WIT stabilizes.

#### Scenario: Hook world end-to-end
- **WHEN** a fixture hook component emitting a log line and writing a KV entry on `handle-event` is instantiated for world `hook` with `storage` granted and invoked with a test event
- **THEN** the log sink captures the line and the KV backend contains the entry

#### Scenario: Contributor world end-to-end
- **WHEN** a fixture contributor returning one provenance-tagged `system-prompt-fragment` contribution for stage `context-assemble` is invoked
- **THEN** the host returns the typed contribution with stage, items, and provenance intact

#### Scenario: Unstable world refused
- **WHEN** instantiation is requested for world `workflow`
- **THEN** the host returns `SandboxError::UnsupportedWorld` naming `workflow`

### Requirement: Error taxonomy stability
Runtime errors SHALL be a `thiserror` enum `SandboxError` (compilation, instantiation, missing-capability, unsupported-world, execution, out-of-fuel, memory-exceeded, timeout) with a pinned mapping into the WIT nine-case `host-error`, guarded by snapshot tests on display strings.

#### Scenario: Mapping pinned
- **WHEN** the error-mapping unit tests run
- **THEN** every `SandboxError` variant maps to exactly one documented `host-error` case and display strings match the pinned snapshots

### Requirement: No vendored-tree coupling
The implementation MUST NOT depend on `librefang-*` crates or `universal-agent-runtime` crates and MUST NOT modify files under `rust/vendor/`.

#### Scenario: Dependency audit
- **WHEN** `cargo tree -p knowme_plugin_host` is inspected and `git status` in `rust/vendor/` is checked
- **THEN** no `librefang` or `universal-agent-runtime` packages appear and no vendored files are modified

