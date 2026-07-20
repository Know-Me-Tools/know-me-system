// TJ-ARCH-MOB-001 compliant
//! Universal Agent Runtime boundary for KnowMe.
//!
//! UAR owns real agent execution, skills, A2UI registry management, tool
//! governance, and general provider routing. KnowMe owns the product shell,
//! local-device provider adapters, and the local conversation projection. This
//! module captures that boundary without linking the full UAR runtime yet.

use futures::{stream::BoxStream, StreamExt};
use gen_ui_protocol::{
    normalize_uar_agui_event, uar_llm_payload_to_inference_request, A2uiAdapter,
    UarLlmRequestPayload, UarNormalizedEventAdapter,
};
use gen_ui_types::events::A2uiEvent;
use gen_ui_types::inference::{InferenceProvider, LocalModelSpec};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::error::AgentError;

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
    /// Future state: in-process UAR kernel owns mobile agent execution.
    EmbeddedKernel,
    /// Future state: HTTP/service UAR owns web/desktop agent execution.
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

/// Host-side bridge that lets a future embedded UAR `ExternalLlmDriver` call
/// KnowMe's local provider without UAR owning model catalogs or Flutter owning
/// prompt construction.
///
/// This is intentionally not the full embedded UAR kernel. It is the narrow
/// adapter the kernel will call:
///
/// `UAR LlmRequest payload → KnowMe InferenceProvider → UAR NormalizedEvent payloads`.
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
        let request =
            uar_llm_payload_to_inference_request(&payload).map_err(AgentError::Interaction)?;
        self.provider.load(&self.model_spec).await?;
        let stream = self.provider.generate_request(&request).await?;
        let adapter = UarNormalizedEventAdapter::new();
        Ok(futures::stream::unfold(
            (stream, adapter, Vec::<serde_json::Value>::new()),
            |(mut stream, mut adapter, mut pending)| async move {
                loop {
                    if let Some(value) = pending.pop() {
                        return Some((Ok(value), (stream, adapter, pending)));
                    }
                    match stream.next().await {
                        Some(event) => {
                            pending = adapter.ingest(&event);
                            pending.reverse();
                        }
                        None => return None,
                    }
                }
            },
        )
        .boxed())
    }
}

impl UarRuntimeStatus {
    pub fn current() -> Self {
        Self {
            authority: "universal-agent-runtime".into(),
            target_mode: default_target_mode(),
            current_state: UarIntegrationState::ProjectionOnly,
            local_provider_role: "knowme_local_provider_backend".into(),
            mobile_sidecar_allowed: false,
            local_provider_bridge_available: true,
            service_runtime_available: service_runtime_available(),
            service_artifact_response_available: service_artifact_response_available(),
            embedded_kernel_available: false,
            embedded_kernel_blocker: Some(
                "The current UAR SDK embedded feature links the broad server runtime and does not yet expose a populated mobile-safe in-process kernel/AppState."
                    .into(),
            ),
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
    UarDeploymentMode::CloudService
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
}

impl UarEventProjector {
    pub fn new(run_id: impl Into<String>) -> Self {
        Self {
            adapter: A2uiAdapter::new(run_id),
        }
    }

    pub fn ingest_agui_payload(&mut self, payload: &serde_json::Value) -> Vec<A2uiEvent> {
        let event = normalize_uar_agui_event(payload);
        self.adapter.ingest(&event)
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
    use gen_ui_types::content_block::ContentBlock;
    use gen_ui_types::events::StreamEvent;
    use gen_ui_types::inference::{InferenceRequest, InferenceToolResult};
    use gen_ui_types::{CoreError, CoreResult};
    use parking_lot::Mutex;

    #[derive(Default)]
    struct RecordingInferenceProvider {
        loads: Mutex<Vec<LocalModelSpec>>,
        requests: Mutex<Vec<InferenceRequest>>,
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
            Ok(futures::stream::iter([
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
            ])
            .boxed())
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
                            r#"{"run_id":"uar-run-1","artifact_id":"artifact-1","status":"accepted"}"#,
                        )
                        .await;
                    } else if request_line.starts_with("POST /api/uar/runs/uar-run-1/cancel ") {
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
            "POST /api/uar/runs/uar-run-1/cancel HTTP/1.1"
        );

        server.abort();
    }
}
