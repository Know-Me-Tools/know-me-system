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

// `not_on_embedded` used to live here — a uniform "not yet on embedded" error
// for control-plane operations the SDK could not serve in-process. Every one of
// them now can, so the helper is gone rather than left as a tempting shortcut
// for the next operation that looks hard to wire up.

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatControlPlaneCatalog {
    pub agents: serde_json::Value,
    pub skills: serde_json::Value,
    pub tools: serde_json::Value,
    pub knowledge_bases: serde_json::Value,
    /// Resources this deployment CANNOT provide, each with a reason.
    ///
    /// Without this a client cannot tell "no skills are configured" from
    /// "skills are unavailable on this platform" — both arrive as `[]`. That
    /// ambiguity is the defect: an offline surface silently omits capabilities
    /// instead of saying they are missing and why.
    ///
    /// Carried ALONGSIDE the arrays rather than replacing them, so existing
    /// consumers that map `skills`/`tools`/`knowledgeBases` keep working
    /// unchanged and can adopt this incrementally.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unavailable: Vec<UnavailableResource>,
}

/// A capability the current deployment cannot serve, and why.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnavailableResource {
    /// Catalog key this refers to: `skills`, `tools`, `knowledgeBases`, …
    pub resource: String,
    /// Human-readable cause, safe to surface directly in a UI.
    pub reason: String,
    /// True when the cause is structural on this platform rather than a
    /// transient outage — the UI should present it as "not supported here",
    /// not "retry later".
    pub platform_limitation: bool,
}

/// The identity an embedded runtime acts as.
///
/// An embedded host is single-user by construction — the device owner. There is
/// no auth service to issue claims, so this is a fixed local identity rather
/// than an authenticated one. It is still explicit rather than defaulted:
/// memories are scoped per user, and silently passing an empty or anonymous
/// context would file every memory under a scope the UI cannot then read back.
fn local_user_context() -> universal_agent_runtime_sdk::runtime::UserContext {
    universal_agent_runtime_sdk::runtime::UserContext {
        user_id: "local".to_string(),
        claims: universal_agent_runtime_sdk::runtime::UserClaims {
            sub: "local".to_string(),
            name: None,
            roles: None,
            exp: 0,
        },
    }
}

impl UnavailableResource {
    #[allow(dead_code, reason = "retained for resources that are still embedded-unavailable")]
    fn embedded(resource: &str) -> Self {
        Self {
            resource: resource.to_string(),
            reason: "not available on the embedded runtime yet (no HTTP service on this platform)"
                .to_string(),
            platform_limitation: true,
        }
    }
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
    /// See `ChatControlPlaneCatalog::unavailable` — same contract.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unavailable: Vec<UnavailableResource>,
}

pub async fn catalog() -> Result<ChatControlPlaneCatalog, AgentError> {
    if let Backend::Embedded = backend().await? {
        // Everything here comes from the in-process runtime — agents, skills,
        // tools and knowledge bases alike. None of it demands a service URL.
        //
        // Shape parity matters here: the service path returns the agents as
        // `{ runtime_agents, federated_agents }` (see `api/discovery.rs`), while
        // the embedded runtime hands back a bare array. Clients parse one shape,
        // so wrap the array to match the service contract — otherwise every
        // embedded surface (iOS/Android/macOS) silently sees an empty agent list
        // while the agents exist and are seeded.
        let runtime = crate::state::embedded_uar()?;
        let agents = runtime.list_agents().await?;
        // Skills and knowledge bases are served in-process by the SDK now, so
        // these are REAL rather than the hardcoded `[]` they used to be.
        let skills = runtime.list_skills().await?;
        let knowledge_bases = runtime.list_knowledge_bases().await?;
        // Tools are now served in-process from both registries, so the marker
        // that used to sit here would be FALSE — leaving it would make the UI
        // claim a capability is missing when it works.
        let tools = runtime.list_tools().await?;
        return Ok(ChatControlPlaneCatalog {
            agents: serde_json::json!({
                "runtime_agents": agents,
                "federated_agents": [],
            }),
            skills,
            tools,
            knowledge_bases,
            unavailable: Vec::new(),
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
        // The service serves every resource, so nothing is advertised as
        // unavailable.
        unavailable: Vec::new(),
    })
}

pub async fn admin_snapshot() -> Result<UarAdminSnapshot, AgentError> {
    if let Backend::Embedded = backend().await? {
        // Every registry the admin UI shows is served in-process: settings,
        // agents, skills, knowledge bases, MCP servers, tools and memories. An
        // embedded host administers exactly what a served one does.
        let runtime = crate::state::embedded_uar()?;
        let (settings, setting_types) = runtime.settings_snapshot().await?;
        let agents = runtime.list_agents().await?;
        let skills = runtime.list_skills().await?;
        let knowledge_bases = runtime.list_knowledge_bases().await?;
        let mcp_servers = runtime.list_mcp_servers().await?;
        let tools = runtime.list_tools().await?;
        // Memories are served for real now. An earlier note here claimed a
        // PersistenceLayer limit; that was a misdiagnosis. `PersistenceLayer`'s
        // memory methods are no-op stubs, but the actual blocker was that no
        // host constructed a `MemoryService` — one is opened at boot, and this
        // reads through it.
        //
        // A failure to open memory is NOT a failure to list the catalog: the
        // rest of the snapshot is still valid and useful, so the error becomes
        // an `unavailable` entry carrying its reason rather than sinking the
        // whole call. An empty list with no explanation is the exact ambiguity
        // the contract exists to remove.
        let (memories, unavailable) = match runtime
            .list_memories(&local_user_context(), None, None)
            .await
        {
            Ok(value) => (value, Vec::new()),
            Err(error) => (
                serde_json::json!([]),
                vec![UnavailableResource {
                    resource: "memories".to_string(),
                    reason: error.to_string(),
                    platform_limitation: false,
                }],
            ),
        };
        return Ok(UarAdminSnapshot {
            settings,
            setting_types,
            agents,
            skills,
            tools,
            knowledge_bases,
            mcp_servers,
            memories,
            unavailable,
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
        unavailable: Vec::new(),
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
        // The result may report `Deferred`: an edit to an already-connected
        // server is stored durably but not applied live, so a healthy
        // connection is not torn down mid-session.
        return crate::state::embedded_uar()?
            .save_mcp_server(name, server)
            .await;
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
        return crate::state::embedded_uar()?.delete_mcp_server(name).await;
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
        // This used to refuse, on the reasoning that a direct admin delete would
        // bypass the memory service's scoping/embedding invariants. That reason
        // no longer applies: the call goes THROUGH `MemoryService`, so those
        // invariants are upheld — and refusing left a device user unable to
        // remove something the runtime remembered about them, which is worse
        // than the risk it was guarding against.
        crate::state::embedded_uar()?.delete_memory(id).await?;
        return Ok(serde_json::json!({ "id": id, "deleted": true }));
    }
    chat::uar_service_client()
        .await?
        .admin()
        .delete_memory(id)
        .await
        .map_err(uar_error)
}

/// Add a memory.
pub async fn add_memory(content: &str) -> Result<serde_json::Value, AgentError> {
    if let Backend::Embedded = backend().await? {
        return crate::state::embedded_uar()?
            .add_memory(content.to_string(), &local_user_context(), None, None)
            .await;
    }
    chat::uar_service_client()
        .await?
        .admin()
        .add_memory(content)
        .await
        .map_err(uar_error)
}

/// Replace a memory's content.
pub async fn update_memory(id: &str, content: &str) -> Result<serde_json::Value, AgentError> {
    if let Backend::Embedded = backend().await? {
        return crate::state::embedded_uar()?
            .update_memory(id, content.to_string())
            .await;
    }
    chat::uar_service_client()
        .await?
        .admin()
        .update_memory(id, content)
        .await
        .map_err(uar_error)
}

pub async fn create_knowledge_base(
    request: &serde_json::Value,
) -> Result<serde_json::Value, AgentError> {
    if let Backend::Embedded = backend().await? {
        // Listing knowledge bases already worked embedded, so a device could
        // SEE them but never add one — the surface read as broken rather than
        // limited. This was the last embedded refusal in the catalog surface.
        return crate::state::embedded_uar()?
            .save_knowledge_base(request)
            .await;
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
        return crate::state::embedded_uar()?.delete_knowledge_base(id).await;
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

#[cfg(test)]
mod tests {
    use super::*;

    // The defect this guards: an empty `skills` array is indistinguishable from
    // "skills are unavailable here" unless something says so. A client that
    // cannot tell them apart silently omits a capability instead of explaining
    // it — which is exactly what an offline surface must not do.
    #[test]
    fn an_embedded_catalog_says_why_its_registries_are_empty() {
        let catalog = ChatControlPlaneCatalog {
            agents: serde_json::json!({ "runtime_agents": [], "federated_agents": [] }),
            skills: serde_json::json!([]),
            tools: serde_json::json!([]),
            knowledge_bases: serde_json::json!([]),
            unavailable: vec![
                UnavailableResource::embedded("skills"),
                UnavailableResource::embedded("tools"),
                UnavailableResource::embedded("knowledgeBases"),
            ],
        };
        let json = serde_json::to_value(&catalog).expect("serializes");

        let listed = json["unavailable"].as_array().expect("unavailable is an array");
        assert_eq!(listed.len(), 3, "every empty registry must be accounted for");

        for entry in listed {
            assert!(
                !entry["reason"].as_str().unwrap_or_default().is_empty(),
                "an unavailable resource with no reason is no better than an empty array"
            );
            assert_eq!(
                entry["platformLimitation"], true,
                "embedded gaps are structural, so a UI must not offer 'retry later'"
            );
        }

        // camelCase on the wire: both clients parse that shape.
        assert!(json.get("unavailable").is_some());
        assert!(listed[0].get("platformLimitation").is_some());
    }

    // A deployment that serves everything must NOT advertise phantom gaps, and
    // the field disappears entirely rather than shipping an empty array.
    #[test]
    fn a_service_catalog_advertises_nothing_as_unavailable() {
        let catalog = ChatControlPlaneCatalog {
            agents: serde_json::json!({}),
            skills: serde_json::json!([{ "id": "s1" }]),
            tools: serde_json::json!([]),
            knowledge_bases: serde_json::json!([]),
            unavailable: Vec::new(),
        };
        let json = serde_json::to_value(&catalog).expect("serializes");
        assert!(
            json.get("unavailable").is_none(),
            "skip_serializing_if keeps the wire clean for deployments with no gaps"
        );
    }
}
