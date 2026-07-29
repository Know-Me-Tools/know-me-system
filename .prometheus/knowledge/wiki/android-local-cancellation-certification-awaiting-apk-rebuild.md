---
type: Reference
id: android-local-cancellation-certification-awaiting-apk-rebuild
title: Android local cancellation certification awaiting APK rebuild
tags:
- model-runtime-per-device
- android-device-cert
- local-cancellation
- litert-lm
- tool-calls
- logcat
links:
- memory-fix-install-watch-armed-for-unplug-safe-ios-verification
sources:
- stdin
timestamp: 2026-07-26T17:49:29.476214+00:00
created_at: 2026-07-26T17:49:29.476214+00:00
updated_at: 2026-07-26T17:49:29.476214+00:00
revision: 0
---

## Phase context

- Phase: `model-runtime-per-device`
- Project: `KnowMe`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system`
- Captured: `2026-07-26T17:39:29Z`
- Position: `model-runtime-per-device › embedded-uar-offline-agents › litertlm-device-proof`
- Status: `executing`
- Monitor: `bbtehaclq` armed

## Phase goals

- Ship `Qwen3.5-9B Q4_K_M` with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM `Gemma 4 E2B`, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Current state

The Android local-cancellation lane is waiting for an APK rebuild carrying the aligned **"Cancelled by user"** wording from commit `450de77`. Monitor `bbtehaclq` is armed and should notify when the build lands.

Certification is blocked until the rebuilt APK is available. Attachment-flow scripting and iOS airplane-mode certification task `5.5` remain open. The broader iOS local-inference workstream continues alongside prior device certification and install-watch work such as [Memory-fix install watch armed for unplug-safe iOS verification](/memory-fix-install-watch-armed-for-unplug-safe-ios-verification.md).

## Planned certification sequence

Run the checks in this order after the build completes:

1. Run `certify-local-cancellation.sh` and record the script verdict.
2. Check `logcat` for the required cancellation classification evidence:
   - `RunCancelled` **present**
   - `generation failed` **absent**
3. Manually stop generation mid-tool-call and verify:
   - the tool block remains preserved on screen
   - the steer bar reports the kept block count

The `logcat` pairing is the proof of the classification fix. A script pass alone is insufficient because the bug under test is cancellation being classified as an error.

## UI assertion caveat

`certify-local-cancellation.sh` passes by grepping the literal string:

```text
Cancelled by user
```

The steer bar now renders this inside a longer status line:

```text
Cancelled by user — kept N blocks, still in context
```

This should still satisfy the script because the check uses a fixed-string substring match:

```sh
rg -F -q "Cancelled by user"
```

Wiring has been verified at code level: `markRunCancelled` assigns `statusMessage: event.message`. However, this exact rendering has not yet been observed on device. If the script fails only because the literal UI-dump assertion does not match the rendered longer line, treat it as a **test assertion mismatch**, not a product regression.

## Expected reporting

Report each result as it arrives, including failures:

- `certify-local-cancellation.sh` verdict
- `logcat` evidence: `RunCancelled` present and `generation failed` absent
- manual mid-tool-call stop result: preserved tool block and kept-count steer bar

# Citations

1. stdin