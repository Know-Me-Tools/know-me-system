---
type: Reference
id: three-tier-model-routing-and-response-provenance-in-knowme
title: Three-tier model routing and response provenance in KnowMe
tags:
- knowme
- model-runtime-per-device
- uar
- model-routing
- provenance
- policy
- flutter
- react
links:
- mobile-embedded-uar-provider-config-without-service-url
- mobile-embedded-uar-provider-config-and-parity
- macos-embedded-uar-host
sources:
- three-tier-model-selection-and-provenance.md
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-25T21:46:33.882615+00:00
created_at: 2026-07-25T21:46:33.882615+00:00
updated_at: 2026-07-25T21:46:33.882615+00:00
revision: 0
---

## Context

- **Project:** KnowMe
- **Phase:** `model-runtime-per-device`
- **Captured:** 2026-07-25
- **Status:** verified

## Intent

Implement Cherry-Studio-style model selection with:

1. Global default model route.
2. Per-agent model override.
3. Per-conversation model override.
4. Per-response provenance chip showing which agent, provider, and model produced each assistant response.

The provenance chip enables route comparison directly in conversation history.

## Existing resolver behavior

UAR already provided a four-scope policy resolver:

```text
Global → Agent → Conversation → Turn
```

Relevant policy shape:

```text
RunPolicy.model: ModelRoute
```

UAR also emitted an `effective_run_policy` artifact once per run.

The missing pieces were:

- Embedded execution ignored the **Global** scope.
- Embedded SDK lacked a conversation-policy admin surface.
- UI had no model-selection controls for the three tiers.
- Provenance artifact sometimes carried an empty model route, producing an incomplete chip.

## Decisions and implementation delta

### Shared run-policy resolution

ADR 0013 introduced `resolve_effective_run_policy_core`, a transport-free resolver shared by both service and embedded paths.

Rationale:

- Identical policy inputs must produce identical effective policy regardless of caller.
- Embedded and service runtimes should not diverge in precedence behavior.

The SDK Runtime also gained settings and agent admin surfaces.

### Embedded conversation-policy admin

ADR 0015 added embedded SDK support for:

- `save` conversation policy
- `get` conversation policy
- `delete` conversation policy
- `effective_config` resolution

Persistence already had conversation-policy methods, and `RunManager` already read them. The missing piece was only the public/admin seam.

### Provenance artifact backfill

ADR 0014 fixed the provenance defect.

Built-in agents intentionally seed an empty `provider.default` because they defer to the registry. This prevents pinned built-in model values from going stale. However, that meant the emitted `effective_run_policy` artifact carried a blank `ModelRoute`, so the UI chip could only display the agent id.

Fix:

- If the resolved route is empty, backfill it from `resolve_default_model()`.
- On embedded runtimes, this resolves to the on-device provider.
- On service runtimes, this resolves to configured `llm_config.model`.
- A fully resolved route is never overwritten, preserving policy precedence.

This was a data-layer fix, not a UI workaround.

### UI model-route picker

Added one catalog-driven provider→model cascading picker per framework:

- React: `ModelRoutePicker`
- Flutter: `ModelRoutePicker`

Picker behavior:

- Wired into global, agent, and conversation scopes.
- Uses provider catalog as source of valid provider/model options.
- Renders the on-device provider as **On-device**.
- Provides an inherit option that clears the route.

This work aligns with embedded-provider configuration behavior described in [Mobile embedded UAR provider config without service URL](/mobile-embedded-uar-provider-config-without-service-url.md) and the broader cross-platform parity requirement in [Mobile embedded-UAR provider config (local⊕cloud) + cross-platform parity](/mobile-embedded-uar-provider-config-and-parity.md). macOS embedded parity is tracked in [macOS as an embedded UAR host — cross-platform parity per deployment mode](/macos-embedded-uar-host.md).

## Verification

Passing tests:

- `effective_run_policy_artifact_backfills_the_registry_default_model`
- `conversation_policy_round_trips_and_effective_config_reflects_the_override`
  - override wins
  - delete reverts to inherited/default route

Static checks:

- `tsc` clean
- `dart analyze lib/` clean
- WCAG 2.2 AA gate passed

Device verification:

```text
agent · provider/model
```

The response chip correctly reports agent, provider, and model.

## Lesson

The blank provenance chip looked like a UI defect, but the renderer was already correct. The artifact producer was emitting empty strings three layers below the UI. Tracing the displayed value back to its producer fixed the issue once for every surface.

## Next experiment

Expose the policy provenance map in the chip tooltip:

```rust
BTreeMap<field, PolicyScope>
```

Goal: show which scope won for each field, not only the resulting model route.

# Citations

1. three-tier-model-selection-and-provenance.md
2. manual:KnowMe/model-runtime-per-device