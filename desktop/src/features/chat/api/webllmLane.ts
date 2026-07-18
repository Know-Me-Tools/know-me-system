// TJ-ARCH-MOB-001 compliant — api layer (called only from stores)
//
// Web local-inference lane: WebLLM (WebGPU), the browser counterpart to the
// desktop/mobile llama.cpp lane in gen_ui_inference.
//
// DOCUMENTED EXCEPTION to the "all inference lives in gen_ui_core" invariant
// (see CLAUDE.md, and the C-105 design doc). WASM cannot drive Metal/CUDA, and
// WebGPU is only reachable from JS — so the web lane cannot be fulfilled in the
// Rust core. This adapter satisfies the SAME intent contract the Rust
// InferenceProvider does (load → stream tokens → unload), emitting the same
// CoreA2uiEvents the Tauri path emits, so the store and every component above it
// stay lane-agnostic. The exception is confined to transport, not semantics.
import type { CoreA2uiEvent } from '@/bridge/a2ui/driver'

// Verified present in @mlc-ai/web-llm@0.2.84's prebuilt catalog. The 0.5B Q4
// build is the reliable zero-configuration default: materially fewer first-run
// shards than the 1.5B build while retaining the same Qwen 2.5 chat template.
const WEB_MODEL_ID = 'Qwen2.5-0.5B-Instruct-q4f16_1-MLC'

export interface WebLlmProgress {
  /** 0..1 */
  progress: number
  text: string
}

/**
 * Whether this browser can run the local lane at all.
 *
 * `navigator.gpu` is the gate — WebLLM has no separate capability probe, and
 * CreateMLCEngine throws without WebGPU. Callers MUST check this and degrade to
 * the cloud lane visibly rather than letting the engine throw.
 */
export function isWebGpuAvailable(): boolean {
  return typeof navigator !== 'undefined' && 'gpu' in navigator
}

let enginePromise: Promise<import('@mlc-ai/web-llm').MLCEngine> | null = null

/**
 * Load the model, or attach to an in-flight//completed load.
 *
 * Idempotent, mirroring InferenceProvider::load's contract: weights are ~1GB and
 * cached by the browser (Cache API), so a second call must never re-download.
 * The module is imported dynamically so the WebLLM bundle stays out of the main
 * chunk for users who never touch the local lane.
 */
export function loadWebLlm(onProgress?: (p: WebLlmProgress) => void): Promise<import('@mlc-ai/web-llm').MLCEngine> {
  if (!isWebGpuAvailable()) {
    return Promise.reject(new Error('WebGPU unavailable — local lane not supported in this browser'))
  }
  if (!enginePromise) {
    enginePromise = import('@mlc-ai/web-llm')
      .then((webllm) =>
        webllm.CreateMLCEngine(WEB_MODEL_ID, {
          initProgressCallback: (r) => onProgress?.({ progress: r.progress, text: r.text }),
        }),
      )
      .catch((e: unknown) => {
        // Don't cache a failed load — a transient failure (network drop
        // mid-download) must be retryable rather than permanently poisoning
        // the lane for the rest of the session.
        enginePromise = null
        throw e
      })
  }
  return enginePromise
}

/**
 * Stream a completion, emitting the same CoreA2uiEvents the Rust/Tauri lane
 * emits so `applyA2uiEvent` handles both identically.
 *
 * `messageId` addresses the assistant message the store already created, exactly
 * as the Tauri path does.
 */
export async function* streamWebLlm(
  prompt: string,
  messageId: string,
  history: { role: 'user' | 'assistant'; text: string }[] = [],
): AsyncGenerator<CoreA2uiEvent> {
  const engine = await loadWebLlm()

  const messages = [
    ...history.map((m) => ({ role: m.role, content: m.text })),
    { role: 'user' as const, content: prompt },
  ]

  // WebLLM's API is OpenAI-chat-completions compatible, so this is the same
  // request/chunk shape the cloud lane produces — no shim needed.
  const stream = await engine.chat.completions.create({ messages, stream: true })

  let text = ''
  for await (const chunk of stream) {
    const delta = chunk.choices[0]?.delta?.content
    if (!delta) continue
    // Accumulate: ContentBlock carries the full text so far, and blockIndex 0
    // means the store replaces rather than appends — matching how the Rust lane
    // streams a single growing text block.
    text += delta
    yield { type: 'contentBlock', messageId, blockIndex: 0, block: { type: 'text', text } }
  }

  yield { type: 'messageComplete', messageId }
}

/** Release the engine + its GPU memory. Safe when nothing is loaded. */
export async function unloadWebLlm(): Promise<void> {
  if (!enginePromise) return
  const engine = await enginePromise.catch(() => null)
  enginePromise = null
  await engine?.unload()
}
