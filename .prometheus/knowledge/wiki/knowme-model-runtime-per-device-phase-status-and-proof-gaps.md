---
type: Reference
id: knowme-model-runtime-per-device-phase-status-and-proof-gaps
title: KnowMe model-runtime-per-device phase status and proof gaps
tags:
- knowme
- mobile-inference
- litert-lm
- qwen35
- uar
- ios
- android
- phase-status
links:
- mobile-local-inference-routing-and-uar-policy-persistence-gap
- karpathy-progress-20260720T201046Z-embedded-uar-offline-agents
sources:
- stdin
timestamp: 2026-07-26T07:42:58.148043+00:00
created_at: 2026-07-26T07:42:58.148043+00:00
updated_at: 2026-07-26T07:42:58.148043+00:00
revision: 0
---

## Phase scope

Active phase: `model-runtime-per-device` for KnowMe.

Primary goals:

- Ship Qwen3.5-9B Q4_K_M with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM Gemma 4 E2B, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Current status

Build state at capture time: 14 `xcodebuild`/`cargo` processes running.

Active phase progress: `4/5` changes complete (`80%`).

| Change | Status |
|---|---|
| `android-litertlm-runtime` | Done; correction recorded: R8 was stripping the JNI bridge |
| `litertlm-native-adapter` | Done |
| `document-uar-karpathy-knowledge-strategy` | Done |
| `litertlm-local-tools` | Done; certified on Samsung |
| `litertlm-device-proof` | Active; only remaining change |

`litertlm-device-proof` open tasks:

- Runtime diagnostics.
- Physical integration scenarios.
- iOS deployment to iPhone 17 Pro Max.
- Android deployment.
- Evidence recording: model, backend, throughput, and fallback.

## Android LiteRT-LM certification

Local mode was certified on Samsung `SM-S936U`.

Verified behavior:

- Local trigger label applies.
- Local mode routes to LiteRT-LM.
- No Universal Agent Runtime run is invoked in this lane.
- Per-conversation isolation holds.

This complements the detailed Android local-mode evidence in [Mobile local inference routing and UAR policy persistence gap](/mobile-local-inference-routing-and-uar-policy-persistence-gap.md).

## iOS proof target

Next required action: install the iOS release build on iPhone 17 Pro Max device `00008150-00125461363A401C` and certify Local mode.

Required checks mirror the Samsung certification:

- Local label changes correctly.
- Generation routes to on-device MLX rather than UAR.
- Per-conversation isolation holds.
- Evidence is recorded for model/backend/throughput/fallback.

## Embedded UAR offline agents child phase

Child phase tracked by `openspec/changes/embed-uar-mobile-offline`: `8` done / `14` open.

Open work includes:

- `3.1` durable sandboxed persistence.
  - This is the blocker for restart persistence: per-conversation mode resets on restart.
  - Captured implementation detail: `InMemoryProvider` at `uar.rs:878`.
- `3.2`-`3.4` embedded FFI admin operations, offline resource advertisement, and seeded-agent protection.
- `4.1`-`4.4` Flutter repo/provider wiring, event-ordering preservation, diagnostics, and desktop alignment.
- `5.1`-`5.6` dependency graph, cross-compile, desktop certification, Android airplane-mode certification, iOS airplane-mode certification, and evidence recording.

This work is related to the in-process mobile UAR direction described in [Correct service-only mobile UAR integration](/karpathy-progress-20260720T201046Z-embedded-uar-offline-agents.md).

## Other open changes

| Change | Progress / remaining work |
|---|---|
| `qwen35-local-tools` | `0/6` |
| `qwen35-device-proof` | `0/5` |
| `qwen35-mobile-vision` | `5/6`; remaining: verify image understanding on both phones |
| `uar-ui-chat-reskin` | `26/29` |
| `uar-ui-token-convergence` | `20/22` |
| `uar-ui-shell-navigation` | `15/17` |
| `uar-ui-a2ui-retoken` | `16/17` |
| `uar-ui-verify-gates` | Closed: `23/23` |

## Phase-state data hygiene issues

Two tracking-state problems were found:

1. `.kbd-orchestrator/phases/model-runtime-per-device/progress.json` is malformed JSON.
   - `python3 -m json.tool` rejects it.
   - Tooling that reads the file may fail or silently skip it.
2. `childPointer` names `embedded-uar-offline-agents`, but that phase directory has no `progress.json`.
   - The child phase has no tracked state file.

These do not block the build, but they make phase counters untrustworthy as source-of-truth progress data. Fix before the next reflect gate.

## Immediate next steps

1. Wait for the active iOS release build to finish.
2. Install on iPhone 17 Pro Max `00008150-00125461363A401C`.
3. Certify Local mode on iOS using the same behavioral checks that passed on Samsung.
4. Repair malformed `model-runtime-per-device/progress.json`.
5. Add missing child-phase `embedded-uar-offline-agents/progress.json`.
6. Complete `litertlm-device-proof` evidence tasks.

# Citations

1. stdin