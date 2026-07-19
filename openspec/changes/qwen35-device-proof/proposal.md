# Qwen3.5 physical-device proof

## Why

Cross-compilation alone cannot prove mobile memory fit, backend correctness, useful output, or thermal behavior.

## What changes

- Expose selected model/backend diagnostics to integration tests.
- Add physical-device scenarios for Markdown/Mermaid, image understanding, and a safe tool call.
- Record fallback decisions and representative throughput.
- Deploy and run on Travis's iPhone and the attached Android device.

## Success criteria

- Both devices identify the expected model or an explicit safe fallback.
- Both complete useful text, vision, and tool scenarios without cloud inference.
- Vulkan failure selects CPU without corrupt output or crashing.
