//! Explicit placement and durable observation. A disconnect never starts a replacement run.
mod journal;
pub use gen_ui_types::execution_profiles::{ExecutionJob, JobState};
pub use journal::{jobs, suspend, resume, cancel};
pub(crate) use journal::claim_observation;

pub use gen_ui_types::execution_profiles::{ExecutionProfile, ProfileKind, ExecutionFrame};
use crate::{error::AgentError, state, uar::UarDeploymentMode};
const PROFILE: &str = "afc.execution-profile.v1";
pub async fn current() -> Result<Option<ExecutionProfile>, AgentError> {
    state::config()?.get_setting(PROFILE).await?.map(|value|
        serde_json::from_value(value).map_err(|_| AgentError::Config("EXECUTION_PROFILE_INVALID".into()))).transpose()
}

pub async fn configure(profile: ExecutionProfile) -> Result<ExecutionProfile, AgentError> {
    if profile.schema_version != 1 || profile.instance_id.trim().is_empty() {
        return Err(AgentError::Config("EXECUTION_PROFILE_INVALID".into()));
    }
    match profile.kind {
        ProfileKind::StandaloneEmbedded => { state::embedded_uar()?; }
        _ => {
            let url = reqwest::Url::parse(profile.service_url.as_deref().unwrap_or_default())
                .map_err(|_| AgentError::Config("EXECUTION_PROFILE_ENDPOINT_REQUIRED".into()))?;
            let local = url.host_str().is_some_and(|host| matches!(host, "localhost" | "127.0.0.1" | "[::1]" | "::1"));
            if !profile.remote_input_approved || (url.scheme() != "https" && !(local && url.scheme() == "http")) ||
                !url.username().is_empty() || url.password().is_some() || url.query().is_some() || url.fragment().is_some() {
                return Err(AgentError::Config("EXECUTION_PROFILE_CONTEXT_OR_ENDPOINT_DENIED".into()));
            }
            // Persisted recovery needs a runs API identity, not the streaming-chat session alias.
            let configured = crate::chat::uar_service_config().await?;
            if configured.service_url.as_deref() != profile.service_url.as_deref() || !configured.has_agent_artifact {
                return Err(AgentError::Config("EXECUTION_PROFILE_COMPILED_AGENT_REQUIRED".into()));
            }
        }
    }
    state::config()?.set_setting(PROFILE, serde_json::to_value(&profile)
        .map_err(|_| AgentError::Config("EXECUTION_PROFILE_INVALID".into()))?).await?;
    Ok(profile)
}

pub(crate) async fn deployment_mode() -> Result<Option<UarDeploymentMode>, AgentError> {
    Ok(current().await?.map(|profile| match profile.kind {
        ProfileKind::StandaloneEmbedded => UarDeploymentMode::EmbeddedLibrary,
        ProfileKind::Home => UarDeploymentMode::HttpService,
        ProfileKind::PersonalCloud | ProfileKind::Enterprise => UarDeploymentMode::CloudService,
    }))
}

pub(crate) async fn guard_remote(endpoint: &str) -> Result<(), AgentError> {
    if let Some(profile) = current().await? {
        if profile.kind != ProfileKind::StandaloneEmbedded &&
            (!profile.remote_input_approved || profile.service_url.as_deref() != Some(endpoint)) {
            return Err(AgentError::Config("EXECUTION_PROFILE_BINDING_CHANGED".into()));
        }
    }
    Ok(())
}

pub(crate) async fn begin(host_run_id: &str, conversation_id: &str, mode: &str) -> Result<(), AgentError> {
    journal::begin(host_run_id, conversation_id, mode, current().await?).await
}
pub(crate) async fn admitted(host_run_id: &str, runtime_run_id: &str) -> Result<(), AgentError> {
    journal::admitted(host_run_id, runtime_run_id).await
}
pub(crate) async fn record_frame(runtime: &str, source_id: Option<u64>, event: &str, data: &serde_json::Value) -> Result<(), AgentError> {
    journal::record_frame(runtime, source_id, event, data).await
}
pub(crate) async fn admitted_session(session: Option<&str>, runtime: &str) -> Result<(), AgentError> {
    journal::admitted_session(session, runtime).await
}
pub(crate) async fn observation_detached(host: &str) -> bool { journal::observation_detached(host).await }
pub(crate) async fn terminal(host_run_id: &str, success: bool) -> Result<(), AgentError> {
    journal::terminal(host_run_id, success).await
}
