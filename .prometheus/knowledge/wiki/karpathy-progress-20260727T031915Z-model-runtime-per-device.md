---
type: Reference
id: karpathy-progress-20260727T031915Z-model-runtime-per-device
title: "Desktop+web four intents shipped; real desktop run blocked on an incomplete MLX download"
tags:
- karpathy-progress
- model-runtime-per-device
- in-progress
sources:
- conversation:operator-agent
timestamp: 2026-07-27T03:19:15Z
created_at: 2026-07-27T03:19:15Z
updated_at: 2026-07-27T03:19:15Z
revision: 1
---

## Intent

Ported Steer/Resume/Retry/Discard to the React codebase serving both Tauri desktop and web, and fixed three engine gaps found in audit: macOS/mlx-c never overrode cancel_generation (its per-token flag was reachable only via download-cancel), Win/Linux/llama.cpp had no cancel check in its token loop at all, and browser WebLLM never called the installed interruptGenerate while cancelActiveRun branched two ways against a three-way start split. Also persisted a new intent column behind an idempotent ALTER, because CREATE TABLE IF NOT EXISTS means a new column never lands on an existing database and the next INSERT then fails for every message. convertMessage now reports incomplete/cancelled so the state is legible to assistant-ui.

## Observed state and verification

Vitest 52/52 across 11 files including the previously untested terminal cancelled state, partial-content preservation, the Resume history-replay invariant, and both cancel-lane routes; tsc clean; gen_ui_inference 7/7 with local-mlxc and 15/15 with local-llama. a11y: light-theme primary was 3.97:1 for a 14px label (fine for the 32 existing icon-only uses at the 3:1 non-text bar) and was fixed to 4.84:1 scoped to this component; RunCancelled was missing aria-live entirely. Commit 44ee525.

## Decision and lesson

Status: in-progress. Preserve evidence, distinguish compile proof from runtime proof, and do not narrow the active goal.

## Next experiment

BLOCKED: the real Tauri desktop run could not be completed. The mlx-c model qwen3.5-4b-mlx-4bit is a 1.1GB model.safetensors.part from 2026-07-22 that never finished downloading, so the macOS local lane cannot load. The app itself built and ran cleanly for an hour with no errors. Finish that download, then run tauri:dev, confirm the Local lane is selected, stop mid-stream and record kept chars plus the rendered 'Cancelled by user', then exercise Resume and Steer and re-check the existing tool UI. Windows/Linux llama.cpp and browser WebLLM end-to-end also remain unverified.
