// TJ-ARCH-MOB-001 compliant
//! flint-forge client — Quarry data plane, A2UI-registry MCP server, AG-UI runs.
//!
//! VERIFIED contract (forge HEAD 2026-07-15, binary `fdb-gateway`, default :8080):
//!  * Quarry REST paths are `/<schema>/<table>` (PostgREST grammar), NOT `/rest/v1`.
//!    RLS tenant scoping rides the Bearer JWT ONLY (no X-Tenant header).
//!  * A2UI registry is an MCP server at `POST /mcp/v1/a2ui` (JSON-RPC 2.0) with a
//!    keep-alive SSE at `/mcp/v1/a2ui/sse`. We register it into gen_ui_mcp::McpRegistry.
//!  * AG-UI runs: `POST /agents/v1/runs` → `GET /agents/v1/{run_id}/events` (SSE).
//!    Event frames are `event:<name>` + `data:<AgUiEvent json>` tagged by `type`.
use crate::flint::token::AuthState;
use gen_ui_mcp::{McpRegistry, McpServerHandle, SseTransport};
use gen_ui_types::content_block::ContentBlock;
use gen_ui_types::events::A2uiEvent;
use gen_ui_types::lifecycle::{RunError, RunPhase};
use gen_ui_types::transport::{EntityRecord, EntityTransport, ListResult};
use gen_ui_types::view::{FilterOp, ViewDescriptor};
use gen_ui_types::{CoreError, CoreResult};
use serde::Deserialize;
use std::sync::Arc;

#[path = "forge_mutations.rs"]
mod mutations;

#[derive(Debug, Clone)]
pub struct ForgeConfig {
    /// e.g. `http://localhost:8080`.
    pub base: String,
    /// Postgres schema the entity tables live in (default `public`).
    pub schema: String,
}

impl Default for ForgeConfig {
    fn default() -> Self {
        Self {
            base: "http://localhost:8080".into(),
            schema: "public".into(),
        }
    }
}

/// Shared auth handle so a refreshed token is seen by every request without rebuilding
/// the client. The gate/agent layer swaps the inner state; forge reads the Bearer.
pub type SharedAuth = Arc<parking_lot::RwLock<AuthState>>;

pub struct ForgeClient {
    http: reqwest::Client,
    config: ForgeConfig,
    auth: SharedAuth,
}

impl ForgeClient {
    pub fn new(http: reqwest::Client, config: ForgeConfig, auth: SharedAuth) -> Self {
        Self { http, config, auth }
    }

    /// Native consumers share the authenticated mutation client without owning its lock.
    pub fn authenticated(config: ForgeConfig, bearer: String) -> CoreResult<Self> {
        let token = crate::flint::Token::parse(bearer)?;
        Ok(Self::new(reqwest::Client::new(), config,
            Arc::new(parking_lot::RwLock::new(AuthState::Authenticated { token }))))
    }

    fn bearer(&self) -> Option<String> {
        self.auth.read().bearer().map(str::to_owned)
    }

    /// Register forge's A2UI registry as an MCP server into the shared registry.
    /// The gate JWT is snapshotted at registration; callers re-register after a
    /// long-lived token rotation (tokens are short-lived — see token.rs).
    pub fn register_a2ui_mcp(&self, registry: &McpRegistry) -> Arc<McpServerHandle> {
        let endpoint = format!("{}/mcp/v1/a2ui", self.config.base.trim_end_matches('/'));
        let transport = SseTransport::new(self.http.clone(), endpoint, self.bearer());
        registry.register(McpServerHandle::new_trusted_local(
            "flint-a2ui-registry",
            Box::new(transport),
        ))
    }

    fn table_url(&self, entity_type: &str) -> String {
        format!(
            "{}/{}/{}",
            self.config.base.trim_end_matches('/'),
            self.config.schema,
            entity_type
        )
    }

    fn apply_bearer(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        match self.bearer() {
            Some(token) => req.bearer_auth(token),
            None => req,
        }
    }
}

/// PostgREST operator string for a FilterOp (`?field=eq.value`).
fn postgrest_op(op: FilterOp) -> &'static str {
    match op {
        FilterOp::Eq => "eq",
        FilterOp::Ne => "neq",
        FilterOp::Lt => "lt",
        FilterOp::Lte => "lte",
        FilterOp::Gt => "gt",
        FilterOp::Gte => "gte",
        FilterOp::In => "in",
        FilterOp::Like => "like",
    }
}

#[async_trait::async_trait]
impl EntityTransport for ForgeClient {
    async fn mutate_idempotent(&self, operation: &str, record: &EntityRecord, key: &str) -> CoreResult<Option<EntityRecord>> {
        self.replay_mutation(operation, record, key).await
    }

    async fn list(&self, view: &ViewDescriptor) -> CoreResult<ListResult> {
        let mut req = self.apply_bearer(self.http.get(self.table_url(&view.entity_type)));
        // PostgREST filter grammar: one query pair per filter.
        for f in &view.filters {
            let raw: serde_json::Value = serde_json::from_str(&f.value_json)
                .unwrap_or(serde_json::Value::String(f.value_json.clone()));
            let val = raw
                .as_str()
                .map(str::to_owned)
                .unwrap_or_else(|| raw.to_string());
            req = req.query(&[(f.field.as_str(), format!("{}.{}", postgrest_op(f.op), val))]);
        }
        if !view.sorts.is_empty() {
            let order = view
                .sorts
                .iter()
                .map(|s| format!("{}.{}", s.field, if s.descending { "desc" } else { "asc" }))
                .collect::<Vec<_>>()
                .join(",");
            req = req.query(&[("order", order)]);
        }
        if let Some(limit) = view.limit {
            req = req.query(&[("limit", limit.to_string())]);
        }
        let resp = req
            .send()
            .await
            .map_err(|e| CoreError::Transient(e.to_string()))?;
        if !resp.status().is_success() {
            return Err(map_status(resp.status()));
        }
        let rows: Vec<serde_json::Value> = resp
            .json()
            .await
            .map_err(|e| CoreError::Serde(e.to_string()))?;
        let items = rows
            .into_iter()
            .map(|row| row_to_record(&view.entity_type, row))
            .collect();
        Ok(ListResult {
            items,
            next_cursor: None,
        })
    }

    async fn get(&self, entity_type: &str, id: &str) -> CoreResult<Option<EntityRecord>> {
        let req = self
            .apply_bearer(self.http.get(self.table_url(entity_type)))
            .query(&[("id", format!("eq.{id}")), ("limit", "1".into())]);
        let resp = req
            .send()
            .await
            .map_err(|e| CoreError::Transient(e.to_string()))?;
        if !resp.status().is_success() {
            return Err(map_status(resp.status()));
        }
        let mut rows: Vec<serde_json::Value> = resp
            .json()
            .await
            .map_err(|e| CoreError::Serde(e.to_string()))?;
        Ok(rows.pop().map(|row| row_to_record(entity_type, row)))
    }

    async fn create(&self, record: &EntityRecord) -> CoreResult<EntityRecord> {
        let value: serde_json::Value =
            serde_json::from_str(&record.data_json).map_err(|e| CoreError::Serde(e.to_string()))?;
        let resp = self
            .apply_bearer(self.http.post(self.table_url(&record.entity_type)))
            .header("Prefer", "return=representation")
            .json(&value)
            .send()
            .await
            .map_err(|e| CoreError::Transient(e.to_string()))?;
        if !resp.status().is_success() {
            return Err(map_status(resp.status()));
        }
        let mut rows: Vec<serde_json::Value> = resp
            .json()
            .await
            .map_err(|e| CoreError::Serde(e.to_string()))?;
        let row = rows
            .pop()
            .ok_or_else(|| CoreError::Terminal("forge create: empty representation".into()))?;
        Ok(row_to_record(&record.entity_type, row))
    }

    async fn update(&self, record: &EntityRecord) -> CoreResult<EntityRecord> {
        let value: serde_json::Value =
            serde_json::from_str(&record.data_json).map_err(|e| CoreError::Serde(e.to_string()))?;
        let url = format!("{}/{}", self.table_url(&record.entity_type), record.id);
        let resp = self
            .apply_bearer(self.http.patch(url))
            .header("Prefer", "return=representation")
            .json(&value)
            .send()
            .await
            .map_err(|e| CoreError::Transient(e.to_string()))?;
        if !resp.status().is_success() {
            return Err(map_status(resp.status()));
        }
        let mut rows: Vec<serde_json::Value> = resp
            .json()
            .await
            .map_err(|e| CoreError::Serde(e.to_string()))?;
        let row = rows
            .pop()
            .ok_or_else(|| CoreError::Terminal("forge update: empty representation".into()))?;
        Ok(row_to_record(&record.entity_type, row))
    }

    async fn delete(&self, entity_type: &str, id: &str) -> CoreResult<()> {
        let url = format!("{}/{}", self.table_url(entity_type), id);
        let resp = self
            .apply_bearer(self.http.delete(url))
            .send()
            .await
            .map_err(|e| CoreError::Transient(e.to_string()))?;
        if !resp.status().is_success() {
            return Err(map_status(resp.status()));
        }
        Ok(())
    }
}

fn row_to_record(entity_type: &str, row: serde_json::Value) -> EntityRecord {
    let id = row
        .get("id")
        .map(|v| {
            v.as_str()
                .map(str::to_owned)
                .unwrap_or_else(|| v.to_string())
        })
        .unwrap_or_default();
    EntityRecord {
        id,
        entity_type: entity_type.to_owned(),
        data_json: row.to_string(),
    }
}

fn map_status(status: reqwest::StatusCode) -> CoreError {
    use reqwest::StatusCode;
    match status {
        StatusCode::CONFLICT | StatusCode::PRECONDITION_FAILED => {
            CoreError::Terminal("conflict: authoritative mutation precondition was refused".into())
        }
        StatusCode::NOT_FOUND => CoreError::NotFound(status.to_string()),
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
            CoreError::Terminal(format!("forge auth: {status}"))
        }
        s if s.is_server_error() || s == StatusCode::TOO_MANY_REQUESTS => {
            CoreError::Transient(s.to_string())
        }
        s => CoreError::Terminal(format!("forge http {s}")),
    }
}

// ── AG-UI event mapping ─────────────────────────────────────────────────────
// forge emits internally-tagged `AgUiEvent` (`{"type":"TextMessageContent",...}`).
// We project the standard lifecycle, text, tool, state, activity, reasoning, raw,
// and custom families into our A2uiEvent/ContentBlock surface.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum AgUiEvent {
    #[serde(alias = "RUN_STARTED")]
    RunStarted {
        #[serde(default, alias = "runId")]
        run_id: String,
    },
    #[serde(alias = "TEXT_MESSAGE_START")]
    TextMessageStart,
    #[serde(alias = "TEXT_MESSAGE_CONTENT")]
    TextMessageContent {
        #[serde(default)]
        delta: String,
    },
    #[serde(alias = "TEXT_MESSAGE_END")]
    TextMessageEnd,
    #[serde(alias = "TOOL_CALL_START")]
    ToolCallStart {
        #[serde(default, alias = "toolCallId")]
        tool_call_id: String,
        #[serde(default, alias = "toolCallName", alias = "toolName")]
        tool_name: String,
    },
    #[serde(alias = "TOOL_CALL_ARGS")]
    ToolCallArgs {
        #[serde(default, alias = "toolCallId")]
        tool_call_id: String,
        #[serde(default)]
        delta: String,
    },
    #[serde(alias = "TOOL_CALL_END")]
    ToolCallEnd {
        #[serde(default, alias = "toolCallId")]
        tool_call_id: String,
    },
    #[serde(alias = "TOOL_CALL_RESULT")]
    ToolCallResult {
        #[serde(default, alias = "toolCallId")]
        tool_call_id: String,
        #[serde(default)]
        content: String,
    },
    #[serde(
        alias = "REASONING_MESSAGE_CONTENT",
        alias = "THINKING_TEXT_MESSAGE_CONTENT"
    )]
    ReasoningMessageContent {
        #[serde(default)]
        delta: String,
    },
    #[serde(alias = "REASONING_MESSAGE_CHUNK")]
    ReasoningMessageChunk {
        #[serde(default)]
        delta: String,
    },
    #[serde(alias = "REASONING_START", alias = "REASONING_MESSAGE_START")]
    ReasoningStart,
    #[serde(
        alias = "REASONING_END",
        alias = "REASONING_MESSAGE_END",
        alias = "THINKING_END",
        alias = "THINKING_TEXT_MESSAGE_END"
    )]
    ReasoningEnd,
    #[serde(alias = "STATE_SNAPSHOT")]
    StateSnapshot { snapshot: serde_json::Value },
    #[serde(alias = "STATE_DELTA")]
    StateDelta { delta: serde_json::Value },
    #[serde(alias = "MESSAGES_SNAPSHOT")]
    MessagesSnapshot { messages: serde_json::Value },
    #[serde(alias = "ACTIVITY_SNAPSHOT")]
    ActivitySnapshot {
        #[serde(default, alias = "messageId")]
        message_id: String,
        #[serde(default, alias = "activityType")]
        activity_type: String,
        content: serde_json::Value,
    },
    #[serde(alias = "ACTIVITY_DELTA")]
    ActivityDelta {
        #[serde(default, alias = "messageId")]
        message_id: String,
        #[serde(default, alias = "activityType")]
        activity_type: String,
        patch: serde_json::Value,
    },
    #[serde(alias = "STEP_STARTED")]
    StepStarted {
        #[serde(default, alias = "stepName")]
        step_name: String,
    },
    #[serde(alias = "STEP_FINISHED")]
    StepFinished {
        #[serde(default, alias = "stepName")]
        step_name: String,
    },
    #[serde(alias = "CUSTOM")]
    Custom {
        #[serde(default)]
        name: String,
        value: serde_json::Value,
    },
    #[serde(alias = "RAW")]
    Raw {
        event: serde_json::Value,
        #[serde(default)]
        source: Option<String>,
    },
    #[serde(alias = "RUN_FINISHED")]
    RunFinished {
        #[serde(default, alias = "runId")]
        run_id: String,
    },
    #[serde(alias = "RUN_ERROR")]
    RunError {
        #[serde(default)]
        message: String,
    },
    #[serde(other)]
    Other,
}

/// Stateful AG-UI stream fold. Tool arguments, RFC 6902 state deltas, and
/// activity deltas are ordered stream protocols and must never be interpreted
/// one frame at a time.
#[derive(Default)]
pub struct AguiStreamAdapter {
    run_id: String,
    pending_tools: std::collections::BTreeMap<String, (String, String)>,
    state: serde_json::Value,
    activities: std::collections::BTreeMap<String, serde_json::Value>,
}

impl AguiStreamAdapter {
    pub fn ingest(&mut self, event: &AgUiEvent) -> Vec<A2uiEvent> {
        match event {
            AgUiEvent::RunStarted { run_id } => {
                self.run_id.clone_from(run_id);
                agui_to_a2ui(event)
            }
            AgUiEvent::ToolCallStart {
                tool_call_id,
                tool_name,
            } => {
                self.pending_tools
                    .insert(tool_call_id.clone(), (tool_name.clone(), String::new()));
                vec![]
            }
            AgUiEvent::ToolCallArgs {
                tool_call_id,
                delta,
            } => {
                if let Some((_, arguments)) = self.pending_tools.get_mut(tool_call_id) {
                    arguments.push_str(delta);
                }
                vec![]
            }
            AgUiEvent::ToolCallEnd { tool_call_id } => {
                let Some((name, arguments)) = self.pending_tools.remove(tool_call_id) else {
                    return vec![];
                };
                vec![A2uiEvent::Block {
                    run_id: self.run_id.clone(),
                    block: ContentBlock::ToolUse {
                        id: tool_call_id.clone(),
                        name,
                        input_json: arguments,
                    },
                }]
            }
            AgUiEvent::StateSnapshot { snapshot } => {
                self.state = snapshot.clone();
                self.scoped(agui_to_a2ui(event))
            }
            AgUiEvent::StateDelta { delta } => {
                let Ok(patch) = serde_json::from_value::<json_patch::Patch>(delta.clone()) else {
                    return vec![self.protocol_error("Invalid AG-UI state patch")];
                };
                if json_patch::patch(&mut self.state, &patch).is_err() {
                    return vec![self.protocol_error("AG-UI state patch could not be applied")];
                }
                vec![artifact_block_for_run(
                    &self.run_id,
                    "agui-state",
                    "agui_state_snapshot",
                    &self.state,
                )]
            }
            AgUiEvent::ActivitySnapshot {
                message_id,
                content,
                ..
            } => {
                self.activities.insert(message_id.clone(), content.clone());
                self.scoped(agui_to_a2ui(event))
            }
            AgUiEvent::ActivityDelta {
                message_id,
                activity_type,
                patch,
            } => {
                let activity = self
                    .activities
                    .entry(message_id.clone())
                    .or_insert(serde_json::Value::Null);
                let Ok(patch_value) = serde_json::from_value::<json_patch::Patch>(patch.clone())
                else {
                    return vec![self.protocol_error("Invalid AG-UI activity patch")];
                };
                if json_patch::patch(activity, &patch_value).is_err() {
                    return vec![self.protocol_error("AG-UI activity patch could not be applied")];
                }
                vec![artifact_block_for_run(
                    &self.run_id,
                    message_id,
                    &format!("agui_activity_{}", activity_type.to_lowercase()),
                    activity,
                )]
            }
            _ => self.scoped(agui_to_a2ui(event)),
        }
    }

    pub fn parse_frame(&mut self, data: &str) -> Vec<A2uiEvent> {
        serde_json::from_str::<AgUiEvent>(data)
            .map(|event| self.ingest(&event))
            .unwrap_or_default()
    }

    fn scoped(&self, events: Vec<A2uiEvent>) -> Vec<A2uiEvent> {
        events
            .into_iter()
            .map(|event| match event {
                A2uiEvent::RunStarted { run_id } => A2uiEvent::RunStarted { run_id },
                A2uiEvent::RunPhaseChanged {
                    phase,
                    status,
                    message,
                    ..
                } => A2uiEvent::RunPhaseChanged {
                    run_id: self.run_id.clone(),
                    phase,
                    status,
                    message,
                },
                A2uiEvent::Block { block, .. } => A2uiEvent::Block {
                    run_id: self.run_id.clone(),
                    block,
                },
                A2uiEvent::RunFinished { run_id } => A2uiEvent::RunFinished { run_id },
                A2uiEvent::RunCancelled { phase, message, .. } => A2uiEvent::RunCancelled {
                    run_id: self.run_id.clone(),
                    phase,
                    message,
                },
                A2uiEvent::RunError { error, .. } => A2uiEvent::RunError {
                    run_id: self.run_id.clone(),
                    error,
                },
                A2uiEvent::ContextTelemetry { payload, .. } => A2uiEvent::ContextTelemetry {
                    run_id: self.run_id.clone(),
                    payload,
                },
            })
            .collect()
    }

    fn protocol_error(&self, message: &str) -> A2uiEvent {
        A2uiEvent::RunError {
            run_id: self.run_id.clone(),
            error: RunError::protocol(message),
        }
    }
}

/// Map a forge AG-UI event to zero or more of our A2UI events. Unhandled variants
/// (state deltas, custom surfaces) yield nothing here and are handled by the A2UI
/// surface layer directly — this path only feeds the streaming ContentBlock fold.
pub fn agui_to_a2ui(ev: &AgUiEvent) -> Vec<A2uiEvent> {
    match ev {
        AgUiEvent::RunStarted { run_id } => vec![A2uiEvent::RunStarted {
            run_id: run_id.clone(),
        }],
        AgUiEvent::TextMessageContent { delta } => {
            vec![A2uiEvent::Block {
                run_id: String::new(),
                block: ContentBlock::Text {
                    text: delta.clone(),
                },
            }]
        }
        AgUiEvent::ToolCallStart {
            tool_call_id,
            tool_name,
        } => vec![A2uiEvent::Block {
            run_id: String::new(),
            block: ContentBlock::ToolUse {
                id: tool_call_id.clone(),
                name: tool_name.clone(),
                input_json: "{}".into(),
            },
        }],
        AgUiEvent::ToolCallResult {
            tool_call_id,
            content,
        } => vec![A2uiEvent::Block {
            run_id: String::new(),
            block: ContentBlock::ToolResult {
                tool_use_id: tool_call_id.clone(),
                output_json: content.clone(),
                is_error: false,
            },
        }],
        AgUiEvent::ReasoningMessageContent { delta }
        | AgUiEvent::ReasoningMessageChunk { delta } => vec![A2uiEvent::Block {
            run_id: String::new(),
            block: ContentBlock::Thinking {
                text: delta.clone(),
            },
        }],
        AgUiEvent::StateSnapshot { snapshot } => vec![artifact_block(
            "agui-state",
            "agui_state_snapshot",
            snapshot,
        )],
        AgUiEvent::StateDelta { delta } => vec![artifact_block(
            "agui-state-delta",
            "agui_state_delta",
            delta,
        )],
        AgUiEvent::MessagesSnapshot { messages } => vec![artifact_block(
            "agui-messages",
            "agui_messages_snapshot",
            messages,
        )],
        AgUiEvent::ActivitySnapshot {
            message_id,
            activity_type,
            content,
        } => vec![artifact_block(
            message_id,
            &format!("agui_activity_{}", activity_type.to_lowercase()),
            content,
        )],
        AgUiEvent::ActivityDelta {
            message_id,
            activity_type,
            patch,
        } => vec![artifact_block(
            message_id,
            &format!("agui_activity_{}_delta", activity_type.to_lowercase()),
            patch,
        )],
        AgUiEvent::StepStarted { step_name } => vec![A2uiEvent::Block {
            run_id: String::new(),
            block: ContentBlock::Skill {
                name: step_name.clone(),
                status: "running".into(),
            },
        }],
        AgUiEvent::StepFinished { step_name } => vec![A2uiEvent::Block {
            run_id: String::new(),
            block: ContentBlock::Skill {
                name: step_name.clone(),
                status: "completed".into(),
            },
        }],
        AgUiEvent::Custom { name, value } => custom_to_a2ui(name, value),
        AgUiEvent::Raw { event, source } => vec![artifact_block(
            source.as_deref().unwrap_or("agui-raw"),
            "agui_raw",
            event,
        )],
        AgUiEvent::RunFinished { run_id } => vec![A2uiEvent::RunFinished {
            run_id: run_id.clone(),
        }],
        AgUiEvent::RunError { message } => vec![A2uiEvent::RunError {
            run_id: String::new(),
            error: RunError {
                code: "provider_error".into(),
                phase: RunPhase::Generation,
                message: message.clone(),
                retryable: true,
                diagnostics: None,
            },
        }],
        AgUiEvent::TextMessageStart
        | AgUiEvent::TextMessageEnd
        | AgUiEvent::ToolCallEnd { .. }
        | AgUiEvent::Other => vec![],
        AgUiEvent::ToolCallArgs {
            tool_call_id,
            delta,
        } => vec![artifact_block(
            tool_call_id,
            "agui_tool_call_args_delta",
            &serde_json::Value::String(delta.clone()),
        )],
        AgUiEvent::ReasoningStart => vec![A2uiEvent::Block {
            run_id: String::new(),
            block: ContentBlock::Skill {
                name: "Reasoning".into(),
                status: "running".into(),
            },
        }],
        AgUiEvent::ReasoningEnd => vec![A2uiEvent::Block {
            run_id: String::new(),
            block: ContentBlock::Skill {
                name: "Reasoning".into(),
                status: "completed".into(),
            },
        }],
    }
}

fn artifact_block(id: &str, kind: &str, value: &serde_json::Value) -> A2uiEvent {
    artifact_block_for_run("", id, kind, value)
}

fn artifact_block_for_run(
    run_id: &str,
    id: &str,
    kind: &str,
    value: &serde_json::Value,
) -> A2uiEvent {
    A2uiEvent::Block {
        run_id: run_id.into(),
        block: ContentBlock::Artifact {
            id: id.into(),
            kind: kind.into(),
            content: serde_json::to_string(value).unwrap_or_else(|_| "null".into()),
        },
    }
}

fn custom_to_a2ui(name: &str, value: &serde_json::Value) -> Vec<A2uiEvent> {
    let string = |key: &str| {
        value
            .get(key)
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_owned()
    };
    let block = match name.to_ascii_lowercase().as_str() {
        "citation" => ContentBlock::Citation {
            source: string("source"),
            quote: string("quote"),
        },
        "memory" => ContentBlock::Memory {
            operation: string("operation"),
            key: string("key"),
            value: value
                .get("value")
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned),
        },
        "a2ui" | "a2ui_surface" => ContentBlock::Artifact {
            id: string("surfaceId"),
            kind: "a2ui".into(),
            content: value
                .get("messages")
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned)
                .unwrap_or_else(|| serde_json::to_string(value).unwrap_or_default()),
        },
        "confirmation" | "confirmation_request" => ContentBlock::Artifact {
            id: string("id"),
            kind: "agui_confirmation".into(),
            content: serde_json::to_string(value).unwrap_or_default(),
        },
        other => ContentBlock::Artifact {
            id: name.into(),
            kind: format!("agui_custom_{other}"),
            content: serde_json::to_string(value).unwrap_or_default(),
        },
    };
    vec![A2uiEvent::Block {
        run_id: String::new(),
        block,
    }]
}

/// Parse one SSE `data:` payload (a JSON AgUiEvent) into A2UI events. Returns an empty
/// vec for keep-alives / unparseable frames rather than erroring the whole stream.
pub fn parse_agui_frame(data: &str) -> Vec<A2uiEvent> {
    match serde_json::from_str::<AgUiEvent>(data) {
        Ok(ev) => agui_to_a2ui(&ev),
        Err(_) => vec![],
    }
}
