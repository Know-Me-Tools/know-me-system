---
type: Reference
id: m5a-knowme-embedded-control-plane-completed-for-uar-admin
title: M5a KnowMe embedded control_plane completed for UAR admin
tags:
- model-runtime-per-device
- uar
- embedded-runtime
- control-plane
- model-routing
- runtime-verification
- knowme
links:
- m5a-finish-delegated-after-killed-embedded-uar-policy-worker
- m5a-embedded-uar-policy-guidance-after-fastembed-fix
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-24T20:54:19.480103+00:00
created_at: 2026-07-24T20:54:19.480103+00:00
updated_at: 2026-07-24T20:54:19.480103+00:00
revision: 0
---

## Phase context

- Phase: `model-runtime-per-device`
- Project: `KnowMe`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system`
- Captured: `2026-07-24T20:30:22Z`
- Position: `model-runtime-per-device › m5a-knowme › done`
- Commit: `ba44060`

## Phase goals

- Ship `Qwen3.5-9B Q4_K_M` with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM `Gemma 4 E2B`, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Completed work

The `control_plane` embedded branch is committed for iOS, Android, and macOS embedded operation when no service URL is configured. This continues the embedded UAR policy work tracked through [M5a finish delegated after killed embedded UAR policy worker](/m5a-finish-delegated-after-killed-embedded-uar-policy-worker.md) and the earlier runtime verification notes such as [M5a embedded UAR policy guidance after fastembed fix](/m5a-embedded-uar-policy-guidance-after-fastembed-fix.md).

Embedded dispatch now uses the in-process SDK admin surface instead of requiring a `UAR_SERVICE_URL` for supported operations:

- `admin_snapshot`
  - Returns real settings.
  - Returns setting types.
  - Returns agents from the in-process runtime.
  - Unblocks the mobile global-policy panel and agent editor load path.
- `update_setting`
  - Writes `run_policy.global` in-process.
  - Makes the global model-route tier writable in embedded mode.
- `save_agent` / `delete_agent`
  - Enables embedded agent editing.
- `catalog`
  - Provides the embedded agent list.

Nine functions that still lack an embedded SDK surface now return an explicit `not available on embedded yet` error instead of silently falling through to a missing service URL path. This preserves the no-silent-capabilities behavior and exposes the real remaining SDK gaps.

## Verification

- `gen_ui_agent`: `68/68` tests passing.
- `gen_ui_ffi`: builds successfully.
- Clippy: no new warnings reported.

## Known gap

The per-conversation model-override tier does **not** work in embedded mode yet.

Reason: the SDK admin surface added in M5a covers settings and agents, but does not yet provide conversation-policy CRUD. As a result:

- Embedded:
  - Global tier: ✅ supported.
  - Per-agent tier: ✅ supported.
  - Per-conversation tier: deferred.
- Desktop/web service path:
  - Global tier: ✅ unchanged.
  - Per-agent tier: ✅ unchanged.
  - Per-conversation tier: ✅ unchanged.

Follow-up requirement: add an SDK conversation-policy surface before enabling embedded per-conversation policy CRUD.

## Overall effort status

| Area | Status |
|---|---|
| Reported bug: `missing_uar_service_url` | ✅ fixed |
| M1–M4 embedded local/cloud + Flutter settings | ✅ complete |
| M6/M6b provenance across surfaces | ✅ complete |
| M7-core macOS embedded host | ✅ complete |
| M5a-uar UAR submodule + fastembed fix | ✅ complete |
| M5a-knowme `control_plane` embedded branch | ✅ complete in `ba44060` |
| M5b model pickers | Pending |
| M7-react macOS React settings | Pending |

## Next work

1. M5b: add global and per-agent model-route pickers over the now embedded-capable resolver across all surfaces.
   - Embedded global picker: unblocked.
   - Embedded per-agent picker: unblocked.
   - Embedded per-conversation picker: deferred pending SDK conversation-policy support.
2. M7-react: update macOS React settings to drop the service form.
3. iOS think-split device certificate: still awaiting operator taps.

# Citations

1. stdin
2. manual:KnowMe/model-runtime-per-device