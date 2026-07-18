# gen_ui_db::sync — local-first sync

> **Substrate: [flint-realtime-fabric][frf] (FRF). ElectricSQL was dropped 2026-07-16**
> (C-106, user decision — see the phase decision-log). FRF is not an Electric consumer:
> `frf-postgres-cdc` opens its own logical-replication slot and reads `pgoutput` directly
> — the same Postgres mechanism Electric consumes. Running both would put two CDC paths
> in contention for replication slots on one WAL, for no gain.

## The shape of it

```text
              ┌──────────── read path ────────────┐
  Postgres WAL ─pgoutput→ frf-postgres-cdc ─EntityChange→ Iggy spine channel
                                                              │
                                          FrfClient::subscribe(channel, consumer, from)
                                                              ▼
                                        EventEnvelope → RowChange → LocalStore
                                                                      (local DB)

              ┌──────────── write path ───────────┐
  local mutation → WriteQueue (idempotency key, backoff, poison) → ForgeWriteSink
                                                                      │
                                                    forge/Quarry (PostgREST) → Postgres
                                                                      │
                                                  …which the WAL picks up, closing the loop
```

**Writes do NOT go to the spine.** `SpineService::Publish` writes to the Iggy broker only
— there is no spine→Postgres writer anywhere in FRF, and `EntityService` is read-only
(`GetEntity`/`WatchEntity`). CDC is strictly Postgres→spine. A write published to the
spine would fan out to live subscribers and then vanish: never persisted, so absent from
the next re-materialisation and never seen by a device that was offline. It would look
like a working demo while silently losing data. Writes go to Postgres via Quarry; CDC
fans them back out.

Also note `WatchEntity` takes a single `entity_id` and streams **one entity's** changes.
It is not a table feed. The spine channel is the table-level lane.

## Pieces

| Piece | Where | Notes |
|---|---|---|
| `FrfSyncTransport` | `frf_transport.rs` | Read lane + write-queue drain. The `SyncTransport` impl. |
| `PgLocalStore` | `local_store.rs` | `LocalStore` over Postgres (desktop pglite-oxide). Needs `feature = "pg"`. |
| `ForgeWriteSink` | `gen_ui_agent::sync_sink` | `WriteSink` over forge/Quarry. Lives at **L3** — the trait is here (L2) and the client is `gen_ui_client` (L2); siblings must not depend on each other. |
| `WriteQueue` | `write_queue.rs` | Idempotency keys, exponential backoff, poison quarantine. Substrate-agnostic — unchanged by the Electric→FRF pivot. |
| `SyncEngine` | `engine.rs` | **Legacy Electric lane.** Superseded by `FrfSyncTransport`; kept until removal is scheduled. `shapes.rs` (the Electric HTTP shape consumer) is dead weight on the FRF path. |

## Platform status — read this before assuming sync works

| Surface | Read lane | Write lane |
|---|---|---|
| **Desktop** (Tauri) | ✅ live — `PgLocalStore` over pglite-oxide | ✅ forge/Quarry |
| **Mobile** (iOS/Android) | ❌ **not wired** — see below | ✅ (platform-agnostic once a store exists) |
| **Web** (wasm32) | ❌ compiled out entirely | — |

**Mobile is blocked on a design decision, not effort.** Mobile has no Postgres:
`gen_ui_ffi::api::boot::run_migrations` registers embedded SurrealDB as *both* config and
memory backend, because pglite-oxide is structurally unsupported on iOS/Android (no child
processes, no JIT). So `PgLocalStore` cannot serve it, and a `SurrealLocalStore` needs
row-level upsert/truncate against `gen_ui_db_graph` — whose public surface is
**intent-level by explicit design** ("never raw SurrealQL"; the Dart FFI contract depends
on it). That is a contract change for that crate. Tracked as C-106 T5.

**Web**: this module is a documented no-op on `wasm32` (see `mod.rs`); tonic/HTTP-2 does
not build for the browser. FRF's browser story is frf-wasm / Connect-web from the JS side,
not `pglite-sync` — the old Electric web lane that used to be documented here is gone.

## Enabling the read lane

The SDK subscribe-driver sits behind `feature = "frf"`, **off by default**: `frf-sdk-rust`
is a private git dep in a *different GitHub org* (`Prometheus-AGS`) than this repo
(`Know-Me-Tools`), so CI's `GITHUB_TOKEN` cannot fetch it and a lock entry alone breaks
CI. Enabling it needs a cross-org deploy key / PAT — a repo-admin action. See the
workspace `Cargo.toml` for the verified recipe (including `[net] git-fetch-with-cli`).

With `frf` off: the write queue still runs and still persists, so an offline-first client
keeps accepting local writes and replays them once a build with the spine is running.
Only the read lane is inert. Decode/apply (`row_change_from_payload`,
`apply_envelope_payload`) are **feature-independent and always tested** — the untestable
surface is deliberately kept to the SDK call itself.

## Boot order (load-bearing)

    migrations → seeds → attach sync

A change cannot be applied to a table that does not exist yet. `attach_sync_shapes` is
step 3 on both surfaces for that reason.

Sync is **opt-in**: with no `sync.frf` setting in the config DB, desktop logs and runs
local-only rather than failing startup — the fabric is a private service most developers
cannot reach. A *malformed* setting is an error, not a silent fallback.

[frf]: https://github.com/Prometheus-AGS/flint-realtime-fabric
