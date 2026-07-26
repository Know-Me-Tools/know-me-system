---
type: Reference
id: knowme-model-runtime-ios-deploy-stable-local-certification-pending
title: KnowMe model runtime iOS deploy stable; Local certification pending
tags:
- model-runtime-per-device
- ios-device-cert
- local-inference
- litert-lm
- qwen
- uar
- phase-tracking
- flutter-release
links:
- memory-fix-install-watch-armed-for-unplug-safe-ios-verification
- m5b-install-monitor-armed-for-model-runtime-picker-verification
- ios-cert-monitor-holding-for-local-send-with-benign-nw-path-noise
- ios-cert-monitor-waiting-on-local-send-with-nw-path-noise
- ios-cert-monitor-still-quiet-before-local-send-taps
sources:
- stdin
timestamp: 2026-07-26T07:57:46.215630+00:00
created_at: 2026-07-26T07:57:46.215630+00:00
updated_at: 2026-07-26T07:57:46.215630+00:00
revision: 0
---

## Phase context

- Phase: `model-runtime-per-device`
- Project: `KnowMe`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system`
- Captured: `2026-07-26T07:49:38Z`
- Position: `model-runtime-per-device › embedded-uar-offline-agents › litertlm-device-proof`
- Status: `executing`

## Phase goals

- Ship `Qwen3.5-9B Q4_K_M` with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM `Gemma 4 E2B`, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## iOS deployment status

The release build compiles, installs, launches, and remains stable on the physical iPhone.

- Built: `Runner.app`, 213.3 MB, release configuration.
- Build result: `Xcode build done` in 448 s.
- Installed bundle: `tools.know-me.knowme`.
- Device: iPhone 17 Pro Max, UDID `3CF53A89-6EDC-544E-8555-1E9BE6F3F452`.
- Runtime state: launched as PID `26732`; same PID remained alive after >65 s.
- Interpretation: no boot-time crash from Rust FFI or MLX initialization.

This continues the iOS runtime certification path from prior install-monitor work such as [Memory-fix install watch armed for unplug-safe iOS verification](/memory-fix-install-watch-armed-for-unplug-safe-ios-verification.md) and [M5b install monitor armed for model runtime picker verification](/m5b-install-monitor-armed-for-model-runtime-picker-verification.md).

## iOS Local mode certification gap

Local mode on iOS is **not yet certified**.

Reason:

- `devicectl` supports install, launch, and process listing in this session.
- It does not provide screen control or usable log streaming for this verification path.
- The operator must manually drive the UI and confirm routing behavior.

Required tap-through:

1. Open a chat on the iPhone.
2. Tap the mode button.
3. Select **Local**.
4. Send a message.
5. Record whether Local routing/generation succeeds.

Android evidence suggests the shared Rust fix should apply to iOS, but that is only an expectation. It is not certification until an actual iOS Local send is observed. This replaces earlier waiting states where iOS certification was blocked on Local-send taps, including [iOS cert monitor holding for Local send with benign nw_path noise](/ios-cert-monitor-holding-for-local-send-with-benign-nw-path-noise.md), [iOS cert monitor waiting on Local send with nw_path noise](/ios-cert-monitor-waiting-on-local-send-with-nw-path-noise.md), and [iOS cert monitor still quiet before local send taps](/ios-cert-monitor-still-quiet-before-local-send-taps.md).

## Phase progress

Active phase `model-runtime-per-device`: **4 / 5 changes complete**.

Completed changes:

- `android-litertlm-runtime`
- `litertlm-native-adapter`
- `document-uar-karpathy-knowledge-strategy`
- `litertlm-local-tools`

Remaining change:

- `litertlm-device-proof`
  - Open work includes runtime diagnostics, physical integration scenarios, Android deploy, evidence recording, and iOS Local certification.
  - iOS deploy is now complete at the install/launch/stability level.

Android status:

- Android Local mode already certified on Samsung `SM-S936U`.
- Android local text-agent inference path uses LiteRT-LM `Gemma 4 E2B`.
- CPU remains the universal correctness baseline.

## Child phase status: embedded UAR offline agents

Child phase `embedded-uar-offline-agents` / `embed-uar-mobile-offline`: **8 done / 14 open**.

Open work:

- `3.1` durable sandboxed persistence.
  - Current impact: per-conversation mode resets after restart.
  - Restart persistence is blocked by `InMemoryProvider` at `uar.rs:878`.
- `3.2`–`3.4` embedded FFI admin operations, offline resource advertisement, seeded-agent protection.
- `4.1`–`4.4` Flutter repository/provider wiring, event-order preservation, diagnostics, desktop alignment.
- `5.1`–`5.6` dependency graph, cross-compile, desktop certification, Android airplane-mode certification, iOS airplane-mode certification, evidence recording.

## Other open workstreams

- `qwen35-local-tools`: 0 / 6 complete.
- `qwen35-device-proof`: 0 / 5 complete.
- `qwen35-mobile-vision`: 5 / 6 complete; remaining work is image-understanding verification on both phones.
- UI tails:
  - `uar-ui-chat-reskin`: 26 / 29 complete.
  - `uar-ui-token-convergence`: 20 / 22 complete.
  - `uar-ui-shell-navigation`: 15 / 17 complete.
  - `uar-ui-a2ui-retoken`: 16 / 17 complete.
- `uar-ui-verify-gates`: closed at 23 / 23.

## Tracking defects

Two phase-tracking defects should be fixed before the next reflect gate:

1. `.kbd-orchestrator/phases/model-runtime-per-device/progress.json` is malformed JSON.
   - `python3 -m json.tool` rejects it.
   - Any reader of that file may fail or silently skip phase state.
2. `childPointer` points to `embedded-uar-offline-agents`, but that phase directory has no `progress.json`.
   - The child phase has no tracked state file.

These defects do not block engineering work, but they make phase counters unreliable as a source of truth.

## Next actions

1. Operator certifies iOS Local mode with the tap-through flow: chat → mode button → Local → send.
2. Repair malformed `.kbd-orchestrator/phases/model-runtime-per-device/progress.json`.
3. Add missing child phase state for `embedded-uar-offline-agents/progress.json`.
4. Implement durable sandboxed persistence for `embed-uar-mobile-offline` task `3.1` so mode selection survives restart.
5. Continue `litertlm-device-proof` evidence recording after physical-device verification.

# Citations

1. stdin