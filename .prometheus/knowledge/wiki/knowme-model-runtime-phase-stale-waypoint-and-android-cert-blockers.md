---
type: Reference
id: knowme-model-runtime-phase-stale-waypoint-and-android-cert-blockers
title: 'KnowMe model-runtime phase: stale waypoint and Android cert blockers'
tags:
- knowme
- model-runtime-per-device
- litert-lm
- surrealdb
- android-certification
- ios-certification
- uar
- stale-waypoint
links:
- ios-cert-monitor-holding-for-local-send-with-benign-nw-path-noise
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-26T16:02:41.696244+00:00
created_at: 2026-07-26T16:02:41.696244+00:00
updated_at: 2026-07-26T16:02:41.696244+00:00
revision: 0
---

## Phase context

- Project: `KnowMe`
- Phase: `model-runtime-per-device`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system`
- Captured: `2026-07-26T15:51:36Z`
- Current position: `model-runtime-per-device › embedded-uar-offline-agents › litertlm-device-proof`
- Status: `executing`
- Progress: changes `4/5`

## Phase goals

- Ship `Qwen3.5-9B Q4_K_M` with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM `Gemma 4 E2B`, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Stale waypoint reconciliation

`/opsx:new desktop-stable-port` was intentionally **not** run.

Reason: `desktop-stable-port` existed only in `current-waypoint.json`, which was stale since `2026-07-22`. It had no corresponding phase directory, OpenSpec change, plan reference, or execution-doc reference. The authoritative phase record was written on `2026-07-25` and disagreed with the waypoint:

| Field | Stale waypoint `2026-07-22` | Phase record `2026-07-25` |
|---|---|---|
| active change | `desktop-stable-port` | `litertlm-device-proof` |
| childPointer | `desktop-stable-port` | `embedded-uar-offline-agents` |

Running the hook command would have materialized a dangling change under a non-existent phase directory. Instead, `current-waypoint.json` was reconciled to `progress.json` and committed as `7e06324`.

Validation performed:

- Cross-checked fields against `progress.json`.
- Preserved a backup.
- Confirmed this explained the session mismatch where the position line continued to show `desktop-stable-port` while actual work was under `litertlm-device-proof`.
- Verified the UAR submodule was already current: `08a6f4c`, `0 ahead / 0 behind`, newest non-Dependabot remote commit.

## Durable persistence finding

A complete `SurrealDbProvider` already exists:

- 1343 lines.
- Implements all 45 `PersistenceLayer` methods.
- Gated behind the `surreal-backend` feature.
- Constructor explicitly supports embedded endpoints, including SurrealKV and in-memory modes.
- Embedded endpoints require no auth, matching the on-device persistence case.

This suggests durable persistence task `3.1` may be wiring work rather than new provider implementation.

Pending verification before wiring:

1. Confirm `normalize_endpoint` accepts a plain sandboxed file path.
2. Confirm `surreal-backend` is buildable from KnowMe mobile targets without pulling server-only dependencies.

If both checks pass, wire `build_local` at:

```text
rust/crates/gen_ui_agent/src/uar.rs:880
```

Target mobile build checks:

- `aarch64-linux-android`
- `aarch64-apple-ios`

## Change 5 Android certification blocker

Change 5 is blocked by Samsung USB-debugging authorization.

Required physical action on Samsung:

1. Accept the **Allow USB debugging?** dialog.
2. Tick **Always allow from this computer**.
3. Verify authorization:

```bash
adb devices
```

Expected output state:

```text
R5GYB4AZD7A	device
```

Blocked state:

```text
R5GYB4AZD7A	unauthorized
```

Once authorized, run:

```bash
cd mobile && flutter build apk --release && adb install -r build/app/outputs/flutter-apk/app-release.apk
```

Then run the Android certification suite under `scripts/android/`:

- `certify-local-model-prompt`
- `certify-local-cancellation`
- `certify-local-tool-call`
- `certify-local-prompt-suite`
- `write-local-certification-report.sh`

The certification report now harvests instrumentation fields:

- `load_ms`
- `generate_ms`
- `chars_per_second`

Accepting the USB-debugging dialog unblocks 7 of 9 evidence items.

## Remaining work not solved by device authorization

Two evidence items still require implementation or execution even after Samsung authorization:

- Attachment flow has no certification script.
- iOS airplane-mode certification task `5.5` has not been run. This is adjacent to prior iOS local-send certification waiting states such as [iOS cert monitor holding for Local send with benign nw_path noise](/ios-cert-monitor-holding-for-local-send-with-benign-nw-path-noise.md), but this session records no airplane-mode run.

## Immediate next actions

1. Verify `normalize_endpoint` accepts a sandboxed file path.
2. Verify `surreal-backend` builds for `aarch64-linux-android` and `aarch64-apple-ios`.
3. If both pass, wire `build_local` to durable SurrealKV at `uar.rs:880`.
4. Wait for Samsung USB-debugging authorization, then build, install, and run Android certification scripts.
5. Add missing attachment-flow certification script.
6. Run iOS airplane-mode certification task `5.5`.

# Citations

1. stdin
2. manual:KnowMe/model-runtime-per-device