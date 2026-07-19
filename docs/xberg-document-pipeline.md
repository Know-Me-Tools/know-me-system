# Xberg document pipeline

KnowMe uses the Rust `gen_ui_document` leaf as the one attachment normalization
boundary for native surfaces (Android, iOS, macOS, and desktop Tauri).

```text
Flutter file picker
  -> chat_send / memory_ingest_attachment (FFI intent)
  -> gen_ui_agent
  -> gen_ui_document (xberg: formats + analysis; OCR where the native target supports it)
  -> bounded text + MIME metadata
  -> local prompt, cloud content parts, or graph-RAG memory
```

## Inference

Every attachment is offered to xberg before inference. iOS and desktop local
multimodal engines retain their original image tensor after extraction, so the
model can use both OCR text and visual features. Android's Qwen3 text-only lane
receives only the normalized xberg text; raw image tensors are intentionally not
sent to that model.

Android and desktop include xberg's bundled Tesseract OCR. The current xberg
Tesseract build does not recognize `aarch64-apple-ios` and emits macOS objects,
so iOS uses xberg's document/format/analysis extractors without that bundled OCR
feature until the upstream mobile target is fixed.

Cloud requests retain their original image/document parts and also include xberg
text when extraction succeeds. This gives vision/document providers richer input
without making local extraction a network dependency.

## RAG

`memory_ingest_attachment` and `memory_ingest_attachments` run the same xberg
processor, prepend the source filename, and store the bounded text as `kind =
"document"` in graph-RAG. Future chunking and citation work can build on that
stable normalized representation instead of reimplementing file parsing.

## Safety and limits

- 25 MiB maximum input per file.
- 512 KiB maximum extracted text carried into a prompt or memory record.
- Six attachments maximum per chat or batch-ingest intent.
- Extraction failures are explicit for local inference/RAG. Cloud uploads retain
  their original part and include a visible extraction warning.

The dependency is pinned to xberg `1.0.0-rc.17`, the last compatible release
before the current xberg regex requirement conflicts with the project's pinned
desktop mistral.rs graph. Upgrade only after validating the complete desktop and
mobile dependency graph.
