---
type: Reference
id: karpathy-progress-20260727T011036Z-model-runtime-per-device
title: "Four intents certified on iPhone; MLX cancellation gap found and fixed"
tags:
- karpathy-progress
- model-runtime-per-device
- in-progress
sources:
- conversation:operator-agent
timestamp: 2026-07-27T01:10:36Z
created_at: 2026-07-27T01:10:36Z
updated_at: 2026-07-27T01:10:36Z
revision: 1
---

## Intent

Deploying the four-intent feature to iOS exposed that the Android engine fix did not transfer: MLX never overrode cancel_generation, so the trait default (a no-op in gen_ui_types/src/inference.rs) swallowed every stop and the engine generated to completion. Everything needed was already present and merely unwired — MlxBridge::cancel is documented as 'ask an in-flight generate to stop', the Swift bridge flips a flag its token loop checks, and the resulting event was already mapped to StreamEvent::Cancelled. Added the override plus aligned the wording from 'MLX generation cancelled' to the shared 'Cancelled by user'. Three integration-harness bugs also had to be fixed: the streaming wait fired on a thinking block rather than text; tap on an off-viewport Resume warned and did nothing; and four find.text assertions passed on that same unreachable button.

## Observed state and verification

iPhone 17 Pro Max iOS 27, lane=local confirmed via chatControlProvider, gates 1-5 passed with backend mlx-metal and modelId qwen3.5-4b-mlx-4bit. Stopped mid-stream at 132 chars keeping 138, status 'Cancelled by user'; all four intents present AND ensureVisible-reachable plus the 'still in context' line; Resume produced 2775 chars continuing from the partial. Zero hit-test warnings with hitTestWarningShouldBeFatal enabled. gen_ui_inference --features local-mlx 26/26 including a mock that counts bridge cancel calls. Commits 21e5602, 5b7c1af.

## Decision and lesson

Status: in-progress. Preserve evidence, distinguish compile proof from runtime proof, and do not narrow the active goal.

## Next experiment

Attachment-flow certification and iOS airplane-mode (task 5.5) remain open. Queue-while-running steering stays deferred pending a Rust mid-run injection point.
