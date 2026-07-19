# Qwen3.5 mobile model runtime

## Why

The current mobile lane uses a 0.5B text model, CPU defaults, a 2K context, and a 128-token output limit. It cannot satisfy KnowMe's useful on-device assistant requirements.

## What changes

- Select Qwen3.5-9B Q4_K_M on capable iOS devices and Qwen3.5-4B Q4_K_M on Android.
- Pin every artifact by repository revision, byte length, and SHA-256.
- Resume interrupted downloads and publish only verified artifacts.
- Enable Metal on iOS and Vulkan with CPU fallback on Android.
- Use an 8K default context and 1,024-token output ceiling.

## Success criteria

- Each platform resolves the intended catalog model without UI-layer inference logic.
- Interrupted downloads resume and corrupt files never become active.
- Runtime diagnostics report the selected model and backend.
- Text generation completes on both physical phones with the configured limits.
