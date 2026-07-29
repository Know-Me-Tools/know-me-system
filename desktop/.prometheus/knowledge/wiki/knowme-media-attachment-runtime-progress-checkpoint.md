---
type: Reference
id: knowme-media-attachment-runtime-progress-checkpoint
title: KnowMe Media Attachment Runtime Progress Checkpoint
tags:
- knowme
- model-runtime
- media-attachments
- flutter-web
- tauri
- react
- rust
- android
links:
- knowme-per-device-model-runtime-download-and-verification-status
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-27T14:17:16.554002+00:00
created_at: 2026-07-27T14:17:16.554002+00:00
updated_at: 2026-07-27T14:17:16.554002+00:00
revision: 0
---

## Context

- **Project:** `KnowMe`
- **Phase:** `model-runtime-per-device`
- **Position:** `model-runtime-per-device › embedded-uar-offline-agents › litertlm-device-proof`
- **Status:** `executing`
- **KBD root:** `/Users/gqadonis/Projects/know-me/know-me-system`
- **Captured:** `2026-07-27T13:46:00Z`

This checkpoint belongs to the same active `model-runtime-per-device` effort tracked in [KnowMe Per-Device Model Runtime Download and Verification Status](/knowme-per-device-model-runtime-download-and-verification-status.md), but records media attachment plumbing progress and remaining runtime certification work.

## Phase goals

- Ship **Qwen3.5-9B Q4_K_M** with matching `mmproj` vision support on **iPhone 17 Pro Max** using Metal.
- Ship Android local text-agent inference through **LiteRT-LM Gemma 4 E2B**, with CPU as the universal correctness baseline.
- Enable:
  - Device-aware model manifests
  - Resumable downloads
  - Integrity checks
  - Runtime gate diagnostics
  - Safe memory/thermal fallback
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## Progress

Current media change list progress: **5 of 8 changes landed and verified**.

| Change | Description | Verification |
|---|---|---|
| C1 | Durable attachment storage in Rust | 8 unit tests; both boot paths wired |
| C2 | Rust decides attachment kind, MIME, and dimensions | 86 Rust tests; 5 hardcoded call sites now read metadata instead of deciding |
| C3 | Flutter local media and staging | 43 Flutter tests; Flutter analyze clean; real Flutter web build against `gen_ui_widgets` passes |
| C4 | React image/video rendering and composer thumbnail | 150 React tests; TypeScript clean; verified live in browser |
| C7 | React OpenGraph thumbnail | Covered by same React suite |
| C8 partial | Attachment lifecycle Rust half | Empty-list guard and test added |

Commits landed:

- `96799a9` — C1 durable attachment storage
- `f11e750` — C2 Rust metadata classification
- `394c1ab` — C4 + C7 React media rendering/OpenGraph thumbnail
- `8cb52ae` — C3 Flutter local media/staging
- `d915939` — C8 Rust lifecycle half

## Verified behavior

The core media classification defect is fixed and was validated in the running app, not only in unit tests:

- PNG produces an `image` block with durable path, alt text, and dimensions.
- PDF remains an `attachment`.
- MP4 becomes `video`.
- Web fallback classification works.

The Android SM-S936U device authorization blocker was cleared: device status is `device`, not `unauthorized`.

## Contract correction: `ContentBlock::Image`

The initial plan incorrectly claimed `ContentBlock::Image` required `alt`, `width`, and `height`.

Actual existing contract fields were only:

- `url`
- `dataBase64`
- `mime`

Resolution:

- Added `alt`, `width`, and `height` as optional fields.
- This preserves decoding compatibility for existing persisted content blocks.

## Web image regression fixed

A browser-only failure escaped unit tests:

- On web, no asset protocol exists.
- A stored path resolved to `''` through `localMediaSrc`.
- The stored-path branch emitted `<img src="">`, creating a silent broken image.

Fix:

- Prevented the stored-path branch from emitting an empty image source.
- Added a regression test for this failure mode.

## Flutter `dart:io` trap closed

A real Flutter web build was run against `gen_ui_widgets` to prove conditional import safety.

Result:

- The build compiles for web.
- This validates that unconditional `dart:io` imports are not present in the relevant web compile path.
- The specific failure mode that previously forced a revert is closed.

## Durable storage root design fix

C1 originally used a single process-wide storage root. That caused test interference:

- One test's orphan sweep could delete files belonging to another test.

Fix:

- Storage functions now take the storage root explicitly.
- This avoids process-global state and makes tests isolated.

## Attachment lifecycle safety guard

Added an empty-list guard to `sweep_orphan_attachments`.

Rationale:

- An empty known-conversation list is more likely to indicate a failed load than a legitimate user state.
- Without the guard, an empty list could delete every attachment on the device.

## Tauri asset protocol and CSP

Tauri asset protocol was enabled with scope limited to:

```text
$APPDATA/attachments/**
```

A CSP was added; it was previously null.

Verification status:

- **Unverified until release bundle build.**
- Vite dev server does not apply the release CSP, so `cargo check` or dev-server testing is insufficient.

## Remaining work

- C5: Flutter download/share.
- C6: React/Tauri download/share.
  - Includes five-place plugin registration.
  - Must be verified through a release bundle, not just `cargo check`.
- C8: UI call site.
- Build Android APK.
- Run on-device attachment verification.
- R1 certification script.
- `kbd-reflect`.
- Commit, push, and open PR.

Operational instruction remains: **do not merge to `main`**.

# Citations

1. [1] stdin
2. [2] manual:KnowMe/model-runtime-per-device