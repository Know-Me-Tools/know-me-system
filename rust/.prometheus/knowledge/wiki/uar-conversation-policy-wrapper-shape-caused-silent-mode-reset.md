---
type: Reference
id: uar-conversation-policy-wrapper-shape-caused-silent-mode-reset
title: UAR conversation policy wrapper shape caused silent mode reset
tags:
- knowme
- uar
- ffi
- flutter
- rust
- mode-picker
- local-inference
- device-certification
links:
- android-litert-lm-local-tab-fix-and-remaining-device-proof-gap
- knowme-mode-picker-fixes-and-unresolved-device-certification
- ios-local-inference-switchover-to-mlx-swift-via-c-vtable
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-26T07:26:13.587147+00:00
created_at: 2026-07-26T07:26:13.587147+00:00
updated_at: 2026-07-26T07:26:13.587147+00:00
revision: 0
---

## Context

- **Project:** KnowMe
- **Phase:** `model-runtime-per-device`
- **Captured:** `2026-07-26T07:11:00Z`
- **KBD root:** `/Users/gqadonis/Projects/know-me/know-me-system`
- **Workstream:** desktop stable port / Android local runtime certification
- **Status:** release APK still building; fix covered by tests but not yet verified on Samsung device

Phase goals remain aligned with the broader local-runtime effort described in [Android LiteRT-LM Local tab fix and remaining device proof gap](/android-litert-lm-local-tab-fix-and-remaining-device-proof-gap.md) and subsequent mode-picker work in [KnowMe mode picker fixes and unresolved device certification](/knowme-mode-picker-fixes-and-unresolved-device-certification.md):

- Ship Android local text-agent inference through LiteRT-LM Gemma 4 E2B, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.
- The older iPhone Qwen3.5-9B target is superseded by the MLX-Swift direction documented in [iOS local inference switchover to MLX-Swift via C vtable](/ios-local-inference-switchover-to-mlx-swift-via-c-vtable.md).

## Root cause

The Local/UAR mode selector appeared silent because the Rust FFI accessor returned the wrong JSON shape.

`uar.rs::conversation_policy` returned the storage wrapper:

```json
{
  "conversation_id": "...",
  "policy": {
    "chat_mode": "..."
  },
  "updated_at": "..."
}
```

Every consumer expected the bare `RunPolicy` shape:

```json
{
  "chat_mode": "..."
}
```

Affected expectations:

- `save_conversation_policy` deserializes its input directly as `RunPolicy`.
- The TypeScript binding declares `Promise<RunPolicy | null>`.
- Flutter assigns the returned object directly into its policy map and reads `policy['chat_mode']`.

Because Flutter received the wrapper, `policy['chat_mode']` was absent and the UI fell back to the default mode. When Flutter later sent the wrapper back to save, Rust deserialized it as `RunPolicy`; since every `RunPolicy` field has `#[serde(default)]`, deserialization succeeded with defaults instead of failing. The write then persisted `chat_mode: "uar"`, silently overwriting the user's selected mode.

This explains the absence of logcat errors: the defect produced a structurally valid default policy, so there was no exception or rejected promise to log.

## Fix

The fix was applied at the accessor boundary rather than in Flutter or React clients:

- `uar.rs::conversation_policy` now returns the bare `RunPolicy`, matching all client contracts.
- React already assumed the bare shape.
- The HTTP path already returned the bare shape.
- Patching Flutter alone would have left the desktop/React bug latent.

Supporting changes:

- Extracted `chat_mode_lane` as a pure function so mode resolution can be tested without live I/O.
- Dart now layers stored policies over defaults so policies written before new fields existed still round-trip as complete `RunPolicy` values rather than sparse patches.

## Regression coverage

New tests explicitly assert that wrapper-shaped data must not be interpreted as a valid mode:

- Rust: `conversation_chat_mode_is_read_from_the_bare_run_policy`
- Dart: `chat_control_policy_shape_test`

Current test status:

- Rust: `72/72` passing.
- Flutter: `31` passing, `3` failing; the failures are unchanged pre-existing baseline failures.

## Pending device verification

The reasoning is backed by traced FFI shape evidence and tests, but hardware certification is still pending. The release APK was still building at capture time.

Required Samsung verification steps:

1. Install the new release APK on the Samsung device.
2. Select **Local**.
3. Confirm the UI label changes and survives app restart.
4. Send a message and verify `send_local` markers.
5. Confirm a second conversation maintains its own independent mode.

The issue is not considered fully closed until those device checks pass. iOS certification remains blocked until the iPhone is reattached.

## Desktop stable port status

Prometheus position at capture:

- **Position:** `model-runtime-per-device › desktop-stable-port › desktop-stable-port`
- **Status:** `executing`
- **Progress:** changes `4/5`
- **Completed:**
  - macOS deployment floor raised to `14.0`.
  - `mlex` / `mlx-c` lane compiles for the first time.
  - `MlxcEngine` certified.
  - Per-target engine matrix verified by full `tauri-plugin` build:
    - macOS: MLX
    - Windows/Linux: `llama.cpp`
- **Remaining:**
  - Runtime certification: launch desktop app and complete an agent turn.
  - Windows/Linux CI cross-build.

# Citations

1. stdin
2. manual:KnowMe/model-runtime-per-device