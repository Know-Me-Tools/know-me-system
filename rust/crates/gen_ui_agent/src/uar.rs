// TJ-ARCH-MOB-001 compliant
//! Universal Agent Runtime boundary for KnowMe.
//!
//! UAR owns real agent execution, skills, A2UI registry management, tool
//! governance, and general provider routing. KnowMe owns the product shell,
//! local-device provider adapters, and the local conversation projection. This
//! module captures that boundary without linking the full UAR runtime yet.

#[cfg(not(any(target_os = "ios", target_os = "android")))]
use futures::{stream::BoxStream, StreamExt};
use gen_ui_protocol::{normalize_uar_agui_event, A2uiAdapter};
use gen_ui_types::events::A2uiEvent;
use serde::{Deserialize, Serialize};

#[cfg(not(any(target_os = "ios", target_os = "android")))]
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
    pub artifact: serde_json::Value,
    pub input: String,
    pub session_id: Option<String>,
}

#[cfg(not(any(target_os = "ios", target_os = "android")))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UarArtifactResponse {
    pub artifact_id: String,
    pub response: serde_json::Value,
}

#[cfg(not(any(target_os = "ios", target_os = "android")))]
pub struct UarProjectedRun {
    pub run_id: String,
    pub events: BoxStream<'static, Result<A2uiEvent, AgentError>>,
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
    #[cfg(not(any(target_os = "ios", target_os = "android")))]
    async fn start_run(&self, request: UarRunRequest) -> Result<UarProjectedRun, AgentError>;
    #[cfg(not(any(target_os = "ios", target_os = "android")))]
    async fn cancel_run(&self, run_id: &str) -> Result<bool, AgentError>;
    #[cfg(not(any(target_os = "ios", target_os = "android")))]
    async fn submit_artifact_response(
        &self,
        run_id: &str,
        response: UarArtifactResponse,
    ) -> Result<(), AgentError>;
}

#[cfg(not(any(target_os = "ios", target_os = "android")))]
#[derive(Debug, Clone)]
pub struct UarServiceRuntime {
    client: universal_agent_runtime_sdk::Client,
}

#[cfg(not(any(target_os = "ios", target_os = "android")))]
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

#[cfg(not(any(target_os = "ios", target_os = "android")))]
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
        let run = self
            .client
            .runs()
            .create(universal_agent_runtime_sdk::CreateRunRequest {
                artifact: request.artifact,
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
            .cancel(run_id)
            .await
            .map(|response| response.cancelled)
            .map_err(uar_sdk_error)
    }

    async fn submit_artifact_response(
        &self,
        run_id: &str,
        response: UarArtifactResponse,
    ) -> Result<(), AgentError> {
        self.client
            .runs()
            .submit_artifact_response(
                run_id,
                universal_agent_runtime_sdk::ArtifactResponseRequest {
                    artifact_id: response.artifact_id,
                    response: response.response,
                },
            )
            .await
            .map(|_| ())
            .map_err(uar_sdk_error)
    }
}

#[cfg(not(any(target_os = "ios", target_os = "android")))]
fn uar_sdk_error(error: universal_agent_runtime_sdk::Error) -> AgentError {
    AgentError::Client(format!("UAR runtime: {error}"))
}

#[cfg(not(any(target_os = "ios", target_os = "android")))]
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

#[cfg(not(any(target_os = "ios", target_os = "android")))]
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
    false
}

#[cfg(not(any(target_os = "ios", target_os = "android")))]
const fn service_artifact_response_available() -> bool {
    true
}

#[cfg(any(target_os = "ios", target_os = "android"))]
const fn service_artifact_response_available() -> bool {
    false
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

/// Current user-visible UAR status. This intentionally reports projection-only
/// until chat execution actually flows through a UAR facade.
pub fn runtime_status() -> UarRuntimeStatus {
    UarRuntimeStatus::current()
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;
    use gen_ui_types::content_block::ContentBlock;

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
}
