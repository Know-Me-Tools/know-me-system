---
type: Reference
id: hybrid-mobile-pem-sync-bridge-phase-and-ci-fixes
title: Hybrid Mobile PEM Sync Bridge Phase and CI Fixes
tags:
- hybrid-mobile
- pem-sync
- mobile-tier
- ci-verification
- flutter
- tauri
- rag-engine
- vault-pairing
links:
- hybrid-mobile-codegen-and-ci-executor-completion
sources:
- stdin
timestamp: 2026-07-27T10:50:05.443552+00:00
created_at: 2026-07-27T10:50:05.443552+00:00
updated_at: 2026-07-27T10:50:05.443552+00:00
revision: 0
---

## Session Metadata

- **Phase:** `pem-sync-bridge-and-mobile-tier`
- **Project:** Hybrid Mobile Architecture Skill
- **KBD root:** `/Users/gqadonis/Projects/hybrid-mobile-architecture-src`
- **Captured:** `2026-07-27T10:48:27Z`
- **Source context:** `manual:Hybrid Mobile Architecture Skill/pem-sync-bridge-and-mobile-tier`
- **Status:** committed and pushed; monitoring PR #17 for complete CI sweep

## Phase Goals

Seeded from `local-first-realtime-sync/reflection.md` recommended next phase.

### PEM sync bridge

- Implement `prometheusSyncTransport` bridging scope streams from local-store row changes into PEM entity reactivity.
- Honor PEM `ListQuery` constraints in transports:
  - filters
  - sorts
  - limit
  - cursor
- Retire the earlier pre-C-104 fetch-everything note.
- Unify optimistic writes on the single existing write queue:
  - PEM pending actions should read `_operation_queue` state.
  - Do **not** introduce a parallel JavaScript queue.

### Real mobile tier

- Add SQLite `LocalStore` implementation behind the existing seam.
- Add `sqlite-vec` `VectorStore` implementation behind the existing `VectorStore` trait.
- Preserve 384-dimensional vector parity.
- Wire the FRB surface for:
  - `attachSyncScopes`
  - `runOneTimeLoads`
- Connect the un-stubbed Dart bridge to `gen_ui_core`.

### RagEngine UI exposure

- Expose `RagEngine` to UIs through:
  - FFI
  - Tauri commands
  - typed request/response objects
- Wire chat-feature retrieval, desktop first, through the layer contract:
  - component → hook → store → invoke

### Vault pairing hardening

- Store an Ed25519 device roster in the document.
- Require a challenge over the DataChannel before deltas flow.
- Propagate revocation.
- Design reference already exists in `references/sync/peer-crdt.md`.

### Explicit carry-forward exclusions

Tracked but upstream-dependent; do not implement in this phase:

- PES gateway on FRF.
- FRF `SyncService` auth.
- Production SignalService signaling swap.

## CI Findings and Fixes

Three commits were added after the PR opened, each addressing real CI failures. The `surfaces` CI job scaffolds a complete hybrid project, so it detected generator-output defects that older `apps/`-snapshot-based CI could not detect. This validates the generator-focused CI direction from the prior [Hybrid Mobile Codegen and CI Executor Completion](/hybrid-mobile-codegen-and-ci-executor-completion.md) work.

### 1. Flutter dependency constraint mismatch

- **Failure:** `flutter pub get` failed outright.
- **Cause:** Phase 1 pinned the app to `riverpod_annotation 4.0.2`, while `scaffold-packages.sh` emitted `^4.0.3`.
- **Why it broke generated apps:** The affected package is a path dependency, so its minimum version was above the app pin and the dependency graph was unsatisfiable.
- **Impact:** Every generated project would have failed before compilation.
- **Resolution:** Align scaffolded package constraints with the app pin.

### 2. Vault feature audit failure

- **Failure:** `audit.sh all` failed on the vault feature.
- **Cause:** Generated vault feature emitted `stores/` and `sync/` but no `hooks/`.
- **Correction history:**
  - First attempted fix used a nonexistent API: `putFact` / `listFacts`.
  - Repository inspection showed the actual API is `agentFacts` / `addAgentFact`.
  - Fix was rewritten against the real API.
- **Verification:** `git diff main..HEAD` showed the branch had not touched vault emission or the audit's vault check, so the gap was pre-existing and newly surfaced by CI.

### 3. TypeScript import mismatch for pgvector support

- **Failure:** `tsc` failed.
- **Cause:** Phase 1 removed `@electric-sql/pglite-pgvector` from `package.json`, but emitted code still imported it.
- **Rationale for restoration:** Although production dropped the dependency for its own reasons, this scaffold emits a 384-dimensional client-RAG surface that needs `@electric-sql/pglite-pgvector`.
- **Resolution:** Restored the dependency from the retained `[sync]` pin.

## Additional Verification Notes

- `Generated workspace compiles` had already passed on the previous run, confirming the Phase 3 inference port builds in CI.
- `npm install` crashes due to an npm `11.16` resolver bug.
  - The crash reproduces identically without the branch changes.
  - It is pre-existing and not on the CI path.
  - `tsc` passes on the installed tree.
  - No fix is claimed for this issue.

## Deployment Catalog Cleanup

- Removed the `knowme-web` build source from the deployment catalog.
- Deliberately did **not** cascade removal into approximately 40 GitOps manifests.
- Reason: those manifests consume the published registry image, not the deleted Dockerfile.
- Decision: preserving a working deployment topology was preferable to tearing it out only to remove a stale build-source reference.

## Deferred Work

- Continue monitoring PR #17 until the full check sweep completes.
- Remaining deferred work:
  - `mistral.rs` port.
  - Physical-device certification run.

# Citations

1. [1] stdin