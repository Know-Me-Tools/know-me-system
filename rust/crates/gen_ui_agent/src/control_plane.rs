// TJ-ARCH-MOB-001 compliant
//! UAR-owned agent/resource discovery and scoped conversation policy.

use serde::Serialize;

use crate::uar::UarDeploymentMode;
use crate::{chat, error::AgentError};

/// Which UAR the control plane talks to: the in-process embedded kernel (mobile
/// and macOS, no service URL) or the HTTP service (Windows/Linux/Web).
enum Backend {
    Embedded,
    Service,
}

/// Resolve the control-plane backend from the platform's deployment mode, mirroring
/// `provider_admin::backend()` so both admin surfaces dispatch identically.
async fn backend() -> Result<Backend, AgentError> {
    match chat::selected_uar_deployment_mode().await? {
        UarDeploymentMode::EmbeddedLibrary => Ok(Backend::Embedded),
        UarDeploymentMode::HttpService | UarDeploymentMode::CloudService => Ok(Backend::Service),
    }
}

/// Uniform "not yet on embedded" error for control-plane operations whose SDK
/// admin surface is not exposed in-process yet (kept explicit rather than falling
/// through to the URL-demanding HTTP client).
fn not_on_embedded(op: &str) -> AgentError {
    AgentError::Config(format!(
        "{op} is not available on the embedded runtime yet (no HTTP service on this platform)"
    ))
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatControlPlaneCatalog {
    pub agents: serde_json::Value,
    pub skills: serde_json::Value,
    pub tools: serde_json::Value,
    pub knowledge_bases: serde_json::Value,
}

/// UAR-owned configuration and runtime resources displayed by the first-party
/// settings clients. Credentials remain masked by UAR and never cross this
/// boundary in a read response.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UarAdminSnapshot {
    pub settings: serde_json::Value,
    pub setting_types: serde_json::Value,
    pub agents: serde_json::Value,
    pub skills: serde_json::Value,
    pub tools: serde_json::Value,
    pub knowledge_bases: serde_json::Value,
    pub mcp_servers: serde_json::Value,
    pub memories: serde_json::Value,
}

pub async fn catalog() -> Result<ChatControlPlaneCatalog, AgentError> {
    if let Backend::Embedded = backend().await? {
        // On embedded, agent definitions come from the in-process runtime; skills/
        // tools/knowledge-bases have no embedded admin surface yet, so they list
        // empty rather than demanding a service URL.
        //
        // Shape parity matters here: the service path returns the agents as
        // `{ runtime_agents, federated_agents }` (see `api/discovery.rs`), while
        // the embedded runtime hands back a bare array. Clients parse one shape,
        // so wrap the array to match the service contract — otherwise every
        // embedded surface (iOS/Android/macOS) silently sees an empty agent list
        // while the agents exist and are seeded.
        let agents = crate::state::embedded_uar()?.list_agents().await?;
        return Ok(ChatControlPlaneCatalog {
            agents: serde_json::json!({
                "runtime_agents": agents,
                "federated_agents": [],
            }),
            skills: serde_json::json!([]),
            tools: serde_json::json!([]),
            knowledge_bases: serde_json::json!([]),
        });
    }
    let client = chat::uar_service_client().await?;
    let api = client.control_plane();
    let (agents, skills, tools, knowledge_bases) = futures::try_join!(
        api.agents(),
        api.skills(),
        api.tools(),
        api.knowledge_bases(),
    )
    .map_err(uar_error)?;
    Ok(ChatControlPlaneCatalog {
        agents,
        skills,
        tools,
        knowledge_bases,
    })
}

pub async fn admin_snapshot() -> Result<UarAdminSnapshot, AgentError> {
    if let Backend::Embedded = backend().await? {
        // Settings + agents come from the in-process runtime (enough for the
        // global run-policy panel and the agent editor). The remaining registries
        // have no embedded admin surface yet and list empty.
        let runtime = crate::state::embedded_uar()?;
        let (settings, setting_types) = runtime.settings_snapshot().await?;
        let agents = runtime.list_agents().await?;
        return Ok(UarAdminSnapshot {
            settings,
            setting_types,
            agents,
            skills: serde_json::json!([]),
            tools: serde_json::json!([]),
            knowledge_bases: serde_json::json!([]),
            mcp_servers: serde_json::json!([]),
            memories: serde_json::json!([]),
        });
    }
    let client = chat::uar_service_client().await?;
    let control = client.control_plane();
    let admin = client.admin();
    let (settings, setting_types, agents, skills, tools, knowledge_bases, mcp_servers, memories) = futures::join!(
        admin.settings(),
        admin.setting_types(),
        control.agents(),
        control.skills(),
        control.tools(),
        control.knowledge_bases(),
        admin.mcp_servers(),
        admin.memories(),
    );
    Ok(UarAdminSnapshot {
        settings: snapshot_value(settings),
        setting_types: snapshot_value(setting_types),
        agents: snapshot_value(agents),
        skills: snapshot_value(skills),
        tools: snapshot_value(tools),
        knowledge_bases: snapshot_value(knowledge_bases),
        mcp_servers: snapshot_value(mcp_servers),
        memories: snapshot_value(memories),
    })
}

pub async fn update_setting(
    key: &str,
    value: &serde_json::Value,
) -> Result<serde_json::Value, AgentError> {
    if let Backend::Embedded = backend().await? {
        // Writes the typed setting (e.g. run_policy.global) in-process, then
        // echoes it back the way the service admin does.
        crate::state::embedded_uar()?
            .set_setting(key, value.clone())
            .await?;
        return Ok(value.clone());
    }
    chat::uar_service_client()
        .await?
        .admin()
        .update_setting(key, value)
        .await
        .map_err(uar_error)
}

pub async fn save_agent(agent: &serde_json::Value) -> Result<serde_json::Value, AgentError> {
    if let Backend::Embedded = backend().await? {
        crate::state::embedded_uar()?.upsert_agent(agent).await?;
        return Ok(agent.clone());
    }
    let client = chat::uar_service_client().await?;
    let id = agent
        .get("id")
        .and_then(serde_json::Value::as_str)
        .filter(|id| !id.trim().is_empty());
    match id {
        Some(id) => client
            .admin()
            .update_agent(id, agent)
            .await
            .map_err(uar_error),
        None => client.admin().create_agent(agent).await.map_err(uar_error),
    }
}

pub async fn delete_agent(id: &str) -> Result<(), AgentError> {
    if let Backend::Embedded = backend().await? {
        return crate::state::embedded_uar()?.delete_agent(id).await;
    }
    chat::uar_service_client()
        .await?
        .admin()
        .delete_agent(id)
        .await
        .map_err(uar_error)
}

pub async fn save_mcp_server(
    name: &str,
    server: &serde_json::Value,
) -> Result<serde_json::Value, AgentError> {
    if let Backend::Embedded = backend().await? {
        return Err(not_on_embedded("MCP server administration"));
    }
    chat::uar_service_client()
        .await?
        .admin()
        .save_mcp_server(name, server)
        .await
        .map_err(uar_error)
}

pub async fn delete_mcp_server(name: &str) -> Result<serde_json::Value, AgentError> {
    if let Backend::Embedded = backend().await? {
        return Err(not_on_embedded("MCP server administration"));
    }
    chat::uar_service_client()
        .await?
        .admin()
        .delete_mcp_server(name)
        .await
        .map_err(uar_error)
}

pub async fn delete_memory(id: &str) -> Result<serde_json::Value, AgentError> {
    if let Backend::Embedded = backend().await? {
        return Err(not_on_embedded("memory administration"));
    }
    chat::uar_service_client()
        .await?
        .admin()
        .delete_memory(id)
        .await
        .map_err(uar_error)
}

pub async fn create_knowledge_base(
    request: &serde_json::Value,
) -> Result<serde_json::Value, AgentError> {
    if let Backend::Embedded = backend().await? {
        return Err(not_on_embedded("knowledge-base administration"));
    }
    chat::uar_service_client()
        .await?
        .admin()
        .create_knowledge_base(request)
        .await
        .map_err(uar_error)
}

pub async fn delete_knowledge_base(id: &str) -> Result<(), AgentError> {
    if let Backend::Embedded = backend().await? {
        return Err(not_on_embedded("knowledge-base administration"));
    }
    chat::uar_service_client()
        .await?
        .admin()
        .delete_knowledge_base(id)
        .await
        .map_err(uar_error)
}

pub async fn conversation_policy(conversation_id: &str) -> Result<serde_json::Value, AgentError> {
    if let Backend::Embedded = backend().await? {
        return crate::state::embedded_uar()?
            .conversation_policy(conversation_id)
            .await;
    }
    chat::uar_service_client()
        .await?
        .control_plane()
        .conversation_policy(conversation_id)
        .await
        .map_err(uar_error)
}

pub async fn save_conversation_policy(
    conversation_id: &str,
    policy: &serde_json::Value,
) -> Result<serde_json::Value, AgentError> {
    if let Backend::Embedded = backend().await? {
        return crate::state::embedded_uar()?
            .save_conversation_policy(conversation_id, policy)
            .await;
    }
    chat::uar_service_client()
        .await?
        .control_plane()
        .save_conversation_policy(conversation_id, policy)
        .await
        .map_err(uar_error)
}

pub async fn delete_conversation_policy(conversation_id: &str) -> Result<(), AgentError> {
    if let Backend::Embedded = backend().await? {
        return crate::state::embedded_uar()?
            .delete_conversation_policy(conversation_id)
            .await;
    }
    chat::uar_service_client()
        .await?
        .control_plane()
        .delete_conversation_policy(conversation_id)
        .await
        .map_err(uar_error)
}

pub async fn effective_config(conversation_id: &str) -> Result<serde_json::Value, AgentError> {
    if let Backend::Embedded = backend().await? {
        return crate::state::embedded_uar()?
            .effective_config(conversation_id)
            .await;
    }
    chat::uar_service_client()
        .await?
        .control_plane()
        .effective_config(conversation_id)
        .await
        .map_err(uar_error)
}

fn uar_error(error: universal_agent_runtime_sdk::Error) -> AgentError {
    AgentError::Client(format!("UAR control plane request failed: {error}"))
}

fn snapshot_value(
    result: Result<serde_json::Value, universal_agent_runtime_sdk::Error>,
) -> serde_json::Value {
    result.unwrap_or_else(|error| {
        serde_json::json!({
            "items": [],
            "error": error.to_string(),
        })
    })
}
