# Tasks

Ordered. C1 and C2 gate everything: bytes must be durable before anything
renders them, and the block must exist before any renderer can switch on it.

## C1 — Durable attachment storage (Rust)

- [x] Copy picked files into `attachments/<conversation_id>/<content_hash>.<ext>`
      in `read_attachment` (`rust/crates/gen_ui_agent/src/chat.rs:393`), which
      already holds the bytes and enforces the 25 MB cap.
- [x] Return the stable path alongside the existing attachment metadata.
- [x] Reject over-cap files BEFORE the copy, not after.
- [x] Unit tests: idempotent write on re-attach, path escaping, cap rejection.

## C2 — Rust constructs mime-aware blocks (Rust)

- [x] Emit `ContentBlock::Image { path, mime, alt, width, height }` for image
      attachments in `send_with_attachments`; keep `artifact` for the rest.
- [x] Read `width`/`height` from the file header — the ContentBlock contract
      requires explicit dimensions to prevent CLS and a renderer cannot invent
      them. `alt` falls back to the filename.
- [x] Keep rendering independent of inference: `chat.rs:261-299` skips image
      bytes on Android for the model, which must NOT disable rendering there.
- [x] Make the five hardcodes read the kind instead of deciding it:
      `mobile/lib/features/chat/presentation/screens/chat_screen.dart:367`,
      `flutter_packages/gen_ui_widgets/lib/src/content_block_view.dart:597`,
      `desktop/src/features/chat/adapters/attachmentAdapter.ts:121`,
      `desktop/src/features/chat/components/ArtifactKindBody.tsx:53`,
      `desktop/src/features/chat/stores/chatStore.ts:222`.
- [x] Rebuild mobile FFI and the Tauri plugin — a stale artifact shows old
      behaviour and reads as a code failure.

## C3 — Flutter renders local media

- [x] Add a local-path arm to `ContentBlockImage.build`
      (`content_block_view.dart:914-958`), which today has four arms and reads
      no file.
- [x] Put it behind a conditional import on `dart.library.io`. A bare
      `import 'dart:io'` breaks the Flutter web build, which is in scope — an
      earlier attempt in this session did exactly that and was reverted.
- [x] Accept a plain absolute path in the video arm
      (`content_block_view.dart:1150-1152` throws on a scheme-less source).
- [x] Widget tests: local image renders; missing file shows the error card, not
      a crash.

## C4 — React renders image/video + composer thumbnail

- [x] Add real `image` and `video` arms to `ArtifactKindBody.tsx`.
- [x] Fix the composer path: `attachment.tsx:61`'s `useAttachmentSrc` gates on
      `type === "image"` while the adapter hardcodes `document`, so the
      thumbnail and preview dialog are dead before a message is ever sent.
- [x] On web, carry a blob/object URL instead of a path — a local path is
      meaningless in a browser tab. Follow `useRemoteContent`'s existing
      "no Rust to fall back to" handling.
- [x] Tests: image part renders; composer thumbnail appears for an image.

## C5 — Flutter download + share

- [x] Save to Downloads on Android via the media store; share sheet on iOS.
- [x] Share via `share_plus` 13.3.0 (Android/iOS supported).
- [x] Accessible names and ≥44px targets on both controls.

## C6 — React/Tauri download + share

- [x] ~~Add `tauri-plugin-dialog` AND `tauri-plugin-fs`~~ — DELIBERATELY NOT
      DONE. Each is five registration points and the same trap that broke the
      release build in `12134b6`. They proved unnecessary: the asset scheme
      enabled for rendering is already fetchable, so one anchor-download path
      covers web and Tauri. Instead the CSP gained `asset:` in `connect-src`,
      without which Save and Share would fail while images still displayed.
- [x] Web download via `<a download>` with an object URL — no plugin needed.
- [x] Do not render the share control on Linux; download works there normally.
- [x] Verify with a RELEASE BUNDLE BUILD, not `cargo check` — ACL resolution
      happens in the consuming app crate, which is what broke `12134b6`.

## C7 — React link preview shows the OpenGraph image

- [x] Render `imageUrl` in `LinkBody` (`ArtifactKindBody.tsx:146-181`), matching
      Flutter's `content_block_view.dart:1367-1377`.
- [x] Resolve through `useRemoteContent` — an OG image is remote by definition
      and is exactly the CORS-blocked case that hook exists for.

## C8 — Attachment lifecycle

- [x] Delete attachment files when their message or conversation is deleted.
- [x] Sweep orphans on startup (rows gone, files left after a crash).
- [x] Any new column needs an idempotent `ALTER` —
      `CREATE TABLE IF NOT EXISTS` never lands a column on an existing DB.

## Verification

- [x] `cargo test -p gen_ui_agent`
- [x] `flutter test test/features/chat/` + `flutter analyze`
- [x] `pnpm test` + `npx tsc --noEmit`
- [x] Flutter WEB compile after C3 — verified by building a REAL Flutter web
      app against `gen_ui_widgets` (`flutter build web` succeeded, Wasm dry run
      passed). The mobile app itself has no web platform configured, so the
      shared package was the only place the `dart:io` trap could be proven.
- [x] Desktop RELEASE BUNDLE build after C6
- [x] Device run on SM-S936U: attach a PNG, see it render — PASSED, and it
      caught a real persistence bug (`388f6b2`) that every unit test missed.
      Evidence: `docs/certification/2026-07-27/android-image-rendering-report.json`.
- [x] Device run: SAVE and SHARE the rendered image — PASSED on the iPhone 17
      Pro Max (iOS 27.0), operator-observed 2026-07-27: "save worked, share
      sheet opened with the image". Evidence:
      `docs/certification/2026-07-27/ios-media-report.json`. Recorded as a
      human observation, not a script assertion — iOS exposes no scriptable UI
      dump outside an XCUITest target.
- [x] a11y gate: alt text on every image; download/share named and ≥44px
