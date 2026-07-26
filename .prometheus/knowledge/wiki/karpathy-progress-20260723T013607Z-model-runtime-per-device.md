---
type: Reference
id: karpathy-progress-20260723T013607Z-model-runtime-per-device
title: "iOS device cert build + 5 knowledge records"
tags:
- karpathy-progress
- model-runtime-per-device
- in-progress
links:
- ios-mlx-switchover-and-qwen3-arch-fix
- ios-local-embedded-lane-certification
sources:
- conversation:operator-agent
timestamp: 2026-07-23T01:36:07Z
created_at: 2026-07-23T01:36:07Z
updated_at: 2026-07-23T01:36:07Z
revision: 1
---

## Intent

Building signed KnowMe for Travis iPhone 17 Pro Max to certify Local + embedded-UAR lanes. Wrote 5 of 7 karpathy Reference records (iOS MLX switchover+arch fix, think-split segmenter, transport-free AG-UI encoder+ADR0012, local-agentic parity+lanes, WebLLM live demo), indexed + logged + pk-ingested cleanly.

## Observed state and verification

pk search returns all 5 new records with tags; none flagged malformed; index.md + log.md updated. iOS xcodebuild signing with team 48VVJ9AXAR in progress.

## Decision and lesson

Status: in-progress. Preserve evidence, distinguish compile proof from runtime proof, and do not narrow the active goal.

## Next experiment

On build+install, operator taps Local then Orchestrator and sends; certify both from idevicesyslog; write record 7; run prometheus learn --lint + pk lint gate.
