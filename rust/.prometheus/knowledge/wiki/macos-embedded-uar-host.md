---
type: Reference
id: macos-embedded-uar-host
title: macOS as an embedded UAR host — cross-platform parity per deployment mode
tags:
- knowme
- model-runtime-per-device
- macos
- uar
- embedded
- desktop
- react
- parity
links:
- mobile-embedded-uar-provider-config
- three-tier-model-selection-and-provenance
- desktop-macos-mlx-floor-raise-and-ios-thinking-split
sources:
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-25T21:45:00+00:00
created_at: 2026-07-25T21:45:00+00:00
updated_at: 2026-07-25T21:45:00+00:00
revision: 0
---

## Context

- **Project:** KnowMe
- **Phase:** `model-runtime-per-device`
- **Captured:** `2026-07-25`
- **Status:** `executing`

## Intent

Make the platform story coherent. "Behaves the same everywhere" had to be defined
precisely, because the UAR deployment mode differs by platform *by design*.

## Observed state

The parity rule that emerged, verified from code rather than assumed:

| Platform | UAR mode | Provider config path |
|---|---|---|
| iOS / Android (Flutter) | `EmbeddedLibrary` | on-device `provider_admin` |
| macOS (React+Tauri) | was `HttpService` → now `EmbeddedLibrary` | on-device `provider_admin` |
| Windows / Linux | `HttpService` | service client (unchanged) |
| Web | REST, login-gated | service client (unchanged) |

The guarantee is therefore: **embedded surfaces behave identically to each other,
service surfaces behave identically to each other.** The service-URL form is
removed only from embedded surfaces, where it is meaningless — it remains correct
on service surfaces.

macOS already ran MLX in-process via `MlxcEngine` but did **not** build an
embedded UAR: its lane resolved to `HttpService`, so it demanded a service URL it
did not need.

Web was explicitly scoped out: the axum server both serves the app and fronts the
UAR, with per-user providers behind a login and a subscription for UAR/BossFang
connectivity. Only the *shared* pieces (model pickers, provenance chip) reach it.

## Decision and delta

- `bootstrap_inner` builds `UarEmbeddedRuntime::build_local` under
  `#[cfg(target_os = "macos")]` from the existing `MlxcEngine` inference plus the
  MLX provider config, so `state::embedded_uar()` is present on macOS.
- `selected_uar_deployment_mode` adds macOS to the `None → EmbeddedLibrary` arm,
  and the force-cloud guard excludes it.
- React settings branch on the **live** runtime status' `targetMode`: when
  `embedded_library`, the URL/key/Save/Test form is replaced by an
  "Embedded · on-device" badge showing kernel readiness. No new FFI was needed —
  `uar_runtime_status()` already reported `targetMode`, and on embedded hosts it
  resolves through the in-process runtime's own status.

## Verification

`cargo check -p tauri-plugin-gen-ui` green (service path intact); `tsc` clean;
settings tests 3/3. A deliberate non-change: `UarRuntimeStatus::current()` — the
*static* per-platform projection — was left alone, because making it claim
`EmbeddedKernel` on macOS would assert a kernel that may not be initialized. A
test caught that and was right to.

## Failure/lesson

"Same behavior on every platform" is the wrong target when the deployment mode is
intentionally different. The useful invariant is parity *within* each mode, and
writing that table down before coding is what kept the service surfaces from
being broken in pursuit of a slogan.

## Next experiment

Runtime-certify the macOS app: launch it, confirm no service URL is required, and
run an agent turn on both local and cloud routes.
