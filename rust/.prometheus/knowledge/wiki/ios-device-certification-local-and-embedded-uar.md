---
type: Reference
id: ios-device-certification-local-and-embedded-uar
title: iOS device certification — Local and embedded-UAR lanes on real hardware
tags:
- knowme
- model-runtime-per-device
- ios
- device-certification
- uar
- local-inference
- liter-llm
links:
- ios-mlx-switchover-and-swift-bridge
- thinking-split-segmenter-and-prethink-state
- uar-embedded-session-memory-fix
- mobile-embedded-uar-provider-config
sources:
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-25T21:46:00+00:00
created_at: 2026-07-25T21:46:00+00:00
updated_at: 2026-07-25T21:46:00+00:00
revision: 0
---

## Context

- **Project:** KnowMe
- **Phase:** `model-runtime-per-device`
- **Captured:** `2026-07-25`
- **Status:** `verified`

## Intent

Certify on physical hardware (iPhone 17 Pro Max) that both on-device
configurations produce real streamed chat responses — not just raw token
generation — and that the app keeps working detached from the debugger.

## Observed state

A key distinction that shaped the whole certification: **iOS's default lane is
`uar`, not `local`.** `default_lane()` returns `LANE_UAR` whenever an embedded
kernel is registered at boot, so earlier "tokens generate" evidence was almost
certainly the embedded-UAR lane. "Local" only activates when the user taps the
Local tab, which persists `active_lane='local'`. The lane carries **no** FFI
argument — `chatSend` reads the persisted lane server-side, so the tab must be
tapped *before* sending.

Both lanes drive the same iOS `MlxEngine` through the Swift vtable, but they
differ in what runs above it: `LANE_LOCAL` is the direct engine (chat + local
tool-calling, no kernel/RAG/memory/skills); `LANE_UAR` is the embedded kernel
running the seeded orchestrator with the full agent loop.

Runtime tell-apart in logs: Local emits KnowMe's own
ContextAssembly → ModelLoading → Generation sequence; UAR emits "Starting
Universal Agent Runtime run".

## Decision and delta

- Division of labor: a physical iPhone cannot be driven by computer-use, so the
  build/install/log-stream half was automated and the tap/send half was the
  operator's, with certification read from device logs plus operator report.
- Release builds are used for certification (`flutter run --release`), and the
  attached `flutter run` is **detached after install** so the app runs free of
  the debugger and survives unplugging — the actual shipping condition.
- Supporting work certified in the same sequence: the liter-llm fork was updated
  at source rather than patched locally (the operator correctly challenged a
  local-patch approach), regenerating the provider catalog to 297 providers; and
  the AG-UI encoder was consolidated transport-free in the UAR (ADR 0012).

## Verification

Operator-certified on device across successive release builds:

- **Local lane:** streamed response with reasoning rendered as a collapsible
  Thinking block, no bare `</think>`.
- **Embedded-UAR lane:** streamed response through the orchestrator, with the
  provenance chip reporting agent · provider/model.
- **Memory:** facts persist across a force-quit/restart; separate conversations
  do not bleed context.
- **Paste:** restored in the chat composer (shadcn_flutter defaults
  `contextMenuBuilder` to null, which suppresses the platform selection toolbar
  entirely — fixed with the package's own `nativeContextMenuBuilder()`).

## Failure/lesson

Two lessons. First, log monitors must filter to *actionable* markers: an initial
monitor woke repeatedly on benign `nw_path` network churn, which is noise that
looks like activity. Second, certification requires knowing which lane is
actually active — the default being `uar` rather than `local` meant "it
generates tokens" had been certifying a different code path than assumed.

## Next experiment

Certify the same two lanes on Android (LiteRT-LM), where the streaming is
synthesized from a single response rather than truly incremental.
