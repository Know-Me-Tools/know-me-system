// TJ-ARCH-MOB-001 compliant
//! Desktop local-inference lane: mistral.rs (Metal on macOS).
//!
//! Fulfils `InferenceProvider` so gen_ui_agent drives a local model through the
//! exact seam it uses for the remote gateway — `generate` yields the same
//! `StreamEvent`s the liter-llm path does, so gen_ui_protocol's A2uiAdapter
//! ingests local and cloud tokens identically and the UI never branches on lane.
use std::sync::Arc;

use async_trait::async_trait;
use futures::stream::{BoxStream, StreamExt};
use gen_ui_types::error::{CoreError, CoreResult};
use gen_ui_types::events::StreamEvent;
use gen_ui_types::inference::{InferenceProvider, LocalModelSpec, SampleParams};
use mistralrs::{
    ChatCompletionChunkResponse, ChunkChoice, Delta, GgufModelBuilder, Model, RequestBuilder,
    Response, TextMessageRole,
};
use tokio::sync::RwLock;

/// Classify a rendered engine error into the shared retryable/terminal split.
///
/// mistral.rs surfaces two unrelated error types across the API we use
/// (`anyhow::Error` from the builder, `mistralrs::error::Error` from streaming),
/// and neither exposes a structured out-of-memory variant — an allocation failure
/// arrives as a boxed candle/Metal error rendered into text. So classification is
/// done on the message. Cheap and honest: a misclassification costs one retry that
/// fails the same way, never a wrong answer.
fn classify(context: &str, rendered: String) -> CoreError {
    let lower = rendered.to_lowercase();
    let retryable = lower.contains("out of memory")
        || lower.contains("oom")
        || lower.contains("insufficient memory")
        || lower.contains("timed out")
        || lower.contains("timeout")
        || lower.contains("connection")
        || lower.contains("network")
        || lower.contains("dns");
    if retryable {
        CoreError::Transient(format!("{context}: {rendered}"))
    } else {
        CoreError::Terminal(format!("{context}: {rendered}"))
    }
}

/// Tokenizer/chat-template source for the GGUF quant repos we load. GGUF repos
/// ship weights only; mistral.rs pulls `tokenizer_config.json` + the chat
/// template from this separate repo (`with_tok_model_id`), so Qwen's template is
/// never hand-authored here.
const QWEN_TOK_REPO: &str = "Qwen/Qwen2.5-1.5B-Instruct";

/// The one catalog entry C-105 ships. `LocalModelSpec.model` is either this id or
/// an absolute path (trait contract), so the catalog stays a lookup, not a type.
const CATALOG_QWEN_1_5B: &str = "qwen2.5-1.5b-instruct-q4";
const CATALOG_QWEN_1_5B_REPO: &str = "Qwen/Qwen2.5-1.5B-Instruct-GGUF";
/// Q4_0, NOT the Q4_K_M the C-105 plan originally named.
///
/// K-quants are BROKEN for Qwen2.5 on the pinned mistral.rs fork: Q4_K_M and
/// Q5_K_M both emit degenerate repetition loops that echo the prompt back
/// ("Say hello in exactly word `` `` ``…"), while Q4_0 and Q8_0 answer
/// correctly. TinyLlama's Q4_K_M is fine, so this is Qwen-architecture-specific
/// K-quant dequantization — almost certainly the same candle version skew that
/// broke safetensors (see the [patch] note in the workspace Cargo.toml; that fix
/// made it compile, but the quant kernels are evidently still mismatched).
///
/// Verified directly against the engine's own API, bypassing all of our code, so
/// this is not a bug in this crate. Q4_0 is the same ~1GB size class and is the
/// honest choice until the fork re-syncs its candle. Re-test the K-quants after
/// any candle/mistral.rs bump — the evidence table is in the C-105 design doc.
const CATALOG_QWEN_1_5B_FILE: &str = "qwen2.5-1.5b-instruct-q4_0.gguf";

/// Top-k candidate pool. The `InferenceProvider` seam's `SampleParams` carries
/// only temperature/top_p/max_tokens (it's frozen — see gen_ui_types), so this
/// lane picks its own: 40 is the long-standing llama.cpp/HF default and is what
/// Qwen2.5's own generation_config recommends. It exists mainly to displace
/// mistral.rs's `top_k = 1` default — see the comment at the call site.
const DEFAULT_TOP_K: usize = 40;

/// mistral.rs-backed engine. `load` is idempotent per the trait contract: loading
/// the spec that is already resident is a no-op rather than a re-download.
pub struct MistralEngine {
    /// `None` until `load`. RwLock (not Mutex) so concurrent `generate` calls
    /// share the model — mistral.rs drives its own internal batching/threading.
    state: RwLock<Option<LoadedModel>>,
}

struct LoadedModel {
    model: Arc<Model>,
    /// What's resident, so `load` can detect the idempotent case.
    spec: LocalModelSpec,
}

impl MistralEngine {
    pub fn new() -> Self {
        Self {
            state: RwLock::new(None),
        }
    }
}

impl Default for MistralEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Resolve a spec's `model` into the (quant repo, gguf filename) mistral.rs needs.
/// An absolute path loads that file directly from disk; anything else must be a
/// known catalog id — an unknown id is Terminal (retrying won't invent a model).
fn resolve(spec: &LocalModelSpec) -> CoreResult<(String, String)> {
    if spec.model.starts_with('/') {
        let path = std::path::Path::new(&spec.model);
        let dir = path.parent().and_then(|p| p.to_str()).ok_or_else(|| {
            CoreError::Terminal(format!("model path has no parent dir: {}", spec.model))
        })?;
        let file = path.file_name().and_then(|f| f.to_str()).ok_or_else(|| {
            CoreError::Terminal(format!("model path has no filename: {}", spec.model))
        })?;
        return Ok((dir.to_string(), file.to_string()));
    }
    match spec.model.as_str() {
        CATALOG_QWEN_1_5B => Ok((CATALOG_QWEN_1_5B_REPO.to_string(), CATALOG_QWEN_1_5B_FILE.to_string())),
        other => Err(CoreError::NotFound(format!(
            "unknown local model id '{other}' — known: {CATALOG_QWEN_1_5B}, or an absolute .gguf path"
        ))),
    }
}

#[async_trait]
impl InferenceProvider for MistralEngine {
    async fn load(&self, spec: &LocalModelSpec) -> CoreResult<()> {
        // Idempotent per the trait contract — re-loading what's resident must not
        // re-download several GB.
        if self
            .state
            .read()
            .await
            .as_ref()
            .is_some_and(|l| &l.spec == spec)
        {
            return Ok(());
        }

        let (repo, file) = resolve(spec)?;

        // `GgufModelBuilder::build()` is `async fn`, but its implementation calls
        // the synchronous `Loader::load_model_from_hf` with no await. Download,
        // dequantization, and device mapping therefore run on the polling thread.
        // Drive the non-Send future on a current-thread runtime inside the global
        // runtime's blocking pool so model loading cannot starve Tokio workers.
        //
        // `spec.context_len` is intentionally unused here: GgufModelBuilder exposes
        // no sequence-length override (verified against the API at our pinned rev) —
        // a GGUF's context length is baked into the file's own metadata. The field
        // stays in the trait for the llama-cpp-2 mobile lane, which does accept one.
        let with_logging = std::env::var_os("GEN_UI_INFERENCE_LOG").is_some();
        let model = gen_ui_runtime::spawn_blocking(move || {
            let mut builder =
                GgufModelBuilder::new(repo, vec![file]).with_tok_model_id(QWEN_TOK_REPO);
            if with_logging {
                builder = builder.with_logging();
            }
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| classify("model load", format!("runtime build failed: {e}")))?
                .block_on(builder.build())
                .map_err(|e| classify("model load", format!("{e:#}")))
        })
        .await
        .map_err(|e| classify("model load", format!("load task failed: {e}")))??;

        *self.state.write().await = Some(LoadedModel {
            model: Arc::new(model),
            spec: spec.clone(),
        });
        tracing::info!(model = %spec.model, "local model loaded");
        Ok(())
    }

    async fn generate(
        &self,
        prompt: &str,
        params: &SampleParams,
    ) -> CoreResult<BoxStream<'static, StreamEvent>> {
        let model = {
            let guard = self.state.read().await;
            let loaded = guard.as_ref().ok_or_else(|| {
                CoreError::Terminal("no local model loaded — call load() first".to_string())
            })?;
            Arc::clone(&loaded.model)
        };

        let request = RequestBuilder::new()
            .add_message(TextMessageRole::User, prompt)
            .set_sampler_temperature(params.temperature as f64)
            .set_sampler_topp(params.top_p as f64)
            // MUST be set explicitly. RequestBuilder::new() starts from
            // SamplingParams::deterministic(), which pins top_k = 1 — pure greedy
            // decoding that overrides temperature and top_p entirely (there is
            // only ever one candidate to sample from). Left at the default, a
            // 1.5B model degenerates into a repetition loop that echoes the
            // prompt back forever, and no temperature value changes it.
            // Verified against a real model: see tests/local_inference_live.rs.
            .set_sampler_topk(DEFAULT_TOP_K)
            .set_sampler_max_len(params.max_tokens as usize);

        // mistral.rs's `Stream<'a>` borrows the `Model` it came from, but the trait
        // contract returns a `BoxStream<'static, _>`. Rather than fight that, drive
        // the borrowed stream inside a task that owns the `Arc<Model>` keeping it
        // alive, and hand the caller the receiving end. The task also outlives this
        // function, so generation continues even if the caller is slow to poll.
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<StreamEvent>();

        gen_ui_runtime::spawn(async move {
            let mut stream = match model.stream_chat_request(request).await {
                Ok(s) => s,
                Err(e) => {
                    // Terminal for this run — report and end the stream rather than
                    // leaving the UI spinning.
                    let _ = tx.send(StreamEvent::Error {
                        message: e.to_string(),
                    });
                    return;
                }
            };

            // Adapt mistral.rs's OpenAI-shaped chunks into StreamEvent. `index`
            // counts deltas within this run, matching the cloud lane's numbering.
            let mut index: u32 = 0;
            while let Some(response) = stream.next().await {
                let event = match response {
                    Response::Chunk(ChatCompletionChunkResponse { choices, .. }) => {
                        match choices.first() {
                            Some(ChunkChoice {
                                delta:
                                    Delta {
                                        content: Some(content),
                                        ..
                                    },
                                ..
                            }) if !content.is_empty() => {
                                let e = StreamEvent::TextDelta {
                                    index,
                                    delta: content.clone(),
                                };
                                index += 1;
                                Some(e)
                            }
                            // Empty/role-only/keep-alive chunk: nothing to render.
                            _ => None,
                        }
                    }
                    // ModelError carries a String + the partial response; the other
                    // two carry a boxed error. All three end the run.
                    Response::ModelError(message, _) => Some(StreamEvent::Error { message }),
                    Response::ValidationError(e) | Response::InternalError(e) => {
                        Some(StreamEvent::Error {
                            message: e.to_string(),
                        })
                    }
                    // Completion/image/speech/embedding/agentic variants cannot occur
                    // on a chat stream_chat_request.
                    _ => None,
                };
                if let Some(event) = event {
                    let is_error = matches!(event, StreamEvent::Error { .. });
                    // Receiver dropped (caller stopped listening) — stop generating.
                    if tx.send(event).is_err() || is_error {
                        return;
                    }
                }
            }

            // Signal completion on clean exhaustion. Mirrors the cloud lane, which
            // also emits Done when a provider omits a final finish_reason.
            let _ = tx.send(StreamEvent::Done);
        });

        Ok(tokio_stream::wrappers::UnboundedReceiverStream::new(rx).boxed())
    }

    async fn unload(&self) -> CoreResult<()> {
        // Safe when nothing is loaded, per the trait contract. Dropping the Arc
        // frees the weights once in-flight generations release their clones.
        *self.state.write().await = None;
        tracing::info!("local model unloaded");
        Ok(())
    }
}
