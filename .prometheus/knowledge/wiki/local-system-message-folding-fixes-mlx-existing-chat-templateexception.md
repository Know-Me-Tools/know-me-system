---
type: Reference
id: local-system-message-folding-fixes-mlx-existing-chat-templateexception
title: Local system-message folding fixes MLX existing-chat TemplateException
tags:
- model-runtime-per-device
- mlx
- system-message-folding
- litert-lm
- android-certification
- uar-persistence
- local-inference
sources:
- stdin
timestamp: 2026-07-26T08:52:22.560251+00:00
created_at: 2026-07-26T08:52:22.560251+00:00
updated_at: 2026-07-26T08:52:22.560251+00:00
revision: 0
---

## Phase context

- Phase: `model-runtime-per-device`
- Project: `KnowMe`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system`
- Captured: `2026-07-26T08:45:56Z`
- Position: `model-runtime-per-device › litertlm-device-proof`
- Status: `executing`
- Progress: changes `4/5`

## Phase goals

- Ship `Qwen3.5-9B Q4_K_M` with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM `Gemma 4 E2B`, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Confirmed device state

- iPhone 17 Pro Max local mode now responds in the previously failing existing conversation.
- Editable local system prompt works in **Settings → Local models**.
- The local system prompt is DB-backed and editable.
- Reset sources the default prompt from Rust so Flutter/UI and Rust defaults cannot drift.
- Commits: `82d0c69` and `ff7d800`.

## Root cause: non-leading `System` messages in existing chats

The MLX `TemplateException` was caused by context assembly producing `System` messages after conversation history had begun. `assemble` places the real system prompt at index `0`; later context slices appended additional `System` messages, which MLX's chat template rejects.

Three context slices were injecting non-leading `System` messages:

1. `truncate_middle` omission marker: `[… N earlier messages omitted …]`
2. Conversation-summary turn prepended to the retained tail
3. Memory/knowledge blocks

The failure only reproduced on an **existing conversation** because fresh chats have too little history to trigger compaction. Existing-history compaction activated the offending slices.

## Fix

Non-leading `System` messages are folded into the leading system message centrally, after all context slices contribute.

Rationale:

- Preserves real context that would be lost if the extra `System` messages were dropped.
- Avoids teaching three independent producers about placement rules.
- Reduces regression risk by enforcing the invariant once in `assemble`.

Invariant after fix:

```text
message[0] may be System
message[1..] must not be System
```

## Validation

Passing checks reported after the fix:

- 4 context tests
- 4 settings tests
- `gen_ui_agent`: `72/72`
- Flutter: `35` passing
- Flutter has `3` pre-existing failures unrelated to this fix

## Open change: `litertlm-device-proof`

Change 5 of 5 remains open; phase progress is `4/5`.

Blocking items:

1. Samsung device shows `adb unauthorized`.
   - The device is displaying the **Allow USB debugging?** prompt.
   - Tapping the prompt unblocks shell access.
   - 7 of 9 required evidence items already have scripts ready.
2. Attachment flow has no certification script.
   - This is real remaining implementation work, not just blocked device access.
3. Load and throughput evidence must be harvested into the certification report.
   - Instrumentation already exists for `load_ms`, `generate_ms`, and `chars_per_second`.

## Durable persistence gap

A separate durable-persistence session landed no commits.

Current verified state:

- `InMemoryProvider` still exists at `rust/crates/gen_ui_agent/src/uar.rs:880`.
- Task `embed-uar-mobile-offline` 3.1 remains unchecked.
- Per-conversation mode still resets on app restart.
- Policies, agents, skills, memory, and run state are still RAM-backed.

This is the largest remaining embedded-runtime gap if restart persistence is prioritized over Android certification.

## Child phase state

The child phase `embedded-uar-offline-agents` is `8` done / `14` open. Remaining work includes:

- Desktop certification
- Android airplane-mode certification
- iOS airplane-mode certification

Correction to carry forward:

- `progress.json` is valid JSON.
- The child phase state lives under `children/embedded-uar-offline-agents/`.

## Next actions

Choose one:

1. Tap **Allow USB debugging** on the Samsung to unblock the `litertlm-device-proof` certification suite.
2. Pick up durable persistence task 3.1 to close the app-restart persistence gap first.

# Citations

1. stdin