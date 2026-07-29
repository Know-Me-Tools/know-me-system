---
type: Reference
id: litert-lm-android-streaming-bridge-for-cancellation-safe-partial-output
title: LiteRT-LM Android streaming bridge for cancellation-safe partial output
tags:
- model-runtime-per-device
- android
- litert-lm
- jni
- streaming
- cancellation
- rust
- kotlin
links:
- m5b-install-monitor-armed-for-model-runtime-picker-verification
sources:
- stdin
- manual:KnowMe/model-runtime-per-device
timestamp: 2026-07-26T21:24:43.121284+00:00
created_at: 2026-07-26T21:24:43.121284+00:00
updated_at: 2026-07-26T21:24:43.121284+00:00
revision: 0
---

## Context

Phase: `model-runtime-per-device` for KnowMe local model runtime certification.

Primary goals in scope:

- iOS: ship `Qwen3.5-9B Q4_K_M` with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Android: ship local text-agent inference through LiteRT-LM `Gemma 4 E2B`, with CPU as the universal correctness baseline.
- Enable device-aware manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

Related iOS runtime/device work continues from picker and install verification such as [M5b install monitor armed for model runtime picker verification](/m5b-install-monitor-armed-for-model-runtime-picker-verification.md), but this note is specifically the Android LiteRT-LM streaming/cancellation lane.

## Verified LiteRT-LM async API surface

The `litertlm-android-0.14.0` AAR was extracted and inspected with `javap`; the streaming design is based on verified bytecode APIs, not assumptions.

Verified API members:

```text
Conversation.sendMessageAsync(Message, MessageCallback, Map)
MessageCallback.onMessage(Message)
MessageCallback.onDone()
MessageCallback.onError(Throwable)
```

A `Flow<Message>` overload also exists, but the callback overload was selected deliberately to keep Kotlin coroutines out of the JNI path.

## JNI bridge constraint

The JNI bridge is strictly one-way:

```text
Rust -> Kotlin static calls
```

There is no callback registration path from Kotlin back into Rust. Therefore, Kotlin must buffer streamed chunks and Rust must poll for them.

## Streaming design

### Kotlin side

- New bridge entrypoint: `startStream`.
- `startStream` uses `Conversation.sendMessageAsync(...)` with `MessageCallback`.
- Callback chunks are buffered in Kotlin.
- Stream buffering uses a dedicated `streamLock`.
- The object monitor is intentionally **not** used for streamed chunk buffering.

Rationale for separate locking:

- The blocking `sendMessage` path holds the object monitor for the entire generation.
- Reusing that monitor would recreate the earlier cancellation deadlock.
- `streamLock` isolates chunk buffering/draining from long-running generation calls.

### Rust side

- New streaming loop: `stream_litert_generation`.
- Rust polls Kotlin `drainStream()` every `30ms`.
- The cancel flag is checked **before each drain**.

Cancellation behavior:

- Stop requests become prompt because the loop checks cancellation before blocking/draining again.
- Text already emitted by LiteRT-LM and buffered by Kotlin remains in the conversation.
- Certification target: a mid-generation stop should report a non-zero kept-block count.

## Fallback behavior

If the installed bridge does not expose `startStream`:

- The call fails safely.
- The runtime logs a warning.
- Generation falls back to the existing blocking `generate_text` path instead of breaking the run.

This preserves compatibility with older bridge installs while enabling cancellation-safe partial output when the streaming bridge is present.

## Compile status and caveats

This is currently the real compile gate for both new code paths:

- Kotlin `startStream` / `drainStream` bridge.
- Rust polling loop under Android `cfg`.

Not yet compile-verified at capture time.

Observed check result:

```sh
cargo check --target aarch64-linux-android
# Finished in 0.28s, but did not actually validate the new Android-gated code
```

Reason this did not prove correctness:

- The `aarch64-linux-android` target has not previously built successfully in this environment.
- Earlier Android target builds hit `ring` / `aws-lc-sys` NDK failures.
- Host checks skip the relevant code because it is guarded by `#[cfg(target_os = "android")]`.
- Therefore the running Gradle build is the first meaningful compilation of both the Kotlin bridge and Rust Android polling path.

## Next verification steps

1. Read the Gradle build result.
2. Fix any Kotlin or Rust compile failures surfaced by the real Android build.
3. Re-run:

```sh
certify-local-cancellation.sh
```

4. Confirm mid-generation cancellation reports a non-zero kept-block count.

That result is the proof that partial generated text survives cancellation.

# Citations

1. stdin
2. manual:KnowMe/model-runtime-per-device