---
type: Reference
id: karpathy-progress-20260726T011118Z-model-runtime-per-device
title: "Mode selector redesign: single picker + per-conversation lane on every send path"
tags:
- karpathy-progress
- model-runtime-per-device
- in-progress
sources:
- conversation:operator-agent
timestamp: 2026-07-26T01:11:18Z
created_at: 2026-07-26T01:11:18Z
updated_at: 2026-07-26T01:11:18Z
revision: 1
---

## Intent

Replaced the fixed Orchestrator/Agent/Local button row with one trigger labelled by the current selection that opens a sheet listing every catalog agent plus Local. Fixed two defects behind 'Local does nothing': (1) the embedded control_plane catalog returned agents as a bare array while the service returns {runtime_agents, federated_agents}, so every embedded surface saw an empty agent list; (2) setMode had no try/catch unlike its siblings load()/patch(), so set_active_lane's deliberate rejection escaped into an async void and state.error was never set. Added a side-effect-free lane_availability probe so unavailable modes render disabled with a reason instead of failing on tap. Audit then found the per-conversation lane fix only covered send() — send_with_attachments, send_content and send_interaction still read the global active_lane setting, so an attachment sent from a Local conversation ran on the global lane.

## Observed state and verification

cargo test -p gen_ui_agent --lib: 71 passed 0 failed, including new chat_modes_map_onto_execution_lanes and every_send_path_resolves_the_lane_per_conversation. flutter test: 28 passed, 3 failed — the same 3 (settings workspace dark/light, narrow-phone categories) fail at baseline with changes stashed, verified by git stash -u. New test/features/chat/chat_mode_picker_test.dart: 5/5 pass covering label-reflects-selection, sheet contents, disabled-with-reason, and rejected-switch-keeps-selection. flutter analyze on the toolbar: no issues. Commits 0a97500 and cf48e98.

## Decision and lesson

Status: in-progress. Preserve evidence, distinguish compile proof from runtime proof, and do not narrow the active goal.

## Next experiment

Install the release APK on Samsung SM-S936U (R5GYB4AZD7A) and verify the picker in a brand-new and an existing conversation: agent list populated, Local selectable, label changes, send_local markers in logcat, and a second conversation retaining its own mode. Physical iPhone was not attached during this session (only the simulator) so iOS verification is still outstanding.
