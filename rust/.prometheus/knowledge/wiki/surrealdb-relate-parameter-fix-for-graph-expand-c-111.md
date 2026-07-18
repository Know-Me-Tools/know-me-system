---
type: Reference
id: surrealdb-relate-parameter-fix-for-graph-expand-c-111
title: SurrealDB RELATE parameter fix for graph_expand C-111
tags:
- surrealdb
- graph-rag
- graph-expand
- rust
- hybrid-mobile
- proof-of-concept
- ci-verification
links:
- t8-resume-status-for-hybrid-mobile-poc-codegen-verification
- hybrid-mobile-poc-phase-codegen-and-ci-execution-context
sources:
- stdin
- manual:Hybrid Mobile Architecture/phase-codegen-and-ci-verification
timestamp: 2026-07-17T01:46:05.916443+00:00
created_at: 2026-07-17T01:46:05.916443+00:00
updated_at: 2026-07-17T01:46:05.916443+00:00
revision: 0
---

## Context

- **Phase:** `phase-codegen-and-ci-verification`
- **Project:** Hybrid Mobile Architecture
- **KBD root:** `$REPO_ROOT`
- **Captured:** `2026-07-17T01:42:22Z`
- **Status:** executing

This work is part of the Hybrid Mobile Architecture PoC/codegen verification effort tracked in [T8 Resume Status for Hybrid Mobile PoC Codegen Verification](/t8-resume-status-for-hybrid-mobile-poc-codegen-verification.md) and the broader [Hybrid Mobile PoC Phase Codegen and CI Execution Context](/hybrid-mobile-poc-phase-codegen-and-ci-execution-context.md).

## Verification result

The C-111 `graph_expand` failure is fixed and verified:

```text
test graph_expand_traverses_relate_edges ... ok
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
cargo clippy → clean
```

The `0 ignored` count confirms the regression test executed rather than being skipped.

## Root cause

`relate()` persisted every edge between two junk records whose IDs were literally `$from` and `$to`.

The prior query used SurrealDB chevrons:

```sql
entity:⟨$from⟩
entity:⟨$to⟩
```

In SurrealDB, `⟨...⟩` is escaped-identifier syntax, not parameter interpolation. Therefore `entity:⟨$from⟩` means the record whose ID is the literal text `$from`.

Consequences:

- `entity:a` genuinely had no persisted edges.
- `graph_expand` was correctly reporting no traversal results.
- The traversal query itself was correct and was not changed.

Evidence:

- A live probe showed edges as `in: entity:$from`, `out: entity:$to`.
- `surrealdb-core` parser source confirmed the interpretation:
  - `char.rs:16` lexes `⟨` as a surrounded identifier.
  - `parse_record_id_key` has no `$param` arm.

This resolves the C-111 either/or to: **edges were not persisted to the intended records**.

## Fix

The endpoint expressions were parenthesized so SurrealDB routes them through `parse_relate_expr`'s `(` arm and evaluates `type::record(...)` with parameters:

```sql
RELATE (type::record('entity', $from))->relates_to->(type::record('entity', $to)) SET rel = $rel;
```

No traversal code was changed.

## Git/session notes

- The `store.rs` fix was already swept into concurrent commit `c523e2d` (`vector-only lane + hybrid/vector toggle (T3)`) by another session via shared-file staging.
- Commit `612620c` records the root cause and prevents the chevron form from being reintroduced as a simplification.
- The session committed documentation only and did not push.
- Push was intentionally deferred because another session had active edits on `main.rs` and C-111 `tasks.md`; pushing shared `main` mid-edit was considered disruptive.

## Follow-up: unchecked SurrealDB statement errors

A deeper bug remains deliberately unfixed in this change:

- `relate()` calls `.query().await?` but never checks `Response::take_errors()`.
- SurrealDB reports statement failures per statement.
- As a result, `relate()` can return `Ok` even when a write statement failed.
- `init()` already handles this correctly.

This missing `take_errors()` check allowed the bad edge writes to remain silent for weeks and should be fixed as a separate task rather than bundled with the syntax correction.

## Phase goal reminder

As revised on `2026-07-15`, the phase deliverable is a working proof-of-concept app under `apps/<name>/`, not merely codegen or CI verification. The supporting objectives remain:

- Run the real PoC codegen pipeline:
  - `flutter_rust_bridge_codegen generate`
  - `dart run build_runner build`
  - `flutter pub get`
  - `pnpm install`
- Resolve or work around the PEM install blocker: `@prometheus-ags/entity-graph-core@workspace:*` outside the PEM monorepo.
- Verify at least one target per surface:
  - macOS Tauri desktop
  - iOS simulator or Android emulator for Flutter
- Wire CI for:
  - `cargo clippy --workspace`
  - `audit.sh all`
  - boundary test suites against the PoC on every push

# Citations

1. stdin
2. manual:Hybrid Mobile Architecture/phase-codegen-and-ci-verification
