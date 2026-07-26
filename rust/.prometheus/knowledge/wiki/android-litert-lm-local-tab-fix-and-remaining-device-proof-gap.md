---
type: Reference
id: android-litert-lm-local-tab-fix-and-remaining-device-proof-gap
title: Android LiteRT-LM Local tab fix and remaining device proof gap
tags:
- knowme
- android
- litert-lm
- local-inference
- uar
- device-certification
- model-runtime-per-device
links:
- ios-local-inference-switchover-to-mlx-swift-via-c-vtable
- ios-certification-of-local-and-embedded-uar-lanes-on-hardware
sources:
- stdin
timestamp: 2026-07-26T00:28:12.609777+00:00
created_at: 2026-07-26T00:28:12.609777+00:00
updated_at: 2026-07-26T00:28:12.609777+00:00
revision: 0
---

## Context

- **Project:** KnowMe
- **Phase:** `model-runtime-per-device`
- **Captured:** `2026-07-25T23:14:58Z`
- **Device under test:** Samsung Android device
- **Runtime:** LiteRT-LM Gemma 4 E2B, CPU as universal correctness baseline
- **App state:** fresh install/run confirmed, PID `25404`, PID-scoped monitor `b4fzeospo` armed

## Phase goals

- Ship Qwen3.5-9B Q4_K_M with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM Gemma 4 E2B with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

> Note: later iOS work formally moved away from the original iPhone Qwen3.5-9B target because of jetsam limits; see [iOS local inference switchover to MLX-Swift via C vtable](/ios-local-inference-switchover-to-mlx-swift-via-c-vtable.md).

## Android Local tab fix

Two fixes were installed on the Samsung and the app was relaunched fresh. The specific bug was in the brand-new-chat path:

- Empty `threadId` caused the active-lane policy write to become a no-op.
- Because the write failed, tapping **Local** before sending in a new chat did not move/select the tab.

### Required operator verification

1. **Existing conversation**
   - Tap **Local**.
   - Expected: Local selects immediately and stays selected.

2. **Brand-new chat**
   - Start a new conversation.
   - Tap **Local** before sending anything.
   - Expected: Local selects; this is the previously broken empty-`threadId` case.

3. **Send from Local**
   - Send a message while Local is selected.
   - Expected log marker: `send_local`.
   - This exercises the `send_local` path containing the tool-shortlist change.
   - Logs should be distinct from the UAR lane marker: `Starting Universal Agent Runtime run`.

## Runtime lane semantics

- **Local lane:** direct engine path for chat plus local tool-calling.
- **Orchestrator/UAR lane:** full agent loop with RAG, memory, and skills.
- Both lanes drive the same LiteRT-LM engine on Android.

This mirrors the lane distinction certified on iOS hardware in [iOS certification of Local and embedded-UAR lanes on hardware](/ios-certification-of-local-and-embedded-uar-lanes-on-hardware.md): the engine can be shared while the runtime above it differs.

## Phase status

| Change | Status |
|---|---|
| `android-litertlm-runtime` | ✅ complete, but was broken until the R8 fix |
| `litertlm-native-adapter` | ✅ complete |
| `litertlm-local-tools` | ✅ complete; certified on device |
| `document-uar-karpathy-knowledge-strategy` | ✅ complete |
| `litertlm-device-proof` | ❌ remaining gap |

Current phase position:

- `model-runtime-per-device › litertlm-device-proof`
- Status: `awaiting_operator`
- Progress: Local tab fix and `progress.json` correction are built, installed, and running on Samsung.
- Honest completion state: **4/5**; `litertlm-device-proof` still requires real evidence.

## Remaining certification: `litertlm-device-proof`

A real device proof run is still required. Evidence to collect:

- Model load time
- Throughput
- Backend used
- Memory state
- Thermal state
- Cancellation behavior
- Markdown rendering
- Mermaid rendering
- Xberg/OCR attachment flow
- One permitted tool call

Several metrics can be captured from logs while the operator drives the UI taps.

## Next action

1. Operator verifies **Local** in an existing conversation.
2. Operator verifies **Local** in a brand-new conversation before first send.
3. Operator sends once from Local to exercise `send_local`.
4. On approval, run structured `litertlm-device-proof` certification and capture the required runtime evidence.

# Citations

1. stdin