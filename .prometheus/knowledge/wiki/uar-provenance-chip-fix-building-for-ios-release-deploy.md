---
type: Reference
id: uar-provenance-chip-fix-building-for-ios-release-deploy
title: UAR provenance chip fix building for iOS release deploy
tags:
- model-runtime-per-device
- uar
- provenance-chip
- flutter-release
- ios-deploy
- on-device-inference
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-24T21:44:39.703458+00:00
created_at: 2026-07-24T21:44:39.703458+00:00
updated_at: 2026-07-24T21:44:39.703458+00:00
revision: 0
---

## Phase context

- Phase: `model-runtime-per-device`
- Project: `KnowMe`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system`
- Captured: `2026-07-24T21:37:30Z`
- Position: `model-runtime-per-device › provenance-chip-fix › build+deploy`
- Status: `executing`

## Phase goals

- Ship `Qwen3.5-9B Q4_K_M` with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM `Gemma 4 E2B`, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Current state

Two monitors are armed:

- `ba99odzcc`: backfill test monitor.
- `b268frpkb`: release build and install monitor.

The release build is compiling the workspace against the edited `manager.rs`, so the UAR provenance fix is expected to be included in the signed binary deployed to the iPhone.

## Implemented fix

- UAR `manager.rs` now backfills `effective_run_policy.model` from `resolve_default_model`.
- An embedded test was added for the backfill behavior.
- `server-full` check passed once.
- Full test suite is still compiling under monitor `ba99odzcc`.

## Deployment lane

- Release build/install to iPhone device `00008150` is running.
- Build session: `brp4zupgi`.
- Monitor: `b268frpkb`.
- Device is connected and reports `iOS 27.0`.

## Required post-install actions

1. Detach the `flutter run` process after installation so the release app continues running after debugger disconnect.
2. Operator re-verifies the Orchestrator bubble shows `orchestrator-agent · On-device` / `agent · provider/model`.
3. Finalize the UAR change:
   - Run formatting.
   - Add/update OpenSpec and ADR material.
   - Bump the KnowMe submodule pointer.
   - Commit the change.

# Citations

1. stdin
2. manual:KnowMe/model-runtime-per-device