---
type: Reference
id: karpathy-progress-20260725T230739Z-model-runtime-per-device
title: "Android CERTIFIED: R8 keep-rule fix unblocks LiteRT-LM; full tool path proven on Samsung"
tags:
- karpathy-progress
- model-runtime-per-device
- verified
sources:
- conversation:operator-agent
timestamp: 2026-07-25T23:07:39Z
created_at: 2026-07-25T23:07:39Z
updated_at: 2026-07-25T23:07:39Z
revision: 1
---

## Intent

Fixed the Android release blocker and certified the complete local tool path on Samsung SM-S936U (android-arm64, Android 16). ROOT CAUSE (corrected mid-investigation): Flutter's release variant runs R8 even though minifyEnabled is absent from build.gradle.kts — proven by build/app/intermediates/r8_art_profile/release/minifyReleaseWithR8 — and the only generated keep rule was MainActivity from the manifest. KnowMeLiteRtLmBridge is reached ONLY from Rust via JNI reflection by fully-qualified name (android_litert_jni.rs), so R8's reachability analysis could not see it and shrank it out; debug builds worked because R8 does not run there, which is why it survived to a release device test. FIX: new android/app/proguard-rules.pro keeping the bridge class AND members (Rust resolves methods reflectively after loadClass, so member shrinking breaks it too) plus the LiteRT-LM API surface crossing JNI, and proguardFiles wired into the release buildType (without which the rules file is ignored). Verified by dex inspection BEFORE touching the device: classes.dex went from only Ltools/knowme/knowme/MainActivity; to including KnowMeLiteRtLmBridge and its nested $RegisteredOpenApiTool.

## Observed state and verification

Device log chain on Samsung R5GYB4AZD7A after the fix: bridge resolves (no ClassNotFoundException) -> liblitertlm_jni.so loads ok -> LitertLmLoader::Initialize, mmap OK, v1.5.0, LlmMetadataProto + SP_Tokenizer parsed -> GPU+CPU accelerators registered, 103.8MB TF_LITE_EMBEDDER compiled -> 'produced tool call id=litert-tool-0 name=knowme_diagnostics__local_runtime_status' (namespace-qualified = from the shortlist; gate admitted it as in-shortlist) -> 'tool-result continuation starting tool_results=1' -> 'produced text chars=393 tool_calls=0 context=tool_result'. tool_result_fallback_applied did NOT fire, so the continuation succeeded natively. Whole chain ~9s including cold graph compilation. NPU warnings (kLiteRtStatusErrorInvalidArgument, DispatchLibraryDir) are expected: the plan certifies the generic CPU artifact first, NPU/SM8750 are later optimizations. Committed+pushed 7330389.

## Decision and lesson

Status: verified. Preserve evidence, distinguish compile proof from runtime proof, and do not narrow the active goal.

## Next experiment

Operator may additionally exercise the Local tab (send_local path) and a mutating tool to see the confirmation gate. Remaining phase item: none for litertlm-local-tools; note progress.json still marks android-litertlm-runtime and litertlm-device-proof complete although this blocker showed android-litertlm-runtime was not actually working on device.
