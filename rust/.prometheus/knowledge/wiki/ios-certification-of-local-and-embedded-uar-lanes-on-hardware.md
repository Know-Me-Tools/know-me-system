---
type: Reference
id: ios-certification-of-local-and-embedded-uar-lanes-on-hardware
title: iOS certification of Local and embedded-UAR lanes on hardware
tags:
- knowme
- ios
- device-certification
- uar
- local-inference
- mlx-swift
- model-runtime-per-device
links:
- ios-local-inference-switchover-to-mlx-swift-via-c-vtable
- local-agentic-parity-and-lanes
- qwen3-prethink-segmenter-for-reasoning-stream-splitting
- ios-mlx-switchover-and-qwen3-arch-fix
sources:
- ios-device-certification-local-and-embedded-uar.md
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-25T21:47:50.785837+00:00
created_at: 2026-07-25T21:47:50.785837+00:00
updated_at: 2026-07-25T21:47:50.785837+00:00
revision: 0
---

## Context

- **Project:** KnowMe
- **Phase:** `model-runtime-per-device`
- **Captured:** `2026-07-25`
- **Status:** verified
- **Device:** physical iPhone 17 Pro Max

## Intent

Certify that both on-device iOS configurations produce real streamed chat responses, not only raw token generation, and that the app continues working detached from the debugger.

## Lane behavior certified

The critical distinction is that iOS defaults to **`uar`**, not `local`:

- `default_lane()` returns `LANE_UAR` whenever an embedded kernel is registered at boot.
- Earlier evidence that “tokens generate” was likely exercising embedded-UAR, not Local.
- The Local lane activates only after the user taps the **Local** tab, which persists `active_lane='local'`.
- The lane is **not** passed as an FFI argument; `chatSend` reads the persisted lane server-side, so the tab must be tapped before sending.

Both lanes use the same iOS `MlxEngine` through the Swift vtable described in [iOS local inference switchover to MLX-Swift via C vtable](/ios-local-inference-switchover-to-mlx-swift-via-c-vtable.md), but differ in the runtime above it:

- **`LANE_LOCAL`**: direct engine path for chat plus local tool-calling; no kernel, RAG, memory, or skills.
- **`LANE_UAR`**: embedded kernel path running the seeded orchestrator with the full agent loop, as detailed in [Full-agentic local parity + LANE_LOCAL vs LANE_UAR](/local-agentic-parity-and-lanes.md).

Runtime log tell-aparts:

- Local emits KnowMe `ContextAssembly` → `ModelLoading` → `Generation` sequence.
- Embedded-UAR emits `Starting Universal Agent Runtime run`.

## Certification procedure and decisions

- A physical iPhone cannot be driven by computer-use automation, so responsibility was split:
  - Build/install/log-stream automation handled from the development machine.
  - Tap/send interaction handled by the operator.
  - Certification read from device logs plus operator report.
- Certification uses release builds: `flutter run --release`.
- After install, the attached `flutter run` process is detached so the app runs free of the debugger and survives unplugging, matching shipping behavior.

## Verified results

Operator-certified across successive release builds:

- **Local lane** streamed a response with reasoning rendered as a collapsible Thinking block and no bare `</think>` leakage. This validates the reasoning stream split behavior covered by [Qwen3 PreThink segmenter for reasoning stream splitting](/qwen3-prethink-segmenter-for-reasoning-stream-splitting.md).
- **Embedded-UAR lane** streamed through the orchestrator, with the provenance chip reporting agent · provider/model.
- **Memory** persisted facts across force-quit/restart.
- **Conversation isolation** held: separate conversations did not bleed context.
- **Paste in chat composer** was restored. Root cause: `shadcn_flutter` defaults `contextMenuBuilder` to null, suppressing the platform selection toolbar entirely. Fix: use the package-provided `nativeContextMenuBuilder()`.

## Supporting work certified in the same sequence

- The `liter-llm` fork was updated at source instead of patched locally after operator pushback on the local-patch approach.
- Provider catalog was regenerated to **297 providers**.
- The AG-UI encoder was consolidated as transport-free in UAR under ADR 0012.
- The certification sequence followed the iOS MLX/device build fixes summarized in [iOS MLX switchover + Qwen3.5→Qwen3-4B arch fix + build fixes](/ios-mlx-switchover-and-qwen3-arch-fix.md).

## Lessons

- Log monitors must filter for actionable markers. An early monitor woke repeatedly on benign `nw_path` network churn, which appeared active but was only noise.
- Certification must explicitly identify the active lane. Because iOS defaults to `uar`, “it generates tokens” can validate a different path than intended unless the Local tab is selected before sending.

## Next experiment

Certify the same Local and embedded-UAR lanes on Android using LiteRT-LM, where streaming is synthesized from a single response rather than emitted incrementally.

# Citations

1. ios-device-certification-local-and-embedded-uar.md
2. manual:KnowMe/model-runtime-per-device