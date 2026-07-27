---
type: Reference
id: mobile-local-inference-routing-and-uar-policy-persistence-gap
title: Mobile local inference routing and UAR policy persistence gap
tags:
- knowme
- mobile-inference
- uar
- litert-lm
- android
- persistence
- run-policy
links:
- karpathy-progress-20260720T201046Z-embedded-uar-offline-agents
- karpathy-progress-20260719T012735Z-android-xberg-runtime
- karpathy-progress-20260718T164250Z-platform-policy
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-26T07:42:05.465905+00:00
created_at: 2026-07-26T07:42:05.465905+00:00
updated_at: 2026-07-26T07:42:05.465905+00:00
revision: 0
---

## Context

Phase: `model-runtime-per-device` for KnowMe.

Primary goals:

- Ship Qwen3.5-9B Q4_K_M with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM Gemma 4 E2B, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Android local mode certification

Local mode was verified on Samsung `SM-S936U`.

| Check | Result |
|---|---|
| Local selectable | Trigger label changes to `Local` |
| Routes to on-device model | `LiteRT-LM generation starting` → `produced text chars=9`; no `Universal Agent Runtime run` |
| Reply renders | Answer appeared in thread |
| Agent list populated | `Orchestrator` + `Default Assistant` from `metadata.title` |
| Per-conversation isolation | Thread A = Local, thread B = Orchestrator; preserved across switching |
| Accessibility | `selected=true/false` per row, `clickable=true`, 48px targets |

This establishes Android runtime behavior, not only compile success. This distinction is consistent with prior mobile integration lessons in [Correct service-only mobile UAR integration](/karpathy-progress-20260720T201046Z-embedded-uar-offline-agents.md) and [Android Xberg and Vulkan runtime integration](/karpathy-progress-20260719T012735Z-android-xberg-runtime.md).

## Root cause: policy accessor returned storage wrapper

Bug location: `uar.rs::conversation_policy`.

The accessor returned the persisted storage wrapper instead of the bare `RunPolicy` expected by all consumers.

Returned shape:

```json
{
  "conversation_id": "...",
  "policy": {
    "chat_mode": "local"
  },
  "updated_at": "..."
}
```

Expected shape:

```json
{
  "chat_mode": "local"
}
```

Affected consumers expected `chat_mode` at the top level:

- `save_conversation_policy` input type.
- TypeScript binding declared type.
- Flutter policy map.
- React surface.

Because every `RunPolicy` field uses `#[serde(default)]`, feeding the wrapper back into save did not fail. Instead, deserialization silently produced defaults, so the write succeeded and overwrote the selection with the default mode, `uar`.

This explains why logcat had no useful error and why earlier hypotheses were wrong:

- R8 stripping was not the issue.
- `NoLocalEngine` was not the issue.
- `patch()` clobbering was not the issue.

## Fix

Fixed the accessor so policy read and write are symmetric: `conversation_policy` returns the bare `RunPolicy`, not `ConversationPolicyRecord`.

The React surface already assumed the bare policy shape, so fixing the Rust accessor repaired both React and Flutter-facing surfaces without per-client patches.

Regression tests were added in Rust and Flutter to assert the wrapper shape must not be accepted/read as a mode. This makes the silent-default failure mode visible in future test runs.

## Remaining issue: restart persistence is not implemented

Mode selection still resets after app restart.

Instrumentation verified the policy write path is correct before restart:

```text
saving chat_mode=local
saved chat_mode=local
```

After restart, reading the same conversation id returns no `chat_mode`.

Cause: `rust/crates/gen_ui_agent/src/uar.rs:878` wires the embedded runtime to `InMemoryProvider`, an `RwLock<HashMap>`, as the persistence layer. A source comment states this is deliberate until the durable-adapter milestone.

Therefore policies live only in RAM and die with the process. This is not a regression from the policy-shape fix; it is the still-unimplemented `embed-uar-mobile-offline` task 3.1. It belongs to the same embedded mobile UAR track as [Correct service-only mobile UAR integration](/karpathy-progress-20260720T201046Z-embedded-uar-offline-agents.md).

Required next implementation:

- Replace `InMemoryProvider` with a sandboxed durable adapter.
- Likely reuse the existing Surreal provider.
- Add a verification gate proving local-vs-UAR mode survives app restart.

## Test status

- Rust: `72/72` passing.
- Flutter: `31` passing / `3` failing.
- The `3` Flutter failures are pre-existing baseline failures.
- Temporary diagnostic tracing was reverted after use.

## iOS status

Runtime certification on iOS remains blocked because `xcrun devicectl` reports both iPhones as `unavailable`.

Current evidence:

- Code compiles for `aarch64-apple-ios`.
- No iOS runtime proof yet.

Per the platform policy in [Current-generation mobile and toolchain policy](/karpathy-progress-20260718T164250Z-platform-policy.md), compile proof must not be treated as physical-device certification.

## Current position

Prometheus position:

- Position: `model-runtime-per-device › desktop-stable-port › desktop-stable-port`
- Status: `executing`
- Progress: `changes 4/5`
- Last verified result: local mode certified on Samsung `SM-S936U`; routing reaches LiteRT-LM and bypasses UAR; per-conversation isolation works.
- Remaining work:
  - Durable embedded persistence adapter for restart survival.
  - iOS device certification after iPhone reattachment.

# Citations

1. stdin
2. manual:KnowMe/model-runtime-per-device