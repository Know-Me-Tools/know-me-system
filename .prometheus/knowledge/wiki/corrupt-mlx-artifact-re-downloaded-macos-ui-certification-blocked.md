---
type: Reference
id: corrupt-mlx-artifact-re-downloaded-macos-ui-certification-blocked
title: Corrupt MLX artifact re-downloaded; macOS UI certification blocked
tags:
- model-runtime-per-device
- mlx
- tauri
- macos-certification
- resumable-downloads
- artifact-integrity
- local-inference
links:
- memory-fix-install-watch-armed-for-unplug-safe-ios-verification
- ios-cert-monitor-holding-for-local-send-with-benign-nw-path-noise
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-27T05:38:50.212033+00:00
created_at: 2026-07-27T05:38:50.212033+00:00
updated_at: 2026-07-27T05:38:50.212033+00:00
revision: 0
---

## Phase context

- Phase: `model-runtime-per-device`
- Project: `KnowMe`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system`
- Captured: `2026-07-27T05:30:45Z`
- Position: `model-runtime-per-device › embedded-uar-offline-agents › litertlm-device-proof`
- Status: `executing`

## Phase goals

- Ship `Qwen3.5-9B Q4_K_M` with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM `Gemma 4 E2B`, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Artifact corruption finding

A partial MLX download was inspected before deletion and was confirmed corrupt, not merely incomplete:

| Offset | Result |
|---|---|
| `0–8MB` | match |
| `512MB` | match |
| `768MB` | match |
| `1024MB` | mismatch |
| tail | mismatch |

This matters because `mlx_download.rs:100` resumes downloads using HTTP Range, and the server advertises `accept-ranges: bytes`. Resuming the corrupted partial would have appended valid bytes onto invalid existing bytes and only failed after another ~1.1 GB download when the final checksum was computed.

## Verified replacement artifact

The artifact was deleted, re-downloaded, and verified against the catalog pin:

```text
size:   2263022529   ✓ exact
sha256: e240c0bdc0ebb0681bf0da0f98d9719fd6ebe269a3633f81542c13e81345651d   ✓ exact
```

Artifact state:

- Full artifact set present.
- No `.part` files remain.
- Original model artifact blocker is cleared.

## macOS desktop certification blocker

The desktop app is running cleanly for 3+ minutes with no Rust/runtime errors, but UI automation cannot drive it.

Root cause:

- The `tauri dev` binary runs without a bundle identifier.
- The screen-control tool requires a matching installed or running application identity.
- Error observed: `doesn't match any installed or running application`.

Attempted workaround:

- Packaged the debug binary into a minimal signed `.app` to provide an identity.
- The app exited immediately on launch because the debug build depends on environment supplied by `tauri dev`.
- The workaround was removed and the working `tauri dev` session restored.

## Current verification status

Completed:

- Vitest: `52/52` passing.
- TypeScript: `tsc` clean.
- Rust tests: `7/7 + 15/15` passing.
- Commit recorded: `44ee525`.

Still open:

- macOS desktop certification: stop mid-stream, record kept chars and rendered `Cancelled by user`, then Resume and Steer, and re-check the tool UI.
- Windows/Linux `llama.cpp` end-to-end verification.
- Browser WebLLM end-to-end verification.
- Attachment-flow certification.
- iOS airplane-mode task `5.5`; this follows the broader iOS local-runtime verification lane documented in [Memory-fix install watch armed for unplug-safe iOS verification](/memory-fix-install-watch-armed-for-unplug-safe-ios-verification.md) and earlier certification monitors such as [iOS cert monitor holding for Local send with benign nw_path noise](/ios-cert-monitor-holding-for-local-send-with-benign-nw-path-noise.md).

## Operator decision needed

Two available paths to finish macOS certification:

1. **Operator drives existing app session**
   - App is already open.
   - Select Local.
   - Send a long prompt.
   - Hit Stop mid-stream.
   - Resume and Steer.
   - Evidence will be read from Rust logs.

2. **Build a release bundle**
   - Run `pnpm tauri:build`.
   - Produces a properly identified `.app` for automation.
   - Cost: full optimized Rust compile.

# Citations

1. stdin
2. manual:KnowMe/model-runtime-per-device