---
type: Reference
id: liter-llm-fork-fix-and-297-provider-catalog-refresh-for-uar
title: liter-llm fork fix and 297-provider catalog refresh for UAR
tags:
- model-runtime-per-device
- liter-llm
- uar
- provider-catalog
- models-dev
- dependency-management
links:
- m5a-embedded-uar-policy-guidance-after-fastembed-fix
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-24T20:57:19.989063+00:00
created_at: 2026-07-24T20:57:19.989063+00:00
updated_at: 2026-07-24T20:57:19.989063+00:00
revision: 0
---

## Context

Phase: `model-runtime-per-device` for KnowMe.

Phase goals:

- Ship `Qwen3.5-9B Q4_K_M` with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM `Gemma 4 E2B`, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Root cause

`GQAdonis/liter-llm` fork `main` at `b0663bc` contained a regression from a `models.dev` sync:

- `ahash`, `arc-swap`, and `async-trait` were reverted from local submodule-only hardcoded dependencies back to `workspace = true`.
- This fails when `liter-llm` is vendored as a nested path dependency inside the UAR workspace, because those workspace dependencies cannot resolve from that location.

## Fix landed

The dependency fix was committed upstream in the fork rather than carried as a local UAR patch:

- `GQAdonis/liter-llm` `main` now points to `c37e98411`.
- All three relevant references were updated to `c37e98411`.
- UAR vendors `liter-llm` unpatched and builds cleanly.

The dormant nested submodule at `prometheus-skill-system/tools/liter-llm` was intentionally left untouched because it is not in any build graph.

## Catalog refresh

The provider catalog generator failed because `cargo metadata` output exceeded Node's default 1 MB `maxBuffer`, causing `ENOBUFS`.

Fix:

- Raised generator `maxBuffer` to 128 MB.
- Regenerated `provider_catalog.json`.
- Provider count increased from `269` to `297`.
- New providers include OpenCode Zen and OpenCode Go, plus the latest `models.dev` sync.
- Updated `SNAPSHOT.md` with the new SHA, provider count, and `liter-llm` commit.

Runtime effect:

- UAR builds against the refreshed 297-provider catalog.
- `ModelCatalog::global()` exposes the new providers.
- Providers now surface in both:
  - embedded provider picker for mobile/macOS,
  - service picker for desktop/web.

## Commits

- `GQAdonis/liter-llm` `main`: `c37e98411` — dependency hardcoding fix, pushed.
- `Prometheus-AGS/universal-agent-runtime` `desktop-stable-port-opsx`: `f68ab06` — catalog refresh and integration, pushed.
- KnowMe `feat/ios-mlx-inference`: `30e9f41` — git dependency revision, UAR pointer, and catalog update.

## Current position

`model-runtime-per-device › liter-llm-update › done`

Status:

- `liter-llm` fixed end-to-end in the fork.
- UAR catalog regenerated from 269 to 297 providers.
- UAR and KnowMe pointers updated.
- Builds are green.

## Next work

Proceed to M5b model-route pickers:

- global tier picker,
- per-agent tier picker,
- embedded route now unblocked after the embedded UAR policy work tracked in [M5a embedded UAR policy guidance after fastembed fix](/m5a-embedded-uar-policy-guidance-after-fastembed-fix.md),
- per-conversation picker deferred until an SDK conversation-policy surface exists.

Then proceed to M7-react:

- macOS React settings should drop the service form.

Open dependency:

- iOS think-split device certificate still requires operator taps.

# Citations

1. stdin
2. manual:KnowMe/model-runtime-per-device