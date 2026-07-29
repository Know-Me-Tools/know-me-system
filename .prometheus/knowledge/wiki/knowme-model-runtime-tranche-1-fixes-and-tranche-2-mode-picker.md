---
type: Reference
id: knowme-model-runtime-tranche-1-fixes-and-tranche-2-mode-picker
title: KnowMe model-runtime tranche 1 fixes and tranche 2 mode picker
tags:
- model-runtime-per-device
- knowme
- desktop-ui
- flutter
- uar
- local-inference
- mode-picker
- accessibility
links:
- m5b-install-monitor-armed-for-model-runtime-picker-verification
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-27T07:47:43.622931+00:00
created_at: 2026-07-27T07:47:43.622931+00:00
updated_at: 2026-07-27T07:47:43.622931+00:00
revision: 0
---

## Phase context

- Phase: `model-runtime-per-device`
- Project: `KnowMe`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system`
- Captured: `2026-07-27T07:46:45Z`
- Source context: `manual:KnowMe/model-runtime-per-device`

## Phase goals

- Ship `Qwen3.5-9B Q4_K_M` with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM `Gemma 4 E2B`, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Tranche 1: assistant thread silent-failure fixes

Commit: `beaffb5` touching `desktop/src/components/assistant-ui/thread.tsx`.

Fixed six silent failures that previously appeared to work because no errors were raised:

- `image` blocks rendered nothing: the adapter emitted a valid part, but the render switch had no image case and hit `default: return null`.
- Replaced the non-exhaustive render switch with a `never` guard to prevent missing block types from silently disappearing.
- Added block coalescing so a streamed turn renders as one bubble instead of N bubbles, matching Flutter behavior.
- Fixed refused cancel hanging on `Cancelling`: both the return value and rejection had been discarded.
- Fixed Flutter correction/steer preamble leaking across conversations and prefixing messages in unrelated threads.
- Guarded Flutter `submitInteraction` so a thrown error no longer leaves the bubble streaming forever.

Validation:

- Desktop tests: `61/61` passing.
- Flutter tests: `40/40` passing.
- Both analyzers clean.
- Flutter guard test was confirmed to fail when the guard was reverted, proving it exercises the fix rather than passing vacuously.

## Tranche 2: unified conversation mode picker

Commit: `0118b5f` touching `desktop/src/features/chat/components/ChatModePicker.tsx`.

### Decision

The separate UAR/on-device switch was deleted. It wrote the lane without the mode, allowing UI and runtime desynchronization: the second row could read `Orchestrator` while inference ran local.

Replacement behavior:

- One trigger chip lists every embedded UAR agent plus `Local`.
- Flutter copy is used verbatim.
- No cloud entry is exposed.
- Selecting `Local` relabels the trigger to `Conversation mode: Local`.

This continues the picker/runtime workstream previously tracked in [M5b install monitor armed for model runtime picker verification](/m5b-install-monitor-armed-for-model-runtime-picker-verification.md).

### Required platform plumbing

Three blockers had to be built before the port could render and switch modes correctly:

1. `lane_availability` had a Rust function and Flutter FFI wrapper, but no desktop path:
   - No Tauri command.
   - No permission entry.
   - No TypeScript binding.
   - Result: React could not render a blocked mode.
2. `switchLane` swallowed all four failure paths, hiding mode-switch failures.
3. Agent label resolution omitted `metadata.title`, causing embedded surfaces to show raw agent IDs.

### Validation

- Vitest: `82/82` passing.
- TypeScript: clean.
- `cargo check`: passing.
- Browser verification completed:
  - Single toolbar visible.
  - Picker opens with expected copy.
  - Selecting `Local` relabels the trigger correctly.

## Known gaps and notes

- **Keyboard activation remains broken**: arrow keys move the highlight, but `Enter` does not select.
  - Verified in a real browser.
  - Tried both `closeOnClick` and routing through Base UI's radio-group contract; neither fixed it.
  - This is a WCAG 2.1.1 failure.
  - Picker is pointer-only until fixed.
  - Gap is recorded in the commit message.
- `desktop/package.json` contains a `lint` script, but no ESLint config exists, so linting cannot run. This is pre-existing.
- Added `.claude/launch.json` to drive the dev server.

## Remaining tranche 2 scope

Additional tranche 2 areas noted but not completed in this session:

- Attachments end-to-end.
- Tool-approval wiring.
- Provenance persistence.
- Artifact sub-kinds.

# Citations

1. stdin
2. manual:KnowMe/model-runtime-per-device