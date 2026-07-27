---
type: Reference
id: uar-durable-persistence-blocked-by-uninitialized-submodule
title: UAR durable persistence blocked by uninitialized submodule
tags:
- knowme
- model-runtime-per-device
- uar
- persistence
- submodules
- litert-lm
- android
links:
- knowme-model-runtime-per-device-phase-status-and-proof-gaps
- mobile-local-inference-routing-and-uar-policy-persistence-gap
sources:
- stdin
timestamp: 2026-07-26T07:44:13.734516+00:00
created_at: 2026-07-26T07:44:13.734516+00:00
updated_at: 2026-07-26T07:44:13.734516+00:00
revision: 0
---

## Context

Phase: `model-runtime-per-device` for KnowMe, under `embedded-uar-offline-agents` / `embed-uar-mobile-offline`.

Relevant phase goals are tracked in [KnowMe model-runtime-per-device phase status and proof gaps](/knowme-model-runtime-per-device-phase-status-and-proof-gaps.md):

- iOS: ship Qwen3.5-9B Q4_K_M with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Android: ship local text-agent inference through LiteRT-LM Gemma 4 E2B, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Current position

- Position: `model-runtime-per-device › embedded-uar-offline-agents › embed-uar-mobile-offline`
- Status: `implementation_ready`
- Progress marker: `changes 4/5`
- Last completed/active item: implement the shared offline embedded UAR kernel for desktop, Android, and iOS.
- Next planned item: `/opsx:new uar-ui-shell-navigation`

## Confirmed root cause

`build_local` currently wires `InMemoryProvider` as the `PersistenceLayer` at:

- `rust/crates/gen_ui_agent/src/uar.rs:633`

The nearby source comment at `uar.rs:623-625` already anticipates replacing this in-memory provider with durable persistence.

This explains the mobile UAR policy persistence gap described in [Mobile local inference routing and UAR policy persistence gap](/mobile-local-inference-routing-and-uar-policy-persistence-gap.md): local-mode policy state is not durable across process restart because the runtime persistence layer is memory-backed.

## Hard blocker

The `universal-agent-runtime` submodule is not checked out:

```text
-0493da782a5b6a716d3acbdbfa3112524a0a25bf rust/vendor/librefang
-5343f29facea00f9618d5f367c85ebe138af98f0 rust/vendor/universal-agent-runtime
```

`rust/vendor/universal-agent-runtime/` is an empty directory. The leading `-` in `git submodule status` indicates the submodule is uninitialized.

This blocks implementation because:

- `providers/surreal.rs` cannot be inspected, so it is unknown whether the existing Surreal provider can point at an app-sandboxed on-device store or assumes a remote WebSocket server.
- The `PersistenceLayer` trait cannot be read, so a mobile-specific adapter cannot be implemented safely.
- `cargo build` cannot resolve `universal_agent_runtime`, preventing compile verification.
- Writing an adapter against an inferred trait would be fabrication and would not satisfy the on-device verification gate.

## Required unblock command

```bash
git submodule update --init --recursive rust/vendor/universal-agent-runtime
```

Likely cause: the repository is being used through a Git worktree, and submodules do not auto-populate in new worktrees.

## Design fork after unblock

Once the submodule is initialized, inspect `providers/surreal.rs` and the `PersistenceLayer` trait to decide between two options:

1. **Reuse the existing Surreal provider** if it can target an app-sandboxed embedded/file-backed store such as `surrealkv://` or RocksDB.
2. **Write a mobile-specific adapter** if the existing provider assumes a remote SurrealDB WebSocket/server endpoint or otherwise cannot operate inside the app sandbox.

The prompt notes `surrealdb_core` appears in boot logs. If that reflects an embedded SurrealKV/RocksDB instance already owned by the app, reusing the existing provider is preferred because it avoids introducing a second on-device database.

## Contract constraints

The chat control policy shape must remain unchanged:

- `conversation_policy` returns the bare `RunPolicy`.
- `save_conversation_policy` accepts the bare `RunPolicy`.
- A storage wrapper must never be interpreted as a chat mode.

Pinned tests to preserve:

- `conversation_chat_mode_is_read_from_the_bare_run_policy` in `rust/crates/gen_ui_agent/src/chat.rs`
- `mobile/test/features/chat/chat_control_policy_shape_test.dart`

## Verification gate after implementation

After durable persistence is implemented:

1. Run the pinned Rust and Dart policy-shape tests.
2. On Samsung Android device:
   - Set chat trigger to `Local`.
   - Force-stop the app.
   - Relaunch.
   - Confirm the trigger still reads `Local`.
   - Send a message.
   - Confirm logcat shows LiteRT-LM local generation and no `Universal Agent Runtime run` remote path.
   - Confirm second-conversation isolation still holds.

Expected local runtime evidence should align with the prior Android certification in [Mobile local inference routing and UAR policy persistence gap](/mobile-local-inference-routing-and-uar-policy-persistence-gap.md): `LiteRT-LM generation starting`, produced text, rendered reply, and no remote UAR run.

# Citations

1. stdin
## Correction (2026-07-26)

The "uninitialized submodule" claim in this note is **wrong** and was retracted the
same session. `rust/vendor/universal-agent-runtime` was fully checked out the whole
time — 15,894 files, clean hashes, `v1.0.0-rc.9-210-g5343f29`. The apparent emptiness
came from a relative-path error in the agent's own shell command (a `cd` from a shell
whose working directory had already changed), not from repo state.

Task 3.1 (durable sandboxed persistence) was therefore **never blocked by the
submodule**. Its real root cause: `rust/crates/gen_ui_agent/Cargo.toml` enables the
`in-memory-backend` feature but not `surreal-backend`, and
`src/uar/persistence/providers/surreal.rs` is `#[cfg(feature = "surreal-backend")]`-
gated — so `InMemoryProvider` was the only provider compiled in.
