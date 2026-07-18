---
type: Reference
id: knowme-poc-wasm-embed-blocking-fix-for-gen-ui-db-graph
title: KnowMe PoC wasm embed_blocking fix for gen_ui_db_graph
tags:
- hybrid-mobile
- knowme-poc
- wasm
- rust-workspace
- gen-ui-db-graph
- embedding
- ci-verification
links:
- knowme-poc-codegen-and-tauri-verification-c-102
- phase-codegen-and-ci-verification-session-status
sources:
- stdin
timestamp: 2026-07-16T10:54:48.532060+00:00
created_at: 2026-07-16T10:54:48.532060+00:00
updated_at: 2026-07-16T10:54:48.532060+00:00
revision: 0
---

## Context

- **Project:** Hybrid Mobile Architecture
- **Phase:** `phase-codegen-and-ci-verification`
- **KBD root:** `~/Projects/hybrid-mobile-architecture-src/.claude/worktrees/pensive-greider-2e206c`
- **Captured:** `2026-07-16T10:45:24Z`
- **Source phase record:** `manual:Hybrid Mobile Architecture/phase-codegen-and-ci-verification`
- **Position:** `phase-codegen-and-ci-verification`
- **Status:** `executing`

This session continues the KnowMe PoC work from [KnowMe PoC Codegen and Tauri Verification C-102](/knowme-poc-codegen-and-tauri-verification-c-102.md) and the broader status tracked in [Phase Codegen and CI Verification Session Status](/phase-codegen-and-ci-verification-session-status.md).

## Revised phase goal

As of `2026-07-15`, the phase deliverable is a **working proof-of-concept application**, not only codegen and CI verification.

The PoC must be built under:

```text
apps/<name>/
```

It must use repository scaffolds and skills, and be based on KnowMe reference documentation in:

```text
docs/reference-app/
```

The PoC should prove the skill package end to end and showcase the broadest practical range of supported capabilities:

- Streaming `ContentBlock` chat
- PEM entity management
- SurrealDB graph-RAG memory
- Local-first sync
- Cross-platform Flutter, Tauri, and web from one Rust core
- Feature subset selected via web research on showcase-app best practices and 2026 on-device AI feasibility

Original codegen and CI goals remain supporting objectives:

- Run real codegen on the PoC:
  - `flutter_rust_bridge_codegen generate`
  - `dart run build_runner build`
  - Full `flutter pub get`
  - Full `pnpm install`
- Confirm pre-codegen warnings clear once generated code and sibling packages exist.
- Resolve or work around the PEM install blocker:

```text
@prometheus-ags/entity-graph-core@workspace:*
```

- Verify the PoC builds and runs on at least one real target per surface:
  - macOS Tauri desktop
  - iOS simulator or Android emulator for Flutter
- Wire CI to run:
  - `cargo clippy --workspace`
  - `audit.sh all`
  - Boundary test suites against the PoC on every push

## Fix applied

`cargo check -p gen_ui_db_graph --target wasm32-unknown-unknown` now passes.

Native linting remains clean:

```bash
cargo clippy -D warnings
```

The fix is confined to:

```text
apps/knowme-poc/rust/crates/gen_ui_db_graph/src/store.rs:110
```

in the main repository checkout:

```text
~/Projects/hybrid-mobile-architecture-src
```

## Implementation detail

`embed_blocking` was changed to use a cfg split:

- **Native variant:** unchanged; still sends the embedder to `gen_ui_runtime::spawn_blocking` so CPU-bound ONNX inference does not run on the async runtime.
- **wasm variant:** calls `self.embedder.embed(&texts)` inline.

Rationale for wasm behavior:

- Browsers do not provide the same blocking thread-pool model.
- Per the crate design documented in `embed.rs`, the wasm embedder is a host-injected JavaScript shim using `transformers.js`, not CPU-bound Rust inference.

All embedding callers already route through `embed_blocking`, including:

- `memory_ingest`
- `memory_search`

Therefore no caller-level changes were needed.

## Verification

Validated commands/results:

```bash
cargo check -p gen_ui_db_graph --target wasm32-unknown-unknown
# passes
```

```bash
cargo clippy -D warnings
# remains clean on native
```

## Follow-up

- This was a standalone pre-existing wasm fix.
- Remaining phase work is step 3:

```text
/kbd-apply 2026-07-15-c103-chat-live-e2e
```

- The fix is currently uncommitted in the main repo checkout and should be folded into the next commit there.

# Citations

1. [1] stdin
