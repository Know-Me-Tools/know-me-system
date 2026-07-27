---
type: Reference
id: karpathy-progress-20260726T071737Z-model-runtime-per-device
title: "Local mode root cause: conversation_policy leaked the storage wrapper"
tags:
- karpathy-progress
- model-runtime-per-device
- in-progress
sources:
- conversation:operator-agent
timestamp: 2026-07-26T07:17:37Z
created_at: 2026-07-26T07:17:37Z
updated_at: 2026-07-26T07:17:37Z
revision: 1
---

## Intent

Device verification disproved three earlier hypotheses (R8 stripping, NoLocalEngine, patch() clobbering) and forced a real trace. uar.rs::conversation_policy returned the storage-layer ConversationPolicyRecord ({conversation_id, policy:{...}, updated_at}) while every consumer expects the bare RunPolicy with chat_mode at the top level: save_conversation_policy deserializes its own input directly as RunPolicy, the TypeScript binding declares Promise<RunPolicy|null>, and Flutter assigns the result straight to state.policy. Flutter therefore read policy['chat_mode'] as absent (label stuck on the default) and sent the wrapper back to save, where every RunPolicy field being serde(default) meant it deserialized to DEFAULTS WITHOUT ERRORING — the write succeeded and persisted chat_mode uar, silently discarding the selection. That silent-success is why logcat was empty: there was structurally no error to log, so log-driven debugging could not find it. Fixed at the accessor (unwrap to record.policy on both read and echo) rather than in each client, because the React store and the HTTP path already assumed the bare shape — Rust was the single deviant, and patching Flutter alone would have left the desktop bug latent.

## Observed state and verification

code-explorer trace with citations: rust/vendor/universal-agent-runtime/src/uar/domain/policy.rs:22-30 (ConversationPolicyRecord struct, no serde flatten), rust/crates/gen_ui_agent/src/uar.rs:985-1014 (pass-through, no reshaping), desktop/.../tauri-plugin-gen-ui/src/index.ts:413 (declares Promise<RunPolicy|null>), desktop/src/features/chat/stores/chatControlStore.ts:102 (assigns policy ?? defaultPolicy() top-level). cargo test -p gen_ui_agent --lib: 72 passed 0 failed. New Rust test conversation_chat_mode_is_read_from_the_bare_run_policy and Dart chat_control_policy_shape_test both assert the WRAPPER shape does NOT read as a mode. flutter test: 31 passed, 3 failed — same 3 pre-existing baseline failures. Commit e81f303. CORRECTION to the prior record: an earlier turn reported the release APK build as succeeded; it had actually FAILED (GeneratedPluginRegistrant.java:44 package dev.flutter.plugins.integration_test does not exist) because running flutter test mid-build regenerated the registrant with a dev-only plugin reference. The APK timestamp I read was stale. Device verification of Local is therefore still NOT done.

## Decision and lesson

Status: in-progress. Preserve evidence, distinguish compile proof from runtime proof, and do not narrow the active goal.

## Next experiment

Rebuild the release APK without running flutter test concurrently, install on Samsung SM-S936U (R5GYB4AZD7A), select Local, and confirm the label changes, survives an app restart, routes through send_local, and that a second conversation keeps its own mode. iPhone remains detached (xcrun devicectl reports both as unavailable) so iOS certification is still outstanding.
