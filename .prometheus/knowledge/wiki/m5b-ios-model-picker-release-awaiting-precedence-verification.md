---
type: Reference
id: m5b-ios-model-picker-release-awaiting-precedence-verification
title: M5b iOS model picker release awaiting precedence verification
tags:
- model-runtime-per-device
- ios-device
- model-picker
- flutter-release
- run-policy
- provenance-chip
- qwen
- litert-lm
links:
- m5b-install-monitor-armed-for-model-runtime-picker-verification
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-25T15:13:19.016589+00:00
created_at: 2026-07-25T15:13:19.016589+00:00
updated_at: 2026-07-25T15:13:19.016589+00:00
revision: 0
---

## Phase context

- Phase: `model-runtime-per-device`
- Project: `KnowMe`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system`
- Captured: `2026-07-25T14:45:58Z`
- Position: `model-runtime-per-device › M5b › device-verify`
- Status: `awaiting_operator`

## Phase goals

- Ship `Qwen3.5-9B Q4_K_M` with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM `Gemma 4 E2B`, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Current state

M5b has fully landed, been pushed, built, installed, launched, and detached on the iPhone. This follows the install-monitor state in [M5b install monitor armed for model runtime picker verification](/m5b-install-monitor-armed-for-model-runtime-picker-verification.md).

- Release build duration: `438.2s`.
- Installed and launched on iPhone device `00008150`.
- `flutter run` was stopped after successful install/launch.
- App is running debugger-free and is unplug-safe.
- All M5b tasks are closed: `34/40/46/47/48`.
- Operator verification remains open for the three-tier model picker behavior and provenance display.

## Verification procedure

Before testing, fully quit and reopen KnowMe from the app switcher so the fresh binary is active.

### 1. Global default model

Path:

`Settings → Runtime & policy → Default model`

Expected behavior:

- New **Default model** picker is present.
- Picker uses a Provider → Model cascade.
- Selecting a cloud provider and model persists the choice to `run_policy.global`.

### 2. Per-agent model

Path:

`Settings → Agents → select an agent → Model`

Expected behavior:

- New **Model** picker appears above the JSON editor.
- Selecting a model different from the global default stores the per-agent override.

### 3. Per-conversation model

Path:

`Chat → policy toolbar horizontal chips → Model chip → bottom-sheet picker`

Expected behavior:

- Policy toolbar includes a **Model** chip.
- Tapping it opens a bottom-sheet picker.
- Selecting a model different from the global and agent models stores the per-conversation override.

## Precedence contract

With all three tiers configured, send a message in the conversation.

Expected resolution order:

1. Conversation model override
2. Agent model override
3. Global default model
4. `On-device` fallback

Expected provenance behavior:

- With all three tiers set, the footer provenance chip shows the **conversation** model.
- After clearing the conversation model by selecting **Agent / global default**, the next message shows the **agent** model.
- After clearing the agent model, the next message shows the **global** default.
- After clearing the global model, the next message shows **On-device**.

## Picker semantics

- Every picker lists configured providers plus **On-device**.
- Selecting the inherit option clears that tier's explicit model selection.
- Conversation-level selection wins over agent-level and global-level selection.

## Remaining plan items

- `M7-react`: macOS desktop React should drop the service-URL form when embedded; tracked as item `#41`.
- `knowledge-capture D`: create 7 Karpathy Reference records and add/verify lint gate; tracked as item `#29`.

# Citations

1. stdin
2. manual:KnowMe/model-runtime-per-device