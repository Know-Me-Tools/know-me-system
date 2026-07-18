// TJ-ARCH-MOB-001 compliant
//! Mobile CPU inference through llama.cpp. Model download is revision- and
//! SHA-256-pinned, atomic, and rooted in the platform app-data cache supplied by
//! the mobile leaf.

use std::num::NonZeroU32;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use futures::{StreamExt, TryStreamExt};
use gen_ui_types::error::{CoreError, CoreResult};
use gen_ui_types::events::StreamEvent;
use gen_ui_types::inference::{InferenceProvider, LocalModelSpec, SampleParams};
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaChatMessage, LlamaModel};
use llama_cpp_2::sampling::LlamaSampler;
use sha2::{Digest, Sha256};
use tokio::io::AsyncWriteExt;
use tokio::sync::RwLock;

const CATALOG_MODEL: &str = "qwen2.5-0.5b-instruct-q4";
const MODEL_FILE: &str = "qwen2.5-0.5b-instruct-q4_k_m.gguf";
const MODEL_REVISION: &str = "9217f5db79a29953eb74d5343926648285ec7e67";
const MODEL_SHA256: &str = "74a4da8c9fdbcd15bd1f6d01d621410d31c6fc00986f5eb687824e7b93d7a9db";
const MODEL_URL: &str = "https://huggingface.co/Qwen/Qwen2.5-0.5B-Instruct-GGUF/resolve/9217f5db79a29953eb74d5343926648285ec7e67/qwen2.5-0.5b-instruct-q4_k_m.gguf";
const DEFAULT_CONTEXT: u32 = 2048;

pub struct LlamaCppEngine {
    cache_dir: PathBuf,
    state: RwLock<Option<LoadedModel>>,
}

struct LoadedModel {
    backend: Arc<LlamaBackend>,
    model: Arc<LlamaModel>,
    spec: LocalModelSpec,
}

impl LlamaCppEngine {
    pub fn new(cache_dir: impl Into<PathBuf>) -> Self {
        Self {
            cache_dir: cache_dir.into(),
            state: RwLock::new(None),
        }
    }
}

fn terminal(context: &str, error: impl std::fmt::Display) -> CoreError {
    CoreError::Terminal(format!("{context}: {error}"))
}

fn resolve_model(spec: &LocalModelSpec, cache_dir: &Path) -> CoreResult<(PathBuf, bool)> {
    let path = PathBuf::from(&spec.model);
    if path.is_absolute() {
        return Ok((path, false));
    }
    if spec.model == CATALOG_MODEL {
        return Ok((cache_dir.join(MODEL_FILE), true));
    }
    Err(CoreError::NotFound(format!(
        "unknown mobile model '{}' — known: {CATALOG_MODEL}, or an absolute GGUF path",
        spec.model
    )))
}

async fn ensure_catalog_model(path: &Path) -> CoreResult<()> {
    if path.is_file() {
        return verify_sha256(path).await;
    }
    let parent = path
        .parent()
        .ok_or_else(|| terminal("model download", "cache path has no parent"))?;
    tokio::fs::create_dir_all(parent)
        .await
        .map_err(|error| terminal("model cache create", error))?;
    let temporary = path.with_extension("gguf.download");
    let response = reqwest::Client::new()
        .get(MODEL_URL)
        .send()
        .await
        .and_then(reqwest::Response::error_for_status)
        .map_err(|error| CoreError::Transient(format!("model download: {error}")))?;
    let mut file = tokio::fs::File::create(&temporary)
        .await
        .map_err(|error| terminal("model temporary file", error))?;
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream
        .try_next()
        .await
        .map_err(|error| CoreError::Transient(format!("model download stream: {error}")))?
    {
        file.write_all(&chunk)
            .await
            .map_err(|error| terminal("model cache write", error))?;
    }
    file.flush()
        .await
        .map_err(|error| terminal("model cache flush", error))?;
    drop(file);
    if let Err(error) = verify_sha256(&temporary).await {
        let _ = tokio::fs::remove_file(&temporary).await;
        return Err(error);
    }
    tokio::fs::rename(&temporary, path)
        .await
        .map_err(|error| terminal("model cache publish", error))?;
    tracing::info!(revision = MODEL_REVISION, path = %path.display(), "mobile model downloaded");
    Ok(())
}

async fn verify_sha256(path: &Path) -> CoreResult<()> {
    let path = path.to_owned();
    gen_ui_runtime::spawn_blocking(move || {
        let bytes = std::fs::read(&path).map_err(|error| terminal("model checksum read", error))?;
        let actual = format!("{:x}", Sha256::digest(bytes));
        if actual == MODEL_SHA256 {
            Ok(())
        } else {
            Err(terminal(
                "model checksum",
                format!("expected {MODEL_SHA256}, got {actual}"),
            ))
        }
    })
    .await
    .map_err(|error| terminal("model checksum task", error))?
}

#[async_trait]
impl InferenceProvider for LlamaCppEngine {
    async fn load(&self, spec: &LocalModelSpec) -> CoreResult<()> {
        if self
            .state
            .read()
            .await
            .as_ref()
            .is_some_and(|loaded| &loaded.spec == spec)
        {
            return Ok(());
        }
        let (path, catalog) = resolve_model(spec, &self.cache_dir)?;
        if catalog {
            ensure_catalog_model(&path).await?;
        } else if !path.is_file() {
            return Err(CoreError::NotFound(format!(
                "GGUF model does not exist: {}",
                path.display()
            )));
        }
        let loaded_spec = spec.clone();
        let loaded = gen_ui_runtime::spawn_blocking(move || {
            let backend = Arc::new(
                LlamaBackend::init().map_err(|error| terminal("llama backend init", error))?,
            );
            let params = LlamaModelParams::default();
            let model = LlamaModel::load_from_file(&backend, &path, &params)
                .map_err(|error| terminal("GGUF load", error))?;
            Ok::<_, CoreError>(LoadedModel {
                backend,
                model: Arc::new(model),
                spec: loaded_spec,
            })
        })
        .await
        .map_err(|error| terminal("GGUF load task", error))??;
        *self.state.write().await = Some(loaded);
        Ok(())
    }

    async fn generate(
        &self,
        prompt: &str,
        params: &SampleParams,
    ) -> CoreResult<futures::stream::BoxStream<'static, StreamEvent>> {
        let (backend, model, context_len) = {
            let state = self.state.read().await;
            let loaded = state
                .as_ref()
                .ok_or_else(|| terminal("mobile inference", "call load before generate"))?;
            (
                Arc::clone(&loaded.backend),
                Arc::clone(&loaded.model),
                loaded.spec.context_len.unwrap_or(DEFAULT_CONTEXT),
            )
        };
        let prompt = prompt.to_owned();
        let params = params.clone();
        let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
        gen_ui_runtime::spawn_blocking(move || {
            if let Err(error) =
                generate_blocking(&backend, &model, &prompt, context_len, &params, &sender)
            {
                let _ = sender.send(StreamEvent::Error {
                    message: error.to_string(),
                });
            }
        });
        Ok(tokio_stream::wrappers::UnboundedReceiverStream::new(receiver).boxed())
    }

    async fn unload(&self) -> CoreResult<()> {
        *self.state.write().await = None;
        Ok(())
    }
}

fn generate_blocking(
    backend: &LlamaBackend,
    model: &LlamaModel,
    prompt: &str,
    context_len: u32,
    params: &SampleParams,
    sender: &tokio::sync::mpsc::UnboundedSender<StreamEvent>,
) -> CoreResult<()> {
    let context_len = NonZeroU32::new(context_len)
        .ok_or_else(|| terminal("mobile inference", "context length cannot be zero"))?;
    let context_params = LlamaContextParams::default().with_n_ctx(Some(context_len));
    let mut context = model
        .new_context(backend, context_params)
        .map_err(|error| terminal("llama context", error))?;
    let formatted = (|| -> CoreResult<String> {
        let template = model
            .chat_template(None)
            .map_err(|error| terminal("chat template", error))?;
        let message = LlamaChatMessage::new("user".to_string(), prompt.to_string())
            .map_err(|error| terminal("chat message", error))?;
        model
            .apply_chat_template(&template, &[message], true)
            .map_err(|error| terminal("apply chat template", error))
    })()
    .unwrap_or_else(|_| prompt.to_string());
    let tokens = model
        .str_to_token(&formatted, AddBos::Always)
        .map_err(|error| terminal("prompt tokenize", error))?;
    if tokens.len() + params.max_tokens as usize > context_len.get() as usize {
        return Err(terminal(
            "mobile inference",
            "prompt plus requested output exceeds the context length",
        ));
    }
    let mut batch = LlamaBatch::new(tokens.len(), 1);
    let last = tokens.len().saturating_sub(1);
    for (position, token) in tokens.into_iter().enumerate() {
        batch
            .add(token, position as i32, &[0], position == last)
            .map_err(|error| terminal("prompt batch", error))?;
    }
    context
        .decode(&mut batch)
        .map_err(|error| terminal("prompt decode", error))?;
    let mut sampler = LlamaSampler::chain_simple([
        LlamaSampler::top_p(params.top_p, 1),
        LlamaSampler::temp(params.temperature),
        LlamaSampler::dist(0xC0FFEE),
    ]);
    let mut decoder = encoding_rs::UTF_8.new_decoder();
    for (position, index) in (batch.n_tokens()..).zip(0..params.max_tokens) {
        let token = sampler.sample(&context, batch.n_tokens() - 1);
        sampler.accept(token);
        if model.is_eog_token(token) {
            break;
        }
        let delta = model
            .token_to_piece(token, &mut decoder, true, None)
            .map_err(|error| terminal("token decode", error))?;
        if !delta.is_empty()
            && sender
                .send(StreamEvent::TextDelta { index, delta })
                .is_err()
        {
            return Ok(());
        }
        batch.clear();
        batch
            .add(token, position, &[0], true)
            .map_err(|error| terminal("token batch", error))?;
        context
            .decode(&mut batch)
            .map_err(|error| terminal("token decode step", error))?;
    }
    let _ = sender.send(StreamEvent::Done);
    Ok(())
}
