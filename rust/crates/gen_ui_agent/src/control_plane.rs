// TJ-ARCH-MOB-001 compliant
//! UAR-owned agent/resource discovery and scoped conversation policy.

use serde::Serialize;

use crate::{chat, error::AgentError};

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
    chat::uar_service_client()
        .await?
        .admin()
        .update_setting(key, value)
        .await
        .map_err(uar_error)
}

pub async fn save_agent(agent: &serde_json::Value) -> Result<serde_json::Value, AgentError> {
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
    chat::uar_service_client()
        .await?
        .admin()
        .save_mcp_server(name, server)
        .await
        .map_err(uar_error)
}

pub async fn delete_mcp_server(name: &str) -> Result<serde_json::Value, AgentError> {
    chat::uar_service_client()
        .await?
        .admin()
        .delete_mcp_server(name)
        .await
        .map_err(uar_error)
}

pub async fn delete_memory(id: &str) -> Result<serde_json::Value, AgentError> {
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
    chat::uar_service_client()
        .await?
        .admin()
        .create_knowledge_base(request)
        .await
        .map_err(uar_error)
}

pub async fn delete_knowledge_base(id: &str) -> Result<(), AgentError> {
    chat::uar_service_client()
        .await?
        .admin()
        .delete_knowledge_base(id)
        .await
        .map_err(uar_error)
}

pub async fn conversation_policy(conversation_id: &str) -> Result<serde_json::Value, AgentError> {
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
    chat::uar_service_client()
        .await?
        .control_plane()
        .save_conversation_policy(conversation_id, policy)
        .await
        .map_err(uar_error)
}

pub async fn delete_conversation_policy(conversation_id: &str) -> Result<(), AgentError> {
    chat::uar_service_client()
        .await?
        .control_plane()
        .delete_conversation_policy(conversation_id)
        .await
        .map_err(uar_error)
}

pub async fn effective_config(conversation_id: &str) -> Result<serde_json::Value, AgentError> {
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
