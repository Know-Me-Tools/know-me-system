---
type: Reference
id: mobile-embedded-uar-provider-config-and-parity
title: Mobile embedded-UAR provider config (local⊕cloud) + cross-platform parity
tags:
- knowme
- model-runtime-per-device
- uar
- embedded-runtime
- provider-admin
- routing-driver
- provenance
- cross-platform-parity
links:
- local-agentic-parity-and-lanes
- transport-free-agui-encoder-in-uar
- ios-mlx-switchover-and-qwen3-arch-fix
- a2ui-content-blocks
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-24T00:00:00+00:00
created_at: 2026-07-24T00:00:00+00:00
updated_at: 2026-07-24T00:00:00+00:00
revision: 0
---

## Intent

Fix the reported mobile bug — Settings showed a "UAR service URL" form and threw
`missing uar_service_url setting or KNOWME_UAR_SERVICE_URL env var` — and deliver
the operator's real requirement: on iOS/Android the UAR is ALWAYS the embedded
in-process kernel (no service URL), configured with **provider + model + keys**,
running EITHER the on-device local model OR a configured cloud provider,
switchable at runtime. Plus a Cherry-Studio-style 3-tier model selection and
per-assistant-bubble provenance. All four platforms must behave the same.

## Observed state / root cause

- Embedded UAR needs no URL: iOS/Android default to `EmbeddedLibrary`
  (`chat.rs` `selected_uar_deployment_mode`/`default_lane`) and the embedded chat
  path never calls `load_uar_service_chat_config()`. The error came ONLY from the
  HTTP admin client (`chat.rs:1416`).
- Two concrete defects: (1) `save_uar_service_config` force-wrote
  `uar_deployment_mode="cloud_service"` on iOS/Android (flipping mobile off
  embedded); (2) `provider_admin::*` was HTTP-only (`uar_service_client()`), so
  the Model-providers tab couldn't work embedded.
- Embedded RunManager holds ONE `primary_driver` for its lifetime and the
  `LlmRequest` carries no model string ⇒ registering providers alone can't switch
  execution; routing MUST live inside the driver.

## Decision and delta (committed on feat/ios-mlx-inference)

- **M1 `UarRoutingDriver`** (gen_ui_agent/uar.rs): one `LlmDriver` the embedded
  kernel holds, dispatching per `stream()` to the on-device bridge OR a lazily
  built, cached cloud `LiterLlmDriver` (reuses UAR's own public `LiterLlmDriver` +
  `build_client_config` ⇒ ZERO submodule edits). `ConfigActiveSelector` reads
  `SETTING_UAR_ACTIVE_PROVIDER` + the cloud `ModelPref`; any gap/read-failure
  degrades to Local so a run is never stranded. `LOCAL_PROVIDER_ID = "on-device"`.
- **M2 on-device `provider_admin` branch**: all 8 fns dispatch on
  `selected_uar_deployment_mode()`; `Embedded` impls (provider_admin_embedded.rs)
  serve `catalog()` from `ModelCatalog::global()` (full 140+) + a synthetic local
  provider entry, persist to the config store + secrets + a `provider_meta:{id}`
  JSON blob; `save_cloud_model`/`get_cloud_model` read/write the SAME keys the
  routing selector reads (end-to-end contract). Identical return types ⇒ Flutter/
  FFI untouched.
- **M3** removed the force-cloud trap (mobile stays embedded).
- **M4** Flutter Settings: dropped the URL/key/session form → "Embedded ·
  on-device" badge; un-gated the provider load; unified picker with the local
  model pinned first + ACTIVE chip + "Use this model" (`setActiveRoute`).
- **M6/M6b provenance**: the `effective_run_policy` artifact is projected to
  `A2uiEvent::ContextTelemetry {provenance:{agentId,providerId,modelId}}` (Rust,
  gen_ui_protocol); Flutter `ChatMessage.provenance` + footer chip; React
  `Message.provenance` + assistant-footer chip. Found+fixed a real gap:
  `foldStream` dropped telemetry (missing `onTelemetry`).
- **M7 core**: macOS desktop now hosts the embedded UAR (gen_ui_host boot builds
  `UarEmbeddedRuntime::build_local` from the MlxcEngine; macOS defaults to
  `EmbeddedLibrary`), so it inherits M1/M2 with no service URL. Win/Linux stay
  service clients.

## Cross-platform parity (verified from code, not assumed)

Parity = same behavior PER deployment mode. iOS/Android/macOS = embedded
(symmetric `#[cfg]` arms; Android uses LiteRT-LM Gemma, iOS/macOS use Qwen3-4B
MLX, same seam). Windows/Linux + Web = service clients (unchanged, no regression).
**Web is a login-gated per-user service client** (axum server fronts the UAR;
keyless local floor, then per-user providers + UAR/BossFang subscription) — left
as-is this effort; parity for web = only the shared UI (pickers, provenance chip).

## Verification

Matrix at the M6-Rust tip: host `cargo test -p gen_ui_agent` 68/68 + gen_ui_protocol
43/43; `cargo check --target aarch64-apple-ios` Finished; `cargo check -p
tauri-plugin-gen-ui` Finished; `dart analyze lib/` clean + 7/7 chat tests; React
`tsc --noEmit` exit 0. Android cross-compile blocked ONLY by a missing NDK
toolchain (`aarch64-linux-android-clang` not on PATH) — an env gap, not a code
defect (code is cfg-symmetric with iOS, which compiled). macOS `gen_ui_host` +
`gen_ui_agent` compile with the embedded path; 68/68 still green.

## Failure / lesson

- "Routing must live in the driver" — registering extra providers into the
  embedded registry can't switch execution because the RunManager reuses one
  driver and the LlmRequest carries no model. Verify the run loop, don't assume.
- Branch hygiene: a `git checkout main` mid-session reverted the working tree to a
  branch without this work; the commits were all safe on `feat/ios-mlx-inference`
  (reflog confirmed). Merged main in cleanly (plugin-system commits were disjoint).
  Lesson: check `git branch --show-current` + reflog before assuming data loss.
- Delegated subagents can die on infra errors mid-flight (connection closed) and
  leave nothing. De-risk large delegated work by pinning the full design (OpenSpec
  proposal/design/specs/tasks) to disk FIRST, so re-delegation is mechanical.

## Next experiment

M5a (UAR submodule, in flight): wire the embedded RunManager to a `SettingsManager`
+ `resolve_effective_run_policy` so `run_policy.global` is honored embedded, and
expose settings/agent admin on the SDK Runtime (OpenSpec
`embedded-run-policy-and-admin-surface`, ADR 0013). Then KnowMe consumers:
control_plane embedded branch → 3-tier model pickers → macOS React settings drop
the service form. See [[local-agentic-parity-and-lanes]].
