---
type: Reference
id: uar-durable-persistence-blocked-by-uninitialized-submodule
title: UAR durable persistence blocked by uninitialized submodule
tags:
- model-runtime-per-device
- universal-agent-runtime
- submodules
- durable-persistence
- surrealdb
- litert-lm
- android-verification
sources:
- stdin
timestamp: 2026-07-26T07:42:38.687053+00:00
created_at: 2026-07-26T07:42:38.687053+00:00
updated_at: 2026-07-26T07:42:38.687053+00:00
revision: 0
---

## Context

Phase: `model-runtime-per-device` for KnowMe. Active goals include:

- iPhone 17 Pro Max local inference with `Qwen3.5-9B Q4_K_M`, matching `mmproj`, and Metal.
- Android local text-agent inference through LiteRT-LM `Gemma 4 E2B`, with CPU as the universal correctness baseline.
- Device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and memory/thermal fallback.
- Higher local generation limits and tool-capable model plumbing for future agent loops.

## Blocker

The `universal-agent-runtime` submodule is not checked out. `rust/vendor/universal-agent-runtime/` is empty, and `git submodule status` shows a leading `-`, meaning the submodule is uninitialized:

```text
-0493da782a5b6a716d3acbdbfa3112524a0a25bf rust/vendor/librefang
-5343f29facea00f9618d5f367c85ebe138af98f0 rust/vendor/universal-agent-runtime
```

This blocks implementation and verification of durable UAR persistence:

- `cargo build` cannot resolve `universal_agent_runtime`.
- `providers/surreal.rs` cannot be inspected, so it is unknown whether the existing Surreal provider can target an app-sandboxed on-device store or whether a mobile-specific adapter is required.
- The `PersistenceLayer` trait surface is unavailable, so implementing an adapter would require guessing at the trait contract.
- On-device verification is unreachable without a compiling build.

Likely cause: the repo is in a new git worktree. Submodules do not auto-populate in new worktrees.

## Required unblock

Initialize the UAR submodule:

```bash
git submodule update --init --recursive rust/vendor/universal-agent-runtime
```

This mutates worktree state, so it requires an operator decision before proceeding.

## Verified findings

- Root cause confirmed in [`rust/crates/gen_ui_agent/src/uar.rs:633`](rust/crates/gen_ui_agent/src/uar.rs:633): `InMemoryProvider` is currently wired as the `PersistenceLayer` in `build_local`.
- The persistence shape contract is constrained by chat policy behavior:
  - `conversation_policy` returns the bare `RunPolicy`.
  - `save_conversation_policy` accepts the bare `RunPolicy`.
  - Storage wrappers must never be read as chat modes.
- Two tests are pinned and should remain behaviorally unchanged:
  - `conversation_chat_mode_is_read_from_the_bare_run_policy` in [`rust/crates/gen_ui_agent/src/chat.rs`](rust/crates/gen_ui_agent/src/chat.rs)
  - [`mobile/test/features/chat/chat_control_policy_shape_test.dart`](mobile/test/features/chat/chat_control_policy_shape_test.dart)

## Design fork after unblock

Once `rust/vendor/universal-agent-runtime` is present:

1. Inspect `PersistenceLayer` to determine the required durable adapter surface.
2. Inspect `providers/surreal.rs` to determine whether it supports an embedded/file-backed app-sandboxed endpoint such as SurrealKV/RocksDB, or assumes a remote WebSocket server.
3. Decide between:
   - reusing the existing `surreal.rs` provider if `surrealdb_core` is already the app-owned embedded durable store, avoiding a second database; or
   - writing a purpose-built mobile persistence adapter if the existing provider assumes a non-mobile/remote topology.

## Verification plan

After implementing durable persistence:

- Keep both pinned policy-shape tests green.
- Run the Samsung Android gate:
  - set chat mode to Local;
  - force-stop the app;
  - relaunch;
  - confirm the trigger still reads Local;
  - send a message and verify it hits LiteRT-LM;
  - confirm logcat has no `Universal Agent Runtime run` fallback marker;
  - verify second-conversation isolation.

# Citations

1. stdin