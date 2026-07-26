---
type: Reference
id: uar-shell-navigation-needs-visual-smoke-persistence-still-submodule-blocked
title: UAR shell navigation needs visual smoke; persistence still submodule-blocked
tags:
- knowme
- model-runtime-per-device
- uar
- navigation
- visual-verification
- persistence
- submodules
links:
- knowme-model-runtime-per-device-phase-status-and-proof-gaps
- knowme-uar-navigation-hook-is-stale-persistence-task-still-blocked
- knowme-uar-persistence-task-remains-blocked-by-submodule-checkout
- uar-durable-persistence-blocked-by-uninitialized-submodule
sources:
- stdin
timestamp: 2026-07-26T07:47:15.149727+00:00
created_at: 2026-07-26T07:47:15.149727+00:00
updated_at: 2026-07-26T07:47:15.149727+00:00
revision: 0
---

## Context

Phase: `model-runtime-per-device` for KnowMe.

Position marker:

- Position: `model-runtime-per-device › embedded-uar-offline-agents › embed-uar-mobile-offline`
- Status: `implementation_ready`
- Progress: `changes 4/5`
- Last: `uar-ui-shell-navigation` planning complete: 4/4 planning artifacts, implementation 15/17
- Next: run visual smoke for `uar-ui-shell-navigation` tasks 3.3 and 5.3, or unblock submodules to resume `embed-uar-mobile-offline` task 3.1

Phase goals remain aligned with [KnowMe model-runtime-per-device phase status and proof gaps](/knowme-model-runtime-per-device-phase-status-and-proof-gaps.md):

- Ship Qwen3.5-9B Q4_K_M with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM Gemma 4 E2B, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## `uar-ui-shell-navigation` state

`uar-ui-shell-navigation` is fully planned and nearly implemented:

- Proposal exists.
- Specs exist.
- Design exists.
- Tasks exist.
- Planning artifact status: `isComplete: true`.
- Implementation status: 15/17 tasks complete.

Command implication:

- `/opsx:continue` creates the next planning artifact.
- There is no next planning artifact for this change.
- Therefore `/opsx:continue uar-ui-shell-navigation` has nothing valid to create.
- The change was not a candidate for `/opsx:new` because it already exists and is planned.

This confirms the stale-hook diagnosis previously captured in [KnowMe UAR navigation hook is stale; persistence task still blocked](/knowme-uar-navigation-hook-is-stale-persistence-task-still-blocked.md).

## Remaining `uar-ui-shell-navigation` tasks

The two unchecked tasks are manual visual verification only, not code work:

### Task 3.3

Verify narrow-layout stacking:

- Chat page fills the shell `main` column.
- Composer clears the bottom navigation.
- Smoke check at:
  - `320px`
  - `1440px`

### Task 5.3

Manual smoke verification:

- Breakpoints:
  - `320px`
  - `1440px`
- Themes:
  - Dark
  - Light
- Check against the KnowMe shell idiom:
  - `320px`: bottom bar, hamburger, sidebar drawer
  - `1440px`: rail, status card
- Attach notes and screenshots.

These tasks should not be marked done until the app is launched, resized, visually inspected, and evidence is attached.

## Parallel open thread: UAR durable persistence

`embed-uar-mobile-offline` task 3.1 remains open: durable persistence for the embedded UAR mobile offline path. This is the same blocked persistence work tracked in [KnowMe UAR persistence task remains blocked by submodule checkout](/knowme-uar-persistence-task-remains-blocked-by-submodule-checkout.md) and [UAR durable persistence blocked by uninitialized submodule](/uar-durable-persistence-blocked-by-uninitialized-submodule.md).

Current blocker:

- `librefang` submodule remains uninitialized.
- `universal-agent-runtime` submodule remains uninitialized.
- Both show 0 files and leading `-` in submodule status.

Required unblock command:

```bash
git submodule update --init --recursive
```

If the command no-ops, capture and inspect the error output.

Remaining task 3.1 work after submodule checkout is fixed:

- Implement the durable persistence adapter.
- Keep the two pinned-shape tests green.
- Verify the on-device restart behavior on the Samsung, including the bug where chat mode does not survive restart.

## Correct next actions

Choose one:

1. Run the `uar-ui-shell-navigation` visual smoke:
   - Launch the app.
   - Capture `320px` and `1440px` in dark and light themes.
   - Verify composer/bottom-nav clearance and rail/status-card behavior.
   - Attach notes/screenshots.
   - Mark tasks 3.3 and 5.3 complete, closing `uar-ui-shell-navigation` at 17/17.
2. Unblock submodules and resume task 3.1:
   - Run `git submodule update --init --recursive`.
   - Implement durable persistence.
   - Preserve pinned-shape tests.
   - Verify Samsung restart persistence.

## Hook correction

The position footer's stale guidance should not re-emit `/opsx:new` or `/opsx:continue` for `uar-ui-shell-navigation`. The accurate next line is:

```text
Next: run the 3.3/5.3 visual smoke to close uar-ui-shell-navigation, or unblock the submodule to resume embed-uar-mobile-offline task 3.1
```

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
