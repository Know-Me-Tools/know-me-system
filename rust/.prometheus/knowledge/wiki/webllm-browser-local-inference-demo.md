---
type: Reference
id: webllm-browser-local-inference-demo
title: WebLLM in-browser local inference — demonstrated live
tags:
- knowme
- model-runtime-per-device
- web
- webllm
- webgpu
- local-inference
- a2ui
links:
- a2ui-content-blocks
- local-agentic-lanes
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-23T01:34:00+00:00
created_at: 2026-07-23T01:34:00+00:00
updated_at: 2026-07-23T01:34:00+00:00
revision: 0
---

## Intent

Make Web — the fourth crucial platform — run a local model on-device (in the
browser), and demonstrate it generating live.

## Observed state

- WebLLM was already substantially wired (not greenfield):
  `$REPO_ROOT/desktop/src/features/chat/api/webllmLane.ts` uses
  `@mlc-ai/web-llm@0.2.84`, model `Qwen2.5-0.5B-Instruct-q4f16_1-MLC`, with
  `isWebGpuAvailable()` gate, `loadWebLlm()` (idempotent, Cache-API cached),
  `streamWebLlm()` (OpenAI-compat stream → `CoreA2uiEvent`s, same shape as the
  Tauri lane), `unloadWebLlm()`.
- `chatStore.ts:175` routes `!isTauri() && lane==='local'` → `streamWebLlm`;
  `laneStore` gates on `isWebGpuAvailable()`. So a browser (non-Tauri) context
  already runs WebLLM in-browser through the shared A2UI ContentBlock path.
- WebGPU is JS-only (WASM cannot drive it) — a documented exception to the
  "all inference in the Rust core" invariant, confined to transport not semantics.

## Decision and delta

No new engine code. Served the KnowMe Vite frontend at `localhost:1420` as a
plain web build (not Tauri) and drove it in a real browser.

## Verification

Live in-browser demonstration:
- Opened `localhost:1420` in a real browser → `isTauri()=false` → WebLLM lane;
  home page showed "Active model: Qwen 2.5 0.5B Q4 · WebLLM · ready".
- On-device → Local, sent "Say hello in one short sentence." → model replied
  "Hello, how can I assist you today?" at **~5.5 tok/s**.
- **Zero cloud-inference network calls** (verified via the browser network tab:
  no openai/anthropic/liter/`/api/uar/run`/completions requests) — genuine
  in-browser WebGPU local inference.

## Failure/lesson

"Demonstrate through the axum web server" conflates two locations: WebLLM runs in
the BROWSER (WebGPU), the server only SERVES the page. Verifying what already
exists (webllmLane + store routing) before building avoided rebuilding a working
engine — the task reduced to serving the frontend and driving it live.

## Next experiment

Extend the Phase-1 `<think>` segmenter to `webllmLane.ts` (currently accumulates
a single text block) when a reasoning-capable web model is adopted; wire an
in-browser model-cache contract mirroring the Rust catalog's SHA-pinned download.
