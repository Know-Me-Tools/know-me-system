---
type: Reference
id: local-tab-state-fix-and-android-litert-lm-progress-correction
title: Local tab state fix and Android LiteRT-LM progress correction
tags:
- model-runtime-per-device
- litert-lm
- android-device-cert
- local-tab
- lane-routing
- progress-json
- flutter-release
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-25T23:15:20.993797+00:00
created_at: 2026-07-25T23:15:20.993797+00:00
updated_at: 2026-07-25T23:15:20.993797+00:00
revision: 0
---

## Phase context

- Phase: `model-runtime-per-device`
- Project: `KnowMe`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system`
- Captured: `2026-07-25T23:13:05Z`
- Position: `model-runtime-per-device › litertlm-device-proof`
- Status: `rebuilding`

## Phase goals

- Ship `Qwen3.5-9B Q4_K_M` with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM `Gemma 4 E2B`, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Fix 1: Local tab selected-state now reflects tapped lane immediately

### Root cause

- The Local tab's selected state was derived only from `state.policy['chat_mode']`.
- `state.policy['chat_mode']` was mutated only inside `patch()`.
- `patch()` early-returned when `conversationId` was empty.
  - Brand-new chats have an empty `threadId`.
  - Result: tapping **Local** in a new chat switched the backend lane but did not update the toolbar selection.
- `patch()` also swallowed persistence failures into `state.error`, which hid the mismatch from the visible tab state.

### Routing truth

- `selectLane` persists `active_lane`.
- Rust routes every send by reading `SETTING_ACTIVE_LANE` in `chat.rs`.
- Therefore the lane had already changed server-side; the UI was stale/misleading.

### Implemented fix

- `setMode` now updates the in-memory mode state immediately when the user taps a lane.
- Policy persistence still runs afterward as before.
- This makes the visible tab state align with the routing authority instead of waiting on `patch()`.

### Verification target

After rebuild/install, tap **Local** in a conversation:

- The Local tab should select immediately.
- This must also work in a brand-new chat where the old `conversationId == ""` path previously caused no visible selection change.

## Fix 2: `progress.json` corrected with provenance

`progress.json` was corrected to distinguish real evidence from unsound historical status.

| Item | Previous status | Corrected status | Rationale |
|---|---:|---:|---|
| `litertlm-local-tools` | `active` | `completed` | Certified on the Samsung device in this session. |
| `litertlm-device-proof` | `completed` | `not completed` | Previous status claimed device certification for a path that crashed at bridge load in release until this session; that evidence could not have existed. |
| `android-litertlm-runtime` | `completed` | `completed` with annotation | Kept completed, but annotated that it was not actually working when first marked; fixed in commit `7330389`. |

Additional handling:

- Overall count remains `4/5`.
- The four completed items now have real evidence.
- The remaining honest gap is `litertlm-device-proof`.
- Reasoning was recorded in a `status_corrections` field rather than silently rewriting history.
- Backup retained at `/tmp/progress.json.bak`.

## Current execution state

- Commit pushed: `2179a83`.
- Rebuild/install running: `bsod839ow`.
- Last completed step: both fixes committed and device rebuild started.

## Remaining phase gap: Android LiteRT-LM device proof

`litertlm-device-proof` still requires a real Android certification run covering:

- Model load time.
- Throughput.
- Backend used.
- Memory behavior.
- Thermal behavior.
- Cancellation behavior.
- Markdown/Mermaid rendering path.
- Attachment flow.
- Permitted tool-call plumbing.

# Citations

1. stdin
2. manual:KnowMe/model-runtime-per-device