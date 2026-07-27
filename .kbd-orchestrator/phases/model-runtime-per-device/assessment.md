# Assessment: model-runtime-per-device

Project: KnowMe
Date: 2026-07-18
Codebase baseline: The Rust core already exposes a llama.cpp-backed mobile inference seam, but the shipped implementation still downloads and loads a pinned Qwen2.5 0.5B text-only model with CPU-default parameters.

## Supersession note — 2026-07-19

This assessment records the pre-recovery baseline. Its Android Qwen3.5,
Vulkan, OpenCL, and MTMD conclusions are superseded by the LiteRT-LM-first
Android plan in `plan.md`, `execution.md`, and
`docs/strategies/local-models/android-litert-lm-six-gates-roadmap.md`.

Current Android direction:

- Runtime: LiteRT-LM behind the Rust `InferenceProvider`.
- Default model: `gemma-4-e2b-it-litertlm`.
- Baseline backend: CPU first.
- Optional GPU/NPU or SM8750-specific artifacts require physical-device gate
  reports, correctness proof, thermal safety, and measured improvement.
- Completion is not a green build; completion requires physical-device evidence.

## IMPLEMENTATION STATUS

- Shared inference seam: **PARTIAL** — `gen_ui_types::inference::InferenceProvider` provides load/generate/unload, but `LocalModelSpec` has only a model id and context length; it has no platform, projector, backend, or capability fields.
- Current local model catalog: **PARTIAL** — `gen_ui_inference/src/llama.rs` hard-codes Qwen2.5-0.5B-Instruct-Q4_K_M, one revision, one SHA-256, and one URL. No Qwen3.5 4B/9B entries exist.
- Download integrity: **PARTIAL** — downloads are atomic and SHA-256 verified, but interrupted downloads do not resume and the downloader does not pin or validate content length.
- Model selection: **STUB** — boot migrates/creates a single `qwen2.5-0.5b-instruct-q4` preference. There is no device-aware iOS/Android catalog resolution or safe fallback policy.
- Native acceleration: **STUB** — `LlamaModelParams::default()` and `LlamaContextParams::default()` are used. `local-llama` does not enable the vendored crate's `metal`, `vulkan`, or `mtmd` features, and no GPU-layer policy or backend diagnostics are exposed.
- Multimodal inference: **MISSING** — the vendored binding contains an `mtmd` wrapper, but the application does not download a matching projector, accept typed image input in the inference contract, or merge image embeddings.
- Tool calling/agent continuation: **MISSING** — local generation receives one flat prompt and streams text. No tool schema formatting, structured-call parser, permission-aware execution loop, iteration bound, or tool-result continuation is connected.
- Generation limits: **PARTIAL** — the engine supports caller-supplied values, but current defaults are 2K context and a 128-token ceiling in the mobile call path. This is inadequate for office tasks, Mermaid/code output, and agent loops.
- Runtime diagnostics: **MISSING** — the selected model, backend, fallback decision, memory preflight, and throughput are not published to the mobile UI or integration tests.
- Physical-device verification: **MISSING** — existing OpenSpec task proposals describe iPhone and Android proof, but no current evidence records a Qwen3.5 load, vision request, tool call, backend, or fallback.

## CROSS-TOOL PROGRESS

- NONE — the new phase has no completed or in-progress changes recorded by another tool. Existing `openspec/changes/qwen35-*` proposals and task lists are unexecuted planning artifacts.

## SPEC GAP SUMMARY

- The functional spec describes llama.cpp through `llama-cpp-2` on mobile with Metal on iOS and Vulkan on Android, but the implementation currently uses CPU-default configuration and a Qwen2.5 0.5B model.
- The functional spec still names Gemma-3n-2B as the default mobile model in one roadmap section, while the current phase decision is Qwen3.5-9B for iPhone and Qwen3.5-4B for Android. The model catalog and docs need one authoritative policy.
- The inference trait is text-only. Adding vision and future tool calls requires a backward-compatible typed request/event contract in the Rust core; Flutter must remain a presentation/FFI client.
- The current model downloader is not resumable, so a multi-gigabyte mobile artifact can restart from zero after interruption.

## BUILD HEALTH

- Rust workspace check: **PASS** — `cd rust && cargo check --workspace` completed successfully in 17.11s.
- Known violations/warnings: the vendored `llama-cpp-2` crate emits existing missing-docs, deprecated API, unused-import, and dead-code warnings. No build errors were observed.
- Desktop and Flutter build health: **UNKNOWN** — not run during this fact-finding assessment.
- Test coverage: **MINIMAL** — there are Flutter chat/A2UI tests and local chat integration scaffolding, but no catalog, resume/checksum, backend-selection, projector, tool-parser, or physical-device tests for this phase.

## CONSTRAINT CHECK

- AGENTS.md violations: **NONE FOUND** in the inspected inference path. The current design keeps LLM/networking in Rust and leaves Flutter as the client layer.
- constraints.md violations: **NONE FOUND** in the inspected path. New work must preserve the Rust core boundary, typed interfaces, and focused tests.

## GOAL PROGRESS

- iPhone Qwen3.5-9B Q4_K_M + mmproj + Metal: **NOT MET** — no artifact, projector, feature flag, GPU policy, or runtime path exists.
- Android Qwen3.5-4B Q4_K_M + mmproj + Vulkan/CPU fallback: **NOT MET** — no artifact, projector, Vulkan feature, fallback, or diagnostics exist.
- Device-aware manifests/downloads/integrity/fallback: **PARTIAL** — one atomic SHA-256-verified catalog download exists, but it is neither device-aware nor resumable and has no fallback policy.
- Generation limits and tool-capable model plumbing: **PARTIAL** — shared sampling fields exist, but current mobile defaults are too small and structured tool execution is absent.

## ASSESSMENT COMPLETE

## Supplemental knowledge-runtime assessment — 2026-07-19

The UAR RAG, Karpathy wiki, and evaluated learning-loop architecture across
mobile, desktop, and web is assessed separately in
`knowledge-base-karpathy-wikis-assessment.md`. Its normative strategy is
`docs/strategies/knowledge-base-karpathy-wikis/README.md`.

This supplemental document does not alter the active LiteRT-LM change or the
phase implementation counters.

---

# Addendum — 2026-07-27: media rendering + download/share

Assessment of a defect the operator hit in live testing: an attached PNG
rendered as a document row, not as the image. Scope then widened to
"images, video, and OpenGraph link thumbnails must display, and media must
be downloadable/shareable, on all four surfaces."

Surfaces: Flutter (iOS, Android) and React (web, Tauri desktop). Flutter has
no web target in this repo (`mobile/web/` absent), so the split is clean.

## Defect 1 — attachments never become image blocks

Five hardcoded sites, none of which consult mime:

| Surface | Site | Code |
|---|---|---|
| Flutter | `mobile/.../chat_screen.dart:367` | `'attachment',` — kind literal |
| Flutter | `gen_ui_widgets/.../content_block_view.dart:597` | `'attachment' => ContentBlockDocumentPreview(` |
| React | `desktop/.../attachmentAdapter.ts:121` | `type: 'document',` |
| React | `desktop/.../ArtifactKindBody.tsx:53` | `case 'attachment': return 'document'` |
| React | `desktop/.../chatStore.ts:222` | `kind: 'attachment',` |

`mimeFor()` at `attachmentAdapter.ts:44` already returns `image/png`
correctly — the value is computed and then discarded.

**Knock-on not visible from the bug report:** because the React adapter
reports `type: 'document'`, assistant-ui's `useAttachmentSrc`
(`attachment.tsx:61`) returns `undefined` for a PNG, so the COMPOSER
thumbnail and its preview dialog are dead too — not only the sent bubble.

### Blocker A — `ContentBlock::Image` is a write-dead variant

Defined at `rust/crates/gen_ui_types/src/content_block.rs:51`, decoded by
both UIs, and **constructed by nothing in `rust/crates/`**. Rust does read
image attachments (`chat.rs:261` `read_attachment`) but routes them only to
`ImageInput` / `ContentPart::ImageUrl` — inference payloads, not transcript
blocks. Changing a block kind alone therefore cannot work; something must
construct an image block for the first time.

### Blocker B — no renderer accepts a local path, and the path is not durable

`ContentBlockImage` (`content_block_view.dart:914-958`) decodes only
`SvgPicture.string` and `Image.memory`, both base64. `ContentBlockVideo`
(`:1150`) throws on a scheme-less source.

Worse, the path itself is transient: `file_picker` returns a CACHED copy on
Android, which is why the package ships `clearTemporaryFiles()`
(file_picker 11.0.1 changelog). Rendering from the picker path would work
during testing and break after a cache purge — a defect that would look
like a regression weeks later.

Durable rendering therefore needs either base64 in the block or a copy into
app-owned storage at attach time. That is a design decision, not a patch.

## Defect 2 — download/share exists nowhere

Confirmed absent on all four surfaces. The nearest existing behaviour:

- `_CopyButton` (`content_block_view.dart:1725`) — clipboard TEXT only; for
  an image its `copyValue` is `url ?? dataBase64`, i.e. it can dump a
  multi-megabyte base64 blob into the system pasteboard.
- `<video controls>` (`ArtifactKindBody.tsx:196`) — the browser's own
  "Save video as…", incidental rather than designed.

### Researched constraints (not assumed)

- **share_plus 13.3.0** supports FILE sharing on Android, iOS, macOS, Web,
  Windows — **not Linux** (pub.dev platform-support matrix). Linux gets text
  via `mailto` only. If Linux desktop matters, share is not uniform and the
  gap must be stated rather than papered over.
- **Tauri v2 needs BOTH `tauri-plugin-dialog` and `tauri-plugin-fs`**, and
  the fs plugin's docs state permissions must be added to `capabilities`.
  Neither plugin is present today (`src-tauri/Cargo.toml:16-20`,
  `package.json:37-40`).

### Registration cost of the Tauri save path — FIVE places

`Cargo.toml`, `package.json`, `lib.rs` plugin registration,
`capabilities/default.json`, and the JS call site. The existing capability
file lists eleven permissions and has no `dialog`/`fs` entry.

This is the same multi-place trap that broke the release build earlier today
(`SetPermissionNotFound`, fixed in `12134b6` by adding the missed fourth
registration). The public failure mode for a missed capability is
`"dialog.open not allowed. Plugin not found"`.

## Divergence to fold in

React's `LinkBody` (`ArtifactKindBody.tsx:146-181`) does NOT render
`imageUrl`, while Flutter's `ContentBlockLinkPreview`
(`content_block_view.dart:1367`) does. OpenGraph thumbnails were explicitly
requested, so React needs that arm regardless of the attachment work.

## Honest scope

This is substantially larger than the three certification items it
interrupted. It spans two codebases, four surfaces, a Rust variant that must
be constructed for the first time, two new Flutter dependencies, two new
Tauri plugins with five registration points, and an unresolved decision
about where attachment bytes live.

The Linux share gap and the byte-durability decision are the two items that
need an explicit call before implementation, not during it.

## Open questions for plan

1. Where do attachment bytes live for durable rendering — base64 inline in
   the block (simple, bloats the transcript store) or a copy into app
   storage with the block holding a stable path (more code, no bloat)?
2. Is Linux desktop in scope? share_plus cannot share files there.
3. Should Rust construct the image block (one place, both UIs benefit) or
   should each UI construct it at attach time (no Rust change, two
   implementations to keep in step)?
