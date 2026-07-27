---
type: Reference
id: transport-free-agui-encoder-in-uar
title: Transport-free AG-UI encoder in the UAR (ADR 0012)
tags:
- knowme
- uar
- ag-ui
- a2ui
- embedded-runtime
- adr
- open-standards
links:
- a2ui-content-blocks
- local-agentic-lanes
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-23T01:32:00+00:00
created_at: 2026-07-23T01:32:00+00:00
updated_at: 2026-07-23T01:32:00+00:00
revision: 0
---

## Intent

When the embedded UAR runs a LOCAL model, it must emit the AG-UI protocol
identically to the cloud path — make the AG-UI encoder a first-class,
transport-free UAR capability the embedded runtime can reach, instead of each
consumer hand-rolling its own.

## Observed state

- The canonical `NormalizedEvent → AG-UI protocol` map (`to_agui_spec_event` +
  `enrich_agui_spec_payload`) lived in
  `$REPO_ROOT/rust/vendor/universal-agent-runtime/src/uar/api/sse.rs`, which is
  `#[cfg(feature = "server")]` and imports `axum`. So the pure mapping was
  UNREACHABLE to any build without the `server` feature.
- The embedded runtime (`embedded-mobile` feature; KnowMe mobile/desktop) never
  enables `server`, so it hand-rolled a 4th duplicate encoder
  (`gen_ui_agent/src/uar.rs::embedded_stream_payload`). Four divergent
  `NormalizedEvent→wire` encoders existed (`api/adapters.rs::to_ag_ui`,
  `sse::to_agui_event`, `sse::to_agui_spec_event`, KnowMe's copy).
- Web research confirmed AG-UI's own architecture: reference SDKs
  (`@ag-ui/encoder`, `ag-ui-protocol`) ship a standalone `EventEncoder` separate
  from the HTTP handler; guidance is that agent runtimes should EXPOSE AG-UI
  event streams. The transport coupling in UAR was incidental, not required.

## Decision and delta

ADR **`docs/adr/0012-transport-free-ag-ui-adapter.md`** + submodule commit
`b5d66de` (KnowMe pointer bump `35f4979`):
- Moved `to_agui_spec_event` + `enrich_agui_spec_payload` into the already-ungated,
  axum-free `src/uar/api/adapters.rs`. `sse.rs` re-exports them
  (`build_sse_response` unchanged).
- Kept them as **free functions, not a trait** — UAR's convention is free
  functions and there is no second transport to abstract over yet (a
  `Transport`/`EventEncoder` trait would be speculative generality; add it when a
  WebSocket/protobuf transport actually lands). The ungated location makes that a
  localized future change.
- The encoder is now reachable from the embedded consumer via the public module
  path `universal_agent_runtime::uar::api::adapters::to_agui_spec_event` — no
  extra re-export needed (P2.2 verified by path).
- Done under an explicit operator override of the submodule's production lock
  (phase `uar-final-production-hardening`); the change is isolated and does not
  touch the BossFang `server-full` work.

DEFERRED (P2.3): wiring KnowMe's `embedded_stream_payload` to the shared encoder
is NOT a drop-in — KnowMe's projector (`ingest_agui_payload`) parses an older
wire shape (`agui.citation.added`/`request_id`/`eventId`) than the canonical
`CUSTOM`/`name`/`runId`. Delegating requires a synchronized projector rewrite +
multi-surface A2UI re-verification against a path that works today. Assessed as a
maintainability cleanup, not a correctness fix; defer until the UAR encoder next
changes or A2UI is already being re-verified.

## Verification

- `cargo check --lib --no-default-features --features embedded-mobile` — GREEN
  (the exact config KnowMe consumes; the encoder is now reachable transport-free).
- The pre-existing `server-full` E0596 (Arc borrow) is unrelated: PROVEN by
  re-running the identical `--features server-full` lib check with the two changed
  files stashed — it reproduces the same error on the unmodified tree.
- The `axum` errors under embedded-mobile are pre-existing auto-discovered helper
  bins (`src/bin/test_db_setup`, `stub-llm`) that lack `required-features`, not the
  library.

## Failure/lesson

"The function is pure" is not the same as "the function is reachable." A pure
`fn` trapped inside an axum-importing, feature-gated module is unreachable to a
non-server build — which is exactly why the duplication arose. The fix is
relocation to an ungated module, not a rewrite. Also: verify a suspected
regression by stashing your change and re-running (truth over fluency) rather
than reasoning that "my change can't have caused it."

## Next experiment

When the UAR production lock lifts and the encoder next changes, do P2.3:
consolidate KnowMe's `embedded_stream_payload` onto the shared encoder + update
the projector to the canonical shape + re-verify A2UI on all four surfaces.
