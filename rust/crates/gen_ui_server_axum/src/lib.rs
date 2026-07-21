// TJ-ARCH-MOB-001 compliant
//! Reusable Axum transport for the shared `gen_ui_host` application services.
//! Static-site composition and process configuration belong to the executable.
#![forbid(unsafe_code)]

use std::convert::Infallible;
use std::time::Duration;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use gen_ui_host::{AppServices, HostError};
use gen_ui_types::events::A2uiEvent;
use serde::{Deserialize, Serialize};
use tower_http::trace::TraceLayer;

pub const API_PREFIX: &str = "/api/v1";

pub fn router(services: AppServices) -> Router {
    Router::new()
        .route("/api/v1/health", get(health))
        .route("/api/v1/ready", get(ready))
        .route("/api/v1/models/active-lane", get(active_lane))
        .route("/api/v1/providers/catalog", get(provider_catalog))
        .route("/api/v1/providers", get(provider_list))
        .route(
            "/api/v1/providers/default-model",
            get(get_default_model).put(save_default_model),
        )
        .route(
            "/api/v1/providers/{id}",
            put(save_provider).delete(delete_provider),
        )
        .route("/api/v1/providers/{id}/test", post(test_provider))
        .route("/api/v1/control-plane/catalog", get(chat_control_catalog))
        .route(
            "/api/v1/conversations/{id}/policy",
            get(get_conversation_policy).put(save_conversation_policy),
        )
        .route("/api/v1/chat/runs", post(start_run))
        .route("/api/v1/chat/runs/{run_id}/events", get(run_events))
        .with_state(services)
        .layer(TraceLayer::new_for_http())
}

#[derive(Serialize)]
struct ProbeResponse {
    status: &'static str,
}

async fn health() -> Json<ProbeResponse> {
    Json(ProbeResponse { status: "ok" })
}

async fn ready(State(services): State<AppServices>) -> Result<Json<ProbeResponse>, ApiError> {
    if services.is_ready() {
        Ok(Json(ProbeResponse { status: "ready" }))
    } else {
        Err(ApiError::unavailable(
            "host_not_ready",
            "application host is not ready",
        ))
    }
}

#[derive(Serialize)]
struct LaneResponse {
    lane: String,
}

async fn active_lane(State(services): State<AppServices>) -> Result<Json<LaneResponse>, ApiError> {
    Ok(Json(LaneResponse {
        lane: services.active_lane().await?,
    }))
}

async fn provider_catalog(
    State(services): State<AppServices>,
) -> Result<Json<Vec<gen_ui_host::ProviderCatalogEntry>>, ApiError> {
    Ok(Json(services.provider_catalog().await?))
}

async fn provider_list(
    State(services): State<AppServices>,
) -> Result<Json<Vec<gen_ui_host::ConfiguredProvider>>, ApiError> {
    Ok(Json(services.providers().await?))
}

async fn save_provider(
    State(services): State<AppServices>,
    Path(id): Path<String>,
    Json(mut request): Json<gen_ui_host::SaveProviderRequest>,
) -> Result<StatusCode, ApiError> {
    if request.id != id {
        return Err(ApiError::bad_request(
            "provider_id_mismatch",
            "path and body provider ids must match",
        ));
    }
    request.id = id;
    services.save_provider(request).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_provider(
    State(services): State<AppServices>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    services.delete_provider(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
struct TestProviderRequest {
    model: Option<String>,
}

async fn test_provider(
    State(services): State<AppServices>,
    Path(id): Path<String>,
    Json(request): Json<TestProviderRequest>,
) -> Result<Json<gen_ui_host::ProviderTestResult>, ApiError> {
    Ok(Json(
        services
            .test_provider(&id, request.model.as_deref())
            .await?,
    ))
}

async fn get_default_model(
    State(services): State<AppServices>,
) -> Result<Json<Option<gen_ui_host::ConfiguredCloudModel>>, ApiError> {
    Ok(Json(services.cloud_model().await?))
}

async fn save_default_model(
    State(services): State<AppServices>,
    Json(request): Json<gen_ui_host::SaveCloudModelRequest>,
) -> Result<StatusCode, ApiError> {
    services.save_cloud_model(request).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn chat_control_catalog(
    State(services): State<AppServices>,
) -> Result<Json<gen_ui_host::ChatControlPlaneCatalog>, ApiError> {
    Ok(Json(services.chat_control_catalog().await?))
}

async fn get_conversation_policy(
    State(services): State<AppServices>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    Ok(Json(services.conversation_policy(&id).await?))
}

async fn save_conversation_policy(
    State(services): State<AppServices>,
    Path(id): Path<String>,
    Json(policy): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, ApiError> {
    Ok(Json(services.save_conversation_policy(&id, &policy).await?))
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StartRunRequest {
    pub message: String,
    #[serde(default)]
    pub history: Vec<ChatMessage>,
    #[serde(default = "default_conversation_id")]
    pub conversation_id: String,
    #[serde(default = "default_branch_id")]
    pub branch_id: String,
    pub byok: Option<ByokConfig>,
}

fn default_conversation_id() -> String {
    "hosted-conversation".into()
}

fn default_branch_id() -> String {
    "main".into()
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ByokConfig {
    pub provider: String,
    pub model: String,
    pub api_key: String,
    pub base_url: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChatRole {
    User,
    Assistant,
}

#[derive(Serialize)]
struct StartRunResponse {
    run_id: String,
    events_url: String,
}

async fn start_run(
    State(services): State<AppServices>,
    Json(request): Json<StartRunRequest>,
) -> Result<(StatusCode, Json<StartRunResponse>), ApiError> {
    let message = request.message.trim();
    if message.is_empty() {
        return Err(ApiError::bad_request(
            "empty_message",
            "message must not be empty",
        ));
    }
    let history = request
        .history
        .into_iter()
        .map(|message| gen_ui_types::inference::InferenceMessage {
            role: match message.role {
                ChatRole::User => gen_ui_types::inference::InferenceRole::User,
                ChatRole::Assistant => gen_ui_types::inference::InferenceRole::Assistant,
            },
            content: message.content,
            name: None,
            tool_call_id: None,
        })
        .collect();
    let conversation_id = request.conversation_id;
    let branch_id = request.branch_id;
    let run_id = if let Some(byok) = request.byok {
        if byok.provider.trim().is_empty()
            || byok.model.trim().is_empty()
            || byok.api_key.trim().is_empty()
        {
            return Err(ApiError::bad_request(
                "invalid_byok",
                "provider, model, and API key are required for BYOK",
            ));
        }
        services
            .start_chat_with_byok(
                message.to_string(),
                history,
                gen_ui_host::EphemeralCloudConfig {
                    provider: byok.provider,
                    model: byok.model,
                    api_key: byok.api_key,
                    base_url: byok.base_url,
                },
                conversation_id,
                branch_id,
            )
            .await?
    } else {
        services
            .start_chat(message.to_string(), history, conversation_id, branch_id)
            .await?
    };
    let events_url = format!("{API_PREFIX}/chat/runs/{run_id}/events");
    Ok((
        StatusCode::ACCEPTED,
        Json(StartRunResponse { run_id, events_url }),
    ))
}

async fn run_events(
    State(services): State<AppServices>,
    Path(run_id): Path<String>,
) -> Result<Sse<impl futures::Stream<Item = Result<Event, Infallible>>>, ApiError> {
    let mut receiver = services.take_run_events(&run_id).await?;
    let event_run_id = run_id;
    let stream = async_stream::stream! {
        let mut active = false;
        loop {
            match receiver.recv().await {
                Ok(event) if event.run_id() == event_run_id => {
                    if matches!(event, A2uiEvent::RunStarted { .. }) {
                        active = true;
                    }
                    if active {
                        let terminal = matches!(
                            event,
                            A2uiEvent::RunFinished { .. }
                                | A2uiEvent::RunCancelled { .. }
                                | A2uiEvent::RunError { .. }
                        );
                        yield Ok(Event::default().event("a2ui").json_data(&event).unwrap_or_else(|_| Event::default().event("serialization_error")));
                        if terminal {
                            break;
                        }
                    }
                }
                Ok(_) => {}
                Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                    let payload = serde_json::json!({
                        "error": {
                            "code": "event_stream_lagged",
                            "message": "event consumer fell behind",
                            "details": { "skipped": skipped }
                        }
                    });
                    yield Ok(Event::default().event("transport_error").json_data(payload).unwrap_or_else(|_| Event::default().event("serialization_error")));
                    break;
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            }
        }
    };
    Ok(Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    ))
}

#[derive(Debug)]
struct ApiError {
    status: StatusCode,
    code: &'static str,
    message: String,
}

impl ApiError {
    fn bad_request(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code,
            message: message.into(),
        }
    }

    fn unavailable(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::SERVICE_UNAVAILABLE,
            code,
            message: message.into(),
        }
    }
}

impl From<HostError> for ApiError {
    fn from(error: HostError) -> Self {
        match error {
            HostError::UnknownRun(_) => Self {
                status: StatusCode::NOT_FOUND,
                code: "run_not_found",
                message: error.to_string(),
            },
            HostError::Initialization(_) => {
                Self::unavailable("host_initialization", error.to_string())
            }
            HostError::Chat(_) => Self {
                status: StatusCode::UNPROCESSABLE_ENTITY,
                code: "chat_unavailable",
                message: error.to_string(),
            },
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = serde_json::json!({
            "error": { "code": self.code, "message": self.message }
        });
        (self.status, Json(body)).into_response()
    }
}
