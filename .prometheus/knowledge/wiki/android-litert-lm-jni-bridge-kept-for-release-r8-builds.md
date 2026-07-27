---
type: Reference
id: android-litert-lm-jni-bridge-kept-for-release-r8-builds
title: Android LiteRT-LM JNI bridge kept for release R8 builds
tags:
- android-release
- litert-lm
- r8
- jni-reflection
- proguard
- model-runtime-per-device
links:
- m5b-install-monitor-armed-for-model-runtime-picker-verification
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-25T23:01:20.114141+00:00
created_at: 2026-07-25T23:01:20.114141+00:00
updated_at: 2026-07-25T23:01:20.114141+00:00
revision: 0
---

## Context

Phase: `model-runtime-per-device` for KnowMe. Android local text-agent inference targets LiteRT-LM `Gemma 4 E2B`, with CPU as the universal correctness baseline. The same phase also includes iOS Qwen/Metal runtime certification, continuing the broader per-device model runtime workstream referenced by [M5b install monitor armed for model runtime picker verification](/m5b-install-monitor-armed-for-model-runtime-picker-verification.md).

## Root cause

The previous assumption that R8/minification was not configured was incorrect. Flutter release variants enable R8 by default even when `minifyEnabled` is absent from `build.gradle.kts`.

Evidence from build intermediates:

```text
build/app/intermediates/r8_art_profile/release/minifyReleaseWithR8   ← R8 is running
aapt_rules.txt:  -keep class tools.knowme.knowme.MainActivity { <init>(); }   ← only keep rule
```

R8 kept only `MainActivity`, because it was referenced from the Android manifest. The LiteRT-LM bridge class was omitted from the release APK because it is reached only from Rust through JNI reflection by fully qualified class name, which is invisible to static reachability analysis.

Debug builds worked because R8 does not run there.

Timeline evidence confirmed this was not a stale-build issue:

- Bridge class compiled at `18:35:21`.
- APK packaged at `18:36:40` in the same build.
- Class was available during build but still omitted from packaged release output.

## Fix

### Add ProGuard/R8 keep rules

Created:

```text
android/app/proguard-rules.pro
```

Rules must keep:

- `tools.knowme.knowme.KnowMeLiteRtLmBridge`
- All members of the bridge class
- LiteRT-LM API surface that crosses the JNI boundary

Member retention is required because Rust resolves methods reflectively after loading the class; keeping only the class is insufficient if R8 removes or renames methods/constructors.

### Wire rules into release build

Updated:

```text
android/app/build.gradle.kts
```

Release `buildType` now includes `proguardFiles(...)`. Without this wiring, `android/app/proguard-rules.pro` is ignored even if the file exists.

## Impact

This unblocks both Android lanes:

- Orchestrator/Agent
- Local

Both paths share the same engine path: the UAR local driver calls the LiteRT engine equivalent to the failed `MlxEngine`-style runtime bridge, so the same missing JNI bridge class broke both lanes.

## Verification plan

1. Wait for release rebuild to complete.
2. Inspect the rebuilt APK before installing on-device.
3. Confirm the descriptor is present in `classes.dex`:

```text
Ltools/knowme/knowme/KnowMeLiteRtLmBridge;
```

4. Install on Samsung.
5. Certify both Android lanes:
   - Orchestrator/Agent
   - Local
6. Continue tool-capable flow verification:
   - shortlist
   - gate
   - execution
   - continuation

## Runtime/build position

```text
Position: model-runtime-per-device › android-litertlm-runtime › fix-rebuilding
Status: executing
Build monitor: b6gsjmb1l
Rebuild id: bim87lisg
```

Last completed actions:

- Wrote keep rules.
- Wired `proguardFiles` into release build type.
- Started release rebuild.

# Citations

1. stdin
2. manual:KnowMe/model-runtime-per-device