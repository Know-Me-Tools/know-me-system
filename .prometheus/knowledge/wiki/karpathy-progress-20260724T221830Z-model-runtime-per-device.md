---
type: Reference
id: karpathy-progress-20260724T221830Z-model-runtime-per-device
title: "M5b Rust backbone: embedded conversation-policy surface + control_plane wiring (3-tier model selection)"
tags:
- karpathy-progress
- model-runtime-per-device
- in_progress
sources:
- conversation:operator-agent
timestamp: 2026-07-24T22:18:30Z
created_at: 2026-07-24T22:18:30Z
updated_at: 2026-07-24T22:18:30Z
revision: 1
---

## Intent

Landed the Rust backbone for full 3-tier (global/agent/conversation) model selection on embedded hosts. UAR submodule (46544c2): SDK Runtime gains save/get/delete_conversation_policy + effective_config (feature=embedded), backed by new RunManager::effective_config (resolves agent from stored policy or default, resolves effective policy via the transport-free core, backfills model) + resolve_agent_or_default + backfill_effective_model (the ADR-0014 backfill factored out and shared with start_run_with_policy). Persistence already had save/load/delete_conversation_policy and RunManager already read it at resolution — the gap was purely the SDK admin surface. ADR 0015 + OpenSpec change embedded-conversation-policy-surface (validate --strict). KnowMe (bb9bb9b): UarEmbeddedRuntime facades + control_plane.rs embedded branch for the 4 per-conversation ops (were not_on_embedded). Precedence conv>agent>global verified by test.

## Observed state and verification

UAR: cargo test server-full,in-memory-backend --lib conversation_policy_round_trips_and_effective_config_reflects_the_override -> ok (no policy=registry default; save openai/gpt-4o=override wins; delete=revert). cargo check server-full + -p universal-agent-runtime-sdk green. KnowMe: cargo test -p gen_ui_agent --lib 68/68; cargo check --target aarch64-apple-ios green. fmt clean. Both pushed: UAR b1dda41..46544c2 (Prometheus-AGS), KnowMe 5af117b..bb9bb9b (Know-Me-Tools).

## Decision and lesson

Status: in_progress. Preserve evidence, distinguish compile proof from runtime proof, and do not narrow the active goal.

## Next experiment

M5b.3 UI: shared catalog-driven provider->model picker on React + Flutter wired into global (GlobalRunPolicyPanel/_GlobalPolicyPanel), per-agent (agent editor), per-conversation (ChatPolicyToolbar/chat_policy_toolbar) tiers. Then device verify precedence + build/deploy.
