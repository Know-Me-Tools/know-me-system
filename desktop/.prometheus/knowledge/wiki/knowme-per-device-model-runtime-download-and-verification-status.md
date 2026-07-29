---
type: Reference
id: knowme-per-device-model-runtime-download-and-verification-status
title: KnowMe Per-Device Model Runtime Download and Verification Status
tags:
- knowme
- model-runtime
- per-device
- resumable-downloads
- sha256-verification
- litert-lm
- qwen
- tauri
links:
- model-runtime-per-device-phase-empty-execution-status
- model-runtime-per-device-executor-completion-marker
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-27T05:31:04.938784+00:00
created_at: 2026-07-27T05:31:04.938784+00:00
updated_at: 2026-07-27T05:31:04.938784+00:00
revision: 0
---

## Context

- **Project:** `KnowMe`
- **Phase:** `model-runtime-per-device`
- **Position:** `model-runtime-per-device › embedded-uar-offline-agents › litertlm-device-proof`
- **Status:** `executing`
- **KBD root:** `/Users/gqadonis/Projects/know-me/know-me-system`
- **Captured:** `2026-07-27T05:24:17Z`

This is an active execution checkpoint for the `model-runtime-per-device` phase, following earlier phase-status records such as [Model Runtime Per Device Phase Empty Execution Status](/model-runtime-per-device-phase-empty-execution-status.md) and [Model Runtime Per Device Executor Completion Marker](/model-runtime-per-device-executor-completion-marker.md).

## Phase goals

- Ship **Qwen3.5-9B Q4_K_M** with matching `mmproj` vision support on **iPhone 17 Pro Max** using Metal.
- Ship Android local text-agent inference through **LiteRT-LM Gemma 4 E2B**, with CPU as the universal correctness baseline.
- Enable:
  - Device-aware model manifests
  - Resumable downloads
  - Integrity checks
  - Runtime gate diagnostics
  - Safe memory/thermal fallback
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Downloader behavior and partial-file decision

The model downloader supports HTTP resume through byte ranges:

- Implementation reference: `mlx_download.rs:100`
- Server capability: `accept-ranges: bytes`

A partial download should not be deleted merely because it is incomplete; resuming can preserve already-downloaded bytes. However, the existing partial was validated against the server and found to be corrupt, not just incomplete.

### Partial validation results

| Offset checked | Result |
|---|---|
| First 8 MB | match |
| 512 MB | match |
| 768 MB | match |
| 1024 MB | mismatch |
| Tail, last 4 MB | mismatch |

The corruption begins somewhere between **768 MB** and **1024 MB**. Resuming from byte `1,149,317,551` would have appended valid data after corrupt data, producing a file that would fail the catalog SHA-256 check after another ~1.1 GB of transfer.

**Decision:** delete and re-download the partial. The reason is data corruption, not staleness.

## Re-download parameters

- **Pinned revision:** `4dcb3d10`
- **Expected target size:** `2,263,022,529` bytes
- **Catalog byte-length check:** matches expected target size
- **Server `content-length`:** matches expected target size
- **Catalog SHA-256 pin:** `e240c0bd…`

The remaining seven files in the model set were already complete and were left untouched:

- `config`
- `tokenizer`
- `index`
- `vocab`
- `merges`
- Other associated non-model-weight files in the set

## Required verification after download

After the re-download completes:

1. Verify SHA-256 against catalog pin `e240c0bd…`.
2. Run `tauri:dev`.
3. Confirm the **Local** lane is selected.
4. Stop a generation mid-stream and record:
   - Characters retained
   - Rendered `Cancelled by user` state
5. Exercise **Resume**.
6. Exercise **Steer**.
7. Re-check the tool UI because the status change touches every message.

## Still unverified / open

- Windows/Linux `llama.cpp` end-to-end path.
- Browser WebLLM end-to-end path.
- Attachment-flow certification.
- iOS airplane-mode verification, task `5.5`.

# Citations

1. [1] stdin
2. [2] manual:KnowMe/model-runtime-per-device