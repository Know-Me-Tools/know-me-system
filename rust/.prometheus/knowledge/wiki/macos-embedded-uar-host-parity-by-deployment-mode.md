---
type: Reference
id: macos-embedded-uar-host-parity-by-deployment-mode
title: macOS embedded UAR host parity by deployment mode
tags:
- knowme
- macos
- uar
- embedded-runtime
- react-tauri
- cross-platform-parity
links:
- mobile-embedded-uar-provider-config-without-service-url
- three-tier-model-routing-and-response-provenance-in-knowme
sources:
- macos-embedded-uar-host.md
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-25T21:47:22.285394+00:00
created_at: 2026-07-25T21:47:22.285394+00:00
updated_at: 2026-07-25T21:47:22.285394+00:00
revision: 0
---

## Context

- **Project:** KnowMe
- **Phase:** `model-runtime-per-device`
- **Captured:** `2026-07-25`
- **Status:** executing
- **Platform:** macOS React+Tauri desktop

## Intent

Define cross-platform parity precisely for UAR, because UAR deployment mode intentionally differs by platform. The invariant is not “identical behavior on every platform”; it is:

- **Embedded surfaces behave identically to other embedded surfaces.**
- **Service surfaces behave identically to other service surfaces.**

This prevents removing service configuration from platforms where it is still meaningful while eliminating it on embedded hosts where it is not.

## Deployment-mode matrix

| Platform | UAR mode | Provider config path |
|---|---|---|
| iOS / Android (Flutter) | `EmbeddedLibrary` | on-device `provider_admin` |
| macOS (React+Tauri) | was `HttpService` → now `EmbeddedLibrary` | on-device `provider_admin` |
| Windows / Linux | `HttpService` | service client, unchanged |
| Web | REST, login-gated | service client, unchanged |

macOS already ran MLX in-process via `MlxcEngine`, but did not build an embedded UAR. Its lane resolved to `HttpService`, causing the app to demand a UAR service URL even though no service URL was needed.

The mobile precedent is documented in [Mobile embedded UAR provider config without service URL](/mobile-embedded-uar-provider-config-without-service-url.md): embedded hosts configure providers, models, and keys through on-device `provider_admin`, not through a service URL form.

## Decision

macOS is an embedded UAR host when running the React+Tauri desktop app:

- `bootstrap_inner` builds `UarEmbeddedRuntime::build_local` under `#[cfg(target_os = "macos")]`.
- The embedded runtime is built from the existing `MlxcEngine` inference path plus the MLX provider config.
- `state::embedded_uar()` is therefore present on macOS.
- `selected_uar_deployment_mode` adds macOS to the `None → EmbeddedLibrary` arm.
- The force-cloud guard excludes macOS so saving runtime settings does not flip macOS out of embedded mode.

## React settings behavior

React settings branch on the live runtime status field `targetMode`:

- When `targetMode == embedded_library`, the URL/key/Save/Test form is hidden.
- The UI shows an `Embedded · on-device` badge instead.
- The badge reports kernel readiness.

No new FFI was required. `uar_runtime_status()` already reported `targetMode`; on embedded hosts it resolves through the in-process runtime status.

The shared model-picker and provenance UI work should still reach web and desktop surfaces where applicable; the routing/provenance behavior is tracked in [Three-tier model routing and response provenance in KnowMe](/three-tier-model-routing-and-response-provenance-in-knowme.md).

## Web scope

Web was explicitly out of scope for the embedded-host change:

- The axum server serves the app and fronts UAR.
- Providers are per-user and login-gated.
- UAR/BossFang connectivity is subscription-gated.
- Web remains a REST/service-client surface.
- Only shared UI pieces, such as model pickers and provenance chips, apply to web.

## Deliberate non-change

`UarRuntimeStatus::current()` was left unchanged.

Rationale: it is a static per-platform projection. Making it claim `EmbeddedKernel` on macOS would assert a kernel that may not be initialized. A test caught this incorrect assumption, and the test was correct.

## Verification

- `cargo check -p tauri-plugin-gen-ui` passed.
- TypeScript compilation passed.
- Settings tests passed: 3/3.
- Service path remained intact for Windows/Linux/Web.

## Runtime certification still needed

Next experiment:

1. Launch the macOS app.
2. Confirm no UAR service URL is required.
3. Run an agent turn on the local route.
4. Run an agent turn on a configured cloud route.

## Lesson

“Same behavior on every platform” is the wrong target when deployment modes differ by design. The useful engineering invariant is parity within each mode. Writing down the deployment-mode table before coding prevented breaking service surfaces while fixing embedded macOS behavior.

# Citations

1. macos-embedded-uar-host.md
2. manual:KnowMe/model-runtime-per-device