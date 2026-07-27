---
type: Reference
id: m5b-model-selection-ui-scoped-after-rust-backbone-lands
title: M5b model selection UI scoped after Rust backbone lands
tags:
- model-runtime-per-device
- m5b
- model-selection
- flutter
- react
- uar
- control-plane
links:
- ios-cert-monitor-awaiting-local-send-after-benign-nw-path-churn
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-25T14:34:17.562677+00:00
created_at: 2026-07-25T14:34:17.562677+00:00
updated_at: 2026-07-25T14:34:17.562677+00:00
revision: 0
---

## Phase context

- Phase: `model-runtime-per-device`
- Project: `KnowMe`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system`
- Captured: `2026-07-25T14:18:08Z`
- Position: `model-runtime-per-device › M5b › M5b.3-UI`
- Status: `executing`

## Phase goals

- Ship `Qwen3.5-9B Q4_K_M` with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM `Gemma 4 E2B`, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Landed work

All Rust backbone work for 3-tier model selection is landed, pushed, and green.

- **M6c provenance backfill**
  - UAR commit: `b1dda41`
  - KnowMe commit: `5af117b`
  - ADR: `0014`
  - Status: device-verified.
- **M5b.1 conversation-policy SDK surface**
  - UAR commit: `46544c2`
  - ADR: `0015`
- **M5b.2 `control_plane` embedded wiring**
  - KnowMe commit: `bb9bb9b`

Confirmed behavior:

- Global, per-agent, and per-conversation model routes resolve on embedded hosts.
- Routes persist correctly.
- Provenance chip reflects the winning route.
- Tests are green.
- Formatting is clean.
- OpenSpec strict validation passes.

## M5b.3 UI scope

M5b.3 is a UI wiring task, not a backend/model-selection primitive task. The provider→model selection primitive already exists on both app surfaces.

### Mobile / Flutter

Existing mobile state and UI already support provider/model selection for the active route:

- `_ProvidersPanel` contains a provider→model selector.
- `snapshot.providers` includes each provider's `models` list.
- `setActiveRoute(providerId, modelId)` already exists.

M5b.3 should reuse this existing provider→model selection behavior for all three policy tiers instead of only the active route:

1. Global setting: `run_policy.global.model`
2. Per-agent model policy
3. Per-conversation toolbar model policy

### React

React has catalog data available through `providerStore`:

- Configured providers
- Per-provider model lists

Required React UI work:

1. Add a model picker to `GlobalRunPolicyPanel`.
2. Upgrade per-agent free-text model inputs to provider/model selection.
3. Add a per-conversation model selector to `ChatPolicyToolbar`.

## Precedence verification target

After React and Flutter UI wiring, build/deploy to iPhone and verify 3-tier precedence end-to-end:

```text
conversation > agent > global
```

This device verification should run in the same iPhone 17 Pro Max local-inference lane tracked by [iOS cert monitor awaiting Local send after benign nw_path churn](/ios-cert-monitor-awaiting-local-send-after-benign-nw-path-churn.md).

## Next actions

- Build React model picker component.
- Wire React global, agent, and conversation tiers.
- Mirror the same 3-tier UI behavior to Flutter using the existing `_ProvidersPanel` provider→model primitive.
- Build and deploy to iPhone.
- Verify route resolution and provenance chip behavior for `conversation > agent > global`.

# Citations

1. stdin
2. manual:KnowMe/model-runtime-per-device