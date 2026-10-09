# AFC C11 — durable business intent source contract

Source adoption: committed KnowMe system `68f7ab8` merged into the retained AFC
worktree at `d2eb2ae1`. Original source checkout and its dirty vendor/iOS/docs changes
remain untouched. This is a bounded production slice of C11, not phase completion.

## Implemented path

Configured desktop notes/memories writes enter the sync queue from the existing
Tauri entity commands. A single local PostgreSQL transaction persists the entity
projection, tombstone where applicable, and `sync_outbox` intent. New submissions
receive a database UUID; transport callers may provide `idempotency_key` to identify
an IPC retry. The queue always reuses the stored identity across process restarts.

Replay sends `Idempotency-Key` to Forge. Its returned row and local command settlement
commit together; later pending edits are reapplied in sequence. Lost responses are
recovered by repeating the identical request against Forge's stored receipt.
Transient failures remain pending with bounded backoff, including prolonged offline
periods. Authoritative terminal refusals remain inspectable as poison records.

Projection scope includes Forge/Fabric endpoints, tenant, channel, device consumer
and token subject. The decoded subject partitions local cache only; Forge verifies
the token and applies RLS. Changing scope clears the previous read cache while keeping
its command history and preventing replay under the new scope. Resnapshot clears
committed live rows while preserving tombstones and pending intent.

## Provider and migration requirements

- Forge production-readiness source `ed72018` and migration
  `0020_mutation_receipts.sql` are required. Older servers must not be credited as
  receipt-backed merely because they accept the header.
- Local additive migration `0004_sync_outbox.sql` runs through the existing startup
  migrator before sync attachment. Rolling back application code does not remove
  pending intents or reverse already committed server effects.
- An authenticated Forge subject is required by that provider's receipt contract.
  Anonymous demo configuration does not establish business mutation authority.

## Remaining C11 delivery work

The Fabric spine subscription is still an explicit unimplemented read-lane path.
This slice does not establish durable CDC cursor/version ordering, retention-gap
transport signaling, or remote revocation delivery. The legacy shape consumer and
mobile Surreal adapter are not advertised as complete alternatives. UI presentation
of pending/committed/quarantined state and manual poison recovery remain separate
consumer work. Existing application callers do not yet supply stable IPC retry IDs.

No compiler, test, build, packaged operation or certification gate ran for this
increment, per the current production-first instruction. Those results remain pending.
