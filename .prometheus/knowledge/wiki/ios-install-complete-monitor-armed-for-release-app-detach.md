---
type: Reference
id: ios-install-complete-monitor-armed-for-release-app-detach
title: iOS install-complete monitor armed for release app detach
tags:
- model-runtime-per-device
- ios-device-cert
- flutter-release
- install-monitor
- provenance-chip
- device-logs
links:
- ios-cert-monitor-holding-for-local-send-with-benign-nw-path-noise
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-24T21:46:45.639882+00:00
created_at: 2026-07-24T21:46:45.639882+00:00
updated_at: 2026-07-24T21:46:45.639882+00:00
revision: 0
---

## Phase context

- Phase: `model-runtime-per-device`
- Project: `KnowMe`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system`
- Captured: `2026-07-24T21:45:42Z`
- Position: `model-runtime-per-device › provenance-chip-fix › installing`
- Status: `executing`
- Device lane: iPhone 17 Pro Max local inference certification

## Phase goals

- Ship `Qwen3.5-9B Q4_K_M` with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM `Gemma 4 E2B`, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Current state

The lane has moved from the prior local-send waiting state represented by [iOS cert monitor holding for Local send with benign nw_path noise](/ios-cert-monitor-holding-for-local-send-with-benign-nw-path-noise.md) into an install-completion wait.

- Install-complete monitor: `ba60wk2qz`
- Monitor state: armed
- Trigger condition: app launch on the physical iPhone device
- Current action: holding for monitor fire

## Next operator action

When Flutter prints the `Flutter run key commands` / VM-service line:

1. Detach `flutter run` rather than stopping the app.
2. Confirm the release app remains alive after debugger disconnect.
3. Re-verify the Orchestrator bubble shows `agent · provider/model`.

## Rationale

Detaching preserves the installed release app process for on-device verification while avoiding debugger lifecycle interference. The Orchestrator bubble check confirms the visible provenance/provider-model plumbing after install and launch.

# Citations

1. stdin
2. manual:KnowMe/model-runtime-per-device