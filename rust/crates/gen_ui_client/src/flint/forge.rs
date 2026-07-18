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
use gen_ui_types::transport::{EntityRecord, EntityTransport, ListResult};
use gen_ui_types::view::{FilterOp, ViewDescriptor};
use gen_ui_types::{CoreError, CoreResult};
use serde::Deserialize;
use std::sync::Arc;

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

    fn bearer(&self) -> Option<String> {
        self.auth.read().bearer().map(str::to_owned)
    }

    /// Register forge's A2UI registry as an MCP server into the shared registry.
    /// The gate JWT is snapshotted at registration; callers re-register after a
    /// long-lived token rotation (tokens are short-lived — see token.rs).
    pub fn register_a2ui_mcp(&self, registry: &McpRegistry) -> Arc<McpServerHandle> {
        let endpoint = format!("{}/mcp/v1/a2ui", self.config.base.trim_end_matches('/'));
        let transport = SseTransport::new(self.http.clone(), endpoint, self.bearer());
        registry.register(McpServerHandle::new(
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
// We translate the subset the ContentBlock contract needs into our A2uiEvent surface
// so the ProtocolPipeline (gen_ui_protocol) folds them into ContentBlocks unchanged.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum AgUiEvent {
    RunStarted {
        #[serde(default)]
        run_id: String,
    },
    TextMessageContent {
        #[serde(default)]
        delta: String,
    },
    ToolCallStart {
        #[serde(default)]
        tool_call_id: String,
        #[serde(default)]
        tool_name: String,
    },
    RunFinished {
        #[serde(default)]
        run_id: String,
    },
    RunError {
        #[serde(default)]
        message: String,
    },
    #[serde(other)]
    Other,
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
                block: ContentBlock::Text {
                    text: delta.clone(),
                },
            }]
        }
        AgUiEvent::ToolCallStart {
            tool_call_id,
            tool_name,
        } => vec![A2uiEvent::Block {
            block: ContentBlock::ToolUse {
                id: tool_call_id.clone(),
                name: tool_name.clone(),
                input_json: "{}".into(),
            },
        }],
        AgUiEvent::RunFinished { run_id } => vec![A2uiEvent::RunFinished {
            run_id: run_id.clone(),
        }],
        AgUiEvent::RunError { message } => vec![A2uiEvent::RunError {
            message: message.clone(),
        }],
        AgUiEvent::Other => vec![],
    }
}

/// Parse one SSE `data:` payload (a JSON AgUiEvent) into A2UI events. Returns an empty
/// vec for keep-alives / unparseable frames rather than erroring the whole stream.
pub fn parse_agui_frame(data: &str) -> Vec<A2uiEvent> {
    match serde_json::from_str::<AgUiEvent>(data) {
        Ok(ev) => agui_to_a2ui(&ev),
        Err(_) => vec![],
    }
}
