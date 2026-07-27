---
type: Reference
id: knowme-ios-provenance-chip-release-awaiting-device-verification
title: KnowMe iOS provenance chip release awaiting device verification
tags:
- model-runtime-per-device
- ios-release
- provenance-chip
- mlx
- flutter-release
- device-verification
links:
- ios-cert-monitor-holding-for-local-send-with-benign-nw-path-noise
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-24T21:56:04.308037+00:00
created_at: 2026-07-24T21:56:04.308037+00:00
updated_at: 2026-07-24T21:56:04.308037+00:00
revision: 0
---

## Phase context

- Phase: `model-runtime-per-device`
- Project: `KnowMe`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system`
- Captured: `2026-07-24T21:46:33Z`
- Position: `model-runtime-per-device › provenance-chip-fix › device-verify`
- Status: `awaiting_operator`
- Prior position: `model-runtime-per-device › provenance-chip-fix › deployed`
- Device lane: iPhone 17 Pro Max local inference certification

## Phase goals

- Ship `Qwen3.5-9B Q4_K_M` with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM `Gemma 4 E2B`, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Current state

The provenance backfill fix has been deployed in a signed release build and installed on Travis's iPhone. The attached `flutter run` process was stopped after successful install/launch, so the app is running without debugger dependency and the device is safe to unplug.

This advances the iOS device lane beyond the earlier cert-monitor wait states such as [iOS cert monitor holding for Local send with benign nw_path noise](/ios-cert-monitor-holding-for-local-send-with-benign-nw-path-noise.md): the release binary is now installed and needs operator verification of the Orchestrator response provenance chip.

## Completed work

- Provenance backfill fix is unit-certified; tests are green.
- Formatting is clean.
- Progress was recorded.
- Signed release build completed in `460.6s`.
- Release build installed and launched on iPhone `00008150`.
- `flutter run` was detached/stopped after launch.
- Release mode has no debugger dependency; the app continues working after unplugging the device.

## Device verification procedure

Operator should verify the installed release binary rather than a warm handoff:

1. Fully quit KnowMe by swiping it out of the iOS app switcher.
2. Reopen KnowMe.
3. Go to the **Orchestrator** tab.
4. Send any prompt, e.g. `Tell me a joke.`
5. Check the footer chip under the assistant reply.

Expected local/on-device chip:

```text
orchestrator-agent · On-device
```

The chip format is:

```text
agent · provider/model
```

For the on-device MLX provider, the provider/model label is rendered as `On-device`.

## Cloud comparison behavior

If the operator switches to a cloud provider in Settings and sends another prompt, the chip should show the agent plus cloud provider/model, for example:

```text
orchestrator-agent · openai/gpt-4o
```

This allows comparison of agent, provider, and model provenance across assistant bubbles.

## Next actions

After operator confirms the Orchestrator bubble shows `orchestrator-agent · On-device` for the local route, and cloud routing shows the expected provider/model:

- Finalize the UAR submodule change.
- Add the OpenSpec change.
- Add the ADR.
- Bump the KnowMe submodule pointer.
- Commit the resulting changes.

# Citations

1. stdin
2. manual:KnowMe/model-runtime-per-device