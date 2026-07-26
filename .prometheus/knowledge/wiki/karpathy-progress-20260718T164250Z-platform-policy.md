---
type: Reference
id: karpathy-progress-20260718T164250Z-platform-policy
title: "Current-generation mobile and toolchain policy"
tags:
- karpathy-progress
- platform-policy
- completed
links: []
sources:
- conversation:operator-agent
timestamp: 2026-07-18T16:42:50Z
created_at: 2026-07-18T16:42:50Z
updated_at: 2026-07-18T17:58:00Z
revision: 2
---

## Intent

The operator explicitly requires KnowMe to prioritize high-impact capabilities on later phones and authorizes raising Dart, Flutter, Android, and iOS minimum versions whenever needed; legacy-device compatibility is not a product constraint.

## Observed state and verification

Operator direction recorded on 2026-07-18. The active A2UI integration requires Dart >=3.10 and Flutter >=3.35.7; the development environment reports Flutter 3.45.0 beta and Dart 3.13.0. Android API 29, arm64-v8a, and iOS 15 are the documented current baselines. A debug APK was built, installed, and exercised on a Samsung SM-S936U running Android API 36; light and dark chat themes and theme persistence across a cold restart were verified.

## Decision and lesson

Status: completed. Preserve evidence, distinguish compile proof from runtime proof, and prefer raising platform baselines when modern security, accessibility, media, or agentic UI capability justifies it.

## Next experiment

At the next major Flutter, Dart, Android, or iOS capability upgrade, reassess and raise these baselines rather than carrying legacy constraints by default; repeat the physical-device verification and update `docs/platform-support.md`.
