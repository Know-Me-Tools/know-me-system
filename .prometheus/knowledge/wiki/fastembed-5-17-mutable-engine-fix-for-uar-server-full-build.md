---
type: Reference
id: fastembed-5-17-mutable-engine-fix-for-uar-server-full-build
title: FastEmbed 5.17 mutable engine fix for UAR server-full build
tags:
- uar
- fastembed
- server-full
- rust
- embedded-runtime
- model-runtime-per-device
- subagent-coordination
links:
- uar-hybrid-app-architecture-phase-status-and-m5a-handoff
- m5a-subagent-alive-during-embedded-uar-policy-implementation
sources:
- stdin
timestamp: 2026-07-24T16:25:32.939444+00:00
created_at: 2026-07-24T16:25:32.939444+00:00
updated_at: 2026-07-24T16:25:32.939444+00:00
revision: 0
---

## Context

- Phase: `uar-hybrid-app-architecture`
- Project: `universal-agent-runtime`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system/rust/vendor/universal-agent-runtime`
- Captured: `2026-07-24T16:20:36Z`
- Branch: `desktop-stable-port-opsx`
- Commit: `9d7c8d9`
- Position: `model-runtime-per-device › uar-embedded-policy › fastembed-fixed`
- Status: `server_full_green`

This work is part of the broader UAR hybrid-app phase tracked in [UAR hybrid app architecture phase status and M5a handoff](/uar-hybrid-app-architecture-phase-status-and-m5a-handoff.md), while the M5a embedded-policy subagent continues separately as described in [M5a subagent alive during embedded UAR policy implementation](/m5a-subagent-alive-during-embedded-uar-policy-implementation.md).

## Problem

`cargo check --locked --no-default-features --features server-full` was failing after `fastembed` 5.17.3 changed `TextEmbedding::embed` to require `&mut self`.

Failure chain:

```text
E0596: cannot borrow `*engine` as mutable
```

Root cause:

- UAR stored the FastEmbed engine as `Arc<TextEmbedding>`.
- `Arc<T>` only provides shared access to `T`.
- The embedding call was executed inside `spawn_blocking`, which also requires captured values to be `Send + 'static`.
- FastEmbed now needs exclusive mutable access because the engine carries session state.

## Fix

Updated `fastembed.rs` with one-file coordinated changes:

- Changed the engine field from `Arc<TextEmbedding>` to `Arc<std::sync::Mutex<TextEmbedding>>`.
- Updated `ensure_initialized` return type accordingly.
- Wrapped constructed engines with `Arc::new(StdMutex::new(...))`.
- Locked the engine inside `spawn_blocking` immediately before calling `.embed()`.
- Added poison-error handling for the standard mutex lock.

Conceptual pattern:

```rust
use std::sync::{Arc, Mutex as StdMutex};

// Shared engine handle with synchronous interior mutability.
Arc<StdMutex<TextEmbedding>>

// Inside spawn_blocking:
let mut engine = engine.lock().map_err(|err| /* poison handling */)?;
engine.embed(inputs, /* ... */)
```

Rationale:

- `std::sync::Mutex` provides the exclusive `&mut TextEmbedding` required by `embed`.
- The mutex guard is acquired only in the blocking closure around the synchronous embedding call.
- The existing async mutex remains responsible for lazy initialization only.
- `ensure_initialized` is private, so the return-type change does not affect external callers.

## Verification

- `cargo check --locked --no-default-features --features server-full` finished successfully.
- `fastembed.rs` is format-clean.
- No clippy errors were introduced in `fastembed.rs`.
- `ensure_initialized` is private; no downstream caller breakage.
- Change is disjoint from the M5a subagent worktree edits.

## Explicitly out of scope

Two pre-existing repository conditions were surfaced but not fixed:

1. The crate has 141 pre-existing strict clippy `-D warnings` failures, including missing `# Errors` docs and raw-string hash issues. These are separate from the FastEmbed API compatibility fix.
2. Vendored git submodules `liter-llm` and `rust-mcp-filesystem` are empty in the main submodule tree. This is unrelated to `server-full` compilation.

## Next steps

- Wait for the M5a embedded-policy subagent to complete.
- Re-run full checks now that `server-full` compiles.
- Bump the KnowMe submodule pointer to include commit `9d7c8d9` plus the M5a result.
- Continue sequence: `M5a-knowme → M5b → M7-react`.
- iOS think-split device certificate remains blocked on operator taps.

# Citations

1. [1] stdin