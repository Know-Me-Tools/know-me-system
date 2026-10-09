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
An attempt is persisted before network IO; a crash/lost reply remains visibly uncertain until the identical command receipt settles. Conflict/precondition refusals are disclosed separately from other rejections. Rejected optimistic rows are removed and their selected shape is resnapshotted.
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

## Authenticated read lane

Fabric SDK source `8966d6b1fef002a663d98f762955b2d273f8c07d` is declared on the native client. Its real ShapeClient reads Gate-selected catalog shape IDs with the bearer; caller predicates cannot widen selection. Configure `sync.frf.shape_gateway` and `shapes: [{shape_id, table}]` for notes and/or memories. Missing selection remains explicit write-only/offline status.

Additive migration `0005_sync_read_cursors.sql` stores opaque handle/offset pairs. A compare-and-set against the previous pair and row application commit together, rejecting stale concurrent batches without interpreting opaque offsets. Provider must-refetch clears live cache and restarts snapshot; tombstones and pending intents remain. Revoked/expired grants clear all disclosed rows and suspend replay/overlay. In-flight mutation receipts remain durable without repopulating revoked rows. Per-shape status aggregation prevents one live shape hiding another revoked grant.

## Consumer source boundary

KnowMe app source 64b1d0a implements the Surreal3.0.5 adapter, native composition, registered row/write/status/recovery APIs and real scope/freshness/outcome settings UI using this shared queue. System entity IPC and guest wrappers now accept optional commandId for retained submission identities; omitting it preserves the existing new-submission behavior. No declaration here certifies an unrun packaged path.

No compiler, test, build, packaged operation or certification gate ran for this
increment, per the current production-first instruction. Those results remain pending.
