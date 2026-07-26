---
type: Reference
id: m5b-model-policy-pickers-ready-for-device-verification
title: M5b model policy pickers ready for device verification
tags:
- model-runtime-per-device
- knowme
- uar
- model-policy
- flutter
- react
- device-verification
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-25T14:45:20.498268+00:00
created_at: 2026-07-25T14:45:20.498268+00:00
updated_at: 2026-07-25T14:45:20.498268+00:00
revision: 0
---

## Phase context

- Phase: `model-runtime-per-device`
- Project: `KnowMe`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system`
- Captured: `2026-07-25T14:33:57Z`
- Position: `model-runtime-per-device › M5b › device-verify`
- Status: `building`

## Phase goals

- Ship `Qwen3.5-9B Q4_K_M` with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM `Gemma 4 E2B`, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## M5b implementation status

M5b is code-complete and pushed in both repositories. Device verification is pending while the release build/install runs.

| Sub-task | Scope | Location / revision |
|---|---|---|
| M5b.1 | UAR SDK conversation-policy surface and `effective_config` | UAR `46544c2`, ADR 0015 |
| M5b.2 | KnowMe `control_plane` embedded wiring | KnowMe `bb9bb9b` |
| M5b.3 | 3-tier model pickers in React and Flutter | KnowMe `d16fb56` |

Validation already completed:

- Backbone tests green.
- `tsc` clean.
- `dart analyze lib/` clean.
- Accessibility gate passed for WCAG 2.2 AA.
- OpenSpec strict validation passed.

## Build and install state

- Build is compiling; Rust is unchanged from the last deploy, so the work is primarily Flutter/Xcode relink and should be faster than a cold build.
- Release build/install job: `bdocegr70`.
- Build monitor: `bw1vri93y` armed.
- Planned install handling: detach `flutter run` after install so the app is debugger-free and unplug-safe.

## Device verification checklist

After the build lands and KnowMe is installed on device:

1. Fully quit and reopen KnowMe.
2. Verify the global picker:
   - Path: Settings → Runtime & policy → **Default model**.
   - Pick a provider/model.
   - Expected persistence target: `run_policy.global`.
3. Verify the per-agent picker:
   - Path: Settings → Agents → select an agent → **Model** picker above the JSON editor.
   - Set a model different from the global default.
4. Verify the per-conversation picker:
   - Path: chat policy toolbar → **Model** chip.
   - Open the picker and set a third model.
5. Verify precedence and provenance:
   - With all three levels set, send a message.
   - Expected precedence: conversation > agent > global.
   - The provenance chip should show the conversation model.
   - Clear the conversation model; expected fallback is the agent model.
   - Clear the agent model; expected fallback is the global model.
   - Clear all model selections; expected fallback is `On-device`.

## Remaining work after verification

- Complete device verification for M5b picker behavior and precedence.
- Continue with M7 React work for macOS React settings.
- Continue knowledge-capture D.

# Citations

1. stdin
2. manual:KnowMe/model-runtime-per-device