---
type: Reference
id: embedded-catalog-agent-shape-parity-and-mode-selector-fixes
title: Embedded catalog agent shape parity and mode selector fixes
tags:
- model-runtime-per-device
- catalog-schema
- mode-selector
- lane-resolution
- flutter
- rust
- litert-lm
links:
- m5b-install-monitor-armed-for-model-runtime-picker-verification
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-26T00:34:59.194084+00:00
created_at: 2026-07-26T00:34:59.194084+00:00
updated_at: 2026-07-26T00:34:59.194084+00:00
revision: 0
---

## Context

Phase: `model-runtime-per-device` for KnowMe.

Goals remain:

- Ship `Qwen3.5-9B Q4_K_M` with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM `Gemma 4 E2B`, with CPU as universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Corrected root cause

Initial diagnosis was wrong: Android `local` was **not** failing because `set_active_lane` rejected it via `NoLocalEngine`.

Verified behavior:

- On Android, the `NoLocalEngine` guard is effectively dead.
- `boot.rs` eagerly registers `LiteRtLmEngine`.
- The constructor is infallible.
- The feature is unconditionally enabled.
- Therefore `state::inference()` succeeds.

The real defect was an embedded/service catalog contract mismatch.

## Catalog shape mismatch

`catalog.agents` serialized differently by runtime path:

| Path | `catalog.agents` shape |
|---|---|
| Service | `{ runtime_agents: [...], federated_agents: [...] }` |
| Embedded: iOS/Android/macOS | bare array `[...]` |

Flutter/Dart reads:

```dart
catalog['agents']['runtime_agents']
```

On embedded builds this returned `null`, causing `_agents()` to return an empty list. Agents were seeded and present, but unreachable from the client parser.

User-visible symptom:

- Agent mode opened an empty sheet.
- Since no agent option was available, mode never changed.

Why CI missed it:

- The existing widget test hand-wrote a fixture in the **service** shape.
- It did not cover the embedded bare-array shape.

## Related embedded-path mismatches

Additional schema mismatches on the same path:

- Agent labels live at `metadata.title`, while Dart reads top-level `name` / `title`.
- Rust serializes `knowledgeBases`, while Dart reads `knowledge_bases`.

## Implemented fixes

### Embedded/service catalog parity

The embedded branch now emits the same shape as the service contract:

```json
{
  "runtime_agents": [],
  "federated_agents": []
}
```

This allows one client parser to work across service and embedded paths, preserving Rule 14 parity.

Changed area: `control_plane.rs`.

### Per-conversation lane resolution

`send()` now resolves lane selection in precedence order:

1. Conversation `chat_mode`
2. Global setting
3. `default_lane()`

Mode mapping includes:

- `agent` → `LANE_UAR`
- `uar` → `LANE_UAR`

Changed area: `chat.rs`.

### Side-effect-free availability probe

Added `lane_availability()` as a non-mutating probe so the picker sheet can disable unavailable modes and expose a reason without changing active lane state.

Changed area: `chat.rs`.

## UI design adjustments

Design audit changed the next UI implementation details:

- Use `shad.openSheetOverlay`, the sanctioned sheet primitive.
  - Existing in-repo precedent: `chat_screen.dart:289`.
  - Avoid Material `showModalBottomSheet`.
- Use `48px` interaction targets because an existing test asserts `>= 48`.

This continues the picker/runtime UI workstream previously tracked in [M5b install monitor armed for model runtime picker verification](/m5b-install-monitor-armed-for-model-runtime-picker-verification.md).

## Current status

- Embedded/service catalog shape parity fixed in Rust.
- Per-conversation lane resolution added.
- Side-effect-free `lane_availability()` probe added.
- `cargo check` is green.
- Current position: `model-runtime-per-device › mode-selector-redesign › MS1`.
- Status: executing.

## Next work

1. Add Rust tests for:
   - lane resolution precedence
   - `lane_availability()` probe behavior
   - embedded/service agent catalog shape parity
2. Add FFI passthrough for the availability probe.
3. Update Flutter state handling:
   - error handling
   - catalog availability in new chat
   - `metadata.title` labels
4. Implement the single-button picker sheet using `shad.openSheetOverlay`.
5. Verify on Samsung Android and iPhone.

# Citations

1. stdin
2. manual:KnowMe/model-runtime-per-device