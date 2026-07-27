---
type: Reference
id: embedded-uar-provider-routing-and-mobile-parity
title: Embedded UAR Provider Routing and Mobile Parity
tags:
- uar
- embedded-runtime
- provider-admin
- routing-driver
- mobile-settings
- cross-platform-parity
- provenance
links:
- karpathy-progress-20260720T201046Z-embedded-uar-offline-agents
- karpathy-progress-20260723T013607Z-model-runtime-per-device
sources:
- rust/.prometheus/knowledge/wiki/mobile-embedded-uar-provider-config-and-parity.md
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-24T15:30:17.621756+00:00
created_at: 2026-07-24T15:30:17.621756+00:00
updated_at: 2026-07-24T15:30:17.621756+00:00
revision: 0
---

## Intent

Fix the mobile Settings bug where iOS/Android displayed a `UAR service URL` form and failed with:

```text
missing uar_service_url setting or KNOWME_UAR_SERVICE_URL env var
```

Required behavior:

- iOS/Android UAR is always the embedded in-process kernel; no service URL is required.
- Embedded UAR is configured by provider, model, and keys.
- Runtime can switch between:
  - on-device local model
  - configured cloud provider
- Settings exposes a Cherry-Studio-style 3-tier model selection flow.
- Assistant bubbles show per-response provenance.
- Platform behavior is consistent by deployment mode.

This supersedes the older service-only/mobile-URL failure mode described in [Correct service-only mobile UAR integration](/karpathy-progress-20260720T201046Z-embedded-uar-offline-agents.md).

## Root Cause

- Embedded UAR does not need a service URL.
  - iOS/Android default to `EmbeddedLibrary` via `chat.rs` `selected_uar_deployment_mode` / `default_lane`.
  - The embedded chat path does not call `load_uar_service_chat_config()`.
  - The observed error came only from the HTTP admin client path at `chat.rs:1416`.
- Two concrete defects caused the bug:
  1. `save_uar_service_config` force-wrote `uar_deployment_mode = "cloud_service"` on iOS/Android, flipping mobile off embedded mode.
  2. `provider_admin::*` was HTTP-only via `uar_service_client()`, so the Model Providers tab could not operate against embedded UAR.
- Embedded `RunManager` holds one `primary_driver` for its lifetime.
- `LlmRequest` carries no model string.
- Therefore, registering providers alone cannot switch execution; routing must occur inside the driver.

## Committed Delta on `feat/ios-mlx-inference`

### M1: `UarRoutingDriver`

Implemented in `gen_ui_agent/uar.rs`.

- A single `LlmDriver` is held by the embedded kernel.
- Each `stream()` dispatches to either:
  - the on-device bridge, or
  - a lazily built and cached cloud `LiterLlmDriver`.
- Reuses UAR public APIs:
  - `LiterLlmDriver`
  - `build_client_config`
- No UAR submodule edits were required.
- `ConfigActiveSelector` reads:
  - `SETTING_UAR_ACTIVE_PROVIDER`
  - cloud `ModelPref`
- Any missing config or read failure degrades to local execution so a run is never stranded.
- Local provider ID is:

```rust
LOCAL_PROVIDER_ID = "on-device"
```

### M2: Embedded `provider_admin` Branch

All 8 `provider_admin` functions now dispatch on `selected_uar_deployment_mode()`.

Embedded implementation: `provider_admin_embedded.rs`.

Behavior:

- `catalog()` is served from `ModelCatalog::global()` with the full 140+ model catalog.
- Adds a synthetic local provider entry.
- Persists provider config to:
  - config store
  - secrets
  - `provider_meta:{id}` JSON blob
- `save_cloud_model` and `get_cloud_model` read/write the same keys consumed by the routing selector.
- Return types are identical to the HTTP implementation, so Flutter/FFI did not need API changes.

### M3: Removed Force-Cloud Trap

- Removed mobile behavior that forced `uar_deployment_mode = "cloud_service"`.
- Mobile remains embedded by default.

### M4: Flutter Settings

- Removed the URL/key/session form from mobile embedded Settings.
- Added an `Embedded · on-device` badge.
- Ungated provider loading.
- Unified picker behavior:
  - local model pinned first
  - active route chip
  - `Use this model` action via `setActiveRoute`

### M6/M6b: Provenance in Assistant Bubbles

- `effective_run_policy` artifact is projected into:

```rust
A2uiEvent::ContextTelemetry {
    provenance: {
        agentId,
        providerId,
        modelId,
    }
}
```

- Rust changes live in `gen_ui_protocol`.
- Flutter:
  - `ChatMessage.provenance`
  - footer provenance chip
- React:
  - `Message.provenance`
  - assistant-footer provenance chip
- Fixed a real telemetry gap: `foldStream` dropped telemetry because it was missing `onTelemetry`.

### M7: macOS Embedded UAR

- macOS desktop now hosts embedded UAR.
- `gen_ui_host` boot builds `UarEmbeddedRuntime::build_local` from `MlxcEngine`.
- macOS defaults to `EmbeddedLibrary`.
- macOS inherits M1/M2 behavior and no longer needs a service URL.
- Windows/Linux remain service clients.

## Cross-Platform Parity

Parity is defined as same behavior per deployment mode, not identical deployment mode on every platform.

| Platform | Deployment mode | Behavior |
| --- | --- | --- |
| iOS | embedded | In-process UAR, no service URL, Qwen3-4B MLX |
| Android | embedded | In-process UAR, no service URL, LiteRT-LM Gemma |
| macOS | embedded | In-process UAR, no service URL, Qwen3-4B MLX |
| Windows | service client | Unchanged |
| Linux | service client | Unchanged |
| Web | service client | Login-gated per-user service client |

Notes:

- iOS/Android/macOS use symmetric `#[cfg]` arms at the UAR seam.
- Android differs only in local model backend: LiteRT-LM Gemma instead of MLX Qwen.
- Web remains as-is:
  - Axum server fronts UAR.
  - Keyless local floor.
  - Per-user providers.
  - UAR/BossFang subscription integration.
  - Shared UI only for pickers and provenance chip.

## Verification

At the M6-Rust tip:

- `cargo test -p gen_ui_agent`: 68/68 passing.
- `gen_ui_protocol`: 43/43 passing.
- `cargo check --target aarch64-apple-ios`: finished.
- `cargo check -p tauri-plugin-gen-ui`: finished.
- `dart analyze lib/`: clean.
- Flutter chat tests: 7/7 passing.
- React `tsc --noEmit`: exit 0.
- macOS `gen_ui_host` + `gen_ui_agent` compile with embedded path.
- Android cross-compile was blocked only by missing NDK toolchain:

```text
aarch64-linux-android-clang not on PATH
```

This was classified as an environment/toolchain gap, not a code defect, because Android code is `cfg`-symmetric with the iOS path that compiled.

The work continues the physical-device certification effort tracked in [iOS device cert build + 5 knowledge records](/karpathy-progress-20260723T013607Z-model-runtime-per-device.md).

## Lessons

- Routing must live in the driver.
  - Embedded `RunManager` reuses one driver.
  - `LlmRequest` has no model field.
  - Registering extra providers in the embedded registry cannot switch execution by itself.
- Verify the run loop before assuming provider registration is sufficient.
- Branch hygiene matters:
  - A mid-session `git checkout main` reverted the working tree to a branch without this work.
  - Commits were safe on `feat/ios-mlx-inference`; reflog confirmed.
  - Main was merged cleanly because plugin-system commits were disjoint.
  - Check `git branch --show-current` and reflog before assuming data loss.
- Delegated subagents can fail on infrastructure errors and leave no durable output.
  - For large delegated work, first pin full design artifacts to disk: OpenSpec proposal, design, specs, and tasks.
  - Re-delegation then becomes mechanical.

## Next Experiment

M5a is in flight in the UAR submodule:

- Wire embedded `RunManager` to a `SettingsManager`.
- Wire `resolve_effective_run_policy` so `run_policy.global` is honored in embedded mode.
- Expose settings and agent admin on SDK Runtime.
- OpenSpec: `embedded-run-policy-and-admin-surface`.
- ADR: `0013`.

Then update KnowMe consumers:

- `control_plane` embedded branch.
- 3-tier model pickers.
- macOS React settings should remove the service form.

# Citations

1. rust/.prometheus/knowledge/wiki/mobile-embedded-uar-provider-config-and-parity.md
2. stdin
3. manual:KnowMe/model-runtime-per-device