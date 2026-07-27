# PLAN: embedded-uar-offline-agents

Project: KnowMe
Date: 2026-07-20
OpenSpec available: YES
Changes to implement: 1

## Change list

1. `embed-uar-mobile-offline`: Replace UAR's placeholder embedded SDK with a complete in-process agent runtime and inject KnowMe's local Android/iOS providers.
   - Scope: UAR core + SDK | KnowMe Rust core/FFI | Flutter mobile | desktop parity | persistence | build/certification
   - Depends on: NONE
   - Library: `cand-001` adapt, `cand-002` adopt, `cand-003` adopt, `cand-004` adopt
   - Recommended agent: Codex
   - Est. complexity: L
   - Complexity score: High
   - Model class: frontier
   - Customer value: HIGH
   - Details: Implement one transport-independent UAR kernel, direct SDK operations, host LLM provider registration, and durable offline state. Connect Android LiteRT-LM and iOS Metal through one shared UAR driver contract, then prove desktop and both mobile targets without connectivity.

## Execution milestone order

1. UAR kernel: failing tests first, then real builder/direct SDK and dependency repair.
2. Host provider: KnowMe `InferenceProvider` ↔ UAR `LlmDriver` plus local provider/model registration.
3. Persistence/control plane: durable embedded records and direct administration/A2UI APIs.
4. Client routing: mobile embedded-first Flutter repository/FFI path, desktop embedded parity, optional service/BYOK preserved.
5. Certification: UAR/KnowMe tests, Android+iOS cross-compilation, desktop, physical Android, and iOS offline functional evidence.

## Ordering rationale

The embedded runtime must be executable and testable before client routing changes; otherwise the UI would only replace one placeholder with another. Provider injection precedes persistence/client wiring because it proves the central offline agent loop. Platform certification comes last but every preceding milestone includes target checks to expose non-mobile dependencies early.

## Trade-offs and explicit scope cuts

- Mobile offline mode supports in-process/native tools; process-spawn and remote MCP resources remain optional and visibly unavailable without their capabilities.
- The first durable backend is selected by cross-compilation evidence, not by assuming the server's SurrealDB profile is mobile-safe.
- This change does not replace either platform's chosen model engine; it makes both engines first-class UAR providers.

## Command

`/opsx:apply embed-uar-mobile-offline`

PLAN COMPLETE

---

# ADDENDUM (2026-07-27): durable persistence — the operator-directed swap

The 2026-07-20 plan above stands as the phase baseline. This addendum plans the
one gap the 2026-07-27 re-assess actually verified, under the operator's
explicit direction.

## Operator decisions (2026-07-27)

| Question | Decision |
|---|---|
| Where does mobile persistence live? | **Embedded SurrealDB inside UAR.** No in-memory providers for agents — stated as "stupid and unsafe" |
| What about existing in-memory state? | **Do not use in-memory state at all.** Embedded SurrealDB everywhere |
| Is desktop in scope? | **Yes** |

The word "agents" matters: this is not a cache. The embedded runtime persists
agents, skills, policies, settings, memory, knowledge, conversations, run state
and A2UI. On `InMemoryProvider` all of it dies with the process — an agent the
user configured is gone on next launch.

## What the re-assess established

`rust/crates/gen_ui_agent/src/uar.rs:880` builds the runtime with
`providers::memory::InMemoryProvider::new()`. That is the whole defect.

## THE BLOCKER — found while planning, not assumed

`providers::surreal` is behind a **feature gate**:

```
# universal-agent-runtime/src/uar/persistence/providers/mod.rs
#[cfg(feature = "surreal-backend")]
pub mod surreal;
```

and `gen_ui_agent/Cargo.toml:61` currently depends on UAR with:

```
features = ["embedded-mobile", "in-memory-backend"]
```

**`surreal-backend` is not enabled, so `SurrealDbProvider` does not exist in
this build.** Swapping the constructor first would not compile. The feature
must be enabled before any call-site change — that ordering is the plan.

## The fortunate part

`SurrealDbProvider::new` normalizes `rocksdb://` → `surrealkv://`
(`surreal.rs:79-80`), and KnowMe already opens its graph store with
`format!("rocksdb://{}", data_dir.join("memory-db"))`
(`gen_ui_host/src/lib.rs:264`). **The two systems already speak the same
endpoint format**, so the connection string is a known quantity rather than a
research task.

Embedded endpoints skip authentication entirely (`surreal.rs` doc comment), so
no credential handling is needed on device.

## Ordered changes

| # | Change | Depends on |
|---|---|---|
| D1 | Enable `surreal-backend` on the UAR dependency; verify it compiles for host, `aarch64-linux-android`, and `aarch64-apple-ios` | — |
| D2 | Replace `InMemoryProvider` at `uar.rs:880` with `SurrealDbProvider`, given a per-install storage path | D1 |
| D3 | Thread the storage path from both boot paths — `gen_ui_host` AND `gen_ui_ffi::api::boot` | D2 |
| D4 | Prove durability: state survives process death | D3 |

### D1 — the gate, first

Nothing else compiles without it. Cross-compilation is the real risk: SurrealKV
pulls storage-engine crates that must build for both mobile triples. If a
target fails, that is a finding to report, not to work around silently.

### D2 — the swap

`SurrealDbProvider::new` is **async** and fallible. The surrounding builder call
is already async, so `.await?` fits — but the error must surface as a boot
failure the user can see, not a silent fallback to memory. A fallback would
recreate exactly the defect being fixed, while appearing to work.

### D3 — two boot paths, not one

This is the trap the media work already hit once: **mobile boots through
`gen_ui_ffi::api::boot` and never touches `gen_ui_host`.** Wiring only the host
leaves Android and iOS on the broken path while desktop looks fixed. Both must
be threaded.

Use the existing `data_dir` in each, alongside `memory-db` — a sibling
directory, not a shared file. Two SurrealDB instances in one process is a real
resource question; a separate `uar-db` keeps schemas independent, and the
operator can consolidate later if measurement justifies it.

### D4 — durability proof

The unit-testable claim: write state, drop the runtime, rebuild against the same
path, read it back. Beyond that, the same on-device gate the media work used —
kill the app, relaunch, confirm the agent still exists.

## Risks

- **Cross-compilation is the live risk.** SurrealKV for two mobile triples is
  unproven here. D1 exists to find that out before any call-site edit.
- **Binary size and cold start.** A second embedded store is not free. Worth
  measuring against the current APK/IPA rather than assuming.
- **No migration path.** Nothing currently on disk to migrate — in-memory state
  never persisted — so first launch after this starts empty by definition. That
  is the correct behaviour, and should be stated rather than discovered.

## NOT in this addendum

The other 13 tasks in `embed-uar-mobile-offline/tasks.md`. The re-assess found
several describe already-shipped, device-certified work, and the file is
untrustworthy as written. **Re-verify each against code before planning it** —
planning from that list would repeat completed work.
