# Qwen3.5 local mobile vision

## Why

The composer accepts images, but local inference currently replaces them with binary metadata. Users reasonably expect the chosen multimodal model to inspect attached images offline.

## What changes

- Download and verify the projector matching each selected model.
- Extend the Rust inference contract with validated image inputs.
- Initialize llama.cpp mtmd and merge image embeddings into the chat request.
- Preserve current text extraction for supported document attachments.

## Success criteria

- An attached image reaches Rust as a typed local inference input.
- The model returns a relevant description without a cloud request.
- Invalid, oversized, or unsupported media produces a typed error.
