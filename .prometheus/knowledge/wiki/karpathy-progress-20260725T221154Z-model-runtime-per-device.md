---
type: Reference
id: karpathy-progress-20260725T221154Z-model-runtime-per-device
title: "litertlm-local-tools: shortlist bounded trusted-first; rest of the change was already implemented"
tags:
- karpathy-progress
- model-runtime-per-device
- in_progress
sources:
- conversation:operator-agent
timestamp: 2026-07-25T22:11:54Z
created_at: 2026-07-25T22:11:54Z
updated_at: 2026-07-25T22:11:54Z
revision: 1
---

## Intent

Took up the phase's outstanding 5th change. Audit found four of its five required behaviors ALREADY implemented by engine-agnostic code built during the iOS work: tool-call detection (litert_lm.rs emits ToolCallStarted/Delta/Complete), schema+permission validation and manual execution through the registry (LocalToolCallGate + LocalToolPolicy with trusted_server/auto_execute), streaming tool events and continuation (continue_with_tool_results + tool_result_fallback_applied), and mutating-tool confirmation (local_tool_requires_confirmation, tested). The genuine gap was shortlisting: local_tool_definitions declared EVERY cached tool from EVERY registered server, while the gate already rejected out-of-shortlist calls — its error text even names 'the shortlisted tool set'. Implemented a bounded, trusted-first shortlist: cap at MAX_LOCAL_TOOL_DECLARATIONS=16, stable-sort trusted-local servers ahead of untrusted so the safest tools survive the cap and ordering is deterministic. Engine-agnostic (lives in send_local), so it applies to Android LiteRT-LM and iOS MLX alike.

## Observed state and verification

New test local_tool_shortlist_is_bounded_and_prefers_trusted_servers registers the untrusted server FIRST (so registry order would lose) and asserts the 16-cap holds against 20 advertised tools and all 4 trusted tools precede untrusted ones. cargo test -p gen_ui_agent --lib 69/69 green; cargo fmt --check clean. Pushed 4a4541d..e07002b. Android cross-compile NOT run: aarch64-linux-android-clang absent from this shell (pre-existing NDK env gap, zero errors referencing KnowMe crates).

## Decision and lesson

Status: in_progress. Preserve evidence, distinguish compile proof from runtime proof, and do not narrow the active goal.

## Next experiment

Two items block calling this change DONE: (a) Android NDK cross-compile or a Gradle build to compile-verify the android cfg paths; (b) device verification on a physical Android device — none is currently attached (adb devices empty). NOTE ordering inconsistency in progress.json: litertlm-device-proof is marked completed although the plan lists it as depending on litertlm-local-tools.
