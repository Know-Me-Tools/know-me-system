# ADR-001 — WASM Target for KnowMe Plugin Modules

**Status:** Accepted
**Date:** 2026-08-03
**Decider:** Travis James
**Supersedes:** open decision A-03 raised against KNOWME-SEC-AGT-001 v1.0
**Related:** PAGS-SPEC-KM-CRM-001, PAGS-SPEC-KM-GOAL-001, ADR-002

---

## Decision

**Target the WebAssembly Component Model on `wasm32-wasip2`, hosted via `wasmtime::component` on a pinned Wasmtime LTS.** Keep the `wasm32-wasip1` + `wasi_snapshot_preview1` adapter path as a documented hedge. Guest modules remain synchronous on WASI 0.2 — do not adopt Component Model async or WASI 0.3 in guests yet.

Amends KNOWME-SEC-AGT-001 §07, which specifies only `runtime: wasm`. The manifest field becomes `runtime: wasm-component` with an explicit `wit_world` binding.

## Why this decision, specifically

KNOWME-SEC-AGT-001 §07 makes static import-surface validation a hard registration gate: *"UAR compares declared capabilities against actual WASM imports — any undeclared import causes rejection."* That single requirement decides the target.

Under core modules (P1), imports are untyped two-level string pairs. Validation means parsing `Payload::ImportSection` with `wasmparser` and string-matching an allowlist. Workable, but "zero capability" is defined **negatively** — as the absence of any WASI import — and the allowlist semantics are hand-maintained.

Under the Component Model, imports are a typed WIT world. `wasmtime::component::Component::component_type()` gives programmatic, typed reflection *before* instantiation, and the component `Linker` refuses to instantiate on interface mismatch. "Zero capability" becomes a **positively specified, auditable fact**: a world that imports nothing but `wasi:clocks/wall-clock` and `wasi:random/random`. Registration validation becomes a typed structural comparison rather than string matching.

Microsoft's Wassette — the closest production analog to the UAR skill model (deny-by-default capability gating on Wasmtime, exposed via MCP) — is components-only and builds its entire security and introspection model on exactly this property.

## Evidence

### Standards status (mid-2026)
- WASI 0.2 (Preview 2) stable since January 2024, latest patch v0.2.12.
- **WASI 0.3 ratified June 11, 2026**, rebasing WASI onto Component Model async primitives; removes `wasi:io`, folding streams/futures into the Canonical ABI.
- WASI 1.0 targeted late 2026 / early 2027, gated on Component Model 1.0.
- Preview 1 is frozen, not deleted. The rustc book: WASIp1 syscalls *"have not changed since then"*, APIs are *"no longer receiving any maintenance"*, and `wasm32-wasip1` is *"intended for historical compatibility."* Bytecode Alliance maintains an explicit long-term backward-compatibility commitment: P1 modules still work, P2 components still work.

**Read:** Preview 1 is a safe-but-stagnant floor on a long sunset track, not a cliff. Choosing it is not risky; it is merely terminal.

### Rust toolchain
- `wasm32-wasip1` and `wasm32-wasip2` both Tier 2, installable via rustup.
- wasip2 reached Tier 2 in Rust 1.82 (2024-10-17), with the `wasm-component-ld` linker.
- Legacy `wasm32-wasi` name removed in Rust 1.84 (Jan 2025).
- `cargo build --target wasm32-wasip2` produces components natively — no post-processing.
- `cargo-component` is being deprecated in favor of native tooling + `wit-bindgen`.
- WASI 0.3 Rust target exists but is **Tier 3** (not in rustup) as of mid-2026 — a direct reason to stay on 0.2 in guests.
- Known gaps: `std::thread` missing on component targets; exporting/importing types still needs third-party bindgen.

None of these block a Rust-only, synchronous, pure-function plugin on wasip2 today.

### Wasmtime
- Documented UAR position is v41 (~Nov 2025). Current is v46 (2026-06-22); v47 published 2026-07-20.
- **LTS policy:** every 12th release (24, 36, 48…) is LTS with 24 months support. Non-LTS releases get ~2 months.
- v36 (2025-08-20) is supported to 2027-08-20. v48 is due August 2026.
- WASI 0.3 support landed in 43+; v46 ships WASI 0.3.0 with Component Model Async **on by default**.

**Security note, action-required:** CVE-2026-27195 affects the `component-model-async` feature, which became default in Wasmtime 39.0.0. A `call_async` future-drop bug can panic the host. Patched in 40.0.4 and 41.0.4; 42.0.0+ unaffected. **If UAR remains on v41 transiently, it must be ≥41.0.4.**

### Overhead for small pure functions
Evidence here is thinner than ideal — flagged as such.

- **Binary size:** componentization adds KB-scale wrapper metadata always, plus the ~100 KB preview1→preview2 adapter *only when the module imports WASI/std*. A truly capability-free module needs no adapter — `wasm-tools component new adder.wasm` runs without `--adapt`. No credible published head-to-head wasip1-vs-wasip2 release-size table exists. **Measure locally on a representative module before finalizing.**
- **Instantiation:** Wasmtime spawns instances in ~5 µs with the pooling allocator plus copy-on-write memory init. Components share the identical compile/instantiate model — `Component::new` compiles synchronously, nothing compiles at instantiation. No published micro-benchmark isolates the component-vs-module delta; maintainer statements imply it is small because instantiation is dominated by memory/table setup. `Engine::precompile_component` removes compilation from the hot path entirely.
- **Per-call marshalling:** primitives and small records that fit the Canonical ABI flat representation cost ≈ zero. Strings/lists incur one `cabi_realloc` per element under today's eager ABI; the planned lazy ABI (non-breaking, opt-in, shipping in a 0.3.x minor) makes this zero-copy.
- **The one measured overhead:** ~3.5x on synchronous cross-call paths, from async task infrastructure (Nick Fitzgerald, Plumbers Summit, reported in "The Road to Component Model 1.0", June 2026). This is a **regression introduced by adding component-model async**, under active repair (Wasmtime #12311), with the stated goal of restoring the pre-async synchronous performance profile.

**Assessment for our workload:** irrelevant. These modules are invoked once per brief run with a structured payload. A 3.5x multiplier on a microsecond-scale cross-call is not on any path that matters, and staying synchronous on WASI 0.2 avoids it entirely.

### Ecosystem signals
| System | Target | Note |
|---|---|---|
| Wassette (Microsoft) | Components only | Closest analog; WIT introspection is the security model |
| wasmCloud (CNCF) | Components only | `wasm32-wasip2`; WASI 0.3 behind a `wasip3` flag |
| Spin / Fermyon | Components | WASI 0.3 RC in Spin v3.5 (Nov 2025) |
| Shopify Functions | **Core module / wasip1** | Bespoke NaN-boxed i64 ABI over stdin/stdout |
| Extism | **Core module** | Generic bytes-in/bytes-out; *"no concrete plans"* for Component Model |
| Helm 4 | **Core module** (via Extism) | Chose Extism because Component Model unsupported in Wazero |

**Reading:** greenfield capability-secure systems standardized on components. The core-module holdouts stayed for reasons that do not apply here — existing bespoke ABI investment (Shopify), or a non-Wasmtime/polyglot runtime constraint (Extism, Helm on Wazero). We are a new Rust + Wasmtime host with a mandatory typed-validation requirement. We match the first group.

### Migration cost
Low in both directions. A wasip1 core module componentizes at build time:

```
wasm-tools component new module.wasm \
  --adapt wasi_snapshot_preview1.reactor.wasm \
  -o component.wasm
```

The cost avoided by starting on components **now** is the data-marshalling contract. Moving from JSON-over-linear-memory (P1) to WIT-typed records (P2) is a breaking interface change touching every module and the host boundary. Do it once, up front.

### Known rough edges (accepted)
- Component debugging tooling immature; componentization errors can be cryptic.
- Guest support beyond Rust uneven — Go via TinyGo/componentize-go; Python/JS via componentize-py/jco. **Non-issue for a Rust-only module set.**
- Component Model is pre-1.0; the ABI will change before standardization, though via a deliberately non-breaking transition.
- Wasmtime ships semver-major monthly. Mitigated by LTS pinning.

## Consequences

**Required actions:**

1. **Pin Wasmtime to an LTS.** Adopt v36 (supported to 2027-08-20) now, or plan the jump to v48 in August 2026. If v41 persists in the interim, ensure ≥41.0.4 for CVE-2026-27195. This decouples UAR from monthly `wasmtime::component` API churn.
2. **Define the module contract in WIT.** One world exporting `run: func(input: <record>) -> result<<record>, error>`, importing only `wasi:clocks/wall-clock` and `wasi:random/random`. Everything else — filesystem, sockets, env, monotonic clock — is absent from the world by construction.
3. **Implement §07 validation via `component_type()`** for typed structural comparison against the declared manifest, plus `wasm-tools component wit` in CI. Retain the `wasmparser` core-module validator only if P1 modules are ever accepted.
4. **Use WIT-typed records at the boundary**, not JSON blobs.
5. **Stay synchronous.** No Component Model async, no WASI 0.3 in guests. Revisit when Wasmtime #12311 lands and the WASI 0.3 Rust target reaches Tier 2.
6. **Amend KNOWME-SEC-AGT-001 §07** — `runtime: wasm-component`, plus a `wit_world` manifest field and the host-function export table (amendment A-02).

**Fallback trigger:** if a required guest language lacks a mature native wasip2 toolchain, fall back to building core modules for wasip1 and componentizing at registration time. UAR still hosts components; typed validation is preserved.

**When pure Preview 1 would have been correct:** broad non-Rust guest support with only core-module paths; a non-Wasmtime runtime without Component Model support; or an existing heavily invested bytes-in/bytes-out ABI. None apply.

## Confidence and caveats

- **High confidence** on standards status, toolchain tiers, Wasmtime LTS policy, and the introspection argument. All primary-sourced.
- **Moderate confidence** on overhead. No controlled component-vs-module benchmark for tiny pure functions exists publicly. The ~5 µs instantiation and ~100 KB adapter figures are primary-sourced but not from a head-to-head comparison. Several 2026 "state of Wasm" sources giving precise cold-start numbers are low-authority or AI-generated and were not relied on.
- **The 3.5x figure is a single maintainer measurement** reported secondhand, describing a regression under repair — directional, not a stable property.
- **This is partly a judgment call, not a settled fact.** Shopify and Extism demonstrate that a well-built core-module ABI remains viable in production. The decision comes down on components specifically because KNOWME-SEC-AGT-001's static-capability-validation requirement is materially better served by typed WIT worlds.
