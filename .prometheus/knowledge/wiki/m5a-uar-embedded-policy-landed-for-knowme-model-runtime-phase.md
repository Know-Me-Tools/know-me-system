---
type: Reference
id: m5a-uar-embedded-policy-landed-for-knowme-model-runtime-phase
title: M5a UAR embedded policy landed for KnowMe model runtime phase
tags:
- model-runtime-per-device
- uar
- embedded-runtime
- runtime-verification
- knowme
- control-plane
- openspec
links:
- m5a-subagent-alive-during-embedded-uar-policy-implementation
- m5a-stall-watch-after-fastembed-fix-in-model-runtime-phase
- m5a-embedded-uar-policy-guidance-after-fastembed-fix
- m5a-subagent-confirmed-alive-despite-bash-classifier-outage
- m5a-finish-delegated-after-killed-embedded-uar-policy-worker
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-24T20:30:45.502240+00:00
created_at: 2026-07-24T20:30:45.502240+00:00
updated_at: 2026-07-24T20:30:45.502240+00:00
revision: 0
---

## Phase context

- Phase: `model-runtime-per-device`
- Project: `KnowMe`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system`
- Captured: `2026-07-24T18:11:12Z`
- Position: `model-runtime-per-device › m5a-knowme › control-plane-embedded`
- Status: `m5a_landed_checkpoint`

## Phase goals

- Ship `Qwen3.5-9B Q4_K_M` with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM `Gemma 4 E2B`, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Completed checkpoint

M5a's blocking cross-repo UAR change is fully landed and verified:

- UAR submodule now supports embedded global-policy resolution.
- UAR SDK now exposes settings and agent admin surfaces for embedded/runtime consumers.
- KnowMe submodule pointer was bumped to `8c87717`.
- `gen_ui_agent` compiles against the bumped UAR pointer.
- Final implementation chain:
  1. `fastembed` fix.
  2. Transport-free policy core.
  3. SDK admin surface.
  4. Six passing tests.
  5. ADR `0013`.
- Gates are green:
  - `server-full`
  - default check
  - formatting
  - `openspec --strict`

This completes the work that was previously in flight across the M5a subagent recovery sequence, including the alive/stall tracking in [M5a subagent alive during embedded UAR policy implementation](/m5a-subagent-alive-during-embedded-uar-policy-implementation.md), [M5a stall watch after fastembed fix in model-runtime phase](/m5a-stall-watch-after-fastembed-fix-in-model-runtime-phase.md), [M5a embedded UAR policy guidance after fastembed fix](/m5a-embedded-uar-policy-guidance-after-fastembed-fix.md), [M5a subagent confirmed alive despite Bash classifier outage](/m5a-subagent-confirmed-alive-despite-bash-classifier-outage.md), and [M5a finish delegated after killed embedded UAR policy worker](/m5a-finish-delegated-after-killed-embedded-uar-policy-worker.md).

## Recovery notes

- Two subagents died during the work:
  - one due to an infrastructure error,
  - one due to user kill.
- Worktree isolation stranded commits on separate branches.
- A pre-existing `fastembed` break blocked `server-full`.
- Recovery actions:
  - fast-forwarded stranded work onto the main branch,
  - fixed `fastembed`,
  - finished tests and ADR directly,
  - avoided further worktree/subagent delegation for the final landing.
- No work was lost.

## Session tally

### KnowMe branch: `feat/ios-mlx-inference`

Committed and verified work includes:

- iOS think-split fix.
- M1 routing driver.
- M2 on-device provider admin.
- M3 always-embedded.
- M4 Flutter settings.
- M6/M6b provenance across Rust, Flutter, and React.
- M7 macOS embedded host.
- Wiki record.
- M5a UAR pointer bump.

### UAR submodule branch: `desktop-stable-port-opsx`

- Full M5a change landed as six commits.

### Verification matrix

- iOS: green.
- Desktop: green.
- React: green.
- Flutter: green.
- Host: green.
- Android: NDK environment only.

## Remaining follow-on work

All remaining items are unblocked consumers of the now-stable SDK surface and are no longer critical path:

1. **M5a-knowme** — implement the embedded `control_plane` branch.
   - Add `EmbeddedLibrary` dispatch branch to `control_plane`'s 14 functions.
   - Delegate to new `UarEmbeddedRuntime` facade methods.
   - Facade methods should call the UAR SDK admin surface.
   - Mirror the prior M2 implementation pattern.
   - This path has been traced and is ready to implement.
2. **M5b** — implement three-tier model pickers over the working resolver.
3. **M7-react** — update macOS React settings to drop the service form.

## External dependency

- iOS think-split device certificate still awaits operator taps.

# Citations

1. stdin
2. manual:KnowMe/model-runtime-per-device