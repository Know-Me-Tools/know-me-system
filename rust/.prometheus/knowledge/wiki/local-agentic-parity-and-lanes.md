---
type: Reference
id: local-agentic-parity-and-lanes
title: Full-agentic local parity + LANE_LOCAL vs LANE_UAR
tags:
- knowme
- model-runtime-per-device
- uar
- embedded-runtime
- lanes
- agentic
- verified-no-code
links:
- local-agentic-lanes
- transport-free-agui-encoder-in-uar
- a2ui-content-blocks
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-23T01:33:00+00:00
created_at: 2026-07-23T01:33:00+00:00
updated_at: 2026-07-23T01:33:00+00:00
revision: 0
---

## Intent

Confirm the embedded UAR runs the FULL agent loop (tool-loop, RAG citations,
memory recall, skill activation) on a LOCAL model — not just chat — and forwards
every event as a ContentBlock, identically to the cloud path.

## Observed state

- Two distinct local lanes exist (`$REPO_ROOT/rust/crates/gen_ui_agent/src/chat.rs`):
  - **LANE_LOCAL** (`send_local`, ~chat.rs:1882) calls
    `engine.generate_request()` DIRECTLY on the `InferenceProvider` with a
    `LocalToolCallGate` — chat + local tool-calling only, NO kernel/RAG/memory/skill.
  - **LANE_UAR** (`start_run` → `UarEmbeddedRuntime::build_local`, uar.rs:626)
    builds the embedded UAR kernel
    (`.local_provider(driver).persistence(InMemory).seed_defaults(true)`) and runs
    the seeded orchestrator on the local model through the `LlmDriver` seam
    (`UarLocalInferenceDriverBridge`, uar.rs:243). This is the full-agentic path.
- iOS/Android default to LANE_UAR when the embedded kernel is available
  (`default_lane()`, chat.rs:1178-1188).

## Decision and delta

NO new code. The audit found the path already complete:
- `UarNativeEventAdapter::adapt` (uar.rs:355-506) maps EVERY agentic
  `StreamEvent` → `NormalizedEvent`: `SkillActivation`, `Citation`, `Citations`,
  `Memory`, full tool loop (Started/Delta/Complete/Ready/ToolResult),
  `ThinkingDelta`, `Custom`, `Cancelled`.
- The embedded kernel wiring + driver bridge already exist and are exercised.

## Verification

`cargo test -p gen_ui_agent --lib` — 2 targeted tests pass:
- `embedded_uar_runs_the_seeded_orchestrator_on_the_local_provider` — the kernel
  runs the seeded orchestrator on the local model, reaches `EmbeddedKernel`
  state, projects thinking + answer ContentBlocks.
- `native_driver_preserves_every_local_semantic_event` — asserts skill +
  2 citations (correct indices) + memory + custom + cancel all survive the
  local→UAR mapping.

## Failure/lesson

"Phase 3 needs code" was a wrong prior. Verifying before building (Rule 1) +
truth over fluency (Rule 5) turned a presumed implementation task into an
audit-and-certify: the forwarding and kernel wiring were already present and
tested. Recording the LANE_LOCAL-vs-LANE_UAR distinction prevents future
confusion — full-agentic-local is LANE_UAR, not LANE_LOCAL.

## Next experiment

Prove both lanes produce real responses on the physical iPhone (Local = direct
engine phase seq; UAR = "Starting Universal Agent Runtime run" + orchestrator-agent).
See [[ios-local-embedded-lane-certification]].
