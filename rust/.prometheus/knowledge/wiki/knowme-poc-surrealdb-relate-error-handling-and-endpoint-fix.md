---
type: Reference
id: knowme-poc-surrealdb-relate-error-handling-and-endpoint-fix
title: KnowMe PoC SurrealDB RELATE error handling and endpoint fix
tags:
- hybrid-mobile
- knowme-poc
- surrealdb
- graph-rag
- relate-query
- ci-verification
- rust
links:
- t8-resume-status-for-hybrid-mobile-poc-codegen-verification
- hybrid-mobile-poc-phase-codegen-and-ci-execution-context
- hybrid-mobile-poc-phase-goals-and-verification-scope
sources:
- stdin
- manual:Hybrid Mobile Architecture/phase-codegen-and-ci-verification
timestamp: 2026-07-17T01:38:15.126491+00:00
created_at: 2026-07-17T01:38:15.126491+00:00
updated_at: 2026-07-17T01:38:15.126491+00:00
revision: 0
---

## Context

- **Phase:** `phase-codegen-and-ci-verification`
- **Project:** Hybrid Mobile Architecture
- **KBD root:** `$REPO_ROOT`
- **Captured:** `2026-07-17T01:36:51Z`
- **Status:** active implementation/debugging session

This work is part of the Hybrid Mobile Architecture KnowMe proof-of-concept effort, whose revised objective is a working application rather than pipeline verification alone. It continues the phase scope tracked in [T8 Resume Status for Hybrid Mobile PoC Codegen Verification](/t8-resume-status-for-hybrid-mobile-poc-codegen-verification.md), [Hybrid Mobile PoC Phase Codegen and CI Execution Context](/hybrid-mobile-poc-phase-codegen-and-ci-execution-context.md), and [Hybrid Mobile PoC phase goals and verification scope](/hybrid-mobile-poc-phase-goals-and-verification-scope.md).

## Phase objective

As of `2026-07-15`, the deliverable is a working proof-of-concept app under:

```text
apps/<name>/
```

The app must use repository scaffolds and skills, guided by KnowMe reference documentation in:

```text
docs/reference-app/
```

The PoC must prove the skill package end-to-end and demonstrate the broadest practical capability set:

- Streaming `ContentBlock` chat
- PEM entity management
- SurrealDB graph-RAG memory
- Local-first sync
- Cross-platform Flutter, Tauri, and web surfaces from one Rust core
- A feature subset selected using web research on showcase-app best practices and 2026 on-device AI feasibility

Supporting objectives remain:

- Run real codegen on the PoC:
  - `flutter_rust_bridge_codegen generate`
  - `dart run build_runner build`
  - `flutter pub get`
  - `pnpm install`
- Confirm pre-codegen warnings clear once generated code and sibling packages exist.
- Resolve or work around the PEM install blocker:

```text
@prometheus-ags/entity-graph-core@workspace:*
```

- Verify builds/runs on at least one real target per surface:
  - macOS Tauri desktop
  - iOS simulator or Android emulator for Flutter
- Wire CI to run on every push:
  - `cargo clippy --workspace`
  - `audit.sh all`
  - boundary test suites against the PoC

## SurrealDB graph bug finding

The ticket correctly identifies that `relate()` discards its SurrealDB `Response`. Four additional functions in `config.rs` also silently discard responses.

However, the initial causal theory was wrong: a malformed `RELATE` using chevron syntax does not surface a statement error that `take_errors()` could catch.

Problematic site:

```text
apps/knowme-poc/rust/crates/gen_ui_db_graph/src/store.rs:200
```

The existing endpoint form is effectively:

```sql
entity:⟨$from⟩
```

In SurrealDB, `⟨...⟩` is escaped-identifier syntax. Therefore `entity:⟨$from⟩` writes an edge to a record literally keyed `"$from"`; it does not interpolate the bound variable and does not fail. There is no statement error for `take_errors()` to report.

The comment around `store.rs:196-197` claiming that the ID remains bound and is not string interpolation is factually wrong.

## Required fix

The fix must address two separate issues:

1. **Silent response handling**
   - Add a shared `pub(crate)` helper that inspects SurrealDB responses and calls `take_errors()`.
   - Apply the helper across all six silent call sites:
     - `relate()`
     - Four additional functions in `config.rs`
     - One other silent response site identified in the session notes

2. **Incorrect RELATE endpoint construction**
   - Replace chevron escaped-identifier endpoint syntax with parenthesized `type::record` syntax so bound variables are used correctly.

Correct endpoint form:

```sql
(type::record('entity', $from))
```

This change is required because response error checking alone can produce green tests while graph traversal remains broken.

## Test strategy

Planned verification:

- Run a live probe against SurrealDB `3.2` to confirm chevron-vs-parenthesized behavior before changing implementation.
- Remove the temporary probe after verification.
- Un-ignore the currently red traversal test:

```text
graph_expand_traverses_relate_edges
```

- Add an error-surfacing behavior test for invalid statements/endpoints where SurrealDB actually emits response errors.
- Avoid a misleading test that expects malformed chevron `RELATE` endpoints to fail; they succeed by writing the wrong record ID.

## Validation commands

After implementation:

```bash
cargo clippy
cargo test -p gen_ui_db_graph --test it
```

Repository policy also requires committing `.prometheus/` session logs with the fix.

# Citations

1. stdin
2. manual:Hybrid Mobile Architecture/phase-codegen-and-ci-verification
