---
type: Reference
id: karpathy-progress-20260725T215956Z-model-runtime-per-device
title: "P2.3 complete: KnowMe consumes the UAR AG-UI encoder (duplicate removed)"
tags:
- karpathy-progress
- model-runtime-per-device
- verified
sources:
- conversation:operator-agent
timestamp: 2026-07-25T21:59:56Z
created_at: 2026-07-25T21:59:56Z
updated_at: 2026-07-25T21:59:56Z
revision: 1
---

## Intent

Removed KnowMe's duplicate NormalizedEvent->AG-UI encoder (embedded_stream_payload, ~149 lines) and delegated to UAR's canonical transport-free to_agui_spec_event + enrich_agui_spec_payload from api/adapters.rs. ADR 0012 had moved that encoder into an ungated module specifically so embedding hosts could reach it; KnowMe carrying a parallel hand-rolled map was the exact drift risk that change existed to eliminate. Ordinal 8 matches the SSE path so live and embedded frames derive identical eventId/sequence. The canonical encoder returns Option (None = no wire frame); those events were already downstream no-ops because normalize_uar_agui_event only acts on recognized types, so the call site skips them. Net -149/+23 lines. This closes the last open item of the iOS/embedded-UAR work stream tracked this session.

## Observed state and verification

cargo test -p gen_ui_agent --lib 68/68 green, including embedded_uar_runs_the_seeded_orchestrator_on_the_local_provider and native_driver_preserves_every_local_semantic_event which exercise this projection path; cargo check --target aarch64-apple-ios green; cargo fmt --check clean. Pushed KnowMe 5d28212..4a4541d.

## Decision and lesson

Status: verified. Preserve evidence, distinguish compile proof from runtime proof, and do not narrow the active goal.

## Next experiment

NOTE: the KBD phase tracker shows 4/5 changes with 'litertlm-local-tools' (Android LiteRT-LM) still active — that change is separate from this session's iOS/embedded-UAR stream and remains outstanding before any phase-level reflection gate.
