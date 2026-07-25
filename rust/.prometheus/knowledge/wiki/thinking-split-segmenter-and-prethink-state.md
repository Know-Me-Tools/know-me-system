---
type: Reference
id: thinking-split-segmenter-and-prethink-state
title: Reasoning split — the PreThink segmenter state for pre-filled think tags
tags:
- knowme
- model-runtime-per-device
- ios
- a2ui
- content-blocks
- thinking
- qwen3
links:
- a2ui-content-blocks
- ios-mlx-switchover-and-swift-bridge
- ios-device-certification-local-and-embedded-uar
sources:
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-25T21:41:00+00:00
created_at: 2026-07-25T21:41:00+00:00
updated_at: 2026-07-25T21:41:00+00:00
revision: 0
---

## Context

- **Project:** KnowMe
- **Phase:** `model-runtime-per-device`
- **Captured:** `2026-07-25`
- **Status:** `verified`

## Intent

Render the model's `<think>…</think>` reasoning as a separate collapsible
Thinking block instead of mixing it into the answer text.

## Observed state

The A2UI emission layer was **already shared** across cloud, local, and UAR
lanes and all four UI surfaces — this was the crucial finding. All three lanes
fold through the same `gen_ui_protocol::A2uiAdapter`, which already maps
`StreamEvent::ThinkingDelta → ContentBlock::Thinking`.

The defect was localized to the *engine* layer: local engines emitted only
`TextDelta`, never `ThinkingDelta`. `mlx/tool_parser.rs` had two states (`Text`,
`InToolCall`), so `<think>` fell through into `Text`.

## Decision and delta

Generalized the tool-call parser into a multi-tag segmenter, reusing its
holdback and char-boundary logic (`safe_emit_len`) so tags split across deltas
still parse.

The subtle part: **Qwen3's chat template pre-fills the opening `<think>` tag**,
so the stream contains only the *closing* `</think>`. A naive "wait for
`<think>`" segmenter therefore never enters the thinking state and emits
everything as text. The fix was a `PreThink` start state for reasoning models:
strip a leading `<think>` if present, defer to the tool path if `<tool_call>`
leads, stream the body as `ThinkingDelta` until `</think>`, then transition to
`Text`. `finish()` flushes any `PreThink` remainder as `ThinkingDelta`.

A `reasoning_format` field on `ModelProfile` selects the segmenter mode per
model preset, so this stays data-driven rather than hardcoded per engine.

## Verification

19/19 mlx tests pass, covering think-only, think+tool interleaved,
split-across-deltas, and no-think passthrough. Device-verified on iPhone: the
reasoning renders as a collapsible Thinking block with no bare `</think>` leaking
into the answer.

## Failure/lesson

The bug looked like a protocol/rendering gap and was actually a tokenizer-
template detail. Before adding layers, confirm which layer is actually missing
the signal — here everything above the engine was already correct, and the real
cause was that the model never emits the opening tag the parser was waiting for.

## Next experiment

Add the harmony `<|channel|>` tag set to the same data-driven segmenter so
non-Qwen reasoning models split identically.
