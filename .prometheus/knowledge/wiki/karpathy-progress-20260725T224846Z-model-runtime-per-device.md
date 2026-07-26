---
type: Reference
id: karpathy-progress-20260725T224846Z-model-runtime-per-device
title: "Android blocker: KnowMeLiteRtLmBridge compiles but is absent from the APK dex"
tags:
- karpathy-progress
- model-runtime-per-device
- blocked
sources:
- conversation:operator-agent
timestamp: 2026-07-25T22:48:46Z
created_at: 2026-07-25T22:48:46Z
updated_at: 2026-07-25T22:48:46Z
revision: 1
---

## Intent

Android device certification on Samsung SM-S936U (android-arm64, Android 16) is BLOCKED by a packaging defect that prevents all LiteRT-LM local inference. Device logs: 'ClassNotFoundException: Didnt find class tools.knowme.knowme.KnowMeLiteRtLmBridge' -> 'LiteRT-LM native adapter load failed ... terminal (do not retry)' -> 'Failed to create driver stream: load local model'. Evidence chain: the FQCN Rust resolves (android_litert_jni.rs:223) is tools.knowme.knowme.KnowMeLiteRtLmBridge; the Kotlin source exists at the matching package path; the compiled KnowMeLiteRtLmBridge.class IS present in BOTH build/app/tmp/kotlin-classes/debug and /release; but the shipped APK classes.dex contains only Ltools/knowme/knowme/MainActivity; (verified by enumerating dex class descriptors). R8/minify and proguard are not configured, so shrinking is not the cause. Conclusion: the class is dropped between Kotlin compilation and dexing — a packaging defect, NOT a naming or code defect. NOTE this blocks android-litertlm-runtime, not litertlm-local-tools, whose tool-shortlist work is code-complete and now compile-verified for aarch64-linux-android by the green cargokit/NDK Gradle build.

## Observed state and verification

Samsung SM-S936U R5GYB4AZD7A, release APK 181.8MB built+installed. Certified working on device: embedded UAR resolving per-agent provider settings provider=knowme-local-litert-lm model=gemma-4-e2b-it-litertlm (3-tier policy + populated provenance on Android); model artifact present at app_flutter/model-cache/litert-lm/gemma-4-E2B-it.litertlm. Failing: JNI FindClass for the bridge. dex enumeration: strings classes.dex | grep -oE 'Ltools/knowme/knowme/[A-Za-z$]*;' returns only MainActivity.

## Decision and lesson

Status: blocked. Preserve evidence, distinguish compile proof from runtime proof, and do not narrow the active goal.

## Next experiment

Fix the Android packaging so KnowMeLiteRtLmBridge reaches classes.dex (investigate the app module dexing/sourceSets and whether the class is reachable from the app variant's compile classpath), then re-run the Samsung certification: Local lane generation, then a tool call to certify shortlist -> gate -> execution -> continuation.
