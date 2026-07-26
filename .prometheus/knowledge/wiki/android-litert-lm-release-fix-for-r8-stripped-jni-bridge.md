---
type: Reference
id: android-litert-lm-release-fix-for-r8-stripped-jni-bridge
title: Android LiteRT-LM release fix for R8-stripped JNI bridge
tags:
- model-runtime-per-device
- android
- litert-lm
- r8
- jni
- release-build
- local-inference
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-25T23:03:59.557549+00:00
created_at: 2026-07-25T23:03:59.557549+00:00
updated_at: 2026-07-25T23:03:59.557549+00:00
revision: 0
---

## Phase context

- Phase: `model-runtime-per-device`
- Project: `KnowMe`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system`
- Captured: `2026-07-25T23:01:01Z`
- Position: `model-runtime-per-device › android-litertlm-runtime › device-verify`
- Status: `awaiting_operator`
- Device lane: Samsung Android LiteRT-LM local inference certification

## Phase goals

- Ship `Qwen3.5-9B Q4_K_M` with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM `Gemma 4 E2B`, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Summary

- Fixed, verified, committed, and pushed the Android LiteRT-LM release fix: `7330389`.
- Corrected APK is installed and running on Samsung.
- App PID: `21155`.
- Device log monitor: `boypxki0g` armed.
- Verification is blocked on operator prompts in Orchestrator/Agent, Local tab, and tool-call flow.

## Root cause

Flutter release variants run **R8 by default**, even when `minifyEnabled` is not explicitly present in `build.gradle.kts`.

Earlier diagnosis incorrectly ruled out R8 because no explicit minification configuration was visible. Build intermediates proved otherwise:

- `minifyReleaseWithR8` ran for the release variant.
- The only generated keep rule before the fix was for `MainActivity` from the manifest.
- The LiteRT-LM bridge is reached only from Rust through JNI reflection by fully qualified class name.
- Because there was no statically visible Java/Kotlin reference, R8 treated the bridge as unused and removed it.
- Debug builds worked because R8 does not run there, allowing the issue to survive until hardware release testing.

## Proof of fix

Dex inspection established the failure and correction:

- Before fix: release dex contained only `MainActivity`.
- After fix: release dex contains:
  - `KnowMeLiteRtLmBridge`
  - `KnowMeLiteRtLmBridge$RegisteredOpenApiTool`

This proves the JNI-reflected bridge and nested tool-registration class now survive release shrinking.

## Runtime paths unblocked

Both Android local inference paths share the same LiteRT engine and are unblocked by the bridge keep fix:

- **Orchestrator/Agent path**: UAR → local driver → LiteRT engine.
- **Local tab path**: `send_local` → LiteRT engine.

The `send_local` path also exercises the tool-shortlist plumbing change.

## Required Samsung verification

### 1. Orchestrator/Agent

Operator should resend the same prompt that previously failed.

Expected result:

- Error box is gone.
- A real streamed answer appears.
- First run may download the approximately `3 GB` Gemma artifact, so a long wait is expected.

### 2. Local tab

Operator should navigate:

`On-device` → `Local` → send a prompt.

Expected result:

- `send_local` executes successfully.
- LiteRT engine returns a local response.
- Tool-shortlist changes are exercised on this path.

### 3. Tool call

Operator should ask for a tool-requiring action, for example:

```text
search my memory for the truck
```

Expected result:

- Tool shortlist is generated.
- Runtime gate evaluates the tool call.
- Tool executes when allowed.
- Model continues after tool output.
- Mutating tools request confirmation before execution.

## Current state

- Commit `7330389` contains the fix and has been pushed.
- Fixed APK is installed on Samsung.
- App is running with PID `21155`.
- PID-scoped logs are being watched for verification markers.
- Next action: operator performs the three Samsung verification prompts above.

# Citations

1. stdin
2. manual:KnowMe/model-runtime-per-device