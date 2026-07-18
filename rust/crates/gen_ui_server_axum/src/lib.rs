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
use axum::routing::{get, post};
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
    Ok(Json(services.provider_catalog()?))
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StartRunRequest {
    pub message: String,
    #[serde(default)]
    pub history: Vec<ChatMessage>,
    pub byok: Option<ByokConfig>,
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

impl ChatRole {
    fn as_str(&self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Assistant => "assistant",
        }
    }
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
        .flat_map(|message| [message.role.as_str().to_string(), message.content])
        .collect();
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
            )
            .await?
    } else {
        services.start_chat(message.to_string(), history).await?
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
                Ok(ref event @ A2uiEvent::RunStarted { ref run_id }) if run_id == &event_run_id => {
                    active = true;
                    yield Ok(Event::default().event("a2ui").json_data(event).unwrap_or_else(|_| Event::default().event("serialization_error")));
                }
                Ok(event @ A2uiEvent::Block { .. }) if active => {
                    yield Ok(Event::default().event("a2ui").json_data(event).unwrap_or_else(|_| Event::default().event("serialization_error")));
                }
                Ok(ref event @ A2uiEvent::RunFinished { ref run_id }) if active && run_id == &event_run_id => {
                    yield Ok(Event::default().event("a2ui").json_data(event).unwrap_or_else(|_| Event::default().event("serialization_error")));
                    break;
                }
                Ok(event @ A2uiEvent::RunError { .. }) if active => {
                    yield Ok(Event::default().event("a2ui").json_data(event).unwrap_or_else(|_| Event::default().event("serialization_error")));
                    break;
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
