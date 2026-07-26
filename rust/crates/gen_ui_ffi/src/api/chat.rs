// TJ-ARCH-MOB-001 compliant
//! Chat + memory/graph-RAG intent surface. Dart sends a turn and folds the
//! resulting A2uiEvent stream (see streams::chat_events) into ContentBlocks.
//! Memory/graph functions are intent-level (`memory_search`, `graph_expand`) —
//! never raw SurrealQL. All three delegate to gen_ui_agent, the SAME
//! implementation tauri-plugin-gen-ui's commands call — no duplicated
//! business logic between the mobile and desktop leaves.
// `pub use` (not `use`) — frb_generated.rs re-exports this module's types via
// `use crate::api::chat::*`, which only sees items visible through a public path.
//
// frb's Result<T,E> detection only matches a literal `Result<...>` return
// type — it does NOT resolve through a generic type alias (verified against
// flutter_rust_bridge_codegen 2.12.0: its alias-parsing filter drops any
// `type Foo<T> = ...` with generics before resolution runs). Every
// frb-exposed fn below spells out `Result<T, CoreError>` literally; using
// `CoreResult<T>` here would silently make Dart receive an opaque blob with
// no field/error access instead of a normal Future<T> that throws on Err.
pub use gen_ui_db_graph::{MemoryHit, MemoryProvenance, RelatedEntity};
pub use gen_ui_types::inference::{InferenceMessage, InferenceRole};
pub use gen_ui_types::CoreError;

/// Decode a typed Rust failure for Flutter's error surfaces. FRB carries
/// workspace-owned error enums as opaque Rust objects, so Dart's default
/// `toString()` cannot expose the safe Display message without this adapter.
pub fn core_error_message(error: CoreError) -> String {
    error.to_string()
}

/// Start a chat turn; returns the run_id whose events arrive on chat_events(run_id).
/// `messages` is the selected conversation's role/text history. The thread id
/// remains transport metadata; persistence is owned by the entity repository.
pub async fn chat_send(
    thread_id: String,
    message: String,
    messages: Vec<InferenceMessage>,
    attachment_paths: Vec<String>,
    branch_id: String,
    user_message_id: String,
) -> Result<String, CoreError> {
    gen_ui_agent::chat::send_with_attachments(
        message,
        messages,
        attachment_paths,
        thread_id,
        branch_id,
        user_message_id,
    )
    .await
    .map_err(Into::into)
}

/// Submit a typed A2UI/AG-UI user interaction and start the agent's response.
/// The JSON envelope is preserved verbatim so Rust remains the sole transport
/// boundary and the agent can correlate surface, component, and context IDs.
pub async fn chat_action(
    thread_id: String,
    interaction_json: String,
    messages: Vec<InferenceMessage>,
    branch_id: String,
) -> Result<String, CoreError> {
    gen_ui_agent::chat::send_interaction(interaction_json, messages, thread_id, branch_id)
        .await
        .map_err(Into::into)
}

/// Approve a host-owned pending local tool call by approval id and start the
/// continuation run. This is intentionally separate from chat_action so the UI
/// cannot spoof tool execution through arbitrary interaction text.
pub async fn chat_approve_tool_confirmation(approval_id: String) -> Result<String, CoreError> {
    gen_ui_agent::chat::approve_tool_confirmation(approval_id)
        .await
        .map_err(Into::into)
}

/// Cancel an active run. Cancellation is cooperative: already received
/// semantic blocks remain persisted, and the stream terminates with a typed
/// `run_cancelled` event rather than fabricated assistant text.
pub fn chat_cancel(run_id: String) -> Result<bool, CoreError> {
    gen_ui_agent::state::cancel_run(&run_id).map_err(Into::into)
}

pub async fn prepare_local_model() -> Result<(), CoreError> {
    gen_ui_agent::chat::prepare_local_model()
        .await
        .map_err(Into::into)
}

pub fn local_model_status() -> Result<String, CoreError> {
    let status = gen_ui_agent::chat::local_model_status()?;
    serde_json::to_string(&status).map_err(|error| CoreError::Serde(error.to_string()))
}

pub fn cancel_model_operation() -> Result<(), CoreError> {
    gen_ui_agent::chat::cancel_model_operation().map_err(Into::into)
}

pub async fn local_runtime_diagnostics() -> Result<String, CoreError> {
    let diagnostics = gen_ui_agent::chat::local_runtime_diagnostics().await?;
    serde_json::to_string(&diagnostics).map_err(|error| CoreError::Serde(error.to_string()))
}

pub fn uar_runtime_status() -> Result<String, CoreError> {
    serde_json::to_string(&gen_ui_agent::uar::runtime_status())
        .map_err(|error| CoreError::Serde(error.to_string()))
}

/// Mobile/Flutter control-plane facade. JSON is intentional here: UAR owns
/// these extensible schemas and the client must preserve fields it does not yet
/// understand while Rust remains the only HTTP boundary.
pub async fn active_lane() -> Result<String, CoreError> {
    gen_ui_agent::chat::active_lane().await.map_err(Into::into)
}

pub async fn set_active_lane(lane: String) -> Result<(), CoreError> {
    gen_ui_agent::chat::set_active_lane(&lane)
        .await
        .map_err(Into::into)
}

/// Which lanes can be selected right now, and why not when they cannot.
///
/// The mode picker calls this to render unavailable modes as disabled with a
/// reason, instead of offering a control that fails when tapped. Side-effect
/// free: it probes registration state only and never loads an engine.
pub async fn lane_availability() -> Result<String, CoreError> {
    let value = gen_ui_agent::chat::lane_availability().await?;
    serde_json::to_string(&value).map_err(|error| CoreError::Serde(error.to_string()))
}

pub async fn uar_service_config() -> Result<String, CoreError> {
    let value = gen_ui_agent::chat::uar_service_config().await?;
    serde_json::to_string(&value).map_err(|error| CoreError::Serde(error.to_string()))
}

pub async fn uar_service_save(
    service_url: String,
    api_key: Option<String>,
    session_id: Option<String>,
) -> Result<(), CoreError> {
    gen_ui_agent::chat::save_uar_service_config(gen_ui_agent::chat::SaveUarServiceRequest {
        service_url,
        api_key,
        agent_artifact_json: String::new(),
        session_id,
    })
    .await
    .map_err(Into::into)
}

pub async fn uar_service_probe() -> Result<String, CoreError> {
    let value = gen_ui_agent::chat::probe_uar_service_config().await?;
    serde_json::to_string(&value).map_err(|error| CoreError::Serde(error.to_string()))
}

pub async fn chat_control_catalog() -> Result<String, CoreError> {
    let value = gen_ui_agent::control_plane::catalog().await?;
    serde_json::to_string(&value).map_err(|error| CoreError::Serde(error.to_string()))
}

pub async fn uar_admin_snapshot() -> Result<String, CoreError> {
    let value = gen_ui_agent::control_plane::admin_snapshot().await?;
    serde_json::to_string(&value).map_err(|error| CoreError::Serde(error.to_string()))
}

pub async fn provider_catalog() -> Result<String, CoreError> {
    let value = gen_ui_agent::provider_admin::catalog().await?;
    serde_json::to_string(&value).map_err(|error| CoreError::Serde(error.to_string()))
}

pub async fn provider_list() -> Result<String, CoreError> {
    let value = gen_ui_agent::provider_admin::list().await?;
    serde_json::to_string(&value).map_err(|error| CoreError::Serde(error.to_string()))
}

pub async fn provider_save(request_json: String) -> Result<(), CoreError> {
    let request =
        serde_json::from_str::<gen_ui_agent::provider_admin::SaveProviderRequest>(&request_json)
            .map_err(|error| CoreError::Serde(error.to_string()))?;
    gen_ui_agent::provider_admin::save(request)
        .await
        .map_err(Into::into)
}

pub async fn provider_delete(id: String) -> Result<(), CoreError> {
    gen_ui_agent::provider_admin::delete(&id)
        .await
        .map_err(Into::into)
}

pub async fn provider_test(id: String, model: Option<String>) -> Result<String, CoreError> {
    let value = gen_ui_agent::provider_admin::test(&id, model.as_deref()).await?;
    serde_json::to_string(&value).map_err(|error| CoreError::Serde(error.to_string()))
}

pub async fn cloud_model_get() -> Result<String, CoreError> {
    let value = gen_ui_agent::provider_admin::get_cloud_model().await?;
    serde_json::to_string(&value).map_err(|error| CoreError::Serde(error.to_string()))
}

pub async fn cloud_model_save(provider_id: String, model_id: String) -> Result<(), CoreError> {
    gen_ui_agent::provider_admin::save_cloud_model(
        gen_ui_agent::provider_admin::SaveCloudModelRequest {
            provider_id,
            model_id,
        },
    )
    .await
    .map_err(Into::into)
}

pub async fn uar_setting_save(key: String, value_json: String) -> Result<String, CoreError> {
    let value = parse_json(&value_json)?;
    let saved = gen_ui_agent::control_plane::update_setting(&key, &value).await?;
    serde_json::to_string(&saved).map_err(|error| CoreError::Serde(error.to_string()))
}

pub async fn uar_agent_save(agent_json: String) -> Result<String, CoreError> {
    let saved = gen_ui_agent::control_plane::save_agent(&parse_json(&agent_json)?).await?;
    serde_json::to_string(&saved).map_err(|error| CoreError::Serde(error.to_string()))
}

pub async fn uar_agent_delete(id: String) -> Result<(), CoreError> {
    gen_ui_agent::control_plane::delete_agent(&id)
        .await
        .map_err(Into::into)
}

pub async fn uar_mcp_server_save(name: String, server_json: String) -> Result<String, CoreError> {
    let saved =
        gen_ui_agent::control_plane::save_mcp_server(&name, &parse_json(&server_json)?).await?;
    serde_json::to_string(&saved).map_err(|error| CoreError::Serde(error.to_string()))
}

pub async fn uar_mcp_server_delete(name: String) -> Result<String, CoreError> {
    let value = gen_ui_agent::control_plane::delete_mcp_server(&name).await?;
    serde_json::to_string(&value).map_err(|error| CoreError::Serde(error.to_string()))
}

pub async fn uar_memory_delete(id: String) -> Result<String, CoreError> {
    let value = gen_ui_agent::control_plane::delete_memory(&id).await?;
    serde_json::to_string(&value).map_err(|error| CoreError::Serde(error.to_string()))
}

pub async fn uar_knowledge_base_create(request_json: String) -> Result<String, CoreError> {
    let value =
        gen_ui_agent::control_plane::create_knowledge_base(&parse_json(&request_json)?).await?;
    serde_json::to_string(&value).map_err(|error| CoreError::Serde(error.to_string()))
}

pub async fn uar_knowledge_base_delete(id: String) -> Result<(), CoreError> {
    gen_ui_agent::control_plane::delete_knowledge_base(&id)
        .await
        .map_err(Into::into)
}

pub async fn conversation_policy_get(conversation_id: String) -> Result<String, CoreError> {
    let value = gen_ui_agent::control_plane::conversation_policy(&conversation_id).await?;
    serde_json::to_string(&value).map_err(|error| CoreError::Serde(error.to_string()))
}

pub async fn conversation_policy_save(
    conversation_id: String,
    policy_json: String,
) -> Result<String, CoreError> {
    let value = gen_ui_agent::control_plane::save_conversation_policy(
        &conversation_id,
        &parse_json(&policy_json)?,
    )
    .await?;
    serde_json::to_string(&value).map_err(|error| CoreError::Serde(error.to_string()))
}

pub async fn conversation_policy_delete(conversation_id: String) -> Result<(), CoreError> {
    gen_ui_agent::control_plane::delete_conversation_policy(&conversation_id)
        .await
        .map_err(Into::into)
}

pub async fn conversation_effective_config(conversation_id: String) -> Result<String, CoreError> {
    let value = gen_ui_agent::control_plane::effective_config(&conversation_id).await?;
    serde_json::to_string(&value).map_err(|error| CoreError::Serde(error.to_string()))
}

fn parse_json(value: &str) -> Result<serde_json::Value, CoreError> {
    serde_json::from_str(value).map_err(|error| CoreError::Serde(error.to_string()))
}

/// The local lane's system prompt, or the shipped default when unset.
pub async fn local_system_prompt() -> Result<String, CoreError> {
    gen_ui_agent::chat::local_system_prompt()
        .await
        .map_err(|error| {
            tracing::error!(%error, "local system prompt read failed");
            error.into()
        })
}

/// Persist the local lane's system prompt; a blank value restores the default.
pub async fn set_local_system_prompt(prompt: String) -> Result<(), CoreError> {
    gen_ui_agent::chat::set_local_system_prompt(&prompt)
        .await
        .map_err(|error| {
            tracing::error!(%error, "local system prompt write failed");
            error.into()
        })
}

/// The shipped default, so Settings can offer "reset" without hardcoding copy
/// that would drift from Rust.
pub fn default_local_system_prompt() -> String {
    gen_ui_agent::chat::DEFAULT_LOCAL_SYSTEM_PROMPT.to_string()
}

pub async fn theme_mode() -> Result<String, CoreError> {
    gen_ui_agent::chat::theme_mode().await.map_err(|error| {
        tracing::error!(%error, "theme preference read failed");
        error.into()
    })
}

pub async fn set_theme_mode(mode: String) -> Result<(), CoreError> {
    gen_ui_agent::chat::set_theme_mode(&mode)
        .await
        .map_err(|error| {
            tracing::error!(%error, %mode, "theme preference write failed");
            error.into()
        })
}

pub async fn resolve_remote_content(url: String, kind: String) -> Result<String, CoreError> {
    let resolved = gen_ui_agent::media::resolve(&url, &kind).await?;
    serde_json::to_string(&resolved).map_err(|error| CoreError::Serde(error.to_string()))
}

/// Ingest a note into memory (embeds on-device via fastembed, upserts into
/// SurrealDB). Returns the assigned record id.
pub async fn memory_ingest(text: String) -> Result<String, CoreError> {
    gen_ui_agent::memory::ingest(text).await.map_err(Into::into)
}

/// Extract a document with xberg and ingest its normalized content into graph-RAG.
pub async fn memory_ingest_attachment(path: String) -> Result<String, CoreError> {
    gen_ui_agent::memory::ingest_attachment(path)
        .await
        .map_err(Into::into)
}

/// Extract and ingest up to six documents with xberg, returning record ids in order.
pub async fn memory_ingest_attachments(paths: Vec<String>) -> Result<Vec<String>, CoreError> {
    gen_ui_agent::memory::ingest_attachments(paths)
        .await
        .map_err(Into::into)
}

/// Hybrid memory search (vector recall + graph expansion + BM25, RRF-fused in Rust).
pub async fn memory_search(query: String, k: u32) -> Result<Vec<MemoryHit>, CoreError> {
    gen_ui_agent::memory::search(query, k)
        .await
        .map_err(Into::into)
}

pub async fn memory_delete_conversation_context(
    conversation_id: String,
) -> Result<usize, CoreError> {
    gen_ui_agent::memory::delete_conversation_context(conversation_id)
        .await
        .map_err(Into::into)
}

pub async fn memory_delete_message_context(message_ids: Vec<String>) -> Result<usize, CoreError> {
    gen_ui_agent::memory::delete_message_context(message_ids)
        .await
        .map_err(Into::into)
}

pub async fn memory_promote_context(ids: Vec<String>) -> Result<usize, CoreError> {
    gen_ui_agent::memory::promote_context(ids)
        .await
        .map_err(Into::into)
}

/// Expand the entity graph around a node to a given depth.
pub async fn graph_expand(entity_id: String, depth: u32) -> Result<Vec<RelatedEntity>, CoreError> {
    gen_ui_agent::memory::graph_expand(entity_id, depth)
        .await
        .map_err(Into::into)
}
