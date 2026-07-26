---
type: Reference
id: local-system-prompt-ui-and-ios-runtime-verification-status
title: Local system prompt UI and iOS runtime verification status
tags:
- model-runtime-per-device
- local-system-prompt
- settings-ui
- accessibility
- ios-device
- template-exception
- uar
- litert-lm
links:
- memory-fix-install-watch-armed-for-unplug-safe-ios-verification
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-26T08:37:32.372516+00:00
created_at: 2026-07-26T08:37:32.372516+00:00
updated_at: 2026-07-26T08:37:32.372516+00:00
revision: 0
---

## Phase context

- Phase: `model-runtime-per-device`
- Project: `KnowMe`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system`
- Captured: `2026-07-26T08:30:20Z`
- Position: `model-runtime-per-device › litertlm-device-proof`
- Status: `executing`
- Progress: changes `4/5`

## Phase goals

- Ship `Qwen3.5-9B Q4_K_M` with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM `Gemma 4 E2B`, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Implemented: DB-backed local system prompt settings UI

Committed in `ff7d800`.

Settings now includes a **Local system prompt** card in **Settings → Local models**, colocated with the model it governs:

- Text area seeded with the effective prompt.
- **Save prompt** button.
- **Reset to default** button.

### Design decisions

- **Reset sources the default from Rust** via `defaultLocalSystemPrompt()` instead of hardcoding the default prompt text in Dart. This prevents Dart/Rust prompt drift when the default changes.
- **Save reads the effective value back** rather than echoing the submitted text. Clearing the editor is the documented restore-default path, so read-back is required to show the restored default instead of leaving an empty editor visible.
- The prompt read is intentionally **outside** the existing UAR `try/catch`. The existing block swallows failures so a cold UAR kernel does not break Settings; the system prompt is only a local settings read and should not be silently blanked by an unrelated UAR service failure.

## Accessibility fixes

The accessibility gate caught two regressions introduced by the new UI:

- `TextArea` had no accessible name. A placeholder is not sufficient because it disappears after text entry, and the visible heading was not programmatically associated.
- Buttons lacked explicit accessible names.

Fixes:

- Added `Semantics` labels.
- Used `excludeSemantics` so controls announce once instead of duplicating labels, e.g. avoiding `Save local system prompt, Save prompt`.

## Tests

Added 4 tests covering prompt round-trip behavior:

- Saved prompt reads back verbatim.
- Clearing the prompt restores the default.
- Reset sources its copy from Rust.
- Unset prompt reports the default.

Suite status:

- `35` passing.
- `3` failing; these are pre-existing settings-workspace baseline failures.

The `TemplateException` fix is test-covered by 4 context tests and was traced to three specific mid-list `System` injectors, but remains pending device verification.

## iOS runtime verification state

- iPhone 17 Pro Max is connected.
- First iOS build was killed because it started before the UI edits and would have installed a stale binary.
- Current iOS release rebuild includes the local system prompt UI changes and is expected to take about 8 minutes.
- After install, operator should:
  - Launch the app.
  - Retry the conversation that previously threw `TemplateException`.
  - Open **Settings → Local models** and verify prompt editing.

This continues the iOS device runtime verification lane after the install-watch flow described in [Memory-fix install watch armed for unplug-safe iOS verification](/memory-fix-install-watch-armed-for-unplug-safe-ios-verification.md).

## Open issues

- **Crash fix not device-verified:** the `TemplateException` fix requires retrying the same failing conversation on-device.
- **Durable persistence unresolved:** the persistence session landed no commits. `InMemoryProvider` at `rust/crates/gen_ui_agent/src/uar.rs:878` likely remains, so per-conversation mode may still reset on restart.
- **Android certification blocked:** Samsung device remains `adb`-unauthorized, blocking change 5 certification suite.

# Citations

1. stdin
2. manual:KnowMe/model-runtime-per-device