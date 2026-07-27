---
type: Reference
id: mobile-embedded-uar-provider-config-without-service-url
title: Mobile embedded UAR provider config without service URL
tags:
- knowme
- model-runtime-per-device
- uar
- embedded
- ios
- android
- providers
- routing
links:
- three-tier-model-selection-and-provenance
- macos-embedded-uar-host
sources:
- mobile-embedded-uar-provider-config.md
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-25T21:46:09.113509+00:00
created_at: 2026-07-25T21:46:09.113509+00:00
updated_at: 2026-07-25T21:46:09.113509+00:00
revision: 0
---

## Context

- **Project:** KnowMe
- **Phase:** `model-runtime-per-device`
- **Captured:** 2026-07-25
- **Status:** verified
- **Platforms:** iOS and Android

## Problem

On mobile, UAR runs as an embedded in-process kernel. There is no UAR service URL. The Settings UI still exposed a `UAR service URL / API key / session` form and produced `missing uar_service_url setting`.

The intended behavior is desktop-like provider configuration without requiring a service URL:

- configure provider, model, and keys;
- keep the on-device local model available;
- allow configured cloud providers to be available alongside the local model;
- allow local/cloud switching at runtime.

## Observed defects

Two KnowMe-side defects caused the mobile failure:

1. **Force-cloud trap**
   - Tapping **Save runtime** wrote `uar_deployment_mode="cloud_service"` on iOS/Android.
   - This flipped mobile out of embedded mode.
   - Subsequent chat turns then demanded a service URL.

2. **HTTP-only provider admin**
   - Every `provider_admin::*` function routed through `uar_service_client()`.
   - On embedded devices, the Model Providers tab could not load a catalog or save provider configuration.

A runtime constraint shaped the fix: embedded `RunManager` holds one `primary_driver` for its lifetime and rebuilds the Orchestrator around that same driver per run. `LlmRequest` also carries no model string. Therefore, merely registering additional providers would list correctly but would not switch execution.

## Decision

### Route inside a single primary driver

Implemented `UarRoutingDriver`, a single `LlmDriver` that wraps:

- the on-device bridge;
- lazily built, cached cloud `LiterLlmDriver`s.

At each `stream()`, the driver reads the persisted active selection and routes to either:

- the local on-device model; or
- the selected cloud provider/model.

Routing lives in the driver because the embedded runtime binds one primary driver for the run lifetime. This reused UAR public `LiterLlmDriver` and `build_client_config`, requiring **zero UAR submodule edits**.

This seam is compatible with the model-routing/provenance work in [Three-tier model selection + per-response provenance](/three-tier-model-selection-and-provenance.md), and the same embedded-vs-service parity rule is used by [macOS as an embedded UAR host — cross-platform parity per deployment mode](/macos-embedded-uar-host.md).

### Add embedded provider-admin path

Added an on-device `provider_admin` branch for all eight provider-admin functions, gated by:

```rust
selected_uar_deployment_mode() == EmbeddedLibrary
```

Properties:

- identical return types to the existing HTTP path;
- unchanged Flutter/FFI surface;
- catalog source: `ModelCatalog::global()` plus a synthetic on-device provider entry.

### Prevent mobile from being forced into cloud mode

Removed the force-cloud write on mobile by guarding it behind non-iOS/Android cfg:

```rust
#[cfg(not(any(target_os = "ios", target_os = "android")))]
```

### Update Flutter settings behavior

Flutter changes:

- removed the service-URL gate in `RustSettingsRepository.load()`;
- replaced the service form with a read-only `Embedded · on-device` badge;
- pinned the on-device model as a first-class provider in the unified picker.

## Verification

Host tests cover:

- local vs cloud routing through a mock selector;
- config round-trip;
- `provider_admin` save → list → delete;
- force-cloud trap does not fire under `cfg(ios)`.

Device verification:

- verified on iPhone with no service URL present.

## Lesson

The initial instinct was to register cloud providers in the registry. That would have fixed provider listing but not execution, because the embedded kernel binds one driver for the run lifetime. Reading the runtime ownership model first exposed the correct routing layer: the single primary `LlmDriver`.

## Next experiment

Extend the routing seam to per-turn model overrides once the SDK exposes a per-request model hint.

# Citations

1. mobile-embedded-uar-provider-config.md
2. manual:KnowMe/model-runtime-per-device