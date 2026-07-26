---
type: Reference
id: litert-lm-gemma-4-e2b-tool-path-certified-on-samsung
title: LiteRT-LM Gemma 4 E2B tool path certified on Samsung
tags:
- model-runtime-per-device
- litert-lm
- gemma
- android-device-cert
- tool-calling
- mcp-registry
- local-inference
links:
- ios-cert-monitor-holding-for-local-send-with-benign-nw-path-noise
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-25T23:08:32.657478+00:00
created_at: 2026-07-25T23:08:32.657478+00:00
updated_at: 2026-07-25T23:08:32.657478+00:00
revision: 0
---

## Phase context

- Phase: `model-runtime-per-device`
- Project: `KnowMe`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system`
- Captured: `2026-07-25T23:06:43Z`
- Position: `model-runtime-per-device › litertlm-local-tools › tool-path-certifying`
- Status: `monitoring`
- Device lane: Android/Samsung LiteRT-LM local text-agent inference

## Phase goals

- Ship `Qwen3.5-9B Q4_K_M` with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM `Gemma 4 E2B`, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Certification result

LiteRT-LM local tool-call plumbing is live on Samsung hardware. `Gemma-4 E2B` generated a namespace-qualified tool call, the runtime gate accepted it as an in-shortlist tool, the MCP registry executed it, and continuation began with the tool result fed back into the model.

Observed log markers:

```text
LiteRT-LM generation produced tool call
  id=litert-tool-0  name=knowme_diagnostics__local_runtime_status
LiteRT-LM tool-result continuation starting  tool_results=1
```

Certified path:

1. `Gemma-4 E2B` saw the tool declarations and generated a tool call.
2. The emitted tool name was namespace-qualified: `knowme_diagnostics__local_runtime_status`.
3. Namespace qualification proves the declaration came from the runtime-produced shortlist.
4. The gate accepted the call as in-shortlist instead of rejecting it.
5. The gate executed the tool through the MCP registry.
6. The runtime started model continuation with `tool_results=1`.

This certifies the intended `litertlm-local-tools` sequence:

```text
shortlisted declarations → detection → validation → execution → continuation
```

## Expected empty initial text

Observed warning:

```text
W  LiteRT-LM generation returned empty text tool_calls=1 context="initial"
```

This is expected for a tool-calling turn: the model emitted a tool call instead of prose, so empty text with `tool_calls=1` is the correct initial-turn shape.

The warning becomes actionable only if the tool-result continuation also returns empty text. That case is covered by the `tool_result_fallback_applied` path and remains under watch.

## Performance observation

End-to-end latency from send to tool call was approximately **5 seconds**, including first-load graph compilation.

## Next verification steps

- Confirm the tool-result continuation returns visible text.
- Check whether `tool_result_fallback_applied` fires.
- Confirm `RunFinished` to close the Agent-lane certification.
- Have the operator tap the Local tab for the `send_local` path and record that certification.
- iOS device-lane certification remains separate and has previously been blocked on local-send operator action, as tracked in [iOS cert monitor holding for Local send with benign nw_path noise](/ios-cert-monitor-holding-for-local-send-with-benign-nw-path-noise.md).

# Citations

1. stdin
2. manual:KnowMe/model-runtime-per-device