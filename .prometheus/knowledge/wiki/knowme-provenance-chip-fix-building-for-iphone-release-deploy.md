---
type: Reference
id: knowme-provenance-chip-fix-building-for-iphone-release-deploy
title: KnowMe provenance chip fix building for iPhone release deploy
tags:
- model-runtime-per-device
- knowme
- ios-release
- provenance-chip
- effective-run-policy
- flutter-release
- uar
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-24T21:45:57.905701+00:00
created_at: 2026-07-24T21:45:57.905701+00:00
updated_at: 2026-07-24T21:45:57.905701+00:00
revision: 0
---

## Phase context

- Phase: `model-runtime-per-device`
- Project: `KnowMe`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system`
- Captured: `2026-07-24T21:44:22Z`
- Position: `model-runtime-per-device › provenance-chip-fix › build+deploy`
- Status: `executing`
- Target device: iPhone `00008150`

## Phase goals

- Ship `Qwen3.5-9B Q4_K_M` with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM `Gemma 4 E2B`, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Provenance chip fix status

- Unit-level fix is certified.
- The embedded `effective_run_policy` artifact now backfills the real provider/model from the registry default instead of emitting blank values.
- Change scope is surgical:
  - one provenance backfill block in `manager.rs`
  - one regression test
- Formatting is clean.
- Passing test: `effective_run_policy_artifact_backfills_the_registry_default_model`
  - profile/features: `server-full`, `in-memory-backend`

## Build and deployment status

- Release build and install are running under `flutter run`/Xcode build.
- Active build session: `brp4zupgi`
- Build monitor: `b268frpkb`
- Current long pole: Rust workspace recompile, including the `manager.rs` edit, plus link.
- Monitor trigger condition: report when build reaches `Installing and launching` or errors.
- Expected post-build behavior: detach `flutter run` so the release app continues running free of the debugger.

## Verification target

After the release app launches detached from the debugger, the operator should verify that the Orchestrator bubble displays:

```text
orchestrator-agent · On-device
```

The confirmation is required before finalizing the UAR change.

## Known out-of-scope issue

- A pre-existing unrelated `VectorMatcher::new` test-compile break was observed.
- It was not addressed as part of this provenance chip fix.

## Next steps

1. Wait for build monitor `b268frpkb` to report install/launch or error.
2. Detach `flutter run` after the release app is running.
3. Operator verifies Orchestrator provenance label as `orchestrator-agent · On-device`.
4. Finalize UAR change:
   - OpenSpec/ADR update
   - KnowMe submodule pointer bump
   - commit

# Citations

1. stdin
2. manual:KnowMe/model-runtime-per-device