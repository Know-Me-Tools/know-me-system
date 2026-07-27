---
type: Reference
id: karpathy-progress-20260726T084513Z-model-runtime-per-device
title: "iOS Local mode certified: mid-list system message fix + editable local prompt"
tags:
- karpathy-progress
- model-runtime-per-device
- in-progress
sources:
- conversation:operator-agent
timestamp: 2026-07-26T08:45:13Z
created_at: 2026-07-26T08:45:13Z
updated_at: 2026-07-26T08:45:13Z
revision: 1
---

## Intent

Operator confirmed on iPhone 17 Pro Max that the previously failing existing conversation now responds in Local mode, and that the editable local system prompt works. Root cause of the MLX TemplateException("System message must be at the beginning.") was three context slices injecting InferenceRole::System messages AFTER history begins: truncate_middle's omission marker, the conversation-summary turn prepended to the retained tail, and the memory/knowledge blocks. ContextBuilder::assemble admits the real system prompt at index 0, so anything those add is necessarily later. A fresh conversation has too little history to trigger compaction, which is exactly why the defect only reproduced on an existing thread — the operator's report that it was an existing conversation was the signal that localized it. Fixed by folding every non-leading System message into the leading one, once, after all slices contribute, rather than dropping the content (it is load-bearing context) or teaching three separate producers about placement. Content and turn order are both preserved. Separately, the hardcoded local system prompt became a settings-backed value with a Settings editor: reset sources its copy from Rust so the default cannot drift, and save reads the effective value back so clearing the field visibly restores the default instead of leaving an empty editor.

## Observed state and verification

Operator confirmation on iPhone 17 Pro Max (00008150-00125461363A401C / 3CF53A89-6EDC-544E-8555-1E9BE6F3F452): the conversation that previously rendered 'MLX generation: TemplateException(message: Optional("System message must be at the beginning."))' now responds, and the local system prompt editor works. Build: Runner.app 213.4MB, release, installed as tools.know-me.knowme, PID 27025 stable. Commits 82d0c69 (context fix + settings backend + FFI) and ff7d800 (Settings UI). Tests: 4 new gen_ui_context tests pinning the merge, the short-conversation no-op, the no-leading-system case, and blank slices not padding the prompt (9/9); gen_ui_agent 72/72; 4 new Dart settings tests (saved prompt reads back verbatim, clearing restores the default, reset sources copy from Rust, unset reports default); Flutter suite 35 pass / 3 pre-existing settings-workspace failures unchanged at baseline. a11y gate on the new editor found and fixed a missing accessible name on the TextArea (a placeholder is not a name — it disappears once text is entered) and unnamed action buttons.

## Decision and lesson

Status: in-progress. Preserve evidence, distinguish compile proof from runtime proof, and do not narrow the active goal.

## Next experiment

Change 5 litertlm-device-proof remains blocked: the Samsung SM-S936U reads adb unauthorized and needs an on-screen 'Allow USB debugging' tap before the certification suite can run. Attachment flow still has no certification script. Durable embedded persistence (embed-uar-mobile-offline task 3.1) is still unstarted — the spawned session landed no commits, so InMemoryProvider at uar.rs:878 still makes per-conversation mode reset on app restart.
