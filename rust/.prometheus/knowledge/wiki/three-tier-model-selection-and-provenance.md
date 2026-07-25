---
type: Reference
id: three-tier-model-selection-and-provenance
title: Three-tier model selection + per-response provenance
tags:
- knowme
- model-runtime-per-device
- uar
- policy
- model-routing
- provenance
- flutter
- react
links:
- mobile-embedded-uar-provider-config
- macos-embedded-uar-host
- uar-embedded-session-memory-fix
sources:
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-25T21:43:00+00:00
created_at: 2026-07-25T21:43:00+00:00
updated_at: 2026-07-25T21:43:00+00:00
revision: 0
---

## Context

- **Project:** KnowMe
- **Phase:** `model-runtime-per-device`
- **Captured:** `2026-07-25`
- **Status:** `verified`

## Intent

Cherry-Studio-style model selection: a global default, a per-agent override, and
a per-conversation override — plus a chip under every assistant response showing
which agent, provider, and model produced it, so routes can be compared at a
glance.

## Observed state

The resolver already existed. UAR ships a four-scope `resolve_run_policy`
(Global → Agent → Conversation → Turn) with `RunPolicy.model: ModelRoute`, and
emits an `effective_run_policy` artifact once per run. The gaps were: the
embedded path ignored the Global scope; the embedded SDK had no
conversation-policy admin surface; and no UI existed for the model tiers.

## Decision and delta

- **UAR (ADR 0013):** a transport-free `resolve_effective_run_policy_core` shared
  by service and embedded paths, so identical inputs yield identical policy
  regardless of caller; plus a settings/agent admin surface on the SDK Runtime.
- **UAR (ADR 0015):** conversation-policy admin (`save`/`get`/`delete`) and an
  `effective_config` resolver on the embedded SDK. Persistence already had the
  conversation-policy methods and `RunManager` already *read* them — only the
  admin seam was missing.
- **UAR (ADR 0014) — the provenance bug:** built-in agents intentionally seed an
  **empty** `provider.default` (they defer to the registry so a pinned model
  cannot go stale), so the emitted artifact carried a blank `ModelRoute` and the
  chip could only show the agent id. Fixed by backfilling the route from
  `resolve_default_model()` when it resolves empty — on embedded that is the
  on-device provider, on service the configured `llm_config.model`. A fully
  resolved route is never overwritten, so precedence is untouched.
- **UI:** one catalog-driven provider→model cascading picker per framework
  (React `ModelRoutePicker`, Flutter `ModelRoutePicker`), wired into all three
  scopes on both surfaces. The on-device provider renders as "On-device"; the
  inherit option clears the route.

## Verification

`effective_run_policy_artifact_backfills_the_registry_default_model` and
`conversation_policy_round_trips_and_effective_config_reflects_the_override`
(override wins, delete reverts) pass. `tsc` clean, `dart analyze lib/` clean,
WCAG 2.2 AA gate passed. Device-verified: the chip reports `agent ·
provider/model`.

## Failure/lesson

The blank chip looked like a UI defect and was a data defect three layers down —
the consumer components were already correct, and the artifact was shipping
empty strings. Tracing the value to its producer, instead of patching the
renderer, fixed it in one place for every surface at once.

## Next experiment

Expose the `provenance: BTreeMap<field, PolicyScope>` map in the chip tooltip so
users can see *which scope* won, not just the resulting model.
