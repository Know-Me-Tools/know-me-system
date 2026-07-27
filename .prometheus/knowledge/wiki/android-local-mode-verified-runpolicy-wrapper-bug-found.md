---
type: Reference
id: android-local-mode-verified-runpolicy-wrapper-bug-found
title: Android Local mode verified; RunPolicy wrapper bug found
tags:
- model-runtime-per-device
- android-local-inference
- litert-lm
- runpolicy
- uar
- persistence
- accessibility
links:
- memory-fix-install-watch-armed-for-unplug-safe-ios-verification
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-26T07:38:10.176161+00:00
created_at: 2026-07-26T07:38:10.176161+00:00
updated_at: 2026-07-26T07:38:10.176161+00:00
revision: 0
---

## Phase context

- Phase: `model-runtime-per-device`
- Project: `KnowMe`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system`
- Captured: `2026-07-26T07:25:48Z`
- Position: `model-runtime-per-device › desktop-stable-port › desktop-stable-port`
- Status: `executing`

## Phase goals

- Ship `Qwen3.5-9B Q4_K_M` with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM `Gemma 4 E2B`, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Verified Android result

Local mode now functions end-to-end on Samsung `SM-S936U`.

### UI selection applies

- Tapping **Local** in the sheet changes the trigger label to `Local`.
- Active state renders as an orange pill with chip icon.
- Verified through `uiautomator` with `content-desc="Local"` and screenshot evidence.
- This is the first verified case where the trigger label changed correctly.

### Routing is genuinely local

Sending a message produced LiteRT-LM logs:

```text
LiteRT-LM native adapter loaded model=gemma-4-e2b-it-litertlm
LiteRT-LM generation starting
LiteRT-LM generation produced text chars=9 tool_calls=0
```

Important absence:

- No `Universal Agent Runtime run` line appeared.
- The send bypassed UAR and hit the on-device Gemma model directly.
- The local model response rendered in the thread.

### Agent list is populated

- The sheet lists `Orchestrator` and `Default Assistant` from `metadata.title`.
- The sheet also lists `Local`.
- This confirms the catalog-shape fix.

### Accessibility state is correct

- Active row exposes `selected="true"`.
- Inactive rows expose `selected="false"`.
- Rows are `clickable="true"` and at least 48 px.

## Root cause

`uar.rs::conversation_policy` returned the storage wrapper:

```text
{ conversation_id, policy, updated_at }
```

Every consumer expected the bare `RunPolicy`.

Because every `RunPolicy` field has `#[serde(default)]`, feeding the wrapper back into save deserialized successfully but defaulted all fields. The write then succeeded and silently overwrote the selection.

Impact:

- No error surfaced in logcat.
- Selection writes appeared to work but were reset to defaults.
- Earlier theories were disproven:
  - R8 issue: not root cause.
  - `NoLocalEngine`: not root cause.
  - `patch()` clobbering: not root cause.

## Remaining persistence gap

Local mode works during the active app session, but selection does not survive app restart.

Observed after force-stop and relaunch:

- Same thread reloads.
- Messages remain present.
- Trigger label reverts to `Orchestrator`.

Current conclusion:

- In-session behavior is fixed.
- Local routing is correct.
- Policy persistence or reload still has a defect.

Instrumentation added on both sides of the Rust policy round-trip:

- `conversation policy saving`
- `conversation policy saved`
- `conversation policy loaded`
- Includes `chat_mode` value.

Next diagnostic target: determine whether the policy write reaches persistence or the read path discards/resets it.

## Build/reporting corrections

- A previously reported successful APK build had actually failed.
- Failure:

```text
GeneratedPluginRegistrant.java:44: package dev.flutter.plugins.integration_test does not exist
```

- Cause: `flutter test` was run concurrently, regenerating the registrant with a dev-only plugin.
- The APK timestamp that was inspected was stale.
- The `patch()`-clobbering theory was wrong because the toolbar only renders when `threadId` is non-empty; therefore the empty-id early return never fires.

## Next steps

1. Capture instrumented `conversation policy saving/saved/loaded` logs.
2. Fix the restart persistence gap.
3. Re-verify:
   - restart survival,
   - per-conversation policy isolation.
4. Resume iOS certification after the iPhone is reattached; the iOS lane remains blocked similarly to [Memory-fix install watch armed for unplug-safe iOS verification](/memory-fix-install-watch-armed-for-unplug-safe-ios-verification.md).

## Desktop stable-port status

- Progress: changes `4/5`.
- macOS deployment floor raised to `14.0`.
- `mlex` / `mlx-c` lane compiles for the first time.
- `MlxcEngine` certified.
- Per-target engine matrix verified by full `tauri-plugin` build:
  - macOS: MLX
  - Windows/Linux: llama.cpp

Remaining desktop work:

- Runtime certification: launch desktop app and complete an agent turn.
- Windows/Linux CI cross-build.

# Citations

1. stdin
2. manual:KnowMe/model-runtime-per-device