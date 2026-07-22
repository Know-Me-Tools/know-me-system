# Current waypoint

- Phase: `model-runtime-per-device`
- Change: `desktop-stable-port`
- Status: `executing`
- Progress: 13 of 18 implementation tasks

## Landed

- **iOS local inference certified on device** (iPhone 17 Pro Max) — Qwen3-4B MLX
  generates tokens. The pinned `qwen3_5`-arch model was repointed to Qwen3-4B
  because MLX-Swift 2.29.1 (latest) does not register `qwen3_5`
  (`unsupportedModelType`). Automatic code signing carries
  `com.apple.developer.kernel.increased-memory-limit`.
- **iOS `<think>` reasoning split** (commit `06f5fd9`) — the MLX lane emits
  `ThinkingDelta`, rendered by the existing collapsible `ContentBlockThinking`
  widget instead of inline content.
- **Submodule PRs #172/#173 MERGED** — parent submodule pointer stable on UAR
  `main`; the earlier dangling-pointer block is resolved.

## Active change: `desktop-stable-port`

Stabilize the macOS/desktop Tauri port so it matches iOS/Android/Web.

- **macOS build floor 10.15 → 14.0** (`68bfe34`/`5343f29`/`ed98ef5`) — removed the
  forced 10.15 below MLX's macOS 14 CMake floor. The `local-mlxc` (`mlex`/mlx-c)
  engine **compiles for the first time** (verified). `MlxcEngine` reuses the
  shared catalog, SHA-verified downloads, memory preflight, and
  `TokenKind::Reasoning → ThinkingDelta`.
- **Per-target engine matrix** (`c557c80`) — macOS → MLX, Windows/Linux →
  llama.cpp, via per-target Cargo features + `cfg` construction. Verified by a
  full `tauri-plugin-gen-ui` macOS build at the 14.0 floor.
- **Karpathy wiki progress log** (`e83fd63`).

## Remaining

- **Runtime cert** (tasks 4.1/4.2/4.4): launch the macOS desktop app, run an
  agent turn on a local model, confirm live A2UI ContentBlocks in the ops console.
- **Win/Linux cross-build** (task 3.3): CI Linux/Windows runners are the gate
  (the `not(macos)` lane is the unchanged `local-llama` config).
- iOS: operator's final visual confirmation of the collapsible Thinking block.

## Known-open UAR submodule defects (not started)

Deliberately untouched while the submodule had concurrent edits:

- Empty-turn history loss (`manager.rs:2039-2041`).
- Context trimming can orphan a tool-call/tool-result pair
  (`context/manager.rs:86-141`).
- `RUN_FINISHED` misreported as `RUN_ERROR` on an unclosed `<think>`;
  `RUN_ERROR`'s producer `code` discarded.

Either of the first two can reproduce a malformed-history symptom on a long
conversation — first place to look if Android regresses.

## Follow-on plan (approved)

- Phase 2 — transport-free AG-UI adapter into the UAR submodule
  (`to_agui_spec_event` is already pure/reachable) + ADR `docs/adr/0012`.
- Phase 4 — in-browser WebLLM local engine for Web parity.
- Phase 5 (rich ContentBlock rendering) is already largely built in
  `flutter_packages/gen_ui_widgets`.
