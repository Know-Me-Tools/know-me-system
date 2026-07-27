---
type: Reference
id: android-litert-lm-release-jni-bridge-certified-on-samsung
title: Android LiteRT-LM release JNI bridge certified on Samsung
tags:
- model-runtime-per-device
- android-release
- litert-lm
- jni
- r8
- local-tools
- mcp-registry
- device-cert
links:
- ios-cert-monitor-holding-for-local-send-with-benign-nw-path-noise
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-25T23:13:26.465574+00:00
created_at: 2026-07-25T23:13:26.465574+00:00
updated_at: 2026-07-25T23:13:26.465574+00:00
revision: 0
---

## Phase context

- Phase: `model-runtime-per-device`
- Project: `KnowMe`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system`
- Captured: `2026-07-25T23:08:11Z`
- Position: `model-runtime-per-device › litertlm-local-tools › CERTIFIED`
- Status: `complete`
- Device: Samsung `SM-S936U`

## Goals in scope

- Ship Android local text-agent inference through LiteRT-LM `Gemma 4 E2B`.
- Use CPU as the universal correctness baseline before NPU/SM8750 optimization work.
- Verify local/agent tool-capable model plumbing for future agent loops.
- Keep the wider phase aligned with the iOS local inference lane, including [iOS cert monitor holding for Local send with benign nw_path noise](/ios-cert-monitor-holding-for-local-send-with-benign-nw-path-noise.md).

## Release-only Android failure

Flutter release builds run **R8 by default**, even when `minifyEnabled` is not explicitly set in `build.gradle.kts`.

Observed failure mode:

- Debug builds worked.
- Release build on hardware failed because R8 retained only `MainActivity` from the manifest.
- `KnowMeLiteRtLmBridge` was removed because it is reached only from Rust through JNI reflection, which is invisible to static reachability analysis.
- Member shrinking was also unsafe because Rust resolves bridge methods reflectively after `loadClass`.

Earlier assumption corrected: inferring that R8 was not involved from `build.gradle.kts` was wrong; build intermediates/dex contents were the authoritative check.

## Fix

Commit `7330389` added release R8 keep rules and wired them via `proguardFiles` on the release `buildType`.

The keep rules preserve:

- `KnowMeLiteRtLmBridge`
- Its members and nested classes
- The LiteRT-LM JNI surface

Dex inspection before device testing confirmed the fix:

- Before: `classes.dex` contained only `MainActivity`.
- After: `classes.dex` contained `KnowMeLiteRtLmBridge` and its nested class.

## Device certification result

The Samsung device proved the full Android LiteRT-LM local tool path in approximately 9 seconds, including cold graph compilation:

```text
bridge resolves (no ClassNotFoundException)
  → liblitertlm_jni.so ok → LitertLmLoader v1.5.0, tokenizer + metadata parsed
  → GPU+CPU accelerators, 103.8MB embedder compiled
  → tool call: knowme_diagnostics__local_runtime_status
  → tool-result continuation (tool_results=1)
  → text chars=393 tool_calls=0 context="tool_result"
```

Certified path for `litertlm-local-tools`:

1. Shortlisted tool declarations were supplied.
2. Model emitted a namespace-qualified tool call: `knowme_diagnostics__local_runtime_status`.
3. Tool detection succeeded.
4. Runtime gate validation succeeded.
5. Execution flowed through the MCP registry.
6. Tool-result continuation succeeded.
7. A visible 393-character answer was produced.
8. `tool_result_fallback_applied` did **not** fire, proving native continuation rather than fallback response synthesis.

The namespace-qualified tool name confirms that the call came through the shortlist mechanism added for local tools.

## Accelerator interpretation

NPU warnings are expected for this milestone. The certified target is the generic CPU correctness artifact first; NPU/SM8750 work remains an optimization follow-up.

## Status corrections needed

`progress.json` still marks both of these as complete:

- `android-litertlm-runtime`
- `litertlm-device-proof`

This release-build blocker proves those statuses were inaccurate if they implied release-build device verification. They should be corrected or annotated to distinguish previous partial/debug validation from the now-certified release path.

## Follow-ups

- Optionally certify the Local tab `send_local` path. It uses the same engine and is likely covered indirectly, but it was not explicitly certified here.
- Optionally test a mutating tool to verify the confirmation gate.
- Correct `progress.json` for `android-litertlm-runtime` and `litertlm-device-proof`.

# Citations

1. stdin
2. manual:KnowMe/model-runtime-per-device