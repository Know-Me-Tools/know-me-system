---
type: Reference
id: knowme-ios-memory-fix-deployed-for-cold-start-verification
title: KnowMe iOS memory fix deployed for cold-start verification
tags:
- model-runtime-per-device
- memory-fix
- ios-device
- uar
- flutter-release
- cold-start-memory
links:
- memory-fix-install-watch-armed-for-unplug-safe-ios-verification
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-25T21:49:25.077474+00:00
created_at: 2026-07-25T21:49:25.077474+00:00
updated_at: 2026-07-25T21:49:25.077474+00:00
revision: 0
---

## Phase context

- Phase: `model-runtime-per-device`
- Project: `KnowMe`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system`
- Captured: `2026-07-25T21:00:20Z`
- Position: `model-runtime-per-device › memory-fix › device-verify`
- Status: `awaiting_operator`
- Device: iPhone `00008150`

## Phase goals

- Ship `Qwen3.5-9B Q4_K_M` with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM `Gemma 4 E2B`, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Deployment state

- Release build installed and launched on Travis's iPhone.
- `flutter run` was detached after successful install/launch; device is safe to unplug.
- Release build duration: `422.4s`.
- Included fixes:
  - Paste fix: `a9ae966`
  - KnowMe memory fix: `b86f43b`
  - UAR fix: `08a6f4c`, ADR0016
- All tests were green before deployment.
- This completes the install/launch step that followed [Memory-fix install watch armed for unplug-safe iOS verification](/memory-fix-install-watch-armed-for-unplug-safe-ios-verification.md).

## Operator verification checklist

Force-quit and reopen KnowMe before testing so the newly installed binary is active.

### 1. Paste behavior

Previously broken. Verify paste callouts appear and work:

- Copy any text.
- Long-press the `Ask KnowMe anything…` input.
- Confirm `Paste / Copy / Select All` appears and works.
- Repeat the same check in both Memory screen inputs.

### 2. Within-conversation memory

On the **Orchestrator** tab:

1. Send: `My name is Travis and I drive a blue truck.`
2. Send: `What's my name and what do I drive?`
3. Expected result: KnowMe answers correctly using the same conversation context.

### 3. Cold-start memory regression test

This is the primary bug certification.

1. Force-quit KnowMe.
2. Reopen KnowMe.
3. Return to the same conversation.
4. Ask: `What did I tell you about my truck?`
5. Expected result: KnowMe still knows the prior truck fact.

Previous failure mode: total amnesia after cold start because the UAR lane only received the current message.

### 4. Cross-conversation isolation

1. Start a new conversation.
2. Ask: `What's my name?`
3. Expected result: KnowMe should **not** know; the remembered context must remain scoped to the original thread.

## Certification criteria

- If check 3 passes, the memory fix is certified.
- If any check fails, capture:
  - Observed behavior
  - Tab where it occurred
  - Relevant conversation/thread context
- Diagnostic path is available for follow-up: session id, seed count, and payload reaching the model.

## Remaining plan

- Await operator verification of paste, within-conversation memory, cold-start memory, and no cross-conversation bleed.
- Remaining workstream items:
  - M7-react: `#41`
  - knowledge-capture D: `#29`

# Citations

1. stdin
2. manual:KnowMe/model-runtime-per-device