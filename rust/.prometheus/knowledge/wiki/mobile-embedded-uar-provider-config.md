---
type: Reference
id: mobile-embedded-uar-provider-config
title: Mobile embedded UAR — provider config without a service URL
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
- embedded-uar-provider-routing-and-mobile-parity
- three-tier-model-selection-and-provenance
- macos-embedded-uar-host
sources:
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-25T21:42:00+00:00
created_at: 2026-07-25T21:42:00+00:00
updated_at: 2026-07-25T21:42:00+00:00
revision: 0
---

## Context

- **Project:** KnowMe
- **Phase:** `model-runtime-per-device`
- **Captured:** `2026-07-25`
- **Status:** `verified`

## Intent

On iOS/Android the UAR is the embedded in-process kernel — there is no service
URL. The Settings screen nonetheless showed a "UAR service URL / API key /
session" form and threw `missing uar_service_url setting`. The operator wanted
provider + model + key configuration like desktop, with **both** the on-device
local model and configured cloud providers available and switchable at runtime.

## Observed state

Two concrete defects, both KnowMe-side:

1. **Force-cloud trap.** Tapping "Save runtime" wrote
   `uar_deployment_mode="cloud_service"` on iOS/Android, flipping mobile out of
   embedded mode so every subsequent chat turn demanded the URL.
2. **HTTP-only admin.** Every `provider_admin::*` function routed through
   `uar_service_client()`, so the Model-providers tab could never load a catalog
   or save a provider on an embedded device.

A third constraint shaped the design: the embedded `RunManager` holds **one**
`primary_driver` for its lifetime and rebuilds the Orchestrator around that same
driver per run, and `LlmRequest` carries no model string. Registering extra
providers therefore cannot switch execution.

## Decision and delta

- **`UarRoutingDriver`** — a single `LlmDriver` that wraps the on-device bridge
  plus lazily-built, cached cloud `LiterLlmDriver`s and picks local-vs-cloud at
  each `stream()` from the persisted active selection. Routing *must* live in
  the driver because of the one-primary-driver constraint above. It reuses UAR's
  public `LiterLlmDriver` + `build_client_config`, so this needed **zero**
  submodule edits.
- **On-device `provider_admin` branch** for all eight functions, gated on
  `selected_uar_deployment_mode() == EmbeddedLibrary`, with identical return
  types so the Flutter/FFI surface was unchanged. The catalog comes from
  `ModelCatalog::global()` plus a synthetic on-device provider entry.
- **Removed the force-cloud write** behind `#[cfg(not(ios/android))]`.
- Flutter dropped the URL-gate in `RustSettingsRepository.load()` and replaced
  the service form with a read-only "Embedded · on-device" badge; the on-device
  model is pinned as a first-class provider in the same unified picker.

## Verification

Host tests cover routing (local vs cloud via a mock selector), the config
round-trip, `provider_admin` save→list→delete, and that the force-cloud trap does
not fire under `cfg(ios)`. Device-verified on iPhone with no service URL present.

## Failure/lesson

The instinct was to "register the cloud provider in the registry" — which would
have listed correctly and executed wrongly, because the kernel binds one driver
for the run's lifetime. Reading how the runtime actually holds the driver, before
designing, is what turned a plausible-but-broken approach into a correct one.

## Next experiment

Extend the same routing seam to per-turn model overrides once the SDK exposes a
per-request model hint.
