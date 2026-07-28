// TJ-ARCH-MOB-001 compliant
//! Universal Agent Runtime boundary for KnowMe.
//!
//! UAR owns real agent execution, skills, A2UI registry management, tool
//! governance, and general provider routing. KnowMe owns the product shell,
//! local-device provider adapters, and the local conversation projection. This
//! module exposes the same facade for the linked mobile kernel and service UAR.

use futures::{stream::BoxStream, StreamExt};
use gen_ui_protocol::{
    normalize_uar_agui_event, uar_llm_payload_to_inference_request, A2uiAdapter,
    UarLlmRequestPayload,
};
use gen_ui_types::events::A2uiEvent;
use gen_ui_types::events::StreamEvent;
use gen_ui_types::inference::{InferenceProvider, LocalModelSpec};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};
pub use universal_agent_runtime::llm::ProviderConfig as UarProviderConfig;
use universal_agent_runtime::{
    config::{build_client_config, LlmConfig},
    llm::{
        registry::ModelConfig, registry::ProtocolSetting, LiterLlmDriver, LlmDriver, LlmRequest,
    },
    normalized::{Citation as UarCitation, NormalizedEvent as UarNormalizedEvent},
};

use crate::error::AgentError;

/// The well-known provider id under which the on-device local model is presented
/// in the unified provider picker. Selecting this id routes the embedded UAR run
/// to the on-device inference bridge instead of a cloud provider.
pub const LOCAL_PROVIDER_ID: &str = "on-device";

/// Config-store setting holding the id of the provider the embedded UAR should
/// route to (either [`LOCAL_PROVIDER_ID`] or a configured cloud provider id).
/// Unset ⇒ the on-device local model (offline-capable first-run default).
pub const SETTING_UAR_ACTIVE_PROVIDER: &str = "uar_active_provider";

/// How this platform should access UAR.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UarDeploymentMode {
    /// Flutter mobile target: linked library/kernel behind Rust FFI.
    EmbeddedLibrary,
    /// Desktop/web target: HTTP/SSE service, possibly supervised as a sidecar.
    HttpService,
    /// Remote authoritative runtime when local capabilities are unavailable.
    CloudService,
}

/// What is actually wired in this build today.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UarIntegrationState {
    /// UAR is pinned and planned, but chat execution is not routed through it.
    NotWired,
    /// UAR event payloads can be projected into KnowMe content blocks.
    ProjectionOnly,
    /// An in-process UAR kernel owns mobile agent execution.
    EmbeddedKernel,
    /// An HTTP/service UAR owns web/desktop agent execution.
    ServiceRuntime,
}

/// Runtime-mode facts the UI/certification tools can surface without implying
/// UAR execution is certified before it is.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UarRuntimeStatus {
    pub authority: String,
    pub target_mode: UarDeploymentMode,
    pub current_state: UarIntegrationState,
    pub local_provider_role: String,
    pub mobile_sidecar_allowed: bool,
    pub local_provider_bridge_available: bool,
    pub service_runtime_available: bool,
    pub service_artifact_response_available: bool,
    pub embedded_kernel_available: bool,
    pub embedded_kernel_blocker: Option<String>,
    pub submodule_path: String,
    pub submodule_commit: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UarRunRequest {
    /// Compiled UAR agent artifact. UAR owns this schema; KnowMe passes it
    /// through and persists only the selected ids/provenance it needs.
    pub artifact: Option<serde_json::Value>,
    pub input: String,
    pub session_id: Option<String>,
    pub agent_id: Option<String>,
    pub run_policy: Option<serde_json::Value>,
    /// Prior conversation turns from the host's durable store, used to seed an
    /// empty embedded session after a cold start so the UAR lane presents the
    /// full thread to the model rather than only the current message.
    #[serde(default)]
    pub history: Vec<UarSeedMessage>,
}

/// A prior conversation turn supplied to seed the embedded session.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UarSeedMessage {
    /// `"user"`, `"assistant"`, `"tool"`, or `"system"`.
    pub role: String,
    /// The message text.
    pub content: String,
    /// The tool-call id when `role == "tool"`.
    #[serde(default)]
    pub tool_call_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UarArtifactResponse {
    pub artifact_id: String,
    pub response: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UarA2uiAction {
    pub surface_id: String,
    pub name: String,
    pub source_component_id: String,
    pub timestamp: Option<String>,
    pub context: serde_json::Value,
    pub client_data_model: Option<serde_json::Value>,
}

pub struct UarProjectedRun {
    pub run_id: String,
    pub events: BoxStream<'static, Result<A2uiEvent, AgentError>>,
}

/// Stable UAR provider identity for Android's in-process LiteRT-LM lane.
///
/// The provider id names the execution boundary, not a particular accelerator.
/// CPU, GPU, and NPU selection remain runtime diagnostics beneath the same
/// provider/model route, so agents and persisted policies survive backend
/// fallback without being rewritten.
pub const ANDROID_LITERT_LM_PROVIDER_ID: &str = "knowme-local-litert-lm";
pub const ANDROID_LITERT_LM_TRUSTED_CONTEXT_TOKENS: u32 = 8_192;
pub const ANDROID_LITERT_LM_MAX_OUTPUT_TOKENS: u32 = 1_024;

/// Build the credential-free UAR registration for Android's pinned LiteRT-LM
/// model artifact.
///
/// The platform leaf supplies `model_id` from `gen_ui_inference::catalog`,
/// keeping the agent layer engine-agnostic while ensuring the exact downloaded
/// artifact becomes UAR's enabled default model.  The capability flags describe
/// the certified text-agent lane: native tools, explicit thinking, constrained
/// structured output, and streaming are available; image tensors are not sent
/// to this model and instead go through document/OCR extraction.
#[must_use]
pub fn android_litert_lm_provider_config(model_id: impl Into<String>) -> UarProviderConfig {
    let model_id = model_id.into();
    UarProviderConfig {
        id: ANDROID_LITERT_LM_PROVIDER_ID.to_string(),
        display_name: "KnowMe on-device · LiteRT-LM".to_string(),
        base_url: String::new(),
        api_key: None,
        protocol: ProtocolSetting::Auto,
        default_model: Some(model_id.clone()),
        models: vec![ModelConfig {
            id: model_id,
            display_name: Some("Gemma 4 E2B Instruct · on-device".to_string()),
            context_window: Some(ANDROID_LITERT_LM_TRUSTED_CONTEXT_TOKENS),
            supports_vision: false,
            supports_tools: true,
            supports_reasoning: true,
            supports_structured_output: true,
            supports_streaming: true,
            max_output_tokens: Some(ANDROID_LITERT_LM_MAX_OUTPUT_TOKENS),
            enabled: true,
        }],
        enabled: true,
    }
}

pub const IOS_QWEN35_TRUSTED_CONTEXT_TOKENS: u32 = 8_192;
pub const IOS_QWEN35_MAX_OUTPUT_TOKENS: u32 = 1_024;

/// Stable UAR provider identity for iOS's in-process MLX-Swift lane.
///
/// The provider id names the execution boundary, not the model file or
/// accelerator. It stays stable across model swaps so persisted agent policies
/// survive an artifact change.
pub const IOS_MLX_PROVIDER_ID: &str = "knowme-local-mlx";

/// Build the credential-free UAR registration for iOS's MLX-Swift model.
///
/// Text-only today: MLXVLM's Swift registry has no Qwen3.5 vision processor
/// yet, so images route through document/OCR extraction rather than image
/// tensors. UAR owns agent orchestration, tool policy, and run events.
#[must_use]
pub fn ios_qwen35_mlx_provider_config(model_id: impl Into<String>) -> UarProviderConfig {
    let model_id = model_id.into();
    UarProviderConfig {
        id: IOS_MLX_PROVIDER_ID.to_string(),
        display_name: "KnowMe on-device · MLX".to_string(),
        base_url: String::new(),
        api_key: None,
        protocol: ProtocolSetting::Auto,
        default_model: Some(model_id.clone()),
        models: vec![ModelConfig {
            id: model_id,
            display_name: Some("Qwen3 4B · on-device (MLX)".to_string()),
            context_window: Some(IOS_QWEN35_TRUSTED_CONTEXT_TOKENS),
            supports_vision: false,
            supports_tools: true,
            supports_reasoning: true,
            supports_structured_output: true,
            supports_streaming: true,
            max_output_tokens: Some(IOS_QWEN35_MAX_OUTPUT_TOKENS),
            enabled: true,
        }],
        enabled: true,
    }
}

/// UAR host driver backed by KnowMe's platform-selected local inference
/// provider.
///
/// UAR remains responsible for agent prompts, policy, tool loops, and run
/// lifecycle.  This adapter performs only the typed engine-boundary conversion:
///
/// `UAR LlmRequest → KnowMe InferenceProvider → UAR NormalizedEvent`.
#[derive(Clone)]
pub struct UarLocalInferenceDriverBridge {
    provider: Arc<dyn InferenceProvider>,
    model_spec: LocalModelSpec,
}

impl UarLocalInferenceDriverBridge {
    pub fn new(provider: Arc<dyn InferenceProvider>, model_spec: LocalModelSpec) -> Self {
        Self {
            provider,
            model_spec,
        }
    }

    pub async fn stream_payload(
        &self,
        payload: UarLlmRequestPayload,
    ) -> Result<BoxStream<'static, Result<serde_json::Value, AgentError>>, AgentError> {
        let stream = self
            .stream(LlmRequest {
                messages: payload.messages,
                tools: payload.tools,
                cache_strategy: None,
                thinking_config: None,
                anthropic_system: None,
                extra_params: payload.extra_params,
            })
            .await
            .map_err(|error| AgentError::Interaction(error.to_string()))?;
        Ok(stream
            .map(|event| {
                event
                    .map_err(|error| AgentError::Interaction(error.to_string()))
                    .and_then(|event| {
                        serde_json::to_value(event)
                            .map_err(|error| AgentError::Interaction(error.to_string()))
                    })
            })
            .boxed())
    }
}

#[async_trait::async_trait]
impl LlmDriver for UarLocalInferenceDriverBridge {
    async fn stream(
        &self,
        request: LlmRequest,
    ) -> anyhow::Result<
        std::pin::Pin<Box<dyn futures::Stream<Item = anyhow::Result<UarNormalizedEvent>> + Send>>,
    > {
        let request = uar_llm_request_to_inference_request(request)?;
        self.provider
            .load(&self.model_spec)
            .await
            .map_err(|error| anyhow::anyhow!("load local model: {error}"))?;
        let source = self
            .provider
            .generate_request(&request)
            .await
            .map_err(|error| anyhow::anyhow!("start local inference: {error}"))?;
        let adapter = UarNativeEventAdapter::new(format!("knowme-local-{}", uuid::Uuid::new_v4()));

        Ok(futures::stream::unfold(
            (source, adapter, VecDeque::new(), false),
            |(mut source, mut adapter, mut pending, mut source_finished)| async move {
                loop {
                    if let Some(event) = pending.pop_front() {
                        return Some((Ok(event), (source, adapter, pending, source_finished)));
                    }
                    if source_finished {
                        return None;
                    }
                    match source.next().await {
                        Some(event) => pending.extend(adapter.ingest(event)),
                        None => {
                            source_finished = true;
                            pending.extend(adapter.finish());
                        }
                    }
                }
            },
        )
        .boxed())
    }
}

/// The resolved model backend a single embedded-UAR run should execute against.
///
/// Produced by an [`ActiveModelSelector`] at `stream()` time and consumed by the
/// [`UarRoutingDriver`] to dispatch either to the on-device inference bridge or a
/// cloud `LiterLlmDriver`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActiveRoute {
    /// Route to the on-device local model (the wrapped inference bridge).
    Local,
    /// Route to a configured cloud provider/model over liter-llm.
    Cloud {
        /// Configured provider id (e.g. `"openai"`), used for cache keying and
        /// the liter-llm `provider/model` address.
        provider_id: String,
        /// Model id within the provider (e.g. `"gpt-4o"`).
        model_id: String,
        /// Optional base-URL override; `None` uses liter-llm auto-detection.
        base_url: Option<String>,
        /// Resolved API key for the provider.
        api_key: String,
    },
}

/// Resolves the currently-selected model backend for the embedded UAR. Kept as a
/// trait so the [`UarRoutingDriver`] is unit-testable with a scripted selector.
#[async_trait::async_trait]
pub trait ActiveModelSelector: Send + Sync {
    /// The route the next run should execute against.
    async fn active(&self) -> Result<ActiveRoute, AgentError>;
}

/// [`ActiveModelSelector`] backed by the KnowMe config store. Reads the selected
/// provider id from [`SETTING_UAR_ACTIVE_PROVIDER`]; the on-device local id (or an
/// unset/dangling selection) resolves to [`ActiveRoute::Local`], any other id to
/// the matching configured cloud provider + its selected model.
pub struct ConfigActiveSelector;

#[async_trait::async_trait]
impl ActiveModelSelector for ConfigActiveSelector {
    async fn active(&self) -> Result<ActiveRoute, AgentError> {
        // Never fail the run because selection couldn't be read. Any error
        // (config store not initialized, transient read failure) degrades to the
        // on-device model, which is always available. `resolve()` holds the real
        // logic; this wrapper enforces the Local fallback.
        Ok(self.resolve().await.unwrap_or(ActiveRoute::Local))
    }
}

impl ConfigActiveSelector {
    async fn resolve(&self) -> Result<ActiveRoute, AgentError> {
        let config = crate::state::config()?;
        let selected = config
            .get_setting(SETTING_UAR_ACTIVE_PROVIDER)
            .await?
            .and_then(|value| value.as_str().map(str::to_owned));

        // Unset or the local id ⇒ on-device. This keeps first-run offline-capable
        // with no keys configured.
        let provider_id = match selected {
            Some(id) if id != LOCAL_PROVIDER_ID => id,
            _ => return Ok(ActiveRoute::Local),
        };

        // A cloud provider is selected: resolve its credentials and model. Any
        // gap (deleted provider, missing key, no model pref) falls back to Local
        // rather than failing the run — the picker can never strand the user.
        let Some(provider) = config.get_provider(&provider_id).await? else {
            return Ok(ActiveRoute::Local);
        };
        if !provider.enabled {
            return Ok(ActiveRoute::Local);
        }
        let Some(api_key) = provider
            .api_key_ref
            .as_deref()
            .map(crate::secrets::resolve_api_key)
            .transpose()?
        else {
            return Ok(ActiveRoute::Local);
        };
        let model_id = config
            .get_model_pref(SURFACE_CHAT, LANE_CLOUD)
            .await?
            .map(|pref| pref.model_id)
            .filter(|model_id| !model_id.is_empty());
        let Some(model_id) = model_id else {
            return Ok(ActiveRoute::Local);
        };

        Ok(ActiveRoute::Cloud {
            provider_id,
            model_id,
            base_url: provider.base_url,
            api_key,
        })
    }
}

const SURFACE_CHAT: &str = "chat";
const LANE_CLOUD: &str = "cloud";

/// The single `LlmDriver` the embedded UAR kernel holds for its lifetime, routing
/// each run to the on-device local model OR a configured cloud provider.
///
/// The embedded `RunManager` builds one driver at construction and reuses it for
/// every run (the per-run `LlmRequest` carries no model string), so backend
/// selection must happen *inside* the driver: on each `stream()` the selector is
/// consulted and the call is dispatched to the wrapped local bridge or a cloud
/// [`LiterLlmDriver`] (built lazily and cached per `(provider_id, model_id)`).
/// Both emit `UarNormalizedEvent`, so the downstream projection is identical.
pub struct UarRoutingDriver {
    local: Arc<UarLocalInferenceDriverBridge>,
    selector: Arc<dyn ActiveModelSelector>,
    cloud_cache: tokio::sync::RwLock<HashMap<(String, String), Arc<LiterLlmDriver>>>,
}

impl UarRoutingDriver {
    /// Wrap the on-device bridge with a routing layer driven by `selector`.
    pub fn new(
        local: Arc<UarLocalInferenceDriverBridge>,
        selector: Arc<dyn ActiveModelSelector>,
    ) -> Self {
        Self {
            local,
            selector,
            cloud_cache: tokio::sync::RwLock::new(HashMap::new()),
        }
    }

    /// Drop all cached cloud drivers. Called when provider config changes so a
    /// re-keyed provider is rebuilt on its next run.
    pub async fn invalidate_cloud_cache(&self) {
        self.cloud_cache.write().await.clear();
    }

    /// Get or build the cloud driver for a `(provider_id, model_id)` selection.
    /// Reuses UAR's own `LiterLlmDriver` + `build_client_config`, so this is the
    /// exact cloud path the service UAR uses — with no submodule change.
    async fn cloud_driver(
        &self,
        provider_id: &str,
        model_id: &str,
        base_url: Option<String>,
        api_key: String,
    ) -> anyhow::Result<Arc<LiterLlmDriver>> {
        let key = (provider_id.to_owned(), model_id.to_owned());
        if let Some(driver) = self.cloud_cache.read().await.get(&key) {
            return Ok(Arc::clone(driver));
        }
        let model = format!("{provider_id}/{model_id}");
        // LlmConfig has no Default derive but every field has a serde default, so
        // build it from a minimal JSON object and override the three fields we set.
        let mut llm_config: LlmConfig = serde_json::from_value(serde_json::json!({}))
            .map_err(|error| anyhow::anyhow!("construct LlmConfig: {error}"))?;
        llm_config.model = model.clone();
        llm_config.api_key = Some(api_key);
        llm_config.base_url = base_url;
        let client_config = build_client_config(&llm_config);
        let driver = Arc::new(LiterLlmDriver::new(client_config, model, None)?);
        self.cloud_cache
            .write()
            .await
            .insert(key, Arc::clone(&driver));
        Ok(driver)
    }
}

#[async_trait::async_trait]
impl LlmDriver for UarRoutingDriver {
    async fn stream(
        &self,
        request: LlmRequest,
    ) -> anyhow::Result<
        std::pin::Pin<Box<dyn futures::Stream<Item = anyhow::Result<UarNormalizedEvent>> + Send>>,
    > {
        match self
            .selector
            .active()
            .await
            .map_err(|error| anyhow::anyhow!("resolve active model: {error}"))?
        {
            ActiveRoute::Local => self.local.stream(request).await,
            ActiveRoute::Cloud {
                provider_id,
                model_id,
                base_url,
                api_key,
            } => {
                let driver = self
                    .cloud_driver(&provider_id, &model_id, base_url, api_key)
                    .await?;
                driver.stream(request).await
            }
        }
    }
}

fn uar_llm_request_to_inference_request(
    request: LlmRequest,
) -> anyhow::Result<gen_ui_types::inference::InferenceRequest> {
    let mut messages = request.messages;
    if let Some(system_blocks) = request.anthropic_system {
        let has_system_message = messages.iter().any(|message| {
            message.get("role").and_then(serde_json::Value::as_str) == Some("system")
        });
        if !has_system_message && !system_blocks.is_empty() {
            messages.insert(
                0,
                serde_json::json!({
                    "role": "system",
                    "content": system_blocks,
                }),
            );
        }
    }
    let mut extra_params = request.extra_params;
    if let Some(thinking) = request.thinking_config {
        let extra = extra_params.get_or_insert_with(|| serde_json::json!({}));
        let object = extra.as_object_mut().ok_or_else(|| {
            anyhow::anyhow!("UAR local-provider extra_params must be a JSON object")
        })?;
        object.insert("thinking".into(), serde_json::to_value(thinking)?);
    }
    uar_llm_payload_to_inference_request(&UarLlmRequestPayload {
        messages,
        tools: request.tools,
        extra_params,
    })
    .map_err(anyhow::Error::msg)
}

#[derive(Debug)]
struct PendingUarTool {
    call_index: usize,
    id: String,
    name: String,
    arguments: String,
}

struct UarNativeEventAdapter {
    request_id: String,
    stream_started: bool,
    stream_finished: bool,
    next_call_index: usize,
    next_citation_index: usize,
    pending_tools: Vec<PendingUarTool>,
    completed_tool_names: Vec<(String, String)>,
}

impl UarNativeEventAdapter {
    fn new(request_id: String) -> Self {
        Self {
            request_id,
            stream_started: false,
            stream_finished: false,
            next_call_index: 0,
            next_citation_index: 0,
            pending_tools: Vec::new(),
            completed_tool_names: Vec::new(),
        }
    }

    fn ingest(&mut self, event: StreamEvent) -> Vec<UarNormalizedEvent> {
        let mut output = Vec::new();
        if !self.stream_started {
            self.stream_started = true;
            output.push(UarNormalizedEvent::StreamStart {
                request_id: self.request_id.clone(),
            });
        }

        match event {
            StreamEvent::MessageStart => {}
            StreamEvent::TextDelta { delta, .. } => {
                output.push(UarNormalizedEvent::MessageDelta { text: delta });
            }
            StreamEvent::ThinkingDelta { delta, .. } => {
                output.push(UarNormalizedEvent::ThinkingDelta { text: delta });
            }
            StreamEvent::ToolCallStarted { id, name } => {
                let call_index = self.next_call_index;
                self.next_call_index += 1;
                self.pending_tools.push(PendingUarTool {
                    call_index,
                    id: id.clone(),
                    name: name.clone(),
                    arguments: String::new(),
                });
                output.push(UarNormalizedEvent::ToolCallDelta {
                    call_index,
                    id: Some(id),
                    name: Some(name),
                    arguments_delta: None,
                });
            }
            StreamEvent::ToolCallDelta { id, delta } => {
                if let Some(tool) = self.pending_tools.iter_mut().find(|tool| tool.id == id) {
                    tool.arguments.push_str(&delta);
                    output.push(UarNormalizedEvent::ToolCallDelta {
                        call_index: tool.call_index,
                        id: Some(id),
                        name: None,
                        arguments_delta: Some(delta),
                    });
                } else {
                    output.push(UarNormalizedEvent::Error {
                        message: format!("tool argument delta arrived before start for '{id}'"),
                        code: Some("knowme_local_tool_state_missing".into()),
                    });
                }
            }
            StreamEvent::ToolCallComplete { id } => {
                if let Some(index) = self.pending_tools.iter().position(|tool| tool.id == id) {
                    let tool = self.pending_tools.remove(index);
                    self.completed_tool_names
                        .push((tool.id.clone(), tool.name.clone()));
                    output.push(UarNormalizedEvent::ToolCallComplete {
                        call_index: tool.call_index,
                        id: tool.id,
                        name: tool.name,
                        arguments_json: tool.arguments,
                    });
                } else {
                    output.push(UarNormalizedEvent::Error {
                        message: format!("tool completion arrived before start for '{id}'"),
                        code: Some("knowme_local_tool_state_missing".into()),
                    });
                }
            }
            StreamEvent::ToolCallReady {
                id,
                name,
                input_json,
            } => {
                let call_index = self
                    .pending_tools
                    .iter()
                    .position(|tool| tool.id == id)
                    .map(|index| self.pending_tools.remove(index).call_index)
                    .unwrap_or_else(|| {
                        let index = self.next_call_index;
                        self.next_call_index += 1;
                        index
                    });
                self.completed_tool_names.push((id.clone(), name.clone()));
                output.push(UarNormalizedEvent::ToolCallComplete {
                    call_index,
                    id,
                    name,
                    arguments_json: input_json,
                });
            }
            StreamEvent::ToolResult {
                tool_use_id,
                output_json,
                is_error,
            } => {
                let name = self
                    .completed_tool_names
                    .iter()
                    .find(|(id, _)| id == &tool_use_id)
                    .map(|(_, name)| name.clone())
                    .unwrap_or_else(|| "unknown_tool".into());
                output.push(UarNormalizedEvent::ToolResult {
                    id: tool_use_id,
                    name,
                    content: output_json,
                    success: !is_error,
                });
            }
            StreamEvent::SkillActivation { name, status } => {
                output.push(UarNormalizedEvent::SkillActivation { name, status });
            }
            StreamEvent::Citation { source, quote } => {
                output.push(self.citation(source, quote));
            }
            StreamEvent::Citations { citations } => {
                output.extend(
                    citations
                        .into_iter()
                        .map(|citation| self.citation(citation.source, citation.quote)),
                );
            }
            StreamEvent::Memory {
                operation,
                key,
                value,
            } => {
                output.push(UarNormalizedEvent::MemoryUpdate {
                    key,
                    value: value.unwrap_or_default(),
                    operation,
                });
            }
            StreamEvent::Custom {
                source,
                event_name,
                payload,
            } => {
                output.push(UarNormalizedEvent::Custom {
                    source,
                    event_name,
                    payload,
                });
            }
            StreamEvent::Error { message } => {
                output.push(UarNormalizedEvent::Error {
                    message,
                    code: Some("knowme_local_provider_error".into()),
                });
            }
            StreamEvent::Cancelled { message } => {
                output.push(UarNormalizedEvent::Error {
                    message,
                    code: Some("knowme_local_cancelled".into()),
                });
            }
            StreamEvent::Done => {
                output.extend(self.finish());
            }
        }
        output
    }

    fn citation(&mut self, source: String, quote: String) -> UarNormalizedEvent {
        let index = self.next_citation_index;
        self.next_citation_index += 1;
        UarNormalizedEvent::CitationAdded(UarCitation {
            index,
            url: source.clone(),
            title: Some(source),
            snippet: (!quote.is_empty()).then_some(quote),
        })
    }

    fn finish(&mut self) -> Vec<UarNormalizedEvent> {
        if self.stream_finished {
            return Vec::new();
        }
        self.stream_finished = true;
        let mut output = Vec::new();
        if !self.stream_started {
            self.stream_started = true;
            output.push(UarNormalizedEvent::StreamStart {
                request_id: self.request_id.clone(),
            });
        }
        output.extend(
            self.pending_tools
                .drain(..)
                .map(|tool| UarNormalizedEvent::Error {
                    message: format!(
                        "local provider ended before tool '{}' ({}) completed",
                        tool.name, tool.id
                    ),
                    code: Some("knowme_local_tool_incomplete".into()),
                }),
        );
        output.push(UarNormalizedEvent::Done);
        output
    }
}

impl UarRuntimeStatus {
    pub fn current() -> Self {
        let embedded_mobile = cfg!(any(target_os = "ios", target_os = "android"));
        Self {
            authority: "universal-agent-runtime".into(),
            target_mode: default_target_mode(),
            current_state: if embedded_mobile {
                UarIntegrationState::EmbeddedKernel
            } else {
                UarIntegrationState::ProjectionOnly
            },
            local_provider_role: "knowme_local_provider_backend".into(),
            mobile_sidecar_allowed: false,
            local_provider_bridge_available: true,
            service_runtime_available: service_runtime_available(),
            service_artifact_response_available: service_artifact_response_available(),
            embedded_kernel_available: embedded_mobile,
            embedded_kernel_blocker: (!embedded_mobile).then(|| {
                "This target did not register an in-process UAR kernel; use its service facade."
                    .into()
            }),
            submodule_path: "rust/vendor/universal-agent-runtime".into(),
            submodule_commit: option_env!("KNOWME_UAR_SUBMODULE_COMMIT").map(str::to_owned),
        }
    }
}

#[async_trait::async_trait]
pub trait UarRuntimeFacade: Send + Sync {
    fn status(&self) -> UarRuntimeStatus;
    async fn start_run(&self, request: UarRunRequest) -> Result<UarProjectedRun, AgentError>;
    async fn cancel_run(&self, run_id: &str) -> Result<bool, AgentError>;
    async fn submit_artifact_response(
        &self,
        run_id: &str,
        response: UarArtifactResponse,
    ) -> Result<UarProjectedRun, AgentError>;
    async fn submit_a2ui_action(
        &self,
        run_id: &str,
        action: UarA2uiAction,
    ) -> Result<UarProjectedRun, AgentError>;
    async fn promote_a2ui_component(
        &self,
        title: &str,
        source: &str,
        description: Option<&str>,
    ) -> Result<serde_json::Value, AgentError>;
}

/// Complete transport-free UAR kernel hosted inside the mobile process.
///
/// KnowMe supplies the platform inference driver and model metadata; UAR owns
/// agent resolution, skills, tools, policy, run lifecycle, and canonical
/// events. No listener, sidecar, loopback socket, or network discovery is used.
#[derive(Clone)]
pub struct UarEmbeddedRuntime {
    runtime: universal_agent_runtime_sdk::Runtime,
    /// The single routing driver the runtime executes every run through. Retained
    /// so provider-admin mutations can invalidate its cached cloud drivers when a
    /// provider is re-keyed or removed (see [`Self::invalidate_routing_cache`]).
    routing: Arc<UarRoutingDriver>,
}

impl std::fmt::Debug for UarEmbeddedRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UarEmbeddedRuntime")
            .field("mode", &"embedded_library")
            .finish_non_exhaustive()
    }
}

impl UarEmbeddedRuntime {
    /// Build a ready embedded runtime around the platform's local model.
    ///
    /// `storage_dir` is where the runtime's SurrealDB instance lives. Everything
    /// the embedded runtime owns — agents, skills, policies, settings, memory,
    /// knowledge, conversations, run state, A2UI — is written there and survives
    /// process death. Passing `None` selects an in-memory store, which is
    /// correct ONLY for tests: an agent the user configured would otherwise be
    /// gone on next launch.
    ///
    /// SurrealDB rather than a plain key-value store because the runtime needs
    /// graph and vector queries over that state, not just persistence.
    pub async fn build_local(
        provider: Arc<dyn InferenceProvider>,
        model_spec: LocalModelSpec,
        provider_config: UarProviderConfig,
        storage_dir: Option<&std::path::Path>,
    ) -> Result<Self, AgentError> {
        // Wrap the on-device bridge in the routing driver so a single embedded
        // run can execute against the local model OR a configured cloud provider,
        // chosen at stream() time from the persisted active selection. Cloud
        // providers are additionally registered into the runtime's provider
        // registry after build (see `register_cloud_providers`) so they list and
        // resolve for policy/telemetry; execution switching is the driver's job.
        let local_bridge = Arc::new(UarLocalInferenceDriverBridge::new(provider, model_spec));
        let routing = Arc::new(UarRoutingDriver::new(
            local_bridge,
            Arc::new(ConfigActiveSelector),
        ));
        let driver: Arc<dyn LlmDriver> = routing.clone();
        let persistence = Self::open_persistence(storage_dir).await?;
        let runtime = universal_agent_runtime_sdk::Runtime::builder()
            .local_provider(driver, provider_config)
            .persistence(persistence)
            .seed_defaults(true)
            .build()
            .await
            .map_err(|error| AgentError::Config(format!("embedded UAR: {error}")))?;
        Ok(Self { runtime, routing })
    }

    /// Open the runtime's persistence layer.
    ///
    /// A durable SurrealDB store when `storage_dir` is supplied; an in-memory
    /// store only when it is not.
    ///
    /// There is deliberately NO fallback from a failed durable open to memory.
    /// Falling back would leave the app running and apparently healthy while
    /// silently discarding every agent, policy and conversation on exit — the
    /// exact defect this change removes, but harder to notice because nothing
    /// looks broken. A store that cannot be opened is a boot failure.
    async fn open_persistence(
        storage_dir: Option<&std::path::Path>,
    ) -> Result<Arc<dyn universal_agent_runtime::uar::persistence::PersistenceLayer>, AgentError>
    {
        let Some(dir) = storage_dir else {
            return Ok(Arc::new(
                universal_agent_runtime::uar::persistence::providers::memory::InMemoryProvider::new(
                ),
            ));
        };

        tokio::fs::create_dir_all(dir).await.map_err(|error| {
            AgentError::Config(format!(
                "embedded UAR: cannot create persistence directory {}: {error}",
                dir.display()
            ))
        })?;

        // `surrealkv://` names the engine UAR actually pins
        // (surrealdb features = ["kv-surrealkv"]). KnowMe's graph store spells
        // the same engine `rocksdb://`, which the provider silently rewrites to
        // surrealkv:// — a legacy spelling that misleads readers into thinking
        // RocksDB is involved. It is not; use the honest scheme here.
        // Embedded endpoints perform no authentication, so no credentials.
        let endpoint = format!("surrealkv://{}", dir.display());
        let provider =
            universal_agent_runtime::uar::persistence::providers::surreal::SurrealDbProvider::new(
                &endpoint, None, None, None, None,
            )
            .await
            .map_err(|error| {
                AgentError::Config(format!(
                    "embedded UAR: cannot open persistent store at {}: {error}",
                    dir.display()
                ))
            })?;
        tracing::info!(path = %dir.display(), "embedded UAR persistence opened (SurrealDB)");
        Ok(Arc::new(provider))
    }

    /// Register (or replace) a cloud provider in the embedded runtime's provider
    /// registry so it lists and resolves for policy/telemetry. Execution switching
    /// between local and cloud remains the routing driver's job; this only keeps
    /// the registry's view of configured providers in sync with the config store.
    pub async fn register_cloud_provider(
        &self,
        config: UarProviderConfig,
    ) -> Result<(), AgentError> {
        self.runtime
            .provider_registry()
            .register_custom_provider(config)
            .await
            .map_err(|error| AgentError::Config(format!("register cloud provider: {error}")))
    }

    /// Drop every cached cloud driver held by the routing driver. Call after any
    /// provider credential/base-URL/selection change so the next run rebuilds the
    /// driver from the updated config rather than reusing a stale client.
    pub async fn invalidate_routing_cache(&self) {
        self.routing.invalidate_cloud_cache().await;
    }

    // ---- embedded admin surface ----
    //
    // Thin JSON facades over the SDK Runtime's in-process settings + agent admin
    // (added in the embedded-run-policy-and-admin-surface UAR change), so the
    // KnowMe `control_plane` embedded branch can administer settings and agents
    // with no HTTP service. Each serializes the typed SDK result to the
    // `serde_json::Value` shape `control_plane`'s HTTP path already returns.

    /// Read a typed setting (e.g. `run_policy.global`); `None` when unset.
    pub async fn get_setting(&self, key: &str) -> Result<Option<serde_json::Value>, AgentError> {
        self.runtime
            .get_setting(key)
            .await
            .map_err(|error| AgentError::Config(format!("get setting {key}: {error}")))
    }

    /// Write a typed setting value.
    pub async fn set_setting(&self, key: &str, value: serde_json::Value) -> Result<(), AgentError> {
        self.runtime
            .set_setting(key, value)
            .await
            .map_err(|error| AgentError::Config(format!("set setting {key}: {error}")))
    }

    /// Current setting values + registered setting types, as `(values, types)`
    /// JSON — the shape the admin snapshot exposes.
    pub async fn settings_snapshot(
        &self,
    ) -> Result<(serde_json::Value, serde_json::Value), AgentError> {
        let snapshot = self
            .runtime
            .settings_snapshot()
            .await
            .map_err(|error| AgentError::Config(format!("settings snapshot: {error}")))?;
        let values = serde_json::to_value(&snapshot.values)
            .map_err(|error| AgentError::Config(format!("serialize settings: {error}")))?;
        let types = serde_json::to_value(&snapshot.types)
            .map_err(|error| AgentError::Config(format!("serialize setting types: {error}")))?;
        Ok((values, types))
    }

    /// All agent definitions as JSON.
    pub async fn list_agents(&self) -> Result<serde_json::Value, AgentError> {
        let agents = self
            .runtime
            .list_agents()
            .await
            .map_err(|error| AgentError::Config(format!("list agents: {error}")))?;
        serde_json::to_value(&agents)
            .map_err(|error| AgentError::Config(format!("serialize agents: {error}")))
    }

    /// List every skill the embedded runtime holds.
    ///
    /// Previously the embedded control plane returned a hardcoded `[]` here,
    /// which a client could not distinguish from "no skills configured". The
    /// SDK now serves this in-process, so the real catalog is available with no
    /// network and no service URL.
    pub async fn list_skills(&self) -> Result<serde_json::Value, AgentError> {
        let skills = self
            .runtime
            .list_skills()
            .await
            .map_err(|error| AgentError::Config(format!("list skills: {error}")))?;
        serde_json::to_value(&skills)
            .map_err(|error| AgentError::Config(format!("serialize skills: {error}")))
    }

    /// List every knowledge base.
    pub async fn list_knowledge_bases(&self) -> Result<serde_json::Value, AgentError> {
        let bases = self
            .runtime
            .list_knowledge_bases()
            .await
            .map_err(|error| AgentError::Config(format!("list knowledge bases: {error}")))?;
        serde_json::to_value(&bases)
            .map_err(|error| AgentError::Config(format!("serialize knowledge bases: {error}")))
    }

    /// List configured MCP servers, read from the database.
    ///
    /// Config files seed the store once at boot; the store is authoritative
    /// afterwards, so a change made through this surface takes effect without a
    /// restart or a file-polling loop.
    pub async fn list_mcp_servers(&self) -> Result<serde_json::Value, AgentError> {
        let servers = self.runtime.list_mcp_servers().await;
        serde_json::to_value(&servers)
            .map_err(|error| AgentError::Config(format!("serialize mcp servers: {error}")))
    }

    /// Save an MCP server.
    ///
    /// Returns the runtime's `SaveResult` verbatim rather than `()`, because the
    /// outcome is not always "applied": an edit to an ALREADY-CONNECTED server is
    /// stored durably but deferred to the next session so in-flight tool calls
    /// are not interrupted. Callers must surface that instead of implying the
    /// change took effect.
    pub async fn save_mcp_server(
        &self,
        name: &str,
        server: &serde_json::Value,
    ) -> Result<serde_json::Value, AgentError> {
        let parsed = serde_json::from_value(server.clone())
            .map_err(|error| AgentError::Config(format!("parse mcp server: {error}")))?;
        let result = self
            .runtime
            .save_mcp_server(name.to_string(), parsed)
            .await
            .map_err(|error| AgentError::Config(format!("save mcp server: {error}")))?;
        serde_json::to_value(&result)
            .map_err(|error| AgentError::Config(format!("serialize save result: {error}")))
    }

    /// Remove an MCP server from the store and the live registry.
    pub async fn delete_mcp_server(&self, name: &str) -> Result<serde_json::Value, AgentError> {
        let result = self
            .runtime
            .delete_mcp_server(name)
            .await
            .map_err(|error| AgentError::Config(format!("delete mcp server: {error}")))?;
        serde_json::to_value(&result)
            .map_err(|error| AgentError::Config(format!("serialize delete result: {error}")))
    }

    /// Create or replace an agent from its JSON artifact.
    pub async fn upsert_agent(&self, agent: &serde_json::Value) -> Result<(), AgentError> {
        let artifact = serde_json::from_value(agent.clone())
            .map_err(|error| AgentError::Config(format!("parse agent artifact: {error}")))?;
        self.runtime
            .upsert_agent(artifact)
            .await
            .map_err(|error| AgentError::Config(format!("upsert agent: {error}")))
    }

    /// Delete an agent by id.
    pub async fn delete_agent(&self, id: &str) -> Result<(), AgentError> {
        self.runtime
            .delete_agent(id)
            .await
            .map_err(|error| AgentError::Config(format!("delete agent {id}: {error}")))
    }

    /// Load the per-conversation run policy, or `null` when none is saved.
    ///
    /// Returns the **bare** `RunPolicy`, not the `ConversationPolicyRecord`
    /// wrapper that persistence stores it in. The wrapper is a storage detail:
    /// `save_conversation_policy` below accepts a bare policy, the TypeScript
    /// binding declares `Promise<RunPolicy | null>`, and Flutter assigns the
    /// result straight to its policy map. Leaking `{conversation_id, policy,
    /// updated_at}` here broke all of them — and silently, because every
    /// `RunPolicy` field is `#[serde(default)]`, so feeding the wrapper back to
    /// save deserialized to defaults instead of erroring and quietly discarded
    /// the user's selection.
    pub async fn conversation_policy(
        &self,
        conversation_id: &str,
    ) -> Result<serde_json::Value, AgentError> {
        let record = self
            .runtime
            .get_conversation_policy(conversation_id)
            .await
            .map_err(|error| AgentError::Config(format!("load conversation policy: {error}")))?;
        match record {
            Some(record) => serde_json::to_value(&record.policy).map_err(|error| {
                AgentError::Config(format!("serialize conversation policy: {error}"))
            }),
            None => Ok(serde_json::Value::Null),
        }
    }

    /// Save or replace the per-conversation run policy; echoes the stored policy
    /// in the same bare shape [`Self::conversation_policy`] returns, so a client
    /// can assign the result back over what it sent.
    pub async fn save_conversation_policy(
        &self,
        conversation_id: &str,
        policy: &serde_json::Value,
    ) -> Result<serde_json::Value, AgentError> {
        let run_policy = serde_json::from_value(policy.clone())
            .map_err(|error| AgentError::Config(format!("parse run policy: {error}")))?;
        let record = self
            .runtime
            .save_conversation_policy(conversation_id, run_policy)
            .await
            .map_err(|error| AgentError::Config(format!("save conversation policy: {error}")))?;
        serde_json::to_value(&record.policy)
            .map_err(|error| AgentError::Config(format!("serialize conversation policy: {error}")))
    }

    /// Delete the per-conversation run policy.
    pub async fn delete_conversation_policy(
        &self,
        conversation_id: &str,
    ) -> Result<(), AgentError> {
        self.runtime
            .delete_conversation_policy(conversation_id)
            .await
            .map_err(|error| AgentError::Config(format!("delete conversation policy: {error}")))
    }

    /// Resolve the effective configuration (agent + requested + effective policy)
    /// for a conversation, mirroring the service path's effective-config shape.
    pub async fn effective_config(
        &self,
        conversation_id: &str,
    ) -> Result<serde_json::Value, AgentError> {
        let config = self.runtime.effective_config(conversation_id).await;
        Ok(serde_json::json!({
            "sessionId": conversation_id,
            "agent": config.agent,
            "requestedPolicy": config.requested_policy,
            "effectivePolicy": config.effective_policy,
        }))
    }

    fn projected_run(
        &self,
        run_id: String,
        history: Vec<universal_agent_runtime::uar::runtime::manager::StreamEvent>,
        receiver: tokio::sync::broadcast::Receiver<
            universal_agent_runtime::uar::runtime::manager::StreamEvent,
        >,
    ) -> UarProjectedRun {
        UarProjectedRun {
            run_id: run_id.clone(),
            events: project_embedded_stream(run_id, history, receiver),
        }
    }

    async fn subscribe_projected(&self, run_id: String) -> Result<UarProjectedRun, AgentError> {
        // Subscribe first, then snapshot history. The projector skips receiver
        // events already represented by the snapshot, closing the run-start
        // race without duplicating any semantic event.
        let receiver = self
            .runtime
            .subscribe(&run_id)
            .await
            .map_err(embedded_sdk_error)?;
        let history = self
            .runtime
            .history_since(&run_id, None)
            .await
            .map_err(embedded_sdk_error)?;
        Ok(self.projected_run(run_id, history, receiver))
    }
}

#[async_trait::async_trait]
impl UarRuntimeFacade for UarEmbeddedRuntime {
    fn status(&self) -> UarRuntimeStatus {
        UarRuntimeStatus {
            target_mode: UarDeploymentMode::EmbeddedLibrary,
            current_state: UarIntegrationState::EmbeddedKernel,
            embedded_kernel_available: true,
            embedded_kernel_blocker: None,
            ..UarRuntimeStatus::current()
        }
    }

    async fn start_run(&self, request: UarRunRequest) -> Result<UarProjectedRun, AgentError> {
        let conversation_id = request.session_id;
        let seed_history: Vec<universal_agent_runtime_sdk::SeedMessage> = request
            .history
            .into_iter()
            .map(|message| universal_agent_runtime_sdk::SeedMessage {
                role: message.role,
                content: message.content,
                tool_call_id: message.tool_call_id,
            })
            .collect();
        let run_id = if let Some(artifact) = request.artifact {
            let artifact = serde_json::from_value(artifact)
                .map_err(|error| AgentError::Config(format!("UAR agent artifact: {error}")))?;
            self.runtime
                .start_run_with_history(
                    artifact,
                    request.input,
                    conversation_id,
                    None,
                    Vec::new(),
                    seed_history,
                )
                .await
        } else {
            // Resolve the built-in agent, then start with seeded history so the
            // agentless (orchestrator) path also repopulates a cold session.
            let agent_id = request.agent_id.as_deref().unwrap_or("orchestrator-agent");
            let artifact = self
                .runtime
                .get_agent(agent_id)
                .await
                .map_err(embedded_sdk_error)?
                .ok_or_else(|| {
                    AgentError::Config(format!("UAR agent '{agent_id}' was not found"))
                })?;
            self.runtime
                .start_run_with_history(
                    artifact,
                    request.input,
                    conversation_id,
                    None,
                    Vec::new(),
                    seed_history,
                )
                .await
        };
        self.subscribe_projected(run_id).await
    }

    async fn cancel_run(&self, run_id: &str) -> Result<bool, AgentError> {
        Ok(self.runtime.cancel_run(run_id).await)
    }

    async fn submit_artifact_response(
        &self,
        run_id: &str,
        response: UarArtifactResponse,
    ) -> Result<UarProjectedRun, AgentError> {
        let continuation = self
            .runtime
            .continue_with_a2ui_interaction(
                run_id,
                serde_json::json!({
                    "artifactId": response.artifact_id,
                    "response": response.response,
                }),
            )
            .await
            .map_err(embedded_sdk_error)?;
        self.subscribe_projected(continuation).await
    }

    async fn submit_a2ui_action(
        &self,
        run_id: &str,
        action: UarA2uiAction,
    ) -> Result<UarProjectedRun, AgentError> {
        let continuation = self
            .runtime
            .continue_with_a2ui_interaction(
                run_id,
                serde_json::json!({
                    "surfaceId": action.surface_id,
                    "name": action.name,
                    "sourceComponentId": action.source_component_id,
                    "timestamp": action.timestamp,
                    "context": action.context,
                    "a2uiClientDataModel": action.client_data_model,
                }),
            )
            .await
            .map_err(embedded_sdk_error)?;
        self.subscribe_projected(continuation).await
    }

    async fn promote_a2ui_component(
        &self,
        title: &str,
        source: &str,
        description: Option<&str>,
    ) -> Result<serde_json::Value, AgentError> {
        use universal_agent_runtime::uar::a2ui::schema::{ArtifactSchema, ArtifactType};

        let parsed = serde_json::from_str::<serde_json::Value>(source).unwrap_or_else(|_| {
            serde_json::json!({
                "type": "object",
                "description": "Host-provided A2UI component source",
                "properties": { "source": { "type": "string" } }
            })
        });
        let slug = title
            .chars()
            .map(|character| {
                if character.is_ascii_alphanumeric() {
                    character.to_ascii_lowercase()
                } else {
                    '-'
                }
            })
            .collect::<String>()
            .trim_matches('-')
            .to_owned();
        let schema = ArtifactSchema {
            schema_id: format!("knowme/{slug}"),
            title: title.to_owned(),
            description: description
                .unwrap_or("User-promoted A2UI component")
                .to_owned(),
            artifact_type: ArtifactType::Display,
            json_schema: parsed,
            render_hint: Some("inline".into()),
            builtin: false,
        };
        self.runtime.a2ui_registry().register(schema.clone()).await;
        serde_json::to_value(schema)
            .map_err(|error| AgentError::Interaction(format!("A2UI component: {error}")))
    }
}

fn embedded_sdk_error(error: universal_agent_runtime_sdk::Error) -> AgentError {
    AgentError::Interaction(format!("embedded UAR: {error}"))
}

fn project_embedded_stream(
    run_id: String,
    history: Vec<universal_agent_runtime::uar::runtime::manager::StreamEvent>,
    receiver: tokio::sync::broadcast::Receiver<
        universal_agent_runtime::uar::runtime::manager::StreamEvent,
    >,
) -> BoxStream<'static, Result<A2uiEvent, AgentError>> {
    let last_history_id = history.last().map(|event| event.id).unwrap_or(0);
    let pending = history.into_iter().collect::<VecDeque<_>>();
    futures::stream::unfold(
        (
            receiver,
            UarEventProjector::new(run_id),
            pending,
            VecDeque::<A2uiEvent>::new(),
            last_history_id,
        ),
        |(mut receiver, mut projector, mut source, mut projected, last_history_id)| async move {
            loop {
                if let Some(event) = projected.pop_front() {
                    return Some((
                        Ok(event),
                        (receiver, projector, source, projected, last_history_id),
                    ));
                }
                let event = if let Some(event) = source.pop_front() {
                    event
                } else {
                    match receiver.recv().await {
                        Ok(event) if event.id > last_history_id => event,
                        Ok(_) => continue,
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                            return Some((
                                Err(AgentError::Interaction(format!(
                                    "embedded UAR event stream lagged by {skipped} events"
                                ))),
                                (receiver, projector, source, projected, last_history_id),
                            ));
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => return None,
                    }
                };
                if let Some(payload) = embedded_stream_payload(&event) {
                    projected.extend(projector.ingest_agui_payload(&payload));
                }
            }
        },
    )
    .boxed()
}

/// Encode one embedded-runtime stream event as an AG-UI frame.
///
/// Delegates to UAR's canonical transport-free encoder (ADR 0012) rather than
/// maintaining a parallel `NormalizedEvent -> AG-UI` map here: the runtime owns
/// the wire contract, and a second hand-rolled copy silently drifts from it.
/// Returns `None` for events that produce no wire frame.
fn embedded_stream_payload(
    event: &universal_agent_runtime::uar::runtime::manager::StreamEvent,
) -> Option<serde_json::Value> {
    use universal_agent_runtime::uar::api::adapters::{
        enrich_agui_spec_payload, to_agui_spec_event,
    };

    let (event_name, payload) = to_agui_spec_event(&event.event)?;
    // Ordinal 8 matches the SSE path so live and embedded frames share the same
    // `eventId`/`sequence` derivation.
    Some(enrich_agui_spec_payload(
        event_name,
        payload,
        &event.id.to_string(),
        8,
    ))
}

#[derive(Debug, Clone)]
pub struct UarServiceRuntime {
    client: universal_agent_runtime_sdk::Client,
}

impl UarServiceRuntime {
    pub fn new(base_url: impl AsRef<str>, api_key: Option<String>) -> Result<Self, AgentError> {
        let client = universal_agent_runtime_sdk::Client::new(base_url.as_ref())
            .map_err(|error| AgentError::Client(format!("UAR client: {error}")))?;
        let client = match api_key {
            Some(api_key) if !api_key.is_empty() => client.with_api_key(api_key),
            _ => client,
        };
        Ok(Self { client })
    }
}

#[async_trait::async_trait]
impl UarRuntimeFacade for UarServiceRuntime {
    fn status(&self) -> UarRuntimeStatus {
        UarRuntimeStatus {
            target_mode: UarDeploymentMode::HttpService,
            current_state: UarIntegrationState::ServiceRuntime,
            ..UarRuntimeStatus::current()
        }
    }

    async fn start_run(&self, request: UarRunRequest) -> Result<UarProjectedRun, AgentError> {
        if request.artifact.is_none() {
            let session_id = request
                .session_id
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
            let stream = self
                .client
                .chat()
                .stream(universal_agent_runtime_sdk::ChatCompletionRequest {
                    messages: vec![universal_agent_runtime_sdk::ChatMessage::text(
                        "user",
                        request.input,
                    )],
                    session_id: Some(session_id.clone()),
                    agent_id: request.agent_id,
                    run_policy: request.run_policy,
                    stream_mode: Some("agui".to_string()),
                    ..Default::default()
                })
                .await
                .map_err(uar_sdk_error)?;
            return Ok(UarProjectedRun {
                run_id: session_id.clone(),
                events: project_sdk_stream(session_id, stream),
            });
        }
        let run = self
            .client
            .runs()
            .create(universal_agent_runtime_sdk::CreateRunRequest {
                artifact: request.artifact.expect("artifact checked above"),
                input: request.input,
                session_id: request.session_id,
            })
            .await
            .map_err(uar_sdk_error)?;
        let stream = self
            .client
            .runs()
            .stream(&run.run_id, None)
            .await
            .map_err(uar_sdk_error)?;
        Ok(UarProjectedRun {
            run_id: run.run_id.clone(),
            events: project_sdk_stream(run.run_id, stream),
        })
    }

    async fn cancel_run(&self, run_id: &str) -> Result<bool, AgentError> {
        self.client
            .runs()
            .cancel_session(run_id)
            .await
            .map(|response| response.cancelled)
            .map_err(uar_sdk_error)
    }

    async fn submit_artifact_response(
        &self,
        run_id: &str,
        response: UarArtifactResponse,
    ) -> Result<UarProjectedRun, AgentError> {
        let ack = self
            .client
            .runs()
            .submit_artifact_response(
                run_id,
                universal_agent_runtime_sdk::ArtifactResponseRequest {
                    artifact_id: response.artifact_id,
                    response: response.response,
                },
            )
            .await
            .map_err(uar_sdk_error)?;
        let stream = self
            .client
            .runs()
            .stream(&ack.continuation_run_id, None)
            .await
            .map_err(uar_sdk_error)?;
        Ok(UarProjectedRun {
            run_id: ack.continuation_run_id.clone(),
            events: project_sdk_stream(ack.continuation_run_id, stream),
        })
    }

    async fn submit_a2ui_action(
        &self,
        run_id: &str,
        action: UarA2uiAction,
    ) -> Result<UarProjectedRun, AgentError> {
        let ack = self
            .client
            .runs()
            .submit_a2ui_action(
                run_id,
                universal_agent_runtime_sdk::A2uiActionRequest {
                    surface_id: action.surface_id,
                    name: action.name,
                    source_component_id: action.source_component_id,
                    timestamp: action.timestamp,
                    context: action.context,
                    a2ui_client_data_model: action.client_data_model,
                },
            )
            .await
            .map_err(uar_sdk_error)?;
        let stream = self
            .client
            .runs()
            .stream(&ack.continuation_run_id, None)
            .await
            .map_err(uar_sdk_error)?;
        Ok(UarProjectedRun {
            run_id: ack.continuation_run_id.clone(),
            events: project_sdk_stream(ack.continuation_run_id, stream),
        })
    }

    async fn promote_a2ui_component(
        &self,
        title: &str,
        source: &str,
        description: Option<&str>,
    ) -> Result<serde_json::Value, AgentError> {
        self.client
            .a2ui()
            .promote_component(title, source, description)
            .await
            .map_err(uar_sdk_error)
    }
}

fn uar_sdk_error(error: universal_agent_runtime_sdk::Error) -> AgentError {
    AgentError::Client(format!("UAR runtime: {error}"))
}

fn project_sdk_stream(
    run_id: String,
    stream: universal_agent_runtime_sdk::EventStream,
) -> BoxStream<'static, Result<A2uiEvent, AgentError>> {
    let projector = UarEventProjector::new(run_id);
    futures::stream::unfold(
        (stream, projector, Vec::<A2uiEvent>::new()),
        |(mut stream, mut projector, mut pending)| async move {
            loop {
                if let Some(event) = pending.pop() {
                    return Some((Ok(event), (stream, projector, pending)));
                }
                match stream.next().await {
                    Some(Ok(event)) => {
                        let payload = sdk_stream_payload_to_agui(event);
                        pending = projector.ingest_agui_payload(&payload);
                        pending.reverse();
                    }
                    Some(Err(error)) => {
                        return Some((Err(uar_sdk_error(error)), (stream, projector, pending)));
                    }
                    None => return None,
                }
            }
        },
    )
    .boxed()
}

fn sdk_stream_payload_to_agui(
    event: universal_agent_runtime_sdk::StreamEvent,
) -> serde_json::Value {
    let universal_agent_runtime_sdk::StreamEvent {
        id,
        event: event_type,
        data,
    } = event;
    match data {
        serde_json::Value::Object(mut object) => {
            if !object.contains_key("type") {
                if let Some(event_type) = event_type {
                    object.insert("type".into(), serde_json::Value::String(event_type));
                }
            }
            if !object.contains_key("eventId") {
                if let Some(id) = id {
                    object.insert("eventId".into(), serde_json::Value::String(id));
                }
            }
            serde_json::Value::Object(object)
        }
        other => serde_json::json!({
            "type": event_type.unwrap_or_else(|| "RAW".into()),
            "eventId": id,
            "value": other,
        }),
    }
}

#[cfg(any(target_os = "ios", target_os = "android"))]
const fn default_target_mode() -> UarDeploymentMode {
    UarDeploymentMode::EmbeddedLibrary
}

#[cfg(not(any(target_os = "ios", target_os = "android")))]
const fn default_target_mode() -> UarDeploymentMode {
    UarDeploymentMode::HttpService
}

#[cfg(not(any(target_os = "ios", target_os = "android")))]
const fn service_runtime_available() -> bool {
    true
}

#[cfg(any(target_os = "ios", target_os = "android"))]
const fn service_runtime_available() -> bool {
    true
}

#[cfg(not(any(target_os = "ios", target_os = "android")))]
const fn service_artifact_response_available() -> bool {
    true
}

#[cfg(any(target_os = "ios", target_os = "android"))]
const fn service_artifact_response_available() -> bool {
    true
}

/// Stateful projector from UAR AG-UI payloads into KnowMe A2UI events.
///
/// Keep one projector per run so streamed tool-call argument chunks share the
/// same `A2uiAdapter` buffer.
pub struct UarEventProjector {
    adapter: A2uiAdapter,
    /// Agent id observed on `RUN_STARTED`, used to back-fill provenance when the
    /// `effective_run_policy` artifact itself omits `agentId`.
    last_agent_id: Option<String>,
}

impl UarEventProjector {
    pub fn new(run_id: impl Into<String>) -> Self {
        Self {
            adapter: A2uiAdapter::new(run_id),
            last_agent_id: None,
        }
    }

    pub fn ingest_agui_payload(&mut self, payload: &serde_json::Value) -> Vec<A2uiEvent> {
        // Remember the RunStart agent id before normalization drops it.
        if payload.get("type").and_then(serde_json::Value::as_str) == Some("RUN_STARTED") {
            if let Some(agent_id) = payload
                .get("input")
                .and_then(|input| input.get("agentId"))
                .and_then(serde_json::Value::as_str)
            {
                self.last_agent_id = Some(agent_id.to_owned());
            }
        }

        let event = normalize_uar_agui_event(payload);
        let mut events = self.adapter.ingest(&event);
        for event in &mut events {
            back_fill_provenance_agent_id(event, self.last_agent_id.as_deref());
        }
        events
    }
}

/// Fill a null `agentId` in a provenance `ContextTelemetry` payload from the
/// RunStart-observed agent id. No-op for any other event or a non-null agentId.
fn back_fill_provenance_agent_id(event: &mut A2uiEvent, fallback_agent_id: Option<&str>) {
    let Some(fallback) = fallback_agent_id else {
        return;
    };
    let A2uiEvent::ContextTelemetry { payload, .. } = event else {
        return;
    };
    let Some(provenance) = payload.get_mut("provenance") else {
        return;
    };
    let missing = provenance
        .get("agentId")
        .map(serde_json::Value::is_null)
        .unwrap_or(true);
    if missing {
        if let Some(object) = provenance.as_object_mut() {
            object.insert(
                "agentId".into(),
                serde_json::Value::String(fallback.to_owned()),
            );
        }
    }
}

/// Baseline UAR capability status. Runtime-specific status can layer configured
/// service or embedded-kernel readiness on top of this baseline.
pub fn runtime_status() -> UarRuntimeStatus {
    UarRuntimeStatus::current()
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;
    use gen_ui_protocol::provenance_from_effective_run_policy;
    use gen_ui_types::content_block::ContentBlock;
    use gen_ui_types::inference::{InferenceRequest, InferenceToolResult};
    use gen_ui_types::{CoreError, CoreResult};
    use parking_lot::Mutex;

    #[derive(Default)]
    struct RecordingInferenceProvider {
        loads: Mutex<Vec<LocalModelSpec>>,
        requests: Mutex<Vec<InferenceRequest>>,
        scripted_events: Mutex<Option<Vec<StreamEvent>>>,
    }

    impl RecordingInferenceProvider {
        fn with_events(events: Vec<StreamEvent>) -> Self {
            Self {
                scripted_events: Mutex::new(Some(events)),
                ..Self::default()
            }
        }
    }

    #[async_trait::async_trait]
    impl InferenceProvider for RecordingInferenceProvider {
        async fn load(&self, spec: &LocalModelSpec) -> CoreResult<()> {
            self.loads.lock().push(spec.clone());
            Ok(())
        }

        async fn generate_request(
            &self,
            request: &InferenceRequest,
        ) -> CoreResult<BoxStream<'static, StreamEvent>> {
            self.requests.lock().push(request.clone());
            let events = self.scripted_events.lock().take().unwrap_or_else(|| {
                vec![
                    StreamEvent::MessageStart,
                    StreamEvent::TextDelta {
                        index: 0,
                        delta: "before".into(),
                    },
                    StreamEvent::ToolCallStarted {
                        id: "tool-1".into(),
                        name: "knowme_diagnostics__local_runtime_status".into(),
                    },
                    StreamEvent::ToolCallDelta {
                        id: "tool-1".into(),
                        delta: r#"{"marker":"#.into(),
                    },
                    StreamEvent::ToolCallDelta {
                        id: "tool-1".into(),
                        delta: r#""KMUAR"}"#.into(),
                    },
                    StreamEvent::ToolCallComplete {
                        id: "tool-1".into(),
                    },
                    StreamEvent::Done,
                ]
            });
            Ok(futures::stream::iter(events).boxed())
        }

        async fn continue_with_tool_results(
            &self,
            _results: &[InferenceToolResult],
        ) -> CoreResult<BoxStream<'static, StreamEvent>> {
            Err(CoreError::Terminal("unused in test".into()))
        }

        async fn unload(&self) -> CoreResult<()> {
            Ok(())
        }
    }

    #[test]
    fn mobile_sidecar_is_not_allowed_in_status_contract() {
        let status = runtime_status();
        assert_eq!(status.authority, "universal-agent-runtime");
        assert_eq!(status.current_state, UarIntegrationState::ProjectionOnly);
        assert!(!status.mobile_sidecar_allowed);
        assert!(status.local_provider_bridge_available);
        assert!(!status.embedded_kernel_available);
        assert!(status.embedded_kernel_blocker.is_some());
    }

    #[tokio::test]
    async fn android_litert_lm_is_a_credential_free_uar_default() {
        const PINNED_MODEL_ID: &str = "gemma-4-e2b-it-litertlm";
        let provider = android_litert_lm_provider_config(PINNED_MODEL_ID);

        assert_eq!(provider.id, ANDROID_LITERT_LM_PROVIDER_ID);
        assert!(provider.enabled);
        assert!(provider.api_key.is_none());
        assert!(provider.base_url.is_empty());
        assert_eq!(provider.default_model.as_deref(), Some(PINNED_MODEL_ID));
        assert_eq!(provider.models.len(), 1);

        let model = &provider.models[0];
        assert_eq!(model.id, PINNED_MODEL_ID);
        assert_eq!(
            model.context_window,
            Some(ANDROID_LITERT_LM_TRUSTED_CONTEXT_TOKENS)
        );
        assert_eq!(
            model.max_output_tokens,
            Some(ANDROID_LITERT_LM_MAX_OUTPUT_TOKENS)
        );
        assert!(!model.supports_vision);
        assert!(model.supports_tools);
        assert!(model.supports_reasoning);
        assert!(model.supports_structured_output);
        assert!(model.supports_streaming);
        assert!(model.enabled);

        let registry = Arc::new(universal_agent_runtime::llm::ProviderRegistry::new());
        registry
            .register_custom_provider(provider)
            .await
            .expect("Android local provider registers");
        registry
            .set_default(ANDROID_LITERT_LM_PROVIDER_ID)
            .await
            .expect("Android local provider becomes default");

        assert_eq!(
            registry.default_model().await,
            Some((
                ANDROID_LITERT_LM_PROVIDER_ID.to_string(),
                PINNED_MODEL_ID.to_string()
            ))
        );
        let resolved = registry
            .resolve_to_llm_config(ANDROID_LITERT_LM_PROVIDER_ID, "")
            .await
            .expect("default local route resolves without credentials");
        assert_eq!(
            resolved.model,
            format!("{ANDROID_LITERT_LM_PROVIDER_ID}/{PINNED_MODEL_ID}")
        );
        assert!(resolved.api_key.is_none());
        assert!(resolved.base_url.is_none());

        let routed = universal_agent_runtime::llm::router::ModelRouter::new(Arc::clone(&registry))
            .route(&universal_agent_runtime::llm::router::RouteRequirements {
                needs_tools: true,
                needs_reasoning: true,
                needs_structured_output: true,
                min_context: Some(ANDROID_LITERT_LM_TRUSTED_CONTEXT_TOKENS.into()),
                preferred_provider: Some(ANDROID_LITERT_LM_PROVIDER_ID.to_string()),
                ..Default::default()
            })
            .await;
        assert_eq!(
            routed.as_deref(),
            Some("knowme-local-litert-lm/gemma-4-e2b-it-litertlm")
        );
    }

    #[tokio::test]
    async fn ios_qwen35_mlx_is_a_credential_free_text_uar_default() {
        const PINNED_MODEL_ID: &str = "qwen3.5-4b-mlx-4bit";
        let provider = ios_qwen35_mlx_provider_config(PINNED_MODEL_ID);

        assert_eq!(provider.id, IOS_MLX_PROVIDER_ID);
        assert!(provider.enabled);
        assert!(provider.api_key.is_none());
        assert!(provider.base_url.is_empty());
        assert_eq!(provider.default_model.as_deref(), Some(PINNED_MODEL_ID));
        assert_eq!(provider.models.len(), 1);

        let model = &provider.models[0];
        assert_eq!(model.id, PINNED_MODEL_ID);
        assert_eq!(
            model.context_window,
            Some(IOS_QWEN35_TRUSTED_CONTEXT_TOKENS)
        );
        assert_eq!(model.max_output_tokens, Some(IOS_QWEN35_MAX_OUTPUT_TOKENS));
        // Text-only today (no Qwen3.5 vision processor in MLXVLM's Swift registry).
        assert!(!model.supports_vision);
        assert!(model.supports_tools);
        assert!(model.supports_reasoning);
        assert!(model.supports_structured_output);
        assert!(model.supports_streaming);
        assert!(model.enabled);

        let registry = Arc::new(universal_agent_runtime::llm::ProviderRegistry::new());
        registry
            .register_custom_provider(provider)
            .await
            .expect("iOS local provider registers");
        registry
            .set_default(IOS_MLX_PROVIDER_ID)
            .await
            .expect("iOS local provider becomes default");

        assert_eq!(
            registry.default_model().await,
            Some((IOS_MLX_PROVIDER_ID.to_string(), PINNED_MODEL_ID.to_string()))
        );
        let resolved = registry
            .resolve_to_llm_config(IOS_MLX_PROVIDER_ID, "")
            .await
            .expect("default local route resolves without credentials");
        assert_eq!(
            resolved.model,
            format!("{IOS_MLX_PROVIDER_ID}/{PINNED_MODEL_ID}")
        );
        assert!(resolved.api_key.is_none());
        assert!(resolved.base_url.is_none());

        let routed = universal_agent_runtime::llm::router::ModelRouter::new(Arc::clone(&registry))
            .route(&universal_agent_runtime::llm::router::RouteRequirements {
                needs_vision: false,
                needs_tools: true,
                needs_reasoning: true,
                needs_structured_output: true,
                min_context: Some(IOS_QWEN35_TRUSTED_CONTEXT_TOKENS.into()),
                preferred_provider: Some(IOS_MLX_PROVIDER_ID.to_string()),
                ..Default::default()
            })
            .await;
        assert_eq!(
            routed.as_deref(),
            Some("knowme-local-mlx/qwen3.5-4b-mlx-4bit")
        );
    }

    #[tokio::test]
    async fn local_inference_driver_bridge_calls_provider_and_returns_uar_events() {
        let provider = Arc::new(RecordingInferenceProvider::default());
        let bridge = UarLocalInferenceDriverBridge::new(
            provider.clone(),
            LocalModelSpec {
                model: "gemma-4-e2b-it-litertlm".into(),
                context_len: Some(8192),
            },
        );

        let events: Vec<_> = bridge
            .stream_payload(UarLlmRequestPayload {
                messages: vec![
                    serde_json::json!({"role": "system", "content": "Be concise."}),
                    serde_json::json!({"role": "user", "content": "Check status."}),
                ],
                tools: vec![serde_json::json!({
                    "type": "function",
                    "function": {
                        "name": "knowme_diagnostics__local_runtime_status",
                        "description": "Status.",
                        "parameters": {"type": "object", "properties": {}}
                    }
                })],
                extra_params: Some(serde_json::json!({
                    "temperature": 0.2,
                    "max_tokens": 64
                })),
            })
            .await
            .unwrap()
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .map(Result::unwrap)
            .collect();

        assert_eq!(provider.loads.lock()[0].model, "gemma-4-e2b-it-litertlm");
        let request = provider.requests.lock()[0].clone();
        assert_eq!(request.messages.len(), 2);
        assert_eq!(
            request.tools[0].name,
            "knowme_diagnostics__local_runtime_status"
        );
        assert_eq!(request.generation_settings.temperature, 0.2);
        assert_eq!(request.generation_settings.max_tokens, 64);

        assert!(events.iter().any(|event| {
            event.get("type").and_then(serde_json::Value::as_str) == Some("message.delta")
                && event
                    .pointer("/data/text")
                    .and_then(serde_json::Value::as_str)
                    == Some("before")
        }));
        let complete = events
            .iter()
            .find(|event| {
                event.get("type").and_then(serde_json::Value::as_str) == Some("tool_call.complete")
            })
            .expect("tool completion exists");
        assert_eq!(
            complete
                .pointer("/data/name")
                .and_then(serde_json::Value::as_str),
            Some("knowme_diagnostics__local_runtime_status")
        );
        assert_eq!(
            complete
                .pointer("/data/arguments_json")
                .and_then(serde_json::Value::as_str),
            Some(r#"{"marker":"KMUAR"}"#)
        );
    }

    #[tokio::test]
    async fn local_inference_bridge_is_a_native_uar_llm_driver() {
        let provider = Arc::new(RecordingInferenceProvider::default());
        let bridge = UarLocalInferenceDriverBridge::new(
            provider.clone(),
            LocalModelSpec {
                model: "gemma-4-e2b-it-litertlm".into(),
                context_len: Some(8192),
            },
        );

        let events = bridge
            .stream(LlmRequest {
                messages: vec![serde_json::json!({
                    "role": "user",
                    "content": "Check status."
                })],
                tools: vec![serde_json::json!({
                    "type": "function",
                    "function": {
                        "name": "knowme_diagnostics__local_runtime_status",
                        "description": "Status.",
                        "parameters": {"type": "object", "properties": {}}
                    }
                })],
                cache_strategy: None,
                thinking_config: None,
                anthropic_system: Some(vec![serde_json::json!({
                    "type": "text",
                    "text": "Be concise."
                })]),
                extra_params: Some(serde_json::json!({
                    "temperature": 0.2,
                    "max_tokens": 64
                })),
            })
            .await
            .expect("native UAR driver starts")
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .map(Result::unwrap)
            .collect::<Vec<_>>();

        let request = provider.requests.lock()[0].clone();
        assert_eq!(
            request.messages[0].role,
            gen_ui_types::inference::InferenceRole::System
        );
        assert_eq!(request.messages[0].content, "Be concise.");
        assert_eq!(request.messages[1].content, "Check status.");
        assert_eq!(request.generation_settings.temperature, 0.2);
        assert_eq!(request.generation_settings.max_tokens, 64);

        assert!(matches!(events[0], UarNormalizedEvent::StreamStart { .. }));
        assert!(events.iter().any(|event| matches!(
            event,
            UarNormalizedEvent::MessageDelta { text } if text == "before"
        )));
        assert!(events.iter().any(|event| matches!(
            event,
            UarNormalizedEvent::ToolCallComplete {
                id,
                name,
                arguments_json,
                ..
            } if id == "tool-1"
                && name == "knowme_diagnostics__local_runtime_status"
                && arguments_json == r#"{"marker":"KMUAR"}"#
        )));
        assert!(matches!(events.last(), Some(UarNormalizedEvent::Done)));
    }

    #[tokio::test]
    async fn embedded_uar_runs_the_seeded_orchestrator_on_the_local_provider() {
        let provider = Arc::new(RecordingInferenceProvider::with_events(vec![
            StreamEvent::MessageStart,
            StreamEvent::ThinkingDelta {
                index: 0,
                delta: "checking".into(),
            },
            StreamEvent::TextDelta {
                index: 0,
                delta: "Offline UAR is ready.".into(),
            },
            StreamEvent::Done,
        ]));
        let runtime = UarEmbeddedRuntime::build_local(
            provider.clone(),
            LocalModelSpec {
                model: "gemma-4-e2b-it-litertlm".into(),
                context_len: Some(8_192),
            },
            android_litert_lm_provider_config("gemma-4-e2b-it-litertlm"),
            // In-memory on purpose: this asserts the runtime builds without a
            // transport, not that it persists. Durability has its own test.
            None,
        )
        .await
        .expect("embedded UAR builds without a transport");

        let projected = runtime
            .start_run(UarRunRequest {
                artifact: None,
                input: "Confirm offline operation.".into(),
                session_id: Some("offline-test-conversation".into()),
                agent_id: None,
                run_policy: None,
                history: Vec::new(),
            })
            .await
            .expect("seeded orchestrator run starts");
        assert_eq!(
            runtime.status().current_state,
            UarIntegrationState::EmbeddedKernel
        );

        let mut events = projected.events;
        let mut saw_thinking = false;
        let mut saw_answer = false;
        let mut saw_finished = false;
        while !saw_finished {
            let event = tokio::time::timeout(std::time::Duration::from_secs(5), events.next())
                .await
                .expect("embedded run event arrives")
                .expect("embedded run stream remains open")
                .expect("embedded event projects");
            match event {
                A2uiEvent::Block {
                    block: ContentBlock::Thinking { .. },
                    ..
                } => saw_thinking = true,
                A2uiEvent::Block {
                    block: ContentBlock::Text { text },
                    ..
                } if text.contains("Offline UAR is ready") => saw_answer = true,
                A2uiEvent::RunFinished { .. } => saw_finished = true,
                _ => {}
            }
        }

        assert!(saw_thinking, "reasoning must survive UAR projection");
        assert!(saw_answer, "answer text must survive UAR projection");
        assert_eq!(provider.requests.lock().len(), 1);
    }

    #[tokio::test]
    async fn native_driver_preserves_every_local_semantic_event() {
        let provider = Arc::new(RecordingInferenceProvider::with_events(vec![
            StreamEvent::ThinkingDelta {
                index: 0,
                delta: "reason".into(),
            },
            StreamEvent::SkillActivation {
                name: "meeting_summary".into(),
                status: "activated".into(),
            },
            StreamEvent::Citations {
                citations: vec![
                    gen_ui_types::events::StreamCitation {
                        source: "handbook.pdf".into(),
                        quote: "Bring receipts.".into(),
                    },
                    gen_ui_types::events::StreamCitation {
                        source: "notes.md".into(),
                        quote: "Follow up Friday.".into(),
                    },
                ],
            },
            StreamEvent::Memory {
                operation: "stored".into(),
                key: "office.format".into(),
                value: Some("concise".into()),
            },
            StreamEvent::Custom {
                source: "litert-lm".into(),
                event_name: "provider.metric".into(),
                payload: serde_json::json!({"ttftMs": 42}),
            },
            StreamEvent::Cancelled {
                message: "Stopped by user".into(),
            },
            StreamEvent::Done,
        ]));
        let bridge = UarLocalInferenceDriverBridge::new(
            provider,
            LocalModelSpec {
                model: "local".into(),
                context_len: None,
            },
        );

        let events = bridge
            .stream(LlmRequest {
                messages: vec![serde_json::json!({"role": "user", "content": "hello"})],
                tools: Vec::new(),
                cache_strategy: None,
                thinking_config: None,
                anthropic_system: None,
                extra_params: None,
            })
            .await
            .unwrap()
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .map(Result::unwrap)
            .collect::<Vec<_>>();

        assert!(matches!(events[0], UarNormalizedEvent::StreamStart { .. }));
        assert!(matches!(
            &events[1],
            UarNormalizedEvent::ThinkingDelta { text } if text == "reason"
        ));
        assert!(events.iter().any(|event| matches!(
            event,
            UarNormalizedEvent::SkillActivation { name, status }
                if name == "meeting_summary" && status == "activated"
        )));
        let citations = events
            .iter()
            .filter_map(|event| match event {
                UarNormalizedEvent::CitationAdded(citation) => Some(citation),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(citations.len(), 2);
        assert_eq!(citations[0].index, 0);
        assert_eq!(citations[1].index, 1);
        assert!(events.iter().any(|event| matches!(
            event,
            UarNormalizedEvent::MemoryUpdate {
                operation,
                key,
                value,
            } if operation == "stored" && key == "office.format" && value == "concise"
        )));
        assert!(events.iter().any(|event| matches!(
            event,
            UarNormalizedEvent::Custom {
                source,
                event_name,
                payload,
            } if source == "litert-lm"
                && event_name == "provider.metric"
                && payload["ttftMs"] == 42
        )));
        assert!(events.iter().any(|event| matches!(
            event,
            UarNormalizedEvent::Error { code: Some(code), .. }
                if code == "knowme_local_cancelled"
        )));
        assert!(matches!(events.last(), Some(UarNormalizedEvent::Done)));
    }

    #[test]
    fn projector_maps_uar_skill_activation_to_visible_block() {
        let mut projector = UarEventProjector::new("run-1");
        let output = projector.ingest_agui_payload(&serde_json::json!({
            "type": "CUSTOM",
            "profile": "uar.agui/1",
            "name": "uar.skill.activated",
            "value": {
                "skill": {"id": "meeting_summary"},
                "selectionMethod": "runtime"
            },
            "runId": "run-1"
        }));

        assert!(matches!(
            &output[0],
            A2uiEvent::Block {
                block: ContentBlock::Skill { name, status },
                ..
            } if name == "meeting_summary" && status == "activated"
        ));
    }

    #[test]
    fn projector_maps_uar_citation_to_visible_block() {
        let mut projector = UarEventProjector::new("run-1");
        let output = projector.ingest_agui_payload(&serde_json::json!({
            "type": "CUSTOM",
            "profile": "uar.agui/1",
            "name": "uar.citation.added",
            "value": {
                "source": "policy.pdf",
                "quote": "Approvals are required before mutating external state."
            },
            "runId": "run-1"
        }));

        assert!(matches!(
            &output[0],
            A2uiEvent::Block {
                block: ContentBlock::Citation { source, quote },
                ..
            } if source == "policy.pdf"
                && quote == "Approvals are required before mutating external state."
        ));
    }

    #[test]
    fn projector_maps_uar_memory_to_visible_block() {
        let mut projector = UarEventProjector::new("run-1");
        let output = projector.ingest_agui_payload(&serde_json::json!({
            "type": "CUSTOM",
            "profile": "uar.agui/1",
            "name": "uar.memory.updated",
            "value": {
                "operation": "stored",
                "key": "office.default_format",
                "value": "concise markdown"
            },
            "runId": "run-1"
        }));

        assert!(matches!(
            &output[0],
            A2uiEvent::Block {
                block: ContentBlock::Memory {
                    operation,
                    key,
                    value
                },
                ..
            } if operation == "stored"
                && key == "office.default_format"
                && value.as_deref() == Some("concise markdown")
        ));
    }

    #[test]
    fn projector_maps_uar_a2ui_artifact_to_a2ui_block() {
        let mut projector = UarEventProjector::new("run-1");
        let output = projector.ingest_agui_payload(&serde_json::json!({
            "type": "agui.artifact_input_request",
            "kind": "artifact_input_request",
            "request_id": "run-1",
            "artifact_id": "artifact-1",
            "artifact_type": "a2ui/confirm",
            "title": "Approve action",
            "content": "{\"type\":\"object\"}",
            "metadata": {"catalogId": "urn:uar:a2ui:catalog:1"}
        }));

        assert!(matches!(
            &output[0],
            A2uiEvent::Block {
                block: ContentBlock::Artifact { kind, content, .. },
                ..
            } if kind == "a2ui"
                && content.contains("agui.artifact_input_request")
                && content.contains("artifact-1")
        ));
    }

    #[test]
    fn projector_maps_effective_run_policy_artifact_to_provenance_telemetry() {
        let mut projector = UarEventProjector::new("run-1");
        let output = projector.ingest_agui_payload(&serde_json::json!({
            "type": "CUSTOM",
            "profile": "uar.agui/1",
            "name": "uar.artifact.available",
            "value": {
                "artifactId": "artifact-1",
                "artifactType": "effective_run_policy",
                "content": {
                    "agent_id": "office-assistant",
                    "model": { "provider_id": "anthropic", "model_id": "claude-opus-4-8" }
                },
                "sourceRunId": "run-1"
            },
            "runId": "run-1"
        }));

        assert_eq!(
            output.len(),
            1,
            "policy artifact must not also emit a block"
        );
        let A2uiEvent::ContextTelemetry { payload, .. } = &output[0] else {
            panic!("expected ContextTelemetry, got {:?}", output[0]);
        };
        let provenance = &payload["provenance"];
        assert_eq!(provenance["agentId"], "office-assistant");
        assert_eq!(provenance["providerId"], "anthropic");
        assert_eq!(provenance["modelId"], "claude-opus-4-8");
    }

    #[test]
    fn projector_back_fills_provenance_agent_id_from_run_started() {
        let mut projector = UarEventProjector::new("run-1");
        // RunStart carries the agent id; the policy artifact omits it.
        let _ = projector.ingest_agui_payload(&serde_json::json!({
            "type": "RUN_STARTED",
            "profile": "uar.agui/1",
            "runId": "run-1",
            "threadId": "run-1",
            "input": { "agentId": "office-assistant" }
        }));
        let output = projector.ingest_agui_payload(&serde_json::json!({
            "type": "CUSTOM",
            "profile": "uar.agui/1",
            "name": "uar.artifact.available",
            "value": {
                "artifactType": "effective_run_policy",
                "content": {
                    "agent_id": null,
                    "model": { "provider_id": "on-device", "model_id": "gemma-3n" }
                }
            },
            "runId": "run-1"
        }));

        let telemetry = output
            .iter()
            .find_map(|event| match event {
                A2uiEvent::ContextTelemetry { payload, .. } => Some(payload),
                _ => None,
            })
            .expect("expected a ContextTelemetry event");
        assert_eq!(telemetry["provenance"]["agentId"], "office-assistant");
        assert_eq!(telemetry["provenance"]["providerId"], "on-device");
        assert_eq!(telemetry["provenance"]["modelId"], "gemma-3n");
    }

    #[test]
    fn provenance_helper_parses_string_encoded_policy() {
        let content = serde_json::json!(
            "{\"agent_id\":\"a\",\"model\":{\"provider_id\":\"p\",\"model_id\":\"m\"}}"
        );
        let provenance = provenance_from_effective_run_policy(&content);
        assert_eq!(provenance["agentId"], "a");
        assert_eq!(provenance["providerId"], "p");
        assert_eq!(provenance["modelId"], "m");
    }

    #[test]
    fn provenance_helper_nulls_missing_parts() {
        let content = serde_json::json!({ "agent_id": "a" });
        let provenance = provenance_from_effective_run_policy(&content);
        assert_eq!(provenance["agentId"], "a");
        assert!(provenance["providerId"].is_null());
        assert!(provenance["modelId"].is_null());
    }

    #[test]
    fn projector_keeps_tool_stream_state_across_uar_events() {
        let mut projector = UarEventProjector::new("run-1");
        assert!(projector
            .ingest_agui_payload(&serde_json::json!({
                "type": "TOOL_CALL_START",
                "toolCallId": "tool-1",
                "name": "docs.search"
            }))
            .is_empty());
        assert!(projector
            .ingest_agui_payload(&serde_json::json!({
                "type": "TOOL_CALL_ARGS",
                "toolCallId": "tool-1",
                "delta": "{\"q\":\"rag\"}"
            }))
            .is_empty());
        let output = projector.ingest_agui_payload(&serde_json::json!({
            "type": "TOOL_CALL_END",
            "toolCallId": "tool-1"
        }));

        assert!(matches!(
            &output[0],
            A2uiEvent::Block {
                block: ContentBlock::ToolUse { id, name, input_json },
                ..
            } if id == "tool-1" && name == "docs.search" && input_json == "{\"q\":\"rag\"}"
        ));
    }

    #[test]
    #[cfg(not(any(target_os = "ios", target_os = "android")))]
    fn sdk_stream_payload_injects_sse_event_type_when_missing() {
        let payload = sdk_stream_payload_to_agui(universal_agent_runtime_sdk::StreamEvent {
            id: Some("cursor-1".into()),
            event: Some("REASONING_MESSAGE_CONTENT".into()),
            data: serde_json::json!({"delta": "reason"}),
        });
        assert_eq!(
            payload.get("type").and_then(serde_json::Value::as_str),
            Some("REASONING_MESSAGE_CONTENT")
        );
        assert_eq!(
            payload.get("eventId").and_then(serde_json::Value::as_str),
            Some("cursor-1")
        );
    }

    #[tokio::test]
    #[cfg(not(any(target_os = "ios", target_os = "android")))]
    async fn sdk_stream_projection_preserves_uar_order() {
        let source = futures::stream::iter([
            Ok(universal_agent_runtime_sdk::StreamEvent {
                id: Some("1".into()),
                event: Some("RUN_STARTED".into()),
                data: serde_json::json!({}),
            }),
            Ok(universal_agent_runtime_sdk::StreamEvent {
                id: Some("2".into()),
                event: Some("TEXT_MESSAGE_CONTENT".into()),
                data: serde_json::json!({"delta": "hello"}),
            }),
            Ok(universal_agent_runtime_sdk::StreamEvent {
                id: Some("3".into()),
                event: Some("RUN_FINISHED".into()),
                data: serde_json::json!({}),
            }),
        ])
        .boxed();

        let output: Vec<_> = project_sdk_stream("run-1".into(), source)
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .map(Result::unwrap)
            .collect();
        assert!(matches!(&output[0], A2uiEvent::RunStarted { .. }));
        assert!(matches!(
            &output[1],
            A2uiEvent::Block {
                block: ContentBlock::Text { text },
                ..
            } if text == "hello"
        ));
        assert!(matches!(&output[2], A2uiEvent::RunFinished { .. }));
    }

    #[tokio::test]
    #[cfg(not(any(target_os = "ios", target_os = "android")))]
    async fn sdk_stream_projection_maps_direct_agui_service_events() {
        let source = futures::stream::iter([
            Ok(universal_agent_runtime_sdk::StreamEvent {
                id: Some("1".into()),
                event: Some("agui.message.delta".into()),
                data: serde_json::json!({"delta": {"text": "hello"}}),
            }),
            Ok(universal_agent_runtime_sdk::StreamEvent {
                id: Some("2".into()),
                event: Some("agui.citation.added".into()),
                data: serde_json::json!({
                    "citation": {
                        "url": "https://example.test",
                        "title": "Example",
                        "snippet": "Cited text."
                    }
                }),
            }),
            Ok(universal_agent_runtime_sdk::StreamEvent {
                id: Some("3".into()),
                event: Some("agui.rag_citations".into()),
                data: serde_json::json!({
                    "citations": [{
                        "marker": 1,
                        "chunk_id": "chunk-a",
                        "document_name": "handbook.pdf",
                        "snippet": "Bring receipts."
                    }]
                }),
            }),
            Ok(universal_agent_runtime_sdk::StreamEvent {
                id: Some("4".into()),
                event: Some("agui.tool_call.complete".into()),
                data: serde_json::json!({
                    "id": "call-1",
                    "name": "docs.search",
                    "arguments_json": "{\"q\":\"uar\"}"
                }),
            }),
            Ok(universal_agent_runtime_sdk::StreamEvent {
                id: Some("5".into()),
                event: Some("agui.cancelled".into()),
                data: serde_json::json!({"message": "User stopped"}),
            }),
        ])
        .boxed();

        let output: Vec<_> = project_sdk_stream("run-1".into(), source)
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .map(Result::unwrap)
            .collect();

        assert!(output.iter().any(|event| matches!(
            event,
            A2uiEvent::Block {
                block: ContentBlock::Text { text },
                ..
            } if text == "hello"
        )));
        assert!(output.iter().any(|event| matches!(
            event,
            A2uiEvent::Block {
                block: ContentBlock::Citation { source, quote },
                ..
            } if source == "Example" && quote == "Cited text."
        )));
        assert!(output.iter().any(|event| matches!(
            event,
            A2uiEvent::Block {
                block: ContentBlock::Citation { source, quote },
                ..
            } if source == "[1] handbook.pdf" && quote == "Bring receipts."
        )));
        assert!(output.iter().any(|event| matches!(
            event,
            A2uiEvent::Block {
                block: ContentBlock::ToolUse { id, name, input_json },
                ..
            } if id == "call-1" && name == "docs.search" && input_json == "{\"q\":\"uar\"}"
        )));
        assert!(output.iter().any(|event| matches!(
            event,
            A2uiEvent::RunCancelled { message, .. } if message == "User stopped"
        )));
    }

    #[tokio::test]
    #[cfg(not(any(target_os = "ios", target_os = "android")))]
    async fn service_runtime_exercises_create_stream_and_artifact_response_over_http() {
        use std::sync::Arc;
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        #[derive(Default)]
        struct Captures {
            create_body: Mutex<String>,
            artifact_body: Mutex<String>,
            cancel_request_line: Mutex<String>,
        }

        async fn read_request(stream: &mut tokio::net::TcpStream) -> (String, String) {
            let mut buffer = Vec::new();
            let mut chunk = [0_u8; 1024];
            loop {
                let read = stream.read(&mut chunk).await.unwrap();
                if read == 0 {
                    break;
                }
                buffer.extend_from_slice(&chunk[..read]);
                if buffer.windows(4).any(|window| window == b"\r\n\r\n") {
                    break;
                }
            }
            let header_end = buffer
                .windows(4)
                .position(|window| window == b"\r\n\r\n")
                .map(|index| index + 4)
                .unwrap_or(buffer.len());
            let headers = String::from_utf8_lossy(&buffer[..header_end]).to_string();
            let content_length = headers
                .lines()
                .find_map(|line| {
                    let (name, value) = line.split_once(':')?;
                    name.eq_ignore_ascii_case("content-length")
                        .then(|| value.trim().parse::<usize>().ok())
                        .flatten()
                })
                .unwrap_or(0);
            while buffer.len() < header_end + content_length {
                let read = stream.read(&mut chunk).await.unwrap();
                if read == 0 {
                    break;
                }
                buffer.extend_from_slice(&chunk[..read]);
            }
            let request_line = headers.lines().next().unwrap_or_default().to_string();
            let body = String::from_utf8_lossy(
                &buffer[header_end..buffer.len().min(header_end + content_length)],
            )
            .to_string();
            (request_line, body)
        }

        async fn respond(
            stream: &mut tokio::net::TcpStream,
            status: &str,
            content_type: &str,
            body: &str,
        ) {
            let response = format!(
                "HTTP/1.1 {status}\r\ncontent-type: {content_type}\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                body.len()
            );
            stream.write_all(response.as_bytes()).await.unwrap();
        }

        let captures = Arc::new(Captures::default());
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server_captures = Arc::clone(&captures);
        let server = tokio::spawn(async move {
            loop {
                let Ok((mut stream, _)) = listener.accept().await else {
                    break;
                };
                let captures = Arc::clone(&server_captures);
                tokio::spawn(async move {
                    let (request_line, body) = read_request(&mut stream).await;
                    if request_line.starts_with("GET /health ") {
                        respond(
                            &mut stream,
                            "200 OK",
                            "application/json",
                            r#"{"status":"ok"}"#,
                        )
                        .await;
                    } else if request_line.starts_with("POST /api/uar/runs ") {
                        *captures.create_body.lock() = body;
                        respond(
                            &mut stream,
                            "200 OK",
                            "application/json",
                            r#"{"run_id":"uar-run-1","stream_url":"/api/uar/runs/uar-run-1/stream"}"#,
                        )
                        .await;
                    } else if request_line.starts_with("GET /api/uar/runs/uar-run-1/stream ") {
                        let sse = concat!(
                            "id: 1\n",
                            "event: agui.message.delta\n",
                            "data: {\"delta\":{\"text\":\"from fake UAR\"}}\n\n",
                            "id: 2\n",
                            "event: agui.done\n",
                            "data: {}\n\n"
                        );
                        respond(&mut stream, "200 OK", "text/event-stream", sse).await;
                    } else if request_line
                        .starts_with("POST /api/uar/runs/uar-run-1/artifact-response ")
                    {
                        *captures.artifact_body.lock() = body;
                        respond(
                            &mut stream,
                            "200 OK",
                            "application/json",
                            r#"{"run_id":"uar-run-1","artifact_id":"artifact-1","continuation_run_id":"uar-run-1","status":"accepted"}"#,
                        )
                        .await;
                    } else if request_line.starts_with("POST /api/uar/sessions/uar-run-1/cancel ") {
                        *captures.cancel_request_line.lock() = request_line;
                        respond(
                            &mut stream,
                            "200 OK",
                            "application/json",
                            r#"{"cancelled":true}"#,
                        )
                        .await;
                    } else {
                        respond(
                            &mut stream,
                            "404 Not Found",
                            "application/json",
                            r#"{"error":"not found"}"#,
                        )
                        .await;
                    }
                });
            }
        });

        let runtime = UarServiceRuntime::new(format!("http://{address}"), None).unwrap();
        let run = runtime
            .start_run(UarRunRequest {
                artifact: Some(serde_json::json!({"agent": "knowme"})),
                input: "hello UAR".into(),
                session_id: Some("session-1".into()),
                agent_id: None,
                run_policy: None,
                history: Vec::new(),
            })
            .await
            .unwrap();
        assert_eq!(run.run_id, "uar-run-1");

        let events: Vec<_> = run
            .events
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .map(Result::unwrap)
            .collect();
        assert!(events.iter().any(|event| matches!(
            event,
            A2uiEvent::Block {
                block: ContentBlock::Text { text },
                ..
            } if text == "from fake UAR"
        )));

        runtime
            .submit_artifact_response(
                "uar-run-1",
                UarArtifactResponse {
                    artifact_id: "artifact-1".into(),
                    response: serde_json::json!({"approved": true}),
                },
            )
            .await
            .unwrap();
        let cancelled = runtime.cancel_run("uar-run-1").await.unwrap();

        let create_body = captures.create_body.lock().clone();
        assert!(create_body.contains("\"input\":\"hello UAR\""));
        assert!(create_body.contains("\"session_id\":\"session-1\""));
        assert!(create_body.contains("\"agent\":\"knowme\""));
        let artifact_body = captures.artifact_body.lock().clone();
        assert!(artifact_body.contains("\"artifact_id\":\"artifact-1\""));
        assert!(artifact_body.contains("\"approved\":true"));
        assert!(cancelled);
        assert_eq!(
            captures.cancel_request_line.lock().as_str(),
            "POST /api/uar/sessions/uar-run-1/cancel HTTP/1.1"
        );

        server.abort();
    }

    // -------- routing driver (local ⊕ cloud selection) --------

    /// A scripted selector so the routing driver is testable without touching the
    /// config store or a live cloud endpoint.
    struct ScriptedSelector(ActiveRoute);

    #[async_trait::async_trait]
    impl ActiveModelSelector for ScriptedSelector {
        async fn active(&self) -> Result<ActiveRoute, AgentError> {
            Ok(self.0.clone())
        }
    }

    fn empty_llm_request() -> LlmRequest {
        LlmRequest {
            messages: vec![serde_json::json!({"role": "user", "content": "hi"})],
            tools: vec![],
            cache_strategy: None,
            thinking_config: None,
            anthropic_system: None,
            extra_params: None,
        }
    }

    #[tokio::test]
    async fn routing_driver_local_route_streams_from_the_on_device_bridge() {
        // Selector says Local ⇒ the request must reach the wrapped inference
        // provider (proving no cloud dispatch), and the on-device stream projects.
        let inference = Arc::new(RecordingInferenceProvider::with_events(vec![
            StreamEvent::MessageStart,
            StreamEvent::TextDelta {
                index: 0,
                delta: "on-device answer".into(),
            },
            StreamEvent::Done,
        ]));
        let bridge = Arc::new(UarLocalInferenceDriverBridge::new(
            inference.clone(),
            LocalModelSpec {
                model: "test-model".into(),
                context_len: None,
            },
        ));
        let driver = UarRoutingDriver::new(bridge, Arc::new(ScriptedSelector(ActiveRoute::Local)));

        let events: Vec<_> = driver
            .stream(empty_llm_request())
            .await
            .expect("local route streams")
            .collect()
            .await;

        // The on-device provider actually generated (Local was honored, not cloud).
        assert_eq!(inference.requests.lock().len(), 1);
        assert!(!events.is_empty());
        assert!(events.iter().all(|event| event.is_ok()));
    }

    #[tokio::test]
    async fn cloud_driver_cache_reuses_one_driver_per_provider_model() {
        // Two builds for the same (provider, model) return the same Arc; a
        // different model builds a distinct driver. Proves keyed caching without
        // needing a live endpoint (LiterLlmDriver::new only constructs a client).
        let bridge = Arc::new(UarLocalInferenceDriverBridge::new(
            Arc::new(RecordingInferenceProvider::default()),
            LocalModelSpec {
                model: "test-model".into(),
                context_len: None,
            },
        ));
        let driver = UarRoutingDriver::new(bridge, Arc::new(ScriptedSelector(ActiveRoute::Local)));

        let a = driver
            .cloud_driver("openai", "gpt-4o", None, "sk-test".into())
            .await
            .expect("build cloud driver a");
        let b = driver
            .cloud_driver("openai", "gpt-4o", None, "sk-test".into())
            .await
            .expect("build cloud driver b");
        assert!(Arc::ptr_eq(&a, &b), "same (provider,model) ⇒ cached driver");

        let c = driver
            .cloud_driver("openai", "gpt-4o-mini", None, "sk-test".into())
            .await
            .expect("build cloud driver c");
        assert!(
            !Arc::ptr_eq(&a, &c),
            "different model ⇒ distinct cached driver"
        );

        driver.invalidate_cloud_cache().await;
        let d = driver
            .cloud_driver("openai", "gpt-4o", None, "sk-test".into())
            .await
            .expect("rebuild after invalidate");
        assert!(
            !Arc::ptr_eq(&a, &d),
            "cache cleared ⇒ fresh driver after invalidate"
        );
    }

    /// D4 — the durability proof, and the whole point of this change.
    ///
    /// Writes state through one persistence layer, DROPS it entirely, opens a
    /// fresh one against the same directory, and reads the state back. If this
    /// fails, agents and conversations do not survive process death and the
    /// swap achieved nothing.
    #[tokio::test]
    async fn embedded_runtime_state_survives_a_dropped_store() {
        let temp = tempfile::tempdir().expect("tempdir");
        let dir = temp.path().join("uar-db");
        let session_id = "durability-probe";
        let prompt = "state written before the store was dropped";

        {
            let store = UarEmbeddedRuntime::open_persistence(Some(&dir))
                .await
                .expect("durable store opens");
            let session =
                universal_agent_runtime::session::SessionStore::new().create_with_id(session_id);
            session.set_system_prompt(prompt.to_string());
            store.save_session(&session).await.expect("save");
            // `store` is dropped here — the process-death stand-in.
        }

        // SurrealKV holds an exclusive lock on its directory and releasing it
        // is NOT synchronous with dropping the Arc: SurrealDB tears the
        // datastore down on a background task. A same-process reopen therefore
        // races that teardown and fails with "already locked by another
        // process".
        //
        // (An earlier comment here blamed RocksDB. Wrong engine — UAR pins
        // surrealdb with kv-surrealkv only. The failure is real and was
        // measured; only the attribution was mistaken.)
        //
        // This is a REAL operational constraint, not a test artifact. An app
        // that relaunches before the OS has reaped its previous process will
        // hit exactly this. Retrying with a short backoff is what a caller must
        // do; encoding it here keeps the behaviour visible rather than hiding
        // it behind a sleep.
        let mut reopened = None;
        for _ in 0..40 {
            match UarEmbeddedRuntime::open_persistence(Some(&dir)).await {
                Ok(store) => {
                    reopened = Some(store);
                    break;
                }
                Err(_) => tokio::time::sleep(std::time::Duration::from_millis(50)).await,
            }
        }
        let reopened = reopened.expect(
            "durable store reopens against the same directory within 2s of the previous \
             handle being dropped",
        );
        let loaded = reopened
            .load_session(session_id)
            .await
            .expect("load succeeds")
            .expect("the session survived the drop");

        assert_eq!(
            loaded.system_prompt().as_deref(),
            Some(prompt),
            "state must round-trip through a dropped store, or nothing was persisted"
        );
    }

    /// The in-memory path is still available, and is explicitly NOT durable —
    /// which is why production must never select it.
    #[tokio::test]
    async fn no_storage_dir_yields_a_non_durable_store() {
        let store = UarEmbeddedRuntime::open_persistence(None)
            .await
            .expect("in-memory store opens");
        let session =
            universal_agent_runtime::session::SessionStore::new().create_with_id("ephemeral");
        store.save_session(&session).await.expect("save");
        drop(store);

        let fresh = UarEmbeddedRuntime::open_persistence(None)
            .await
            .expect("second in-memory store opens");
        assert!(
            fresh
                .load_session("ephemeral")
                .await
                .expect("load")
                .is_none(),
            "in-memory state must NOT survive — if it did, this test proves nothing"
        );
    }
}
