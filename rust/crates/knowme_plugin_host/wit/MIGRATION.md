# MIGRATION — legacy plugin dialects → `knowme:plugin@1.0.0`

This document maps every `knowme:plugin@0.1.0` (`wit/knowme-plugin.wit`) and
`librefang:plugin@0.1.0` (`rust/vendor/librefang/crates/librefang-skills/wit/`)
interface, function, and error case to its `knowme:plugin@1.0.0` equivalent
(`wit/v1/`), or records it as dropped with a reason.

**Deprecation note:** `wit/knowme-plugin.wit` (`knowme:plugin@0.1.0`) is
deprecated as of this change. It is superseded by `knowme:plugin@1.0.0` and
will be deleted in `plugin-manifest-v2` once the manifest validates `worlds`
against the 1.0.0 vocabulary.

## knowme:plugin@0.1.0 → 1.0.0

| 0.1.0 construct | 1.0.0 target | Notes |
|---|---|---|
| `interface host` | split into `host-log`, `host-kv`, `host-uar`, `host-events` | single fat interface replaced by granular capabilities (D2) |
| `host.log-level` enum | `host-log.log-level` | unchanged |
| `host.log(level, message)` | `host-log.log(level, message)` | unchanged; infallible |
| `host.storage-get(key) -> option<list<u8>>` | `host-kv.get(namespace, key) -> result<option<list<u8>>, host-error>` | namespace parameter added; error channel typed |
| `host.storage-set(key, value) -> result<_, string>` | `host-kv.set(namespace, key, value) -> result<_, host-error>` | string errors map: `io` / `invalid-argument` / `policy-denied` (cross-namespace) |
| `host.storage-delete(key) -> result<_, string>` | `host-kv.delete(namespace, key) -> result<_, host-error>` | same error mapping |
| `host.uar-invoke(skill, payload-json) -> result<string, string>` | `host-uar.invoke(skill, payload-json) -> result<string, host-error>` | Cedar-gated semantics preserved (D3); policy rejections become `policy-denied`, timeouts `timeout`, host faults `internal` |
| `host.emit-event(topic, payload-json) -> result<_, string>` | `host-events.emit(topic, event: ag-ui-event) -> result<_, host-error>` | JSON payload promoted to a typed AG-UI event record (D5) |
| `interface plugin` | `interface lifecycle` + per-kind world exports | `init`/`shutdown` shared; `handle-event` moves to the `hook` world |
| `plugin.init() -> result<_, string>` | `lifecycle.init() -> result<_, string>` | unchanged |
| `plugin.handle-event(topic, payload-json) -> result<_, string>` | `hook.handle-event(topic, event: ag-ui-event) -> result<_, string>` | typed envelope (D5) |
| `plugin.shutdown()` | `lifecycle.shutdown()` | unchanged |
| `world knowme-plugin` | `world hook` | guests retarget; `hello-knowme` migrated in this change |

## librefang:plugin@0.1.0 → 1.0.0

### `host.wit`

| 0.1.0 construct | 1.0.0 target | Notes |
|---|---|---|
| `interface host-types` | `interface host-types` | same name, same role |
| `host-error.capability-denied(string)` | `host-error.capability-denied(string)` | 1:1 |
| `host-error.path-denied(string)` | `host-error.path-denied(string)` | 1:1 |
| `host-error.ssrf-denied(string)` | `host-error.ssrf-denied(string)` | 1:1 |
| `host-error.io(string)` | `host-error.io(string)` | 1:1 |
| `host-error.invalid-argument(string)` | `host-error.invalid-argument(string)` | 1:1 |
| `host-error.timeout` | `host-error.timeout` | 1:1 |
| `host-error.internal(string)` | `host-error.internal(string)` | 1:1 |
| — (new) | `host-error.policy-denied(string)` | Cedar/manifest policy rejections (was a bare string in knowme 0.1.0) |
| — (new) | `host-error.budget-exceeded(string)` | token/byte/time budget exhaustion |
| `interface fs` (`read`, `write`, `list-entries`) | `interface fs` | ported verbatim (semantically identical) |
| `interface net` → renamed `host-net` | `http-request`/`http-response`/`fetch` ported 1:1 | rename per D2/D3; SSRF-guard contract documented on `fetch` |
| `interface kv` (`get`, `set`) | `interface host-kv` | explicit `namespace` parameter added; values `string` → `list<u8>` (absorbs knowme 0.1.0 byte storage); `delete` added from knowme 0.1.0 |
| `interface agent` (`send`, `spawn`) | **dropped**, because the A2A-shaped `agent` world (`start-task`/`poll-task`/`cancel-task`) supersedes ad-hoc agent messaging, and inter-plugin/agent notification routes through `host-events` | reintroduce under `plugin-agent-world` if real guests need it |
| `interface env` (`read`) | `interface env` | ported verbatim |
| `interface time` (`now`) | `interface time` | ported verbatim |

### `world.wit`

| 0.1.0 construct | 1.0.0 target | Notes |
|---|---|---|
| `interface plugin-types` | **dropped**, because 1.0.0 guest exports return `result<_, string>` (knowme 0.1.0 style); a typed guest-error variant is deferred until real guests demonstrate the need (additive later under D6) | see cases below |
| `plugin-error.invalid-input(string)` | **dropped**, because guest export errors are plain strings in 1.0.0 (see `plugin-types` note) | |
| `plugin-error.internal(string)` | **dropped**, because guest export errors are plain strings in 1.0.0 (see `plugin-types` note) | |
| `world plugin` (single `export run`) | **dropped**, because one universal `run` loses compile-time contract checking (D4); skill-style guests target the `hook` or `contributor` world instead | |
| `export run() -> result<_, plugin-error>` | per-kind exports: `hook.handle-event` / `contributor.contribute` | closest equivalents for skill-style guests |
| instantiation-time capability subset (`MissingCapability`) | unchanged semantics, all seven worlds | host wires declared subset; undeclared imports fail at instantiation |

## uar:plugin@0.1.0 (vendored UAR) — flagged, not mapped

`rust/vendor/universal-agent-runtime/wit/uar-plugin.wit` (`uar:plugin@0.1.0`)
is referenced by UAR's `plugin_loader.rs`, but its `bindgen!` integration is
still pending upstream and its error model (`plugin-error`:
`capability-denied`, `resource-exhausted`, `guest-trap`, `host-rejected`, …)
belongs to the UAR loader, not to the KnowMe host surface. It remains
functional inside the vendored tree behind the `wasm-runtime` feature.
Migrating UAR plugins to `knowme:plugin@1.0.0` is a separate change with its
own gate (design risk register); when it lands, `capability-grant` maps to
the 1.0.0 capability-subset model and `plugin-error` cases map onto
`host-error` (`resource-exhausted` → `budget-exceeded`, `host-rejected` →
`policy-denied`).

## Parity checklist (drift watch)

The 1.0.0 `fs` / `env` / `time` / `host-net` (ex-`net`) / `host-types`
interfaces are semantic copies of the librefang originals (D2). The two
trees can drift pre-stabilization; `plugin-conformance-harness` adds a
fixture instantiating a librefang-dialect guest against this mapping. Any
intentional divergence must be recorded in this document.
