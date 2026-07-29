---
type: Reference
id: android-litert-lm-certification-queued-after-device-authorization
title: Android LiteRT-LM certification queued after device authorization
tags:
- model-runtime-per-device
- android-device-cert
- litert-lm
- gemma
- adb
- local-inference
- certification
links:
- ios-cert-monitor-holding-for-local-send-with-benign-nw-path-noise
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-26T16:25:09.510454+00:00
created_at: 2026-07-26T16:25:09.510454+00:00
updated_at: 2026-07-26T16:25:09.510454+00:00
revision: 0
---

## Phase context

- Phase: `model-runtime-per-device`
- Project: `KnowMe`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system`
- Captured: `2026-07-26T16:02:12Z`
- Position: `model-runtime-per-device › embedded-uar-offline-agents › litertlm-device-proof`
- Status: `executing`

## Goals

- Ship `Qwen3.5-9B Q4_K_M` with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM `Gemma 4 E2B`, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Current state

- Samsung `SM-S936U` is authorized; `adb` reports `R5GYB4AZD7A device` and shell responds.
- Device authorization cleared the blocker for 7 of 9 evidence items.
- Release APK is rebuilding with local inference performance instrumentation from commit `c92fc80`:
  - `load_ms`
  - `generate_ms`
  - `chars_per_second`
- Rust cross-compile is active and expected to take several minutes; monitoring should report completion rather than polling.
- `GeneratedPluginRegistrant` was removed before rebuilding to avoid `integration_test` contamination that caused an earlier build failure.
- 14 processes were active at capture time.

## Certification automation

The Android certification scripts are fully self-driving and use `uiautomator dump` plus `adb shell input tap/text`; no operator taps are required after APK installation.

Planned suite:

- `certify-local-model-prompt` — exact-response check.
- `certify-local-cancellation` — mid-generation cancellation.
- `certify-local-tool-call` — permitted tool call.
- `certify-local-prompt-suite` — prompt cases: `exact`, `office`, `mermaid`, `code`, `json`; covers Markdown/Mermaid requirements.
- `write-local-certification-report.sh` — records backend, memory, thermal, and instrumentation markers:
  - `load_ms`
  - `generate_ms`
  - `chars_per_second`

## Evidence discipline

A prior status marked this change `COMPLETE`, but that evidence was void because the Android release path was crashing at bridge load when the claim was made. The suite results must be reported exactly as returned, including any failures; completion must not be inferred merely because the suite ran.

## Remaining open items

These remain open regardless of Android certification outcome:

- Attachment flow has no certification script yet and requires real scripting/work.
- iOS airplane-mode certification, task `5.5`, has not been run. This is separate from the existing iOS local-send certification waiting/monitoring lane such as [iOS cert monitor holding for Local send with benign nw_path noise](/ios-cert-monitor-holding-for-local-send-with-benign-nw-path-noise.md).

## Next steps

1. Install the fresh release APK after the rebuild completes.
2. Run the full Android local certification suite.
3. Report each script's actual result.
4. Keep attachment-flow scripting and iOS airplane-mode certification open after the suite.

# Citations

1. stdin
2. manual:KnowMe/model-runtime-per-device