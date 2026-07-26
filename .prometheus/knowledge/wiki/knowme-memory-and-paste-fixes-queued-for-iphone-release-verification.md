---
type: Reference
id: knowme-memory-and-paste-fixes-queued-for-iphone-release-verification
title: KnowMe memory and paste fixes queued for iPhone release verification
tags:
- model-runtime-per-device
- knowme
- uar-memory
- session-isolation
- flutter-release
- ios-device
- chat-input
links:
- m5b-install-monitor-armed-for-model-runtime-picker-verification
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-25T20:59:40.383844+00:00
created_at: 2026-07-25T20:59:40.383844+00:00
updated_at: 2026-07-25T20:59:40.383844+00:00
revision: 0
---

## Phase context

- Phase: `model-runtime-per-device`
- Project: `KnowMe`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system`
- Captured: `2026-07-25T20:51:44Z`
- Position: `model-runtime-per-device › memory-fix › building`
- Status: `executing`
- Device: iPhone `00008150`
- Build monitor: `bmzkbuegg`
- Build/run id: `bjg28e0cb`

## Phase goals

- Ship `Qwen3.5-9B Q4_K_M` with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM `Gemma 4 E2B`, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Build state

A signed release build/install to iPhone `00008150` is running. This is a fuller rebuild, expected to take roughly 7–8 minutes, because the Rust workspace changed, including `gen_ui_agent` and the UAR submodule.

- Confirmed app HEAD: `b86f43b`
- UAR submodule fix: `08a6f4c`
- Device connected before release build kickoff.
- After install, detach `flutter run` so the phone is unplug-safe, continuing the same device-install workflow pattern used in [M5b install monitor armed for model runtime picker verification](/m5b-install-monitor-armed-for-model-runtime-picker-verification.md).

## Included fixes

| Area | Commit(s) | Expected verification |
| --- | --- | --- |
| Chat paste | `a9ae966` | Long-pressing the chat input shows the system Paste/Copy/Select All callout; paste works. |
| UAR memory | KnowMe `b86f43b` + UAR `08a6f4c` | Orchestrator remembers earlier turns, including after force-quit/reopen. |
| Session isolation | Same as memory fix | Switching conversations no longer bleeds context between threads. |

## Validation already green before device install

- Embedded tests: `9/9`
- `gen_ui_agent`: `68/68`
- iOS config checks: green
- Formatting: green
- OpenSpec strict validation: green

## Operator verification plan after install

1. **Paste behavior**
   - Copy any text.
   - Long-press the `Ask KnowMe anything…` input.
   - Verify the Paste/Copy/Select All callout appears.
   - Verify Paste inserts the copied text.

2. **Within-conversation memory**
   - In **Orchestrator**, send: `My name is Travis and I drive a blue truck.`
   - Then send: `What's my name and what do I drive?`
   - Expected: answer correctly includes `Travis` and `blue truck`.

3. **Cold-start memory**
   - Force-quit KnowMe by swiping it out of the app switcher.
   - Reopen KnowMe.
   - Return to the same conversation.
   - Ask: `What did I tell you about my truck?`
   - Expected: the orchestrator still knows the truck context.
   - Regression addressed: previous behavior was total amnesia after force-quit.

4. **No cross-conversation bleed**
   - Start a new conversation.
   - Ask: `What's my name?`
   - Expected: the new thread should not know the name from the previous conversation.

## Remaining plan

- Build lands.
- Detach `flutter run` for unplug-safe operation.
- Operator verifies paste, within-conversation memory, cold-start memory after force-quit, and session isolation.
- Continue with M7-react (`#41`).
- Continue with knowledge-capture D (`#29`).

# Citations

1. stdin
2. manual:KnowMe/model-runtime-per-device