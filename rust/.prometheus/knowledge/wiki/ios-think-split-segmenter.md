---
type: Reference
id: ios-think-split-segmenter
title: <think> reasoning split into ThinkingDelta (multi-tag segmenter)
tags:
- knowme
- model-runtime-per-device
- reasoning
- thinking
- a2ui
- content-blocks
- mlx
- segmenter
links:
- a2ui-content-blocks
- ios-mlx-switchover-and-qwen3-arch-fix
- local-agentic-lanes
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-23T01:31:00+00:00
created_at: 2026-07-23T01:31:00+00:00
updated_at: 2026-07-23T01:31:00+00:00
revision: 0
---

## Intent

Stop Qwen3's `<think>…</think>` reasoning from rendering as plain assistant
message content on device; render it as a distinct collapsible thinking block.

## Observed state

- On-device screenshot showed the model's chain-of-thought printed inline as the
  answer.
- Root cause was engine-only: the MLX lane emitted every token as
  `StreamEvent::TextDelta`, never `ThinkingDelta`. The shared
  `gen_ui_protocol::A2uiAdapter` already maps `ThinkingDelta →
  ContentBlock::Thinking`, and the Flutter `ContentBlockView`
  (`$REPO_ROOT/flutter_packages/gen_ui_widgets/lib/src/content_block_view.dart`)
  already has a complete stateful collapsible `ContentBlockThinking` widget. So
  the only missing piece was the engine emitting the right event.

## Decision and delta

Commit **`06f5fd9`**:
- Added `gen_ui_types::inference::ReasoningFormat { None, Qwen3Think, Harmony }`
  — a per-model property for how a model marks reasoning tokens.
- Generalized `$REPO_ROOT/rust/crates/gen_ui_inference/src/mlx/tool_parser.rs`
  from a tool-call-only state machine into a **multi-tag segmenter**: it now
  recognizes `<think>…</think>` → `ThinkingDelta` alongside `<tool_call>` → tool
  events, reusing the same bounded-holdback + char-boundary-safe tag detection
  (`safe_emit_len`). `ToolCallParser::with_reasoning(format)` gates the think
  split; `::new()` stays tool-call-only.
- `MlxArtifactSet.reasoning_format` (catalog) is the authoritative source the
  engine reads; both Qwen3 MLX sets = `Qwen3Think`. `mlx.rs::stream_generation`
  constructs the parser from the loaded model's `reasoning_format`.
- 6 new parser tests (think-only, think disabled, open/close split across deltas,
  think+tool interleave, no-markup passthrough, unterminated flush).

Per-engine reasoning reality: iOS MLX/Qwen3 = `Qwen3Think`; Android
LiteRT/Gemma-4 = `None` (instruct model, no reasoning markup); macOS `mlxc`
already emits `ThinkingDelta` via the mlex `TokenKind::Reasoning` classifier
(cleaner, tokenizer-level); desktop llama deferred.

## Verification

- `cargo test -p gen_ui_inference --features local-mlx --lib` — 15 parser tests
  pass (incl. the 6 new). gen_ui_types + gen_ui_agent suites green.
- Installed on device; awaiting the operator's visual confirmation of the
  collapsible Thinking block (folds into the device-cert record).

## Failure/lesson

When a UI symptom looks like a rendering bug, trace the event pipeline before
touching the UI: the renderer + shared adapter were already correct; the defect
was one missing event type at the engine layer. The fix stayed a 1-crate,
1-state-machine change because the surrounding contract was sound.

## Next experiment

Confirm the Thinking block renders on device on the Local lane; extend the same
segmenter to the WebLLM lane (`webllmLane.ts` currently accumulates one text
block) if a reasoning-capable web model is adopted.
