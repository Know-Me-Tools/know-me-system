---
type: Reference
id: model-runtime-phase-status-after-p2-3-ag-ui-encoder-dedup
title: Model runtime phase status after P2.3 AG-UI encoder dedup
tags:
- model-runtime-per-device
- litert-lm
- android-tools
- ag-ui
- uar
- ios-mlx
- phase-status
links:
- memory-fix-install-watch-armed-for-unplug-safe-ios-verification
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-25T22:14:21.983398+00:00
created_at: 2026-07-25T22:14:21.983398+00:00
updated_at: 2026-07-25T22:14:21.983398+00:00
revision: 0
---

## Phase context

- Phase: `model-runtime-per-device`
- Project: `KnowMe`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system`
- Captured: `2026-07-25T22:00:18Z`
- Current position: `model-runtime-per-device › litertlm-local-tools`
- Status: `awaiting_steer`

## Phase goals

- Ship `Qwen3.5-9B Q4_K_M` with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM `Gemma 4 E2B`, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Delivered in this session

P2.3 is complete at commit `4a4541d`.

- KnowMe now calls the UAR canonical AG-UI encoder instead of maintaining its own copy.
- Net diff: `-149/+23` lines.
- Test status: `68/68` green, including the two tests that exercise the AG-UI projection path.
- iOS target compiles.
- Formatting is clean.

## Corrected phase status

Earlier status that P2.3 was the only remaining item was incorrect. The task list for this session only covered the iOS/embedded-UAR workstream, not the entire `model-runtime-per-device` phase.

The KBD phase tracker shows `4/5` changes complete:

```text
model-runtime-per-device: 4/5 changes
✅ android-litertlm-runtime
✅ litertlm-native-adapter
✅ litertlm-device-proof
✅ document-uar-karpathy-knowledge-strategy
🔵 litertlm-local-tools  ← still active
```

The remaining phase change is `litertlm-local-tools`, an Android LiteRT-LM local tools workstream last touched on `2026-07-19` by `codex`. The reflection gate is correctly blocked until `litertlm-local-tools` lands.

## Pushed work inventory

| Work | Commits / references |
|---|---|
| iOS MLX + think-split, device-certified | earlier; follows prior iOS certification/install lanes such as [Memory-fix install watch armed for unplug-safe iOS verification](/memory-fix-install-watch-armed-for-unplug-safe-ios-verification.md) |
| Mobile embedded-UAR provider config | M1–M4 |
| 3-tier model selection + provenance | `d16fb56`, UAR ADRs 0013–0015 |
| Provenance backfill fix | `5af117b` / UAR `b1dda41` |
| Memory fix, device-certified | `b86f43b` / UAR `08a6f4c`, ADR 0016 |
| Chat paste fix | `a9ae966` |
| macOS React embedded settings | `da95881` |
| 7 knowledge records | `5d28212`; pk-ingested and lint-clean |
| AG-UI encoder de-duplication | `4a4541d` |

## Next decision

Two valid next actions:

1. Pick up `litertlm-local-tools`, the outstanding fifth `model-runtime-per-device` phase change for Android LiteRT-LM local tools.
2. Stop here with this session's iOS/embedded-UAR stream complete and pushed.

# Citations

1. stdin
2. manual:KnowMe/model-runtime-per-device