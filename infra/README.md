<!-- TJ-ARCH-MOB-001 compliant -->
# KnowMe PoC — realtime/sync infrastructure

**The realtime substrate is [flint-realtime-fabric][frf] (FRF), not ElectricSQL.**
See the phase decision-log (2026-07-16, option B) for the full rationale. Short version:
FRF is how this org does realtime, and it is *not* an Electric consumer — `frf-postgres-cdc`
opens its own logical-replication slot and reads `pgoutput` directly, which is the same
Postgres mechanism Electric consumes. Running both would put two CDC paths in contention
for replication slots on one WAL, for no gain.

## Do not hand-roll a second stack

FRF **already ships a maintained `compose.yml`** (gateway + Iggy spine + Keto + flint-gate
+ Postgres configured with `wal_level=logical`). Duplicating it here would guarantee drift.
Bring the fabric up from the FRF checkout:

```sh
# One-time: FRF's gateway is BUILT FROM SOURCE (no published image), so the first
# `up` compiles the workspace. Expect it to be slow once, then cached.
cd $FRF_HOME            # e.g. ~/Projects/prometheus/flint-realtime-fabric
docker compose up -d    # gateway :28080 (HTTP) / :29090 (gRPC), postgres :15432
```

Pinned FRF revision for this PoC (Rule 22/23 — matches the `frf-sdk-rust` git dep pinned
in `rust/Cargo.toml`, so the client and the server speak the same frozen `proto-v1`):

    9ba04ae6ce41be796ae149609414b17a0d0d376b

## What this directory owns

Only the **PoC-specific** database objects layered on top of FRF's Postgres — the
`notes`/`memories` tables, their `CREATE PUBLICATION`, and the channel mapping the CDC
consumer publishes onto. See `knowme-sync.sql` (T3d).

## Boot order invariant

    FRF stack up → migrations (tables exist) → seeds → attach sync
                                                       ^ `attach_sync_shapes`

A shape/channel subscription MUST NOT be attached before its target table exists locally.

[frf]: https://github.com/Prometheus-AGS/flint-realtime-fabric
