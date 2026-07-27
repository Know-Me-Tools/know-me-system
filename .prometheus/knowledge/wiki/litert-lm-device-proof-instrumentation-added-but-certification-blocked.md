---
type: Reference
id: litert-lm-device-proof-instrumentation-added-but-certification-blocked
title: LiteRT-LM device proof instrumentation added but certification blocked
tags:
- knowme
- model-runtime-per-device
- litert-lm
- android
- device-certification
- proof-gap
- ios
links:
- knowme-model-runtime-per-device-phase-status-and-proof-gaps
sources:
- stdin
timestamp: 2026-07-26T07:59:45.405939+00:00
created_at: 2026-07-26T07:59:45.405939+00:00
updated_at: 2026-07-26T07:59:45.405939+00:00
revision: 0
---

## Context

Phase: `model-runtime-per-device` for KnowMe. Current position: `model-runtime-per-device › litertlm-device-proof`; status: `executing`; progress: `changes 4/5`.

The phase goals remain aligned with [KnowMe model-runtime-per-device phase status and proof gaps](/knowme-model-runtime-per-device-phase-status-and-proof-gaps.md):

- Ship Qwen3.5-9B Q4_K_M with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM Gemma 4 E2B, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Status corrections

Two previously reported tracking defects were retracted:

1. `progress.json` is valid JSON; `python3 -m json.tool` parses it successfully. The earlier malformed-JSON claim came from a quoting failure in inline Python.
2. The child phase does have a `progress.json`; it is located at `phases/model-runtime-per-device/children/embedded-uar-offline-agents/`, not the top-level `phases/` directory.

`execution.md` had claimed `litertlm-device-proof` was completed, but `progress.json` `status_corrections` had already reverted it to `not_completed`. The original completion evidence was invalid because the Android release path crashed at bridge load until 2026-07-25, so certification could not have been obtained. The stale ledger was reconciled to match the correction.

## Evidence audit for `litertlm-device-proof`

Nine required evidence items were audited against the existing Android certification harness:

| Required evidence | Current coverage |
|---|---|
| Backend | Covered by `write-local-certification-report.sh` |
| Memory | Covered by `write-local-certification-report.sh` |
| Thermal | Covered by `write-local-certification-report.sh` |
| Cancellation | Covered by `certify-local-cancellation.sh` |
| Markdown/Mermaid | Covered by `certify-local-prompt-suite.sh` |
| Permitted tool call | Covered by `certify-local-tool-call.sh` |
| Load time | Added: `load_ms` captured on adapter load |
| Throughput | Added: `generate_ms`, `chars`, `chars_per_second` |
| Attachment flow | Not covered; no certification script yet |

## Instrumentation changes

Commit: `c92fc80`.

Changes made for LiteRT-LM Android local certification:

- Added adapter-load timing via `load_ms`.
- Added generation timing and throughput markers:
  - `generate_ms`
  - `chars`
  - `chars_per_second`
- Updated `write-local-certification-report.sh` to harvest:
  - timing markers,
  - throughput markers,
  - observed model id,
  - hardcoded `expectedBackend`.
- The report now exposes mismatches between observed model id and expected backend instead of assuming correctness.
- Missing markers remain `null`, not `0`, so a run that never generated cannot appear to have a valid zero-valued measurement.

Rationale: previous engine output only logged `chars=9` with no duration, so throughput was not derivable from existing output.

## Verification completed

Verified before device access was lost:

- Compiles for `aarch64-linux-android`.
- Compiles for host.
- Bash syntax is valid.
- Embedded Python syntax is valid.
- Parser was tested against real Samsung SM-S936U log lines, not invented samples:
  - `modelLoadMs: 4821`
  - `generateMs: 4609`
  - `charsPerSecond: 1`

## Current blockers

### Samsung Android certification blocked by USB authorization

The Samsung device disconnected mid-work and returned as `adb unauthorized`. The phone is showing an `Allow USB debugging?` prompt that requires a physical tap.

Required action:

- Tap **Allow** on the USB debugging dialog.
- Enable **Always allow from this computer**.

After authorization, the next step is to reinstall the instrumented APK and run the full certification suite to capture all nine required evidence items.

### Attachment-flow proof missing

Attachment flow has no certification script yet. This remains open independently of device authorization.

### iOS local mode uncertified

The iOS app is installed and running on iPhone 17 Pro Max with PID `26732`, but Local mode remains uncertified. `devicectl` provides no screen control, so the iOS path also requires physical tap-through.

## Next actions

1. Authorize USB debugging on the Samsung device.
2. Reinstall the instrumented Android APK.
3. Run the full Android certification suite and capture all nine evidence items.
4. Add an attachment-flow certification script.
5. Complete iOS Local-mode tap-through certification on iPhone 17 Pro Max.

# Citations

1. stdin