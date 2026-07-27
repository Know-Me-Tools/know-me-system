# PLAN: model-runtime-per-device

Project: KnowMe
Date: 2026-07-18
OpenSpec available: YES
Changes to implement: 5

## CHANGE LIST (ordered)

1. `android-litertlm-runtime`: Replace Android Qwen/llama.cpp/OpenCL with LiteRT-LM Gemma 4 E2B.
   - Scope: Rust inference core, Android FFI feature selection, model cache, diagnostics
   - Depends on: NONE
   - Recommended agent: Codex
   - Est. complexity: High
   - Complexity score: High
   - Model class: frontier
   - Customer value: HIGH
   - Details: Android defaults to `gemma-4-e2b-it-litertlm` from `litert-community/gemma-4-E2B-it-litert-lm`, revision `9262660a1676eed6d0c477ab1a86344430854664`, file `gemma-4-E2B-it.litertlm`, size `2,588,147,712`, digest `181938105e0eefd105961417e8da75903eacda102c4fce9ce90f50b97139a63c`. CPU is the universal correctness baseline. Qualcomm SM8750-specific artifacts are optimization candidates only after physical-device proof.

2. `litertlm-native-adapter`: Link the official LiteRT-LM Android SDK behind `InferenceProvider`.
   - Scope: Native Android build, Rust/JNI adapter, streaming callbacks, prompt/template handling
   - Depends on: `android-litertlm-runtime`
   - Recommended agent: Codex
   - Est. complexity: High
   - Complexity score: High
   - Model class: frontier
   - Customer value: HIGH
   - Details: Use the official Android AAR through a Rust-owned JNI bridge so Flutter remains UI-only. Preserve structured messages, LiteRT-LM prompt-template application, constrained decoding, multimodal content shape, and manual tool-call loops. Direct C++ vendoring remains a future option if the Android SDK cannot expose required capabilities.

3. `litertlm-local-tools`: Add bounded structured tool-call continuation for Android local inference.
   - Scope: Rust agent/inference orchestration, MCP registry integration, AG-UI/A2UI events, parser and policy tests
   - Depends on: `litertlm-native-adapter`
   - Recommended agent: Codex
   - Est. complexity: High
   - Complexity score: High
   - Model class: frontier
   - Customer value: HIGH
   - Details: Send only shortlisted registered tool declarations. Detect model tool calls, validate schema and permissions, execute manually through the existing registry, stream tool events, and continue with tool results. Mutating tools require confirmation.

4. `litertlm-device-proof`: Verify the complete Android text, document, and safe-tool path on physical devices.
   - Scope: Rust diagnostics, Flutter integration tests, Android build/deployment evidence
   - Depends on: `litertlm-local-tools`
   - Recommended agent: Manual
   - Est. complexity: High
   - Complexity score: High
   - Model class: frontier
   - Customer value: HIGH
   - Details: Certify the attached Samsung first with the generic artifact on CPU, then test GPU/NPU and SM8750-specific artifacts only as optimizations. Record load time, throughput, backend, memory, thermal state, cancellation, Markdown/Mermaid, Xberg/OCR attachment flow, and one permitted tool call.

5. `document-uar-karpathy-knowledge-strategy`: Create and publish the evidence-backed UAR RAG and Karpathy wiki strategy.
   - Scope: architecture and research documents only
   - Depends on: NONE
   - Recommended agent: Roo Code (Architect mode)
   - Est. complexity: M
   - Complexity score: Medium
   - Model class: medium
   - Customer value: HIGH
   - Details: Review and finalize the assessment draft in `docs/strategies/knowledge-base-karpathy-wikis/README.md` and its persistent research package. Preserve current-source evidence, platform-specific placement, artifact authority, learning-loop gates, sync/privacy/licensing constraints, explicit contradictions, and a strict no-code acceptance boundary. The full change contract is in `knowledge-base-karpathy-wikis-plan.md`.

## EXECUTION ROUND ORDER

Round 1: `android-litertlm-runtime`
Round 2: `litertlm-native-adapter`
Round 3: `litertlm-local-tools`
Round 4: `litertlm-device-proof`

Independent document round: `document-uar-karpathy-knowledge-strategy` may run
in parallel with any runtime round because it changes architecture and research
artifacts only.

## SUPERSEDED WORK

Android Qwen GGUF, llama.cpp OpenCL, Vulkan, MTMD/projector support, and direct
vendor OpenCL dynamic-library loading are no longer the Android production plan.
They may remain only as quarantined developer fallback experiments until
LiteRT-LM certification is complete.

## DOCUMENT-ONLY SUPPLEMENT

The ordered creation/finalization plan for the UAR RAG and Karpathy wiki
strategy is `knowledge-base-karpathy-wikis-plan.md`. Its OpenSpec change is
`document-uar-karpathy-knowledge-strategy`.

## COMMANDS TO RUN

`/opsx:new document-uar-karpathy-knowledge-strategy`

---

# ADDENDUM (2026-07-27): chat media rendering, download, and share

The five-change list above is the phase baseline and is NOT superseded. This
addendum plans one additional OpenSpec change, `chat-media-rendering`, arising
from the media defect assessed the same day (see the matching addendum in
`assessment.md`). Tasks live in
`openspec/changes/chat-media-rendering/tasks.md`.

## The defect in one paragraph

A PNG attached in the composer renders as a grey document chip on every surface.
`ContentBlock::Image` is defined in `gen_ui_types` and decoded by both UIs but
**constructed by nothing**, so no amount of renderer work fixes this alone — the
block never arrives. Five sites hardcode the kind `attachment` regardless of
mime. And even with a correct block, Flutter's `ContentBlockImage` has no
file-path arm and its video arm hard-rejects scheme-less sources, so a local
path renders as an error card.

## Operator decisions (resolved before planning)

| Question | Decision | Consequence for this plan |
|---|---|---|
| Where do attachment bytes live? | **Copy into app storage**, stable path in the block | Adds C1 ahead of everything; keeps the transcript store small; needs a deletion story (C8) |
| Is Linux desktop in scope? | **Download everywhere, share where supported** | C6 ships save on all four surfaces; the share control is *absent* on Linux, not dead |
| Who constructs the image block? | **Rust, once** | C2 is the single construction point; both UIs become consumers. Requires a mobile FFI rebuild and a Tauri plugin rebuild |

The Rust-constructs decision is what makes this tractable: without it, the five
mime-blind hardcodes become five mime-*aware* hardcodes and drift again.

## ORDERED CHANGE LIST (addendum)

Ordering rationale: **bytes must be durable before anything renders them** (C1),
**the block must exist before any renderer can switch on it** (C2), and the
renderers then land per surface in parallel (C3/C4). Export (C5/C6) depends on a
stable path from C1 but not on the renderers. C7 is independent and could ship
first, but it is smaller and rides along. C8 closes the loop C1 opens.

| # | Change | Surface | Depends on | Recommended agent |
|---|---|---|---|---|
| C1 | Durable attachment storage in Rust | Rust | — | rust-reviewer |
| C2 | Rust constructs mime-aware blocks | Rust | C1 | rust-reviewer |
| C3 | Flutter renders local media | Flutter | C2 | flutter-reviewer |
| C4 | React renders image/video + composer thumbnail | React | C2 | typescript-reviewer |
| C5 | Flutter download + share | Flutter | C1 | flutter-reviewer |
| C6 | React/Tauri download + share | React + Rust | C1 | typescript-reviewer |
| C7 | React link preview shows the OpenGraph image | React | — | typescript-reviewer |
| C8 | Attachment lifecycle: delete + orphan sweep | Rust | C1 | rust-reviewer |

### C1 — Durable attachment storage (Rust)

`read_attachment` (`rust/crates/gen_ui_agent/src/chat.rs:393`) already has the
bytes and enforces the 25 MB cap. Extend it to copy the file into an app-owned
`attachments/<conversation_id>/<content_hash>.<ext>` directory and return that
path alongside the existing metadata.

Content-hash naming makes the copy idempotent: re-attaching the same file twice
writes once. This is the change that makes rendering survive Android's cache
purge — `file_picker` hands back a **purgeable** cached path (the package ships
`clearTemporaryFiles()` for exactly this reason), so a renderer pointed at the
picker path passes testing and fails weeks later.

Verification: unit tests for idempotent write, path escaping, and the size cap
rejecting *before* the copy rather than after.

### C2 — Rust constructs mime-aware blocks

The single most important change. In `send_with_attachments`, emit
`ContentBlock::Image { path, mime, alt, width, height }` for image attachments
and keep `artifact`/`attachment` for everything else. `width`/`height` are read
from the file header, not guessed — the ContentBlock contract requires explicit
dimensions to prevent CLS, and a renderer cannot invent them. `alt` is mandatory
per the contract; with no caption available, the filename is the honest fallback.

Then the five mime-blind hardcodes stop deciding the kind and start reading it:
`chat_screen.dart:367`, `content_block_view.dart:597`,
`attachmentAdapter.ts:121`, `ArtifactKindBody.tsx:53`, `chatStore.ts:222`.

Note: `chat.rs:261-299` currently routes image bytes to *inference* only (and
skips Android entirely). Rendering and inference are separate concerns — C2 must
not make rendering conditional on `is_image && !android`, or Android gets a
working model path and a broken picture.

### C3 — Flutter renders local media

`ContentBlockImage.build` (`content_block_view.dart:914-958`) has four arms and
none reads a file. Add a local-path arm.

**The portability trap, stated so it is not repeated:** an earlier attempt in
this session added a bare `import 'dart:io' show File;`. That breaks Flutter
web, which is in scope. The local-path arm must be behind a conditional import
(`dart.library.io`), or the web build fails at compile time.

Video: `content_block_view.dart:1150-1152` throws `FormatException` on a
scheme-less source. A `file://` URI from C1 has a scheme and passes, but the
check should accept a plain absolute path too rather than depending on the
caller to pre-format it.

### C4 — React renders image/video + composer thumbnail

Two distinct dead paths:

1. The transcript: `ArtifactKindBody.tsx` needs real `image` and `video` arms,
   reading the block C2 now sends.
2. The composer: `attachment.tsx:61`'s `useAttachmentSrc` gates on
   `s.attachment.type === "image"`, and the adapter hardcodes `'document'` — so
   the composer thumbnail and the preview dialog are dead for images *before*
   the message is ever sent. Fixing the adapter (C2) is necessary but the
   thumbnail path needs its own verification.

Web has no Rust: a local path is meaningless in a browser tab. On web the block
carries a blob/object URL instead; `useRemoteContent` already models "no Rust to
fall back to" correctly and is the pattern to follow.

### C5 — Flutter download + share

`share_plus` 13.3.0 for share (Android/iOS ✅). Save-to-Downloads on Android via
the media store, share-sheet-to-Files on iOS. Both controls need accessible
names and ≥44px targets per the a11y gate.

### C6 — React/Tauri download + share

Tauri v2 save needs **both** `tauri-plugin-dialog` and `tauri-plugin-fs`, and
each is a **five-place** registration: `Cargo.toml`, `package.json`, `lib.rs`,
`capabilities/default.json`, call site.
`desktop/src-tauri/capabilities/default.json` currently has eleven permissions
and no dialog/fs entry.

This is the same trap that broke today's release build (fixed in `12134b6`):
`cargo check -p tauri-plugin-gen-ui` passes while the consuming app cannot
build, because ACL resolution happens in the app crate. **Verify with a release
bundle build, not a plugin check.**

Web download is an `<a download>` with an object URL — no plugin, no permission.
On Linux the share control is not rendered (`share_plus` has no Linux file
support and Tauri has no share sheet); download works normally.

### C7 — React link preview shows the OpenGraph image

Flutter's link preview already renders `imageUrl` by delegating to
`ContentBlockImage` (`content_block_view.dart:1367-1377`). React's `LinkBody`
(`ArtifactKindBody.tsx:146-181`) does not. This is a pure cross-surface
divergence and the operator explicitly asked for OpenGraph thumbnails.

Use `useRemoteContent` — an OG image is remote by definition and is exactly the
CORS-blocked case that hook exists for.

### C8 — Attachment lifecycle

C1 writes files that nothing deletes. Delete a message or a conversation and its
attachment bytes leak forever. Wire deletion into the existing message- and
conversation-delete paths, plus a startup sweep for orphans (rows gone, files
left behind after a crash).

Not optional polish: a chat app that silently grows unbounded on disk is a
defect, and it is C1's direct consequence.

## VERIFICATION (addendum)

Per change: `cargo test -p gen_ui_agent`, `flutter test test/features/chat/` +
`flutter analyze`, `pnpm test` + `npx tsc --noEmit`.

Cross-surface gates that no unit test covers:

- **Release bundle build** after C6 — the ACL trap is invisible to `cargo check`.
- **Flutter web compile** after C3 — the `dart:io` trap is invisible to a mobile build.
- **Device run** on the SM-S936U: attach a PNG, see it render, save it, share it.
- **a11y**: alt text present on every image, download/share named and ≥44px,
  status never by colour alone.

## RISKS (addendum)

- **Two rebuilds.** C2 changes the Rust contract, so mobile FFI *and* the Tauri
  plugin must be rebuilt. A stale artifact will show old behaviour and read as a
  code failure.
- **Existing conversations.** Blocks persisted before C2 still say `attachment`.
  Decide explicitly: leave them (honest — those messages never had the bytes
  stored durably) rather than backfilling from paths that may no longer exist.
- **The migration trap.** Any new column needs an idempotent `ALTER`;
  `CREATE TABLE IF NOT EXISTS` never lands a column on an existing DB.
- **Certification is queued behind this**: `certify-local-attachment-flow.sh`
  (written, needs a manual pick), the iOS deploy, and airplane-mode task 5.5.

## COMMANDS TO RUN (addendum)

`/opsx:new chat-media-rendering`

## PLAN COMPLETE
