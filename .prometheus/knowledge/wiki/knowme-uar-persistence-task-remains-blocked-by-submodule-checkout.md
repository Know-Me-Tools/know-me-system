---
type: Reference
id: knowme-uar-persistence-task-remains-blocked-by-submodule-checkout
title: KnowMe UAR persistence task remains blocked by submodule checkout
tags:
- knowme
- model-runtime-per-device
- uar
- persistence
- submodules
- android
- openspec
links:
- knowme-model-runtime-per-device-phase-status-and-proof-gaps
- uar-durable-persistence-blocked-by-uninitialized-submodule
- mobile-local-inference-routing-and-uar-policy-persistence-gap
sources:
- stdin
timestamp: 2026-07-26T07:45:23.042289+00:00
created_at: 2026-07-26T07:45:23.042289+00:00
updated_at: 2026-07-26T07:45:23.042289+00:00
revision: 0
---

## Context

Phase: `model-runtime-per-device` for KnowMe, in position:

`model-runtime-per-device › embedded-uar-offline-agents › embed-uar-mobile-offline`

Status marker:

- Status: `implementation_ready`
- Progress: `changes 4/5`
- Last: implement the shared offline embedded UAR kernel for desktop, Android, and iOS
- Suggested next hook: `/opsx:new uar-ui-shell-navigation`

The phase goals remain the same as tracked in [KnowMe model-runtime-per-device phase status and proof gaps](/knowme-model-runtime-per-device-phase-status-and-proof-gaps.md):

- Ship Qwen3.5-9B Q4_K_M with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM Gemma 4 E2B, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Current blocker

Task 3.1 of `embed-uar-mobile-offline` remains unchecked and unimplemented: durable persistence for the embedded UAR mobile offline path. This is the same persistence gap previously recorded in [UAR durable persistence blocked by uninitialized submodule](/uar-durable-persistence-blocked-by-uninitialized-submodule.md) and related to the policy persistence gap in [Mobile local inference routing and UAR policy persistence gap](/mobile-local-inference-routing-and-uar-policy-persistence-gap.md).

The submodule checkout did not take effect in the active worktree:

- The relevant submodule directory still contains `0` files.
- `git submodule status` still shows a leading `-` on both submodule entries.
- Therefore the durable persistence task remains blocked exactly as before.

Both submodules are uninitialized and likely needed:

- `rust/vendor/universal-agent-runtime`
- `librefang`

Use the unqualified recursive init/update from the active worktree:

```bash
git submodule update --init --recursive
```

Potential failure causes to check:

- Private repository authentication failure.
- Network failure.
- Command was run from the main checkout instead of this worktree. Worktrees have their own submodule working trees; populating the main repo does not populate this worktree.

The command output is needed if it fails or silently no-ops.

## Navigation hook mismatch

The hook suggestion `/opsx:new uar-ui-shell-navigation` would create a new OpenSpec change and advance to the next change (`5/5`). It does **not** complete task 3.1 and would leave the Samsung restart persistence bug open.

Two valid interpretations exist:

1. Park task 3.1 and proceed to the navigation change:
   - Run `/opsx:new uar-ui-shell-navigation`.
   - Durable persistence remains unchecked.
   - Samsung chat-mode restart persistence bug remains live.
2. Retry submodule initialization and return to task 3.1:
   - Run `git submodule update --init --recursive` in this worktree.
   - If errors occur, capture/paste them.
   - Resume implementation after the UAR source is available.

The session did **not** invoke `/opsx:new` because doing so would create a project artifact and move phase state forward on a likely misunderstanding.

## Remaining implementation plan for task 3.1

After submodules are initialized:

1. Read the UAR persistence abstractions:
   - `PersistenceLayer` trait
   - `surreal.rs`
2. Decide whether to reuse existing persistence or write a mobile-safe adapter based on actual trait/runtime constraints.
3. Implement durable persistence in place of the current non-durable adapter.
4. Keep the two pinned-shape tests green.
5. Run on-device Samsung restart verification to prove the chat-mode persistence bug is fixed.

## Required next decision

Choose one:

- `Yes, proceed with the navigation change` — run `/opsx:new uar-ui-shell-navigation`, leaving task 3.1 unchecked.
- `Retry the submodule` — run `git submodule update --init --recursive` and return to task 3.1 once the checkout is populated.

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
