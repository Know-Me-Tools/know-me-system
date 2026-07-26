---
type: Reference
id: knowme-model-runtime-per-device-phase-status
title: KnowMe Model Runtime Per Device Phase Status
tags:
- knowme
- model-runtime
- uar
- submodule-pointer
- gen-ui-agent
- mobile-inference
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-24T18:11:29.088803+00:00
created_at: 2026-07-24T18:11:29.088803+00:00
updated_at: 2026-07-24T18:11:29.088803+00:00
revision: 0
---

## Phase goals

- Ship `Qwen3.5-9B Q4_K_M` with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through `LiteRT-LM Gemma 4 E2B`, with CPU as the universal correctness baseline.
- Enable:
  - Device-aware model manifests
  - Resumable downloads
  - Integrity checks
  - Runtime gate diagnostics
  - Safe memory and thermal fallback
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Current position

- Phase: `model-runtime-per-device`
- Project: `KnowMe`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system`
- Position: `model-runtime-per-device › uar-embedded-policy › m5a-pointer-verify`
- Status: `verifying_knowme_build`
- Captured at: `2026-07-24T18:05:08Z`

## M5a UAR submodule status

M5a is complete and committed on branch `desktop-stable-port-opsx`.

Commit chain:

```text
26ccde2 → 9d7c8d9 → 4b5c0da → b60d627 → 554b219 → 58e85e6
```

Completed scope:

- Transport-free policy core
- Embedded global-scope resolution
- SDK admin surface
- ADR `0013`
- Formatting finishers

Verification status:

- 6/6 tests passing
- `server-full` green
- default gates green
- `fmt` green
- `openspec` green

## KnowMe pointer bump

The KnowMe UAR submodule pointer bump is staged:

```text
b5d66de → 58e85e6
```

The pointer is not committed yet. `gen_ui_agent` is being verified against the new UAR SDK/resolver before committing the pointer bump because M5a changed the SDK and resolver surfaces consumed by KnowMe.

Build monitor:

```text
bcyr2p6ky
```

Commit criteria:

1. `gen_ui_agent` build passes against the staged UAR pointer.
2. Commit the KnowMe submodule pointer bump.

## Recovery notes

The M5a push encountered multiple disruptions:

- Two subagents died due to infrastructure/kill events.
- Worktree isolation stranded commits.
- A pre-existing `fastembed` break blocked `server-full`.

Recovery actions:

- Consolidated stranded work through fast-forward.
- Fixed the `fastembed` break directly.
- Finished tests and ADR work in the main tree.
- No work was lost.

## Next sequence

After `gen_ui_agent` verifies green:

1. Commit the KnowMe submodule pointer bump.
2. Continue `M5a-knowme`:
   - Implement/control the `control_plane` embedded branch now that the SDK admin surface exists.
3. Continue `M5b`:
   - Implement 3-tier model pickers.
4. Continue `M7-react`:
   - macOS settings work.

## External dependency

The iOS think-split device certificate is still awaiting operator taps.

# Citations

1. stdin
2. manual:KnowMe/model-runtime-per-device