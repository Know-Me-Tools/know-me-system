---
type: Reference
id: karpathy-progress-20260726T162425Z-model-runtime-per-device
title: "litertlm-device-proof: 6 of 9 evidence items captured; cancellation is a real product defect"
tags:
- karpathy-progress
- model-runtime-per-device
- in-progress
sources:
- conversation:operator-agent
timestamp: 2026-07-26T16:24:25Z
created_at: 2026-07-26T16:24:25Z
updated_at: 2026-07-26T16:24:25Z
revision: 1
---

## Intent

Samsung SM-S936U authorization cleared, so the certification suite ran against a fresh instrumented release APK. PASSED: exact-response prompt (visible token KMOK110807), prompt suite exact+mermaid+code (covers the Markdown/Mermaid item), and the certification report capturing backend/memory/thermal plus the new timing markers. The permitted tool call also genuinely succeeded on device — the UI shows knowme_diagnostics running -> Tool completed -> complete -> TOOLSTATUSOK — but its script reported FAIL because it scrapes uiautomator before the UI settles AND expects LiteRT-LM generation markers on a run that went through the Orchestrator/UAR lane, where no such marker can appear. That is a script assumption defect, not a product failure. CANCELLATION IS A REAL PRODUCT DEFECT: generation ran to completion (generate_ms=16566, 1506 chars) despite the cancel tap. Tracing the cause, every publish_cancelled in chat.rs sits at a phase boundary (Queued/ToolExecution/Generation) with cancelled() checked BETWEEN steps, and generate_text is a single blocking call with no interior cancellation check — the AtomicBool at litert_lm.rs:200 guards only the model-download loop. So a stop tap cannot take effect until generation finishes.

## Observed state and verification

Device samsung SM-S936U, API 36, SoC SM8750, arm64-v8a. APK built 11:06 with commit c92fc80 instrumentation. certify-local-model-prompt: G3/G4/G5/G6 PASS, visible token KMOK110807. certify-local-prompt-suite CASES=exact,mermaid,code: 'Prompt suite passed'. write-local-certification-report.sh performance block (machine-captured, not estimated): modelLoadMs=249, generateMs=5554, generatedChars=154, charsPerSecond=27, observedBackend=gemma-4-e2b-it-litertlm. A separate run logged modelLoadMs=485, generate_ms=16566, chars=1506, chars_per_second=90. Thermal AP=45.6C. Memory totalPssKb=3049781. certify-local-cancellation: CANCEL FAIL with generation_seen=1 cancel_tapped=1; logcat shows generation completing normally with no interrupt. Source: grep of cancelled() call sites in chat.rs shows all at phase boundaries; crates/gen_ui_inference/src/litert_lm.rs:200 AtomicBool is inside the download path only.

## Decision and lesson

Status: in-progress. Preserve evidence, distinguish compile proof from runtime proof, and do not narrow the active goal.

## Next experiment

Two defects to fix before this change can close: (1) add an interior cancellation check to the LiteRT-LM generation path so a stop tap interrupts an in-flight generate_text rather than waiting for it to finish; (2) fix certify-local-tool-call.sh to settle the UI before scraping and to not require LiteRT-LM markers on UAR-lane runs. Still unscripted: attachment flow. Still unrun: iOS airplane-mode certification (task 5.5).
