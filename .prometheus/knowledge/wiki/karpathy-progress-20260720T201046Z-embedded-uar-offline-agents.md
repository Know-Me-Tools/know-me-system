---
type: Reference
id: karpathy-progress-20260720T201046Z-embedded-uar-offline-agents
title: "Correct service-only mobile UAR integration"
tags:
- karpathy-progress
- embedded-uar-offline-agents
- in-progress
sources:
- conversation:operator-agent
timestamp: 2026-07-20T20:10:46Z
created_at: 2026-07-20T20:10:46Z
updated_at: 2026-07-20T20:10:46Z
revision: 1
---

## Intent

Make Android and iOS run the Universal Agent Runtime as an in-process Rust library and register the installed local model as the UAR provider so agents remain functional without network access.

## Observed state and verification

Static inspection proves the Android UAR lane calls send_uar_service, requires uar_service_url, and links universal-agent-runtime-sdk with only the http-client feature. UarRuntimeStatus reports ProjectionOnly and embedded_kernel_available=false. A UarLocalInferenceDriverBridge exists but no in-process UAR kernel consumes it.

## Decision and lesson

Status: in-progress. Preserve evidence, distinguish compile proof from runtime proof, and do not narrow the active goal.

## Next experiment

Assess UAR AppState construction and dependency portability, research current Android Rust embedding patterns, then define and implement a mobile-safe EmbeddedRuntime facade with a host-supplied LLM driver.
