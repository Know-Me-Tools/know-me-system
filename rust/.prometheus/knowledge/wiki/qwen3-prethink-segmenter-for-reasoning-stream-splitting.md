---
type: Reference
id: qwen3-prethink-segmenter-for-reasoning-stream-splitting
title: Qwen3 PreThink segmenter for reasoning stream splitting
tags:
- knowme
- model-runtime-per-device
- ios
- a2ui
- thinking
- qwen3
- mlx
- segmenter
links:
- ios-think-split-segmenter
- desktop-macos-mlx-floor-raise-and-ios-thinking-split
sources:
- thinking-split-segmenter-and-prethink-state.md
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-25T21:45:44.535672+00:00
created_at: 2026-07-25T21:45:44.535672+00:00
updated_at: 2026-07-25T21:45:44.535672+00:00
revision: 0
---

## Context

- **Project:** KnowMe
- **Phase:** `model-runtime-per-device`
- **Captured:** 2026-07-25
- **Status:** verified

## Intent

Render model `<think>…</think>` reasoning as a separate collapsible **Thinking** block instead of mixing it into assistant answer text.

## Observed state

- The A2UI emission/rendering path was already shared across cloud, local, and UAR lanes and all four UI surfaces.
- All lanes fold through `gen_ui_protocol::A2uiAdapter`, which already maps:
  - `StreamEvent::ThinkingDelta` → `ContentBlock::Thinking`
- The defect was localized to the engine layer:
  - Local engines emitted only `TextDelta`.
  - They never emitted `ThinkingDelta`.
  - `mlx/tool_parser.rs` had only two states: `Text` and `InToolCall`.
  - As a result, `<think>` content fell through into plain text.

## Decision

Generalize the MLX tool-call parser into a multi-tag segmenter that can split both tool calls and reasoning spans. This builds on the implementation described in [<think> reasoning split into ThinkingDelta (multi-tag segmenter)](/ios-think-split-segmenter.md).

Key behavior:

- Reuse existing bounded holdback logic.
- Preserve char-boundary safety via `safe_emit_len`.
- Correctly parse tags split across streaming deltas.
- Emit reasoning body as `ThinkingDelta` rather than `TextDelta`.

## Critical Qwen3 detail: pre-filled opening tag

Qwen3's chat template pre-fills the opening `<think>` tag. The model stream may therefore contain only the closing tag:

```text
reasoning tokens...
</think>
answer tokens...
```

A segmenter that waits to observe an opening `<think>` never enters the thinking state and emits all reasoning as text.

## Segmenter state change

Add a `PreThink` start state for reasoning models:

- At stream start:
  - Strip a leading `<think>` if present.
  - If `<tool_call>` leads, defer to the tool-call path.
  - Otherwise treat the initial stream as reasoning content.
- While in `PreThink`:
  - Emit body text as `ThinkingDelta`.
  - Continue until `</think>` is observed.
  - Transition to `Text` after the closing tag.
- On `finish()`:
  - Flush any remaining `PreThink` buffer as `ThinkingDelta`.

## Model-driven selection

A `reasoning_format` field on `ModelProfile` selects segmenter behavior per model preset. This keeps reasoning parsing data-driven instead of hardcoding behavior per engine.

Relevant formats include Qwen-style `<think>` tags and planned Harmony channel tags, as also noted in [Desktop macOS MLX floor raise + iOS thinking-split](/desktop-macos-mlx-floor-raise-and-ios-thinking-split.md).

## Verification

- `19/19` MLX tests passed.
- Test coverage includes:
  - think-only streams
  - think + tool interleaving
  - tags split across deltas
  - no-think passthrough
- Device-verified on iPhone:
  - reasoning renders as a collapsible Thinking block
  - no bare `</think>` leaks into answer text

## Lesson

The issue initially looked like a protocol or rendering gap, but the upper layers were already correct. The root cause was a tokenizer/chat-template detail: the model stream did not include the opening tag the parser expected. Before adding protocol or UI layers, verify which layer is actually failing to emit the required semantic signal.

## Next experiment

Add the Harmony `<|channel|>` tag set to the same data-driven segmenter so non-Qwen reasoning models split reasoning identically.

# Citations

1. thinking-split-segmenter-and-prethink-state.md
2. manual:KnowMe/model-runtime-per-device