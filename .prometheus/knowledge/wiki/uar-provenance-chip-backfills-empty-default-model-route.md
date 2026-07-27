---
type: Reference
id: uar-provenance-chip-backfills-empty-default-model-route
title: UAR provenance chip backfills empty default model route
tags:
- model-runtime-per-device
- uar
- provenance-chip
- model-routing
- orchestrator-agent
- on-device-inference
- mlx
links:
- ios-cert-monitor-holding-for-local-send-with-benign-nw-path-noise
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-24T21:37:48.134633+00:00
created_at: 2026-07-24T21:37:48.134633+00:00
updated_at: 2026-07-24T21:37:48.134633+00:00
revision: 0
---

## Context

Phase: `model-runtime-per-device` for KnowMe.

Primary goals:

- Ship `Qwen3.5-9B Q4_K_M` with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM `Gemma 4 E2B`, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Problem

The embedded-UAR assistant footer provenance chip showed only:

```text
orchestrator-agent
```

It did not include the provider/model route, so users could not compare which agent, provider, and model executed a run.

Expected examples:

```text
orchestrator-agent · On-device
orchestrator-agent · openai/gpt-4o
```

## Root cause

The provenance chip reads `model.provider_id` and `model.model_id` from the per-run `effective_run_policy` artifact.

The built-in orchestrator agent intentionally seeds an empty `provider.default`, because it defers default selection to the provider registry. During run start this produced an empty `ModelRoute`, so the `effective_run_policy` artifact shipped blank provider/model fields. UI consumers therefore had no route data and rendered only the agent name.

## Fix

Implemented in one place in the UAR submodule:

- File: `manager.rs`
- Function: `start_run_with_policy`
- Change: after resolving `effective_policy`, backfill `model` from `resolve_default_model()` when the route is missing or empty.

Resolution behavior:

- Embedded builds: resolves to the on-device provider plus MLX model through the registry default wired via `with_provider_registry` and `set_default`.
- Service builds: resolves to `llm_config.model`.

The `effective_run_policy` artifact now reports the true executing route rather than the orchestrator agent's intentionally blank seed route.

## UI impact

No consumer changes are required.

Existing renderers already format `agent · provider/model`:

- Flutter: `_ProvenanceChip`
- React: `provenanceLabel`

Existing label behavior maps `on-device` to `On-device`, so local orchestrator output should display:

```text
orchestrator-agent · On-device
```

Cloud orchestrator output should display routes such as:

```text
orchestrator-agent · openai/gpt-4o
```

## Verification state

- iOS Local + Orchestrator-to-local path was certified by the operator; related certification activity followed the local-send monitor lane documented in [iOS cert monitor holding for Local send with benign nw_path noise](/ios-cert-monitor-holding-for-local-send-with-benign-nw-path-noise.md).
- Tasks `27/28` closed.
- Cert monitor stopped.
- Added embedded test for the route backfill behavior.
- `server-full` check passed once.
- Full test run was still compiling at capture time.
- Active monitor for backfill test result: `ba99odzcc`.

## Follow-up after green tests

1. Run `cargo fmt`.
2. Add OpenSpec change/ADR in UAR.
3. Bump the KnowMe submodule pointer.
4. Commit the fix.
5. Rebuild/redeploy so the operator sees `agent · provider/model` on the Orchestrator bubble.

# Citations

1. stdin
2. manual:KnowMe/model-runtime-per-device