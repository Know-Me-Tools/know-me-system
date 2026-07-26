---
type: Reference
id: m5a-embedded-uar-policy-handoff-during-model-runtime-phase
title: M5a embedded UAR policy handoff during model runtime phase
tags:
- model-runtime-per-device
- uar
- embedded-runtime
- openspec
- branch-hygiene
- mobile-runtime
- runtime-verification
links:
- model-runtime-per-device-executor-session-completion
- karpathy-progress-20260723T013607Z-model-runtime-per-device
- uar-hybrid-app-architecture-phase-status-and-m5a-handoff
- embedded-uar-provider-routing-and-mobile-parity
sources:
- stdin
timestamp: 2026-07-24T15:52:23.147535+00:00
created_at: 2026-07-24T15:52:23.147535+00:00
updated_at: 2026-07-24T15:52:23.147535+00:00
revision: 0
---

## Phase context

Phase: `model-runtime-per-device`  
Project: KnowMe  
KBD root: `/Users/gqadonis/Projects/know-me/know-me-system`  
Captured: `2026-07-24T15:31:54Z`

## Goals

- Ship `Qwen3.5-9B Q4_K_M` with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM `Gemma 4 E2B`, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Current position

`model-runtime-per-device › uar-embedded-policy › m5a-impl-in-flight`

Status: `knowledge_captured_holding`

The session continues the per-device runtime work tracked in [Model runtime per-device executor session completion](/model-runtime-per-device-executor-session-completion.md) and the prior iOS certification effort in [iOS device cert build + 5 knowledge records](/karpathy-progress-20260723T013607Z-model-runtime-per-device.md). It also depends on embedded UAR architecture work described in [UAR hybrid app architecture phase status and M5a handoff](/uar-hybrid-app-architecture-phase-status-and-m5a-handoff.md) and the mobile parity fixes in [Embedded UAR Provider Routing and Mobile Parity](/embedded-uar-provider-routing-and-mobile-parity.md).

## Completed work

- M5a OpenSpec authored and committed:
  - proposal
  - design
  - specs
  - tasks
- OpenSpec validation passed: `4/4 valid`.
- M5a implementation delegated to a worktree-isolated subagent using a pinned design.
- Session knowledge captured to the karpathy wiki and committed.
- New wiki record is clean, not malformed, and committed at `4c52392`.
- KnowMe working tree remained intact; HEAD is `4c52392`.
- Branch hygiene preserved: no conflict with the in-flight M5a submodule implementation.

## Commits and state

- `26ccde2`: M5a OpenSpec authored and committed.
- `4c52392`: session knowledge captured to wiki; `pk-clean`.

## Deliberate hold

No further consumer work should proceed until the M5a implementation lands. The following areas are blocked because they depend on M5a and would risk conflicts with the active subagent:

- `M5a-knowme`
- `M5b`
- `M7-react`
- UAR submodule work
- chat/settings surfaces that consumers will modify

Rationale: advancing these areas before the delegated M5a subagent finishes would either duplicate the subagent's work or introduce merge conflicts.

## Verification required after M5a lands

When the M5a implementation subagent completes:

1. Verify both cargo configurations:
   - `server-full`
   - default config
2. Run the parity test.
3. Verify ADR `0013`.
4. Run strict OpenSpec validation:

```bash
openspec --strict
```

5. Bump the KnowMe submodule pointer.
6. Commit the submodule pointer update.

## Follow-on execution order

After M5a verification and submodule pointer bump:

1. `M5a-knowme`: wire `control_plane` embedded behavior.
2. `M5b`: implement/verify pickers.
3. `M7-react`: React consumer work.
4. Continue iOS think-split device certification after operator taps on the phone.

## Operational lesson

The session preserved high-value decisions and parity/branch-hygiene lessons without touching active implementation surfaces. This kept M1–M7-core decisions, parity decisions, delegated-failure lessons, and branch-hygiene guidance captured losslessly while avoiding conflicts with the in-flight M5a subagent.

# Citations

1. [1] stdin