---
type: Reference
id: android-litert-lm-uar-provider-resolution-marker-on-samsung
title: Android LiteRT-LM UAR provider resolution marker on Samsung
tags:
- model-runtime-per-device
- android-device-cert
- litert-lm
- uar
- gemma
- local-inference
- tool-shortlist
links:
- m5b-install-monitor-armed-for-model-runtime-picker-verification
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-25T22:49:23.351331+00:00
created_at: 2026-07-25T22:49:23.351331+00:00
updated_at: 2026-07-25T22:49:23.351331+00:00
revision: 0
---

## Phase context

- Phase: `model-runtime-per-device`
- Project: `KnowMe`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system`
- Captured: `2026-07-25T22:45:33Z`
- Position: `model-runtime-per-device › litertlm-local-tools › device-verify`
- Status: `monitoring`
- Device lane: Samsung Android local text-agent inference certification

## Phase goals

- Ship `Qwen3.5-9B Q4_K_M` with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM `Gemma 4 E2B`, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Certified marker

Observed first real Android device marker on Samsung:

```text
universal_agent_runti..: Using per-agent provider settings
  provider=knowme-local-litert-lm  model=gemma-4-e2b-it-litertlm
```

This marker certifies:

- Embedded UAR is live on Android and resolving run policy in-process.
- Per-agent provider settings are being applied on device.
- M5b three-tier resolution works on Android, not only iOS.
- Android lane resolved to the planned provider/model pair:
  - Provider: `knowme-local-litert-lm`
  - Model: `gemma-4-e2b-it-litertlm`
- The selected artifact is the generic CPU artifact intended as the first universal correctness baseline.
- Provenance data is populated: provider and model are both non-empty, confirming M6c backfill behavior on Android.

## Path distinction

The observed marker came from the **Orchestrator/Agent** path, where UAR resolves an agent provider. This is valid for certifying embedded UAR and per-agent policy resolution.

The tool-shortlist change still needs certification through the **On-device → Local** tab because that path exercises `send_local`.

## Required next verification

Watch the PID-scoped Samsung monitor `bbh3665mr` for the expected first-run sequence:

1. `Downloading`
2. Integrity/verification step
3. `ModelLoading`
4. Token deltas

The initial `Gemma 4 E2B` pull is expected to be approximately 3 GB and may take time on first run.

After download/load completes, operator actions needed:

1. Open **On-device → Local** tab and send a prompt to exercise `send_local` and the shortlist path.
2. Send a tool-requiring prompt to certify the full sequence:
   - shortlist
   - gate
   - execution
   - continuation

## Monitoring state

- Position: `model-runtime-per-device › litertlm-local-tools › device-verify`
- Status: `monitoring`
- Current progress: provider/model resolution marker certified.
- Monitor: `bbh3665mr`, PID-scoped, clean of system noise.
- Awaiting: model download/load markers and Local-tab send.

## Related context

This Android certification lane complements the iOS model-runtime device certification work, including prior install and picker verification tracked in [M5b install monitor armed for model runtime picker verification](/m5b-install-monitor-armed-for-model-runtime-picker-verification.md).

# Citations

1. stdin
2. manual:KnowMe/model-runtime-per-device