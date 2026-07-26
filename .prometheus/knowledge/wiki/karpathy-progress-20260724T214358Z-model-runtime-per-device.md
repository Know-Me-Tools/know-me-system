---
type: Reference
id: karpathy-progress-20260724T214358Z-model-runtime-per-device
title: "Provenance chip: backfill effective_run_policy model route from registry default"
tags:
- karpathy-progress
- model-runtime-per-device
- in_progress
sources:
- conversation:operator-agent
timestamp: 2026-07-24T21:43:58Z
created_at: 2026-07-24T21:43:58Z
updated_at: 2026-07-24T21:43:58Z
revision: 1
---

## Intent

Embedded-UAR assistant bubbles showed only the agent id (orchestrator-agent) with no provider/model. Root cause: the seeded orchestrator/default agent leaves policy.provider.default empty (defers to registry), so the embedded RunManager built an empty ModelRoute and the effective_run_policy artifact shipped blank provider_id/model_id; provenance_from_effective_run_policy then yielded null provider/model and the Flutter/React chip rendered only the agent. Fix in UAR submodule manager.rs start_run_with_policy: after resolving effective_policy, backfill effective_policy.model from RunManager::resolve_default_model() when the route is None or has an empty provider_id/model_id. On embedded that resolves to the on-device provider + MLX model (registry default set via with_provider_registry + set_default in src/embedded.rs); on service builds it falls back to llm_config.model. The artifact now carries the true executing route. Consumer chips unchanged (Flutter _ProvenanceChip, React provenanceLabel already render agent · provider/model, on-device -> 'On-device').

## Observed state and verification

cargo test --no-default-features --features server-full,in-memory-backend --lib effective_run_policy_artifact_backfills_the_registry_default_model -> 1 passed. cargo fmt --check clean. Pre-existing unrelated compile break in tests/test_provider_resolution.rs + siblings (stale VectorMatcher::new 2-arg vs Arc<dyn EmbeddingBackend>+threshold, fastembed refactor) left untouched (out of scope).

## Decision and lesson

Status: in_progress. Preserve evidence, distinguish compile proof from runtime proof, and do not narrow the active goal.

## Next experiment

Release build+deploy to iPhone 00008150 (running); operator device re-verify Orchestrator bubble shows agent · provider/model; then openspec change + ADR in UAR submodule, bump KnowMe submodule pointer, commit.
