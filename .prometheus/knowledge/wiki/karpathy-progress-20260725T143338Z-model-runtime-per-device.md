---
type: Reference
id: karpathy-progress-20260725T143338Z-model-runtime-per-device
title: "M5b complete: 3-tier model-route pickers on React + Flutter (global/agent/conversation)"
tags:
- karpathy-progress
- model-runtime-per-device
- in_progress
sources:
- conversation:operator-agent
timestamp: 2026-07-25T14:33:38Z
created_at: 2026-07-25T14:33:38Z
updated_at: 2026-07-25T14:33:38Z
revision: 1
---

## Intent

Completed the full 3-tier (Cherry-Studio-style) model selection UI on both surfaces over the embedded resolver. Shared catalog-driven provider->model cascading picker: React ModelRoutePicker (desktop/src/features/settings/components/ModelRoutePicker.tsx, sources useProviderStore catalog: configured providers + on-device) and Flutter ModelRoutePicker (mobile/lib/features/settings/presentation/widgets/model_route_picker.dart, sources snapshot.providers). Wired into 3 scopes each: GLOBAL (React GlobalRunPolicyPanel + Flutter _GlobalPolicyPanel -> run_policy.global.model), PER-AGENT (React agent editor free-text inputs upgraded to picker; Flutter picker reads/writes the policy JSON controller), PER-CONVERSATION (React ChatPolicyToolbar Model popover + Flutter chat_policy_toolbar Model modal -> patchPolicy model). Precedence conv>agent>global resolves in UAR (M5b.1/M5b.2 backbone); the M6c provenance chip reflects the winner. on-device provider renders as 'On-device' everywhere; inherit option clears the route.

## Observed state and verification

React: npx tsc --noEmit clean (no ModelRoutePicker/UarAdminPanels/ChatPolicyToolbar errors). Flutter: dart analyze lib/ -> No issues found. a11y-gate WCAG 2.2 AA pass: accessible names on all selects (aria-label / InputDecoration labelText), 44px targets (_ModeButton minWidth/minHeight 44, h-11 triggers), keyboard operable (Radix Select / Material dropdown), status by text not color (modelSummary chips). Committed+pushed KnowMe bb9bb9b..d16fb56. M5b.1 UAR 46544c2 (ADR0015) + M5b.2 KnowMe bb9bb9b already landed.

## Decision and lesson

Status: in_progress. Preserve evidence, distinguish compile proof from runtime proof, and do not narrow the active goal.

## Next experiment

Device verify: build+deploy release to iPhone (running bdocegr70), confirm all 3 pickers render + precedence conv>agent>global takes effect + provenance chip matches. Then remaining plan items: M7-react (macOS React drop service form when embedded), knowledge-capture D (7 karpathy records).
