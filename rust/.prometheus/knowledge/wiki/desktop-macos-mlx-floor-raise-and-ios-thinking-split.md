---
type: Reference
id: desktop-macos-mlx-floor-raise-and-ios-thinking-split
title: Desktop macOS MLX floor raise + iOS thinking-split
tags:
- knowme
- model-runtime-per-device
- macos
- mlx
- mlex
- desktop-stable-port
- ios
- thinking
- a2ui
- content-blocks
links:
- desktop-stable-port-openspec-change
- a2ui-content-blocks
- mlx-engine-policy
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-22T17:20:00+00:00
created_at: 2026-07-22T17:20:00+00:00
updated_at: 2026-07-22T17:20:00+00:00
revision: 0
---

## Context

- **Project:** KnowMe
- **Phase:** `model-runtime-per-device`
- **Captured:** `2026-07-22`
- **Status:** `executing`

Two coupled advances landed while stabilizing per-device local inference: the
iOS on-device reasoning-rendering fix, and the macOS desktop MLX build unblock
that opens the `desktop-stable-port` OpenSpec change.

## iOS thinking-split (commit 06f5fd9)

Qwen3's `<think>…</think>` reasoning was rendering as plain assistant content on
device. Root cause was engine-only: the MLX lane emitted every token as
`TextDelta`, never `ThinkingDelta`. The shared `gen_ui_protocol::A2uiAdapter`
already maps `ThinkingDelta → ContentBlock::Thinking`, and the Flutter
`ContentBlockView` (`flutter_packages/gen_ui_widgets`) already has a complete
collapsible `ContentBlockThinking` widget — so the fix was purely to make the
engine emit the right event.

- `gen_ui_types::inference::ReasoningFormat { None, Qwen3Think, Harmony }` — a
  per-model property.
- `mlx/tool_parser.rs` generalized into a multi-tag segmenter: splits `<think>`
  → `ThinkingDelta` alongside `<tool_call>`, reusing the bounded-holdback +
  char-boundary detection. `with_reasoning(format)` gates the think split.
- `MlxArtifactSet.reasoning_format` (catalog) is the authoritative source the
  engine reads; both Qwen3 MLX sets = `Qwen3Think`.
- 6 new parser tests. Installed on the iPhone 17 Pro Max; awaiting final visual
  confirmation of the collapsible Thinking block.

Per-engine reasoning reality: iOS MLX/Qwen3 needs the split; Android
LiteRT/Gemma-4 has no reasoning markup (`None`); macOS `mlxc` already emits
`ThinkingDelta` via the mlex `TokenKind::Reasoning` classifier; desktop llama is
deferred.

## macOS MLX floor raise (commits 68bfe34 parent, 5343f29 submodule, ed98ef5 pointer)

`MACOSX_DEPLOYMENT_TARGET=10.15` was force-set (whisper.cpp/llama.cpp
`std::filesystem`) below MLX's macOS 14 CMake floor, so the scaffolded
`local-mlxc` (`mlex`/mlx-c) engine could not compile. Raised to 14.0 in
`rust/.cargo/config.toml`, `desktop/src-tauri/.cargo/config.toml`, and the Tauri
bundle `macOS.minimumSystemVersion` (14.0 ⊃ 10.15, so nothing regresses).

**Verified: `cargo build -p gen_ui_inference --features local-mlxc` compiles and
links fully on macOS (5m49s), no `MLX requires macOS >= 14.0`.** The macOS MLX
lane builds for the first time. `MlxcEngine` already uses the shared catalog,
SHA-verified downloads, memory preflight, and `TokenKind::Reasoning →
ThinkingDelta`; mlxc tests green.

Desktop engine matrix wired (`tauri-plugin-gen-ui`): per-target features and
`cfg` construction select macOS → `MlxcEngine`, Windows/Linux → `LlamaCppEngine`.

## OpenSpec change

`desktop-stable-port` (spec-driven, all 4 artifacts, `openspec validate` passes)
tracks the macOS stabilization: floor raise, mlex certification, engine matrix,
and full-agentic A2UI parity on the desktop shell. Committed in the UAR
submodule branch `desktop-stable-port-opsx`. Applied through task 3.2; remaining:
3.3 (Win/Linux no-regression), group 4 (shell boot + A2UI parity, needs a running
desktop app), group 5 (make MLX the macOS default, KBD/wiki finalization).
