---
type: Reference
id: iphone-local-prompt-ui-and-mlx-templateexception-fix-deployed
title: iPhone local prompt UI and MLX TemplateException fix deployed
tags:
- model-runtime-per-device
- ios-device
- mlx
- local-inference
- system-prompt
- template-exception
- accessibility
- litert-lm
links:
- memory-fix-install-watch-armed-for-unplug-safe-ios-verification
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-26T08:46:22.471505+00:00
created_at: 2026-07-26T08:46:22.471505+00:00
updated_at: 2026-07-26T08:46:22.471505+00:00
revision: 0
---

## Phase context

- Phase: `model-runtime-per-device`
- Project: `KnowMe`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system`
- Captured: `2026-07-26T08:37:08Z`
- Position: `model-runtime-per-device › litertlm-device-proof`
- Status: `executing`
- Progress: changes `4/5`

## Phase goals

- Ship `Qwen3.5-9B Q4_K_M` with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM `Gemma 4 E2B`, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Deployed state

A fresh `Runner.app` was installed and launched on the iPhone 17 Pro Max:

- Bundle ID: `tools.know-me.knowme`
- App size: `213.4MB`
- Launch status: stable
- PID: `27025`
- Bundle path: new install path beginning `704E631A…`, distinct from the earlier `A4887FF6…` path
- Deployment lineage continues the iOS model-runtime install/verification path after [Memory-fix install watch armed for unplug-safe iOS verification](/memory-fix-install-watch-armed-for-unplug-safe-ios-verification.md).

Included changes:

- MLX `TemplateException` fix: commit `82d0c69`
- Editable local system prompt UI: commit `ff7d800`
- New UI location: **Settings → Local models**
  - Adds a **Local system prompt** card below the model card.
  - Provides a text area, **Save prompt**, and **Reset to default**.
  - Saving an edited prompt should display `Local system prompt saved.`
  - Clearing the field and saving restores the shipped default instead of sending an empty prompt.

## MLX TemplateException root cause and fix

Failure mode:

- Existing Local-mode conversations could fail with `MLX generation: TemplateException(...)`.
- Fresh chats did not reproduce because they had too little history to trigger context compaction.

Root cause:

- Three context slices could inject `System` messages after conversation history had already begun:
  - `[… N earlier messages omitted …]` marker
  - Conversation summary
  - Memory/knowledge blocks
- `assemble` places the real system prompt at index `0`.
- Any later `System` message violates the MLX chat template, which rejects non-leading system messages.
- The bug therefore mainly affected existing conversations with enough history/context to compact.

Fix:

- Centrally fold all non-leading `System` messages into the leading system prompt after all context slices contribute.
- This preserves real context instead of dropping it.
- The centralized approach avoids teaching each of the three context producers about placement rules, reducing regression risk.

## Verification status

Automated coverage is green for the changed areas, but device-level behavior still needs operator confirmation:

- Context tests: `4` green
- Settings tests: `4` green
- Rust tests: `81` green
- Flutter suite: `35` pass / `3` pre-existing failures
- App launch: clean on iPhone 17 Pro Max
- Device verification: not yet complete

Required operator checks:

1. Retry the previously failing existing conversation:
   - Use the same thread.
   - Switch/use Local mode.
   - Ask a prompt that previously failed.
   - Expected result: answer is generated instead of `MLX generation: TemplateException(...)`.
2. Open **Settings → Local models**:
   - Confirm the **Local system prompt** card appears below the model card.
   - Edit and save the prompt.
   - Confirm `Local system prompt saved.` appears.
   - Clear the field and save.
   - Confirm the shipped default is restored rather than an empty prompt being stored.

`devicectl` can install, launch, and list processes, but cannot drive the UI or read the required on-screen result, so these checks require manual taps.

## Accessibility fixes

The accessibility gate caught two bugs in the newly added prompt UI:

- Text area had no accessible name; placeholder text was insufficient because it disappears when typing.
- Buttons were unnamed.

Both accessibility issues were fixed before deployment.

## Remaining blockers and risks

- Durable persistence is likely still absent:
  - The persistence session landed no commits.
  - `git log` shows no persistence changes.
  - `InMemoryProvider` at `rust/crates/gen_ui_agent/src/uar.rs:878` likely remains.
  - Per-conversation mode may still reset on app restart.
- Change 5 remains blocked:
  - Samsung device is still `adb unauthorized`.
  - Android certification suite cannot run until USB debugging authorization is accepted.
- This phase remains at changes `4/5` until iPhone operator verification and Samsung authorization unblock the remaining certification work.

## Next actions

1. Operator retries the failing existing iPhone conversation in Local mode.
2. Operator verifies the new local system prompt editor in **Settings → Local models**.
3. Authorize Samsung USB debugging to unblock Android certification.
4. Decide whether durable persistence must be implemented now or tracked separately from this deployment.

# Citations

1. stdin
2. manual:KnowMe/model-runtime-per-device