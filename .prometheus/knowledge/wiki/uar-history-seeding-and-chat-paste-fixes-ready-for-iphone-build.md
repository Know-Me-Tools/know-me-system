---
type: Reference
id: uar-history-seeding-and-chat-paste-fixes-ready-for-iphone-build
title: UAR history seeding and chat paste fixes ready for iPhone build
tags:
- model-runtime-per-device
- uar
- orchestrator
- conversation-history
- ios-build
- chat-paste
- session-isolation
links:
- m5b-install-monitor-armed-for-model-runtime-picker-verification
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-25T20:52:02.395875+00:00
created_at: 2026-07-25T20:52:02.395875+00:00
updated_at: 2026-07-25T20:52:02.395875+00:00
revision: 0
---

## Phase context

- Phase: `model-runtime-per-device`
- Project: `KnowMe`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system`
- Captured: `2026-07-25T15:12:59Z`
- Position: `model-runtime-per-device › memory-fix › ready-to-build`
- Status: `awaiting_go`

## Phase goals

- Ship `Qwen3.5-9B Q4_K_M` with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM `Gemma 4 E2B`, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Fixes landed

### UAR/Orchestrator memory loss and context bleed

Root cause was limited to the UAR/Orchestrator lane:

- `send_uar` discarded conversation history and depended on a non-durable in-memory session.
  - After force-quit/reopen, the model received only the current message.
- A global `uar_session_id` setting could collapse all conversations into one shared session.
  - This allowed cross-conversation context bleed.

Fixes landed in both repos:

- UAR commit `08a6f4c` / ADR `0016`:
  - Added a `SeedMessage` seam.
  - Added `start_run_with_history`.
  - Repopulates an empty embedded session from the host thread before the current message.
  - Seeding is idempotent: warm sessions are not duplicated.
  - Tests prove cold-session seeding works and warm sessions are not re-seeded.
- KnowMe commit `b86f43b`:
  - `send_uar` now builds seed messages from the history it receives and passes them to UAR.
  - Embedded sessions are keyed strictly by `conversation_id`.
  - The global session-id override applies only to the service path, not embedded per-conversation sessions.

The Local lane was already correct because it sent full history; only the Orchestrator/Agent lane was affected.

### Chat paste support

Commit `a9ae966` restores platform paste/copy menus:

- `shad.TextField` had defaulted `contextMenuBuilder` to `null`, suppressing the native Paste/Copy callout.
- Fixed by using `TextField.nativeContextMenuBuilder()` on:
  - chat composer
  - both memory fields

## Validation status

- Fixes are committed and pushed.
- `gen_ui_agent`: `68/68` passing.
- iOS configuration: green.
- Embedded suite: `9/9` passing.
- OpenSpec: strict-valid.
- Tasks closed: `49/50`.

This build follows the model runtime picker/device installation workstream captured in [M5b install monitor armed for model runtime picker verification](/m5b-install-monitor-armed-for-model-runtime-picker-verification.md).

## Release gate and operator checks

Build/deploy is paused until explicit operator approval. The next build should bundle both memory and paste fixes for iPhone deployment.

After installing on device, verify:

1. Paste works in the chat box.
2. Orchestrator remembers earlier turns after force-quit/reopen.
3. Switching between conversations does not mix context.

## Remaining plan

- M7-react: task `#41`.
- Knowledge-capture D: task `#29`.

# Citations

1. stdin
2. manual:KnowMe/model-runtime-per-device