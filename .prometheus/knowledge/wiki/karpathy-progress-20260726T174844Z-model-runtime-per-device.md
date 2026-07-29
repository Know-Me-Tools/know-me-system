---
type: Reference
id: karpathy-progress-20260726T174844Z-model-runtime-per-device
title: "Cancellation certified on device; LiteRT-LM cannot preserve partial text because it is non-streaming"
tags:
- karpathy-progress
- model-runtime-per-device
- in-progress
sources:
- conversation:operator-agent
timestamp: 2026-07-26T17:48:44Z
created_at: 2026-07-26T17:48:44Z
updated_at: 2026-07-26T17:48:44Z
revision: 1
---

## Intent

CANCEL PASS on Samsung SM-S936U with the decisive evidence: 'generation failed' appears ZERO times, replaced by 'generation cancellation requested' then 'generation cancelled by user' 5-16ms later. Before the fix this exact scenario logged 'LiteRT-LM generation failed error=terminal ... Java exception was thrown' and completed a 1506-char run after the stop tap. The steer bar renders on device with Steer/Retry/Discard and all three accessible names present in the UI dump. HOWEVER a manual cancel 12s into a long generation showed the status as plain 'Cancelled by user' with NO kept-count, and the UI dump confirms the cancelled assistant turn holds no content blocks at all. This is not a UI defect: LiteRT-LM is non-streaming. TextDelta is pushed exactly once inside send_litert_response_events (litert_lm.rs:421) after the single blocking generate_text call returns. Cancel before it returns and the Err arm runs instead, so no text ever crossed the JNI boundary and there is nothing to preserve. The earlier exploration conclusion that 'partial content is already preserved everywhere' holds for the Flutter/protocol layers and for streaming engines (MLX), but NOT for the Android local lane. The zero-block branch I wrote is therefore behaving correctly — it refuses to claim kept work that does not exist.

## Observed state and verification

Device SM-S936U, APK built 12:39 carrying commits d3dc64a, 673201b, 450de77. certify-local-cancellation.sh: CANCEL PASS. logcat grep -c 'generation failed' = 0. Run 1 (script): 12:40:58.584 generation starting -> 12:41:03.407 cancellation requested -> 12:41:03.423 cancelled by user (16ms). Run 2 (manual, cancel at +12s): 12:45:42.008 cancellation requested -> 12:45:42.013 cancelled by user (5ms), with NO 'generation timing' or 'produced text' line, proving no text was emitted. UI dump content-desc values on the stopped turn: 'Cancelled by user', 'Steer from here — write a new instruction', 'Retry this response', 'Discard this stopped response' — no content blocks. Source: crates/gen_ui_inference/src/litert_lm.rs:421 single TextDelta push; grep for TextDelta/MessageDelta in mlx.rs returns 0 matches, so MLX uses a different emission path.

## Decision and lesson

Status: in-progress. Preserve evidence, distinguish compile proof from runtime proof, and do not narrow the active goal.

## Next experiment

Decide whether the Android local lane should stream incrementally so partial text survives a cancel. Options: (a) accept non-streaming and document that Android keeps thinking/tool blocks but not partial answer text, (b) adopt LiteRT-LM's async/callback generation API if the AAR exposes one, emitting TextDelta per chunk. Note the preservation machinery downstream (protocol flush, checkpointing, history replay) is already correct and tested — only the engine's emission granularity is the gap. Also still open: attachment-flow certification script, iOS airplane-mode certification (task 5.5).
