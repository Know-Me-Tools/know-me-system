// TJ-ARCH-MOB-001 compliant
//! Mobile CPU inference through llama.cpp. Model download is revision- and
//! SHA-256-pinned, atomic, and rooted in the platform app-data cache supplied by
//! the mobile leaf.

use std::num::NonZeroU32;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::catalog::{
    artifact_set_for_id, ArtifactSpec, ANDROID_QWEN35_4B, ANDROID_QWEN35_4B_ID, IPHONE_QWEN35_9B_ID,
};
use async_trait::async_trait;
use futures::{StreamExt, TryStreamExt};
use gen_ui_types::error::{CoreError, CoreResult};
use gen_ui_types::events::StreamEvent;
#[cfg(feature = "local-llama-mtmd")]
use gen_ui_types::inference::InferenceRequest;
use gen_ui_types::inference::{InferenceProvider, LocalModelSpec, SampleParams};
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaChatMessage, LlamaModel};
#[cfg(feature = "local-llama-mtmd")]
use llama_cpp_2::mtmd::{
    mtmd_default_marker, MtmdBitmap, MtmdContext, MtmdContextParams, MtmdInputText,
};
use llama_cpp_2::sampling::LlamaSampler;
use reqwest::header::RANGE;
use reqwest::StatusCode;
use sha2::{Digest, Sha256};
use std::io::Read;
use tokio::io::AsyncWriteExt;
use tokio::sync::RwLock;

const DEFAULT_CONTEXT: u32 = 8192;
const DEFAULT_MAX_MODEL_BYTES: u64 = 8 * 1024 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeDiagnostics {
    pub model_id: String,
    pub backend: String,
    pub fallback_reason: Option<String>,
}

pub struct LlamaCppEngine {
    cache_dir: PathBuf,
    state: RwLock<Option<LoadedModel>>,
    diagnostics: RwLock<Option<RuntimeDiagnostics>>,
    #[cfg(feature = "local-llama-mtmd")]
    multimodal: RwLock<Option<Arc<MtmdContext>>>,
}

struct LoadedModel {
    backend: Arc<LlamaBackend>,
    model: Arc<LlamaModel>,
    spec: LocalModelSpec,
    diagnostics: RuntimeDiagnostics,
}

impl LlamaCppEngine {
    pub fn new(cache_dir: impl Into<PathBuf>) -> Self {
        Self {
            cache_dir: cache_dir.into(),
            state: RwLock::new(None),
            diagnostics: RwLock::new(None),
            #[cfg(feature = "local-llama-mtmd")]
            multimodal: RwLock::new(None),
        }
    }

    pub async fn diagnostics(&self) -> Option<RuntimeDiagnostics> {
        self.diagnostics.read().await.clone()
    }

    /// Ensure the projector paired with a catalog model is present and
    /// checksum-verified. The mtmd loader calls this after model selection so
    /// a mismatched projector can never be attached to a GGUF.
    pub async fn ensure_projector(&self, model_id: &str) -> CoreResult<PathBuf> {
        let set = artifact_set_for_id(model_id).ok_or_else(|| {
            CoreError::NotFound(format!(
                "unknown multimodal model '{model_id}' — no projector is cataloged"
            ))
        })?;
        preflight_artifact(set.projector)?;
        let path = self.cache_dir.join(set.projector.file_name);
        ensure_catalog_model(&path, set.projector).await?;
        Ok(path)
    }

    /// Download the paired projector and initialize llama.cpp's MTMD context
    /// against the currently loaded text model. The context is cached until
    /// unload, so each image turn does not rebuild the vision encoder.
    #[cfg(feature = "local-llama-mtmd")]
    pub async fn ensure_multimodal(&self, model_id: &str) -> CoreResult<()> {
        if self.multimodal.read().await.is_some() {
            return Ok(());
        }
        let projector = self.ensure_projector(model_id).await?;
        let model = {
            let state = self.state.read().await;
            let loaded = state
                .as_ref()
                .ok_or_else(|| terminal("multimodal inference", "call load before images"))?;
            if loaded.spec.model != model_id {
                return Err(terminal(
                    "multimodal inference",
                    format!(
                        "projector model {model_id} does not match loaded {}",
                        loaded.spec.model
                    ),
                ));
            }
            Arc::clone(&loaded.model)
        };
        let projector = projector.to_string_lossy().into_owned();
        let context = gen_ui_runtime::spawn_blocking(move || {
            MtmdContext::init_from_file(&projector, &model, &MtmdContextParams::default())
                .map_err(|error| terminal("mtmd init", error))
        })
        .await
        .map_err(|error| terminal("mtmd init task", error))??;
        if !context.support_vision() {
            return Err(terminal(
                "mtmd init",
                "projector does not expose vision support",
            ));
        }
        *self.multimodal.write().await = Some(Arc::new(context));
        Ok(())
    }
}

fn terminal(context: &str, error: impl std::fmt::Display) -> CoreError {
    CoreError::Terminal(format!("{context}: {error}"))
}

fn resolve_model(
    spec: &LocalModelSpec,
    cache_dir: &Path,
) -> CoreResult<(PathBuf, Option<ArtifactSpec>)> {
    let path = PathBuf::from(&spec.model);
    if path.is_absolute() {
        return Ok((path, None));
    }
    if let Some(set) = artifact_set_for_id(&spec.model) {
        return Ok((cache_dir.join(set.model.file_name), Some(set.model)));
    }
    Err(CoreError::NotFound(format!(
        "unknown mobile model '{}' — expected a Qwen3.5 catalog id or an absolute GGUF path",
        spec.model,
    )))
}

async fn ensure_catalog_model(path: &Path, artifact: ArtifactSpec) -> CoreResult<()> {
    if path.is_file() && verify_artifact(path, artifact).await.is_ok() {
        return Ok(());
    }
    let parent = path
        .parent()
        .ok_or_else(|| terminal("model download", "cache path has no parent"))?;
    tokio::fs::create_dir_all(parent)
        .await
        .map_err(|error| terminal("model cache create", error))?;
    let temporary = path.with_extension("gguf.part");
    let existing = tokio::fs::metadata(&temporary)
        .await
        .map(|metadata| metadata.len())
        .unwrap_or(0);
    let mut request = reqwest::Client::new().get(artifact.url);
    if existing > 0 && existing < artifact.byte_len {
        request = request.header(RANGE, format!("bytes={existing}-"));
    }
    let response = request
        .send()
        .await
        .and_then(reqwest::Response::error_for_status)
        .map_err(|error| CoreError::Transient(format!("model download: {error}")))?;
    let append = should_append_download(existing, artifact.byte_len, response.status());
    let mut options = tokio::fs::OpenOptions::new();
    options.write(true).create(true);
    if append {
        options.append(true);
    } else {
        options.truncate(true);
    }
    let mut file = options
        .open(&temporary)
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
    if let Err(error) = verify_artifact(&temporary, artifact).await {
        let _ = tokio::fs::remove_file(&temporary).await;
        return Err(error);
    }
    tokio::fs::rename(&temporary, path)
        .await
        .map_err(|error| terminal("model cache publish", error))?;
    tracing::info!(
        revision = artifact.revision,
        bytes = artifact.byte_len,
        path = %path.display(),
        "mobile model downloaded"
    );
    Ok(())
}

fn preflight_artifact(artifact: ArtifactSpec) -> CoreResult<()> {
    let max_bytes = std::env::var("KNOWME_LOCAL_MODEL_MAX_BYTES")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(DEFAULT_MAX_MODEL_BYTES);
    if artifact.byte_len > max_bytes {
        return Err(CoreError::Transient(format!(
            "model {} is {} bytes, above the {} byte mobile preflight budget",
            artifact.file_name, artifact.byte_len, max_bytes
        )));
    }
    Ok(())
}

fn gpu_backend_name() -> &'static str {
    #[cfg(target_os = "ios")]
    {
        return "metal";
    }
    #[cfg(target_os = "android")]
    {
        return "vulkan";
    }
    "gpu"
}

fn should_append_download(existing: u64, expected: u64, status: StatusCode) -> bool {
    existing > 0 && existing < expected && status == StatusCode::PARTIAL_CONTENT
}

fn load_model_with_backend(
    backend: Arc<LlamaBackend>,
    path: PathBuf,
    spec: LocalModelSpec,
    fallback_reason: Option<String>,
) -> CoreResult<LoadedModel> {
    let supports_gpu = backend.supports_gpu_offload();
    let attempts = if supports_gpu {
        vec![true, false]
    } else {
        vec![false]
    };
    let mut last_error = String::from("no model load attempt");
    for use_gpu in attempts {
        let params = if use_gpu {
            LlamaModelParams::default().with_n_gpu_layers(u32::MAX)
        } else {
            LlamaModelParams::default()
        };
        match LlamaModel::load_from_file(&backend, &path, &params) {
            Ok(model) => {
                let backend_name = if use_gpu {
                    gpu_backend_name().to_string()
                } else {
                    "cpu".to_string()
                };
                let mut reason = fallback_reason.clone();
                if supports_gpu && !use_gpu && reason.is_none() {
                    reason = Some("GPU model load failed; CPU fallback selected".to_string());
                }
                return Ok(LoadedModel {
                    backend,
                    model: Arc::new(model),
                    spec: spec.clone(),
                    diagnostics: RuntimeDiagnostics {
                        model_id: spec.model,
                        backend: backend_name,
                        fallback_reason: reason,
                    },
                });
            }
            Err(error) => last_error = error.to_string(),
        }
    }
    Err(terminal("GGUF load", last_error))
}

async fn verify_artifact(path: &Path, artifact: ArtifactSpec) -> CoreResult<()> {
    let path = path.to_owned();
    gen_ui_runtime::spawn_blocking(move || {
        let metadata =
            std::fs::metadata(&path).map_err(|error| terminal("model checksum metadata", error))?;
        if metadata.len() != artifact.byte_len {
            return Err(terminal(
                "model byte length",
                format!("expected {}, got {}", artifact.byte_len, metadata.len()),
            ));
        }
        let mut file =
            std::fs::File::open(&path).map_err(|error| terminal("model checksum read", error))?;
        let mut digest = Sha256::new();
        let mut buffer = [0_u8; 1024 * 1024];
        loop {
            let read = file
                .read(&mut buffer)
                .map_err(|error| terminal("model checksum read", error))?;
            if read == 0 {
                break;
            }
            digest.update(&buffer[..read]);
        }
        let actual = format!("{:x}", digest.finalize());
        if actual == artifact.sha256 {
            Ok(())
        } else {
            Err(terminal(
                "model checksum",
                format!("expected {}, got {actual}", artifact.sha256),
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
        let (path, artifact) = resolve_model(spec, &self.cache_dir)?;
        if let Some(artifact) = artifact {
            preflight_artifact(artifact)?;
            ensure_catalog_model(&path, artifact).await?;
        } else if !path.is_file() {
            return Err(CoreError::NotFound(format!(
                "GGUF model does not exist: {}",
                path.display()
            )));
        }
        let backend =
            Arc::new(LlamaBackend::init().map_err(|error| terminal("llama backend init", error))?);
        let loaded_spec = spec.clone();
        let primary_result = gen_ui_runtime::spawn_blocking({
            let backend = Arc::clone(&backend);
            let path = path.clone();
            move || load_model_with_backend(backend, path, loaded_spec, None)
        })
        .await
        .map_err(|error| terminal("GGUF load task", error))?;
        let primary_model = spec.model.clone();
        let loaded = match primary_result {
            Ok(loaded) => loaded,
            Err(primary_error) if spec.model == IPHONE_QWEN35_9B_ID => {
                let fallback = ANDROID_QWEN35_4B;
                preflight_artifact(fallback.model)?;
                let fallback_path = self.cache_dir.join(fallback.model.file_name);
                ensure_catalog_model(&fallback_path, fallback.model).await?;
                let fallback_spec = LocalModelSpec {
                    model: ANDROID_QWEN35_4B_ID.to_string(),
                    context_len: spec.context_len,
                };
                gen_ui_runtime::spawn_blocking({
                    let backend = Arc::clone(&backend);
                    move || {
                        load_model_with_backend(
                            backend,
                            fallback_path,
                            fallback_spec,
                            Some(format!(
                                "primary model {} failed: {primary_error}",
                                primary_model
                            )),
                        )
                    }
                })
                .await
                .map_err(|error| terminal("fallback GGUF load task", error))??
            }
            Err(error) => return Err(error),
        };
        *self.diagnostics.write().await = Some(loaded.diagnostics.clone());
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

    #[cfg(feature = "local-llama-mtmd")]
    async fn generate_request(
        &self,
        request: &InferenceRequest,
        params: &SampleParams,
    ) -> CoreResult<futures::stream::BoxStream<'static, StreamEvent>> {
        if request.images.is_empty() {
            return self.generate(&request.prompt, params).await;
        }
        if request.images.len() > 4 {
            return Err(terminal(
                "multimodal inference",
                "at most four images are supported per turn",
            ));
        }
        if request
            .images
            .iter()
            .any(|image| image.bytes.len() > 25 * 1024 * 1024)
        {
            return Err(terminal(
                "multimodal inference",
                "an image exceeds the 25 MB limit",
            ));
        }
        let model_id = {
            let state = self.state.read().await;
            let loaded = state
                .as_ref()
                .ok_or_else(|| terminal("multimodal inference", "call load before generate"))?;
            loaded.spec.model.clone()
        };
        self.ensure_multimodal(&model_id).await?;
        let (backend, model, mtmd, context_len) = {
            let state = self.state.read().await;
            let loaded = state
                .as_ref()
                .ok_or_else(|| terminal("multimodal inference", "call load before generate"))?;
            let mtmd = self
                .multimodal
                .read()
                .await
                .as_ref()
                .ok_or_else(|| terminal("multimodal inference", "MTMD is not initialized"))
                .map(Arc::clone)?;
            (
                Arc::clone(&loaded.backend),
                Arc::clone(&loaded.model),
                mtmd,
                loaded.spec.context_len.unwrap_or(DEFAULT_CONTEXT),
            )
        };
        let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
        let request = request.clone();
        let params = params.clone();
        gen_ui_runtime::spawn_blocking(move || {
            if let Err(error) = generate_multimodal_blocking(
                &backend,
                &model,
                &mtmd,
                &request,
                context_len,
                &params,
                &sender,
            ) {
                let _ = sender.send(StreamEvent::Error {
                    message: error.to_string(),
                });
            }
        });
        Ok(tokio_stream::wrappers::UnboundedReceiverStream::new(receiver).boxed())
    }

    async fn unload(&self) -> CoreResult<()> {
        *self.state.write().await = None;
        #[cfg(feature = "local-llama-mtmd")]
        {
            *self.multimodal.write().await = None;
        }
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

#[cfg(feature = "local-llama-mtmd")]
fn generate_multimodal_blocking(
    backend: &LlamaBackend,
    model: &LlamaModel,
    mtmd: &MtmdContext,
    request: &InferenceRequest,
    context_len: u32,
    params: &SampleParams,
    sender: &tokio::sync::mpsc::UnboundedSender<StreamEvent>,
) -> CoreResult<()> {
    let context_len = NonZeroU32::new(context_len)
        .ok_or_else(|| terminal("multimodal inference", "context length cannot be zero"))?;
    let context_params = LlamaContextParams::default().with_n_ctx(Some(context_len));
    let mut context = model
        .new_context(backend, context_params)
        .map_err(|error| terminal("multimodal context", error))?;
    let bitmaps = request
        .images
        .iter()
        .map(|image| {
            MtmdBitmap::from_buffer(mtmd, &image.bytes, false)
                .map_err(|error| terminal("image decode", error))
        })
        .collect::<CoreResult<Vec<_>>>()?;
    let bitmap_refs = bitmaps.iter().collect::<Vec<_>>();
    let input = MtmdInputText {
        text: format!("{}\n{}", mtmd_default_marker(), request.prompt),
        add_special: true,
        parse_special: true,
    };
    let chunks = mtmd
        .tokenize(input, &bitmap_refs)
        .map_err(|error| terminal("image tokenize", error))?;
    let mut n_past = chunks
        .eval_chunks(mtmd, &context, 0, 0, 512, true)
        .map_err(|error| terminal("image embedding", error))?;
    if n_past as u32 + params.max_tokens > context_len.get() {
        return Err(terminal(
            "multimodal inference",
            "image prompt plus requested output exceeds the context length",
        ));
    }
    let mut sampler = LlamaSampler::chain_simple([
        LlamaSampler::top_p(params.top_p, 1),
        LlamaSampler::temp(params.temperature),
        LlamaSampler::dist(0xC0FFEE),
    ]);
    let mut decoder = encoding_rs::UTF_8.new_decoder();
    let mut batch = LlamaBatch::new(1, 1);
    for index in 0..params.max_tokens {
        let token = sampler.sample(&context, -1);
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
            .add(token, n_past, &[0], true)
            .map_err(|error| terminal("token batch", error))?;
        context
            .decode(&mut batch)
            .map_err(|error| terminal("token decode step", error))?;
        n_past += 1;
    }
    let _ = sender.send(StreamEvent::Done);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::{ANDROID_QWEN35_4B_ID, IPHONE_QWEN35_9B_ID};

    #[test]
    fn resolves_each_catalog_model_to_its_pinned_file() {
        let temp = Path::new("/tmp/knowme-model-cache");
        let (iphone, iphone_artifact) = resolve_model(
            &LocalModelSpec {
                model: IPHONE_QWEN35_9B_ID.to_string(),
                context_len: Some(8192),
            },
            temp,
        )
        .expect("iPhone catalog model should resolve");
        assert!(iphone.ends_with("Qwen3.5-9B-Q4_K_M.gguf"));
        assert_eq!(iphone_artifact.expect("artifact").byte_len, 5_680_522_464);

        let (android, android_artifact) = resolve_model(
            &LocalModelSpec {
                model: ANDROID_QWEN35_4B_ID.to_string(),
                context_len: Some(8192),
            },
            temp,
        )
        .expect("Android catalog model should resolve");
        assert!(android.ends_with("Qwen3.5-4B-Q4_K_M.gguf"));
        assert_eq!(android_artifact.expect("artifact").byte_len, 2_740_937_888);
    }

    #[test]
    fn range_download_only_appends_on_partial_response() {
        assert!(should_append_download(10, 100, StatusCode::PARTIAL_CONTENT));
        assert!(!should_append_download(10, 100, StatusCode::OK));
        assert!(!should_append_download(
            100,
            100,
            StatusCode::PARTIAL_CONTENT
        ));
        assert!(!should_append_download(0, 100, StatusCode::PARTIAL_CONTENT));
    }

    #[test]
    fn preflight_rejects_artifacts_above_configured_default_budget() {
        let oversized = ArtifactSpec {
            byte_len: DEFAULT_MAX_MODEL_BYTES + 1,
            ..crate::catalog::ANDROID_QWEN35_4B.model
        };
        assert!(preflight_artifact(oversized).is_err());
    }
}
