---
type: Reference
id: pem-sync-bridge-and-mobile-tier-phase-status-for-pr-17
title: PEM sync bridge and mobile tier phase status for PR 17
tags:
- pem-sync
- hybrid-mobile-architecture
- local-first-sync
- sqlite
- sqlite-vec
- rag-engine
- vault-pairing
- ci-status
links:
- uar-hybrid-app-architecture-phase-status-and-m5a-handoff
sources:
- stdin
- manual:Hybrid Mobile Architecture Skill/pem-sync-bridge-and-mobile-tier
timestamp: 2026-07-27T11:09:12.978963+00:00
created_at: 2026-07-27T11:09:12.978963+00:00
updated_at: 2026-07-27T11:09:12.978963+00:00
revision: 0
---

## Phase context

- Phase: `pem-sync-bridge-and-mobile-tier`
- Project: `Hybrid Mobile Architecture Skill`
- KBD root: `/Users/gqadonis/Projects/hybrid-mobile-architecture-src`
- Captured: `2026-07-27T11:08:22Z`
- PR under watch: `https://github.com/Know-Me-Tools/hybrid-mobile-architecture-skill/pull/17`

## Phase goals

Seeded from `local-first-realtime-sync/reflection.md` → Recommended Next Phase.

### PEM sync bridge

- Implement `prometheusSyncTransport` to bridge scope streams from local-store row changes into PEM entity reactivity.
- Honor PEM `ListQuery` semantics in transports:
  - filters
  - sorts
  - limit
  - cursor
- Retire the pre-`C-104` “fetch everything” note.
- Unify optimism on the single write queue:
  - PEM pending actions should read `_operation_queue` state.
  - Do **not** introduce a parallel JavaScript queue.

### Mobile tier

- Implement SQLite `LocalStore` behind the existing seam.
- Implement `sqlite-vec` `VectorStore` behind the same `VectorStore` trait.
- Preserve 384-dimensional embedding parity.
- Wire Flutter Rust Bridge surface for:
  - `attachSyncScopes`
  - `runOneTimeLoads`
- Connect the Dart bridge to `gen_ui_core`; remove current stubs.
- This mobile persistence direction is consistent with the SQLite + `sqlite-vec` target described in [UAR hybrid app architecture phase status and M5a handoff](/uar-hybrid-app-architecture-phase-status-and-m5a-handoff.md).

### RAG engine UI exposure

- Expose `RagEngine` to UI surfaces through:
  - FFI
  - Tauri commands
  - typed request/response contracts
- Wire chat-feature retrieval, desktop first, through the layer contract:
  - component → hook → store → invoke

### Vault pairing hardening

- Store the Ed25519 device roster in the document.
- Require challenge over the DataChannel before deltas flow.
- Propagate revocations.
- Design reference already exists in `references/sync/peer-crdt.md`.

### Carry-forward only; do not build in this phase

Tracked but upstream-dependent work:

- PES gateway on FRF.
- FRF `SyncService` auth.
- SignalService production signaling replacement.

## PR #17 CI status

Current state at capture time:

| Check | Result |
|---|---|
| Audit gates | ✅ pass |
| Scaffold structure + manifests | ✅ pass |
| validate | ✅ pass |
| Generated workspace compiles | ⏳ running |
| Generated surfaces — analyze + typecheck | ⏳ running |
| build | ⏳ running |

Notes:

- Three checks were green and three were still running.
- `surfaces` is the long pole because it runs the full hybrid scaffold.
- `Generated workspace compiles` had passed on an earlier run against identical Rust code, so a repeat pass was expected.
- `Generated surfaces — analyze + typecheck` remained the real unknown:
  - it had failed twice before;
  - the latest fix specifically targeted that job.

## Next actions

- Continue monitoring PR #17 for:
  - `surfaces`
  - `compile`
  - `build`
- No active intervention until the pending checks resolve.
- Deferred work remains:
  - `mistral.rs` port
  - physical-device certification run

# Citations

1. stdin
2. manual:Hybrid Mobile Architecture Skill/pem-sync-bridge-and-mobile-tier