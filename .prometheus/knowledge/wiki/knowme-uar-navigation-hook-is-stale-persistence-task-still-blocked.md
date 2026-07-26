---
type: Reference
id: knowme-uar-navigation-hook-is-stale-persistence-task-still-blocked
title: KnowMe UAR navigation hook is stale; persistence task still blocked
tags:
- knowme
- model-runtime-per-device
- uar
- openspec
- submodules
- persistence
- navigation
links:
- knowme-model-runtime-per-device-phase-status-and-proof-gaps
- knowme-uar-persistence-task-remains-blocked-by-submodule-checkout
- uar-durable-persistence-blocked-by-uninitialized-submodule
sources:
- stdin
timestamp: 2026-07-26T07:46:30.163490+00:00
created_at: 2026-07-26T07:46:30.163490+00:00
updated_at: 2026-07-26T07:46:30.163490+00:00
revision: 0
---

## Context

Phase: `model-runtime-per-device` for KnowMe.

Position marker:

- Position: `model-runtime-per-device › embedded-uar-offline-agents › embed-uar-mobile-offline`
- Status: `implementation_ready`
- Progress: `changes 4/5`
- Last: implement the shared offline embedded UAR kernel for desktop, Android, and iOS
- Next marker: `/opsx:continue uar-ui-shell-navigation`

Phase goals remain aligned with [KnowMe model-runtime-per-device phase status and proof gaps](/knowme-model-runtime-per-device-phase-status-and-proof-gaps.md):

- Ship Qwen3.5-9B Q4_K_M with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM Gemma 4 E2B, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Important command correction

`uar-ui-shell-navigation` is **not** a new change. It already has a full artifact set:

- Proposal
- Design
- Tasks
- Specs
- Verification notes

It is nearly complete: **15 of 17 tasks are already done**.

Do **not** run:

```bash
/opsx:new uar-ui-shell-navigation
```

Reason: `/opsx:new` creates a change from scratch. Reusing it for this existing change could error or scaffold over the completed artifact set, destroying completed tasks and verification notes.

Correct continuation command:

```bash
/opsx:continue uar-ui-shell-navigation
```

Before continuing, read the remaining 2 tasks to confirm the intended work.

## Current blocker: embedded UAR durable persistence

Task 3.1 in `embed-uar-mobile-offline` remains unchecked at `openspec/changes/embed-uar-mobile-offline/tasks.md:17`:

- Durable persistence adapter work is untouched.
- The two pinned-shape tests still need to remain green.
- On-device restart verification on Samsung is still required.

This is the same blocked persistence path described in [KnowMe UAR persistence task remains blocked by submodule checkout](/knowme-uar-persistence-task-remains-blocked-by-submodule-checkout.md) and [UAR durable persistence blocked by uninitialized submodule](/uar-durable-persistence-blocked-by-uninitialized-submodule.md).

Re-verification during the session found both submodules still uninitialized:

- `git submodule status` shows leading `-` entries.
- Submodule directories contain 0 files.

Therefore, task 3.1 is still blocked and no durable-persistence implementation can proceed until submodules are initialized.

## Available next actions

1. **Fix the submodule and resume task 3.1**
   
   Run:

   ```bash
   git submodule update --init --recursive
   ```

   If it fails, capture and inspect the error. Likely causes include private-repository authentication failure or running the command outside the intended worktree.

2. **Continue the existing navigation change**

   Read the 2 remaining tasks, then run:

   ```bash
   /opsx:continue uar-ui-shell-navigation
   ```

3. **Start a genuinely new change**

   Only use `/opsx:new` with a different intended change name. Recreating `uar-ui-shell-navigation` requires explicit confirmation because it risks overwriting already-completed work.

## Decision recorded

The stale hook recommendation must not be followed literally. The correct branch is either:

- unblock submodules and resume durable persistence task 3.1, or
- continue the existing `uar-ui-shell-navigation` change.

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
