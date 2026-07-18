// TJ-ARCH-MOB-001 compliant
//! MCP server registry. One [`McpRegistry`] per process holds the connected
//! servers keyed by name; `tools/list` results are cached per server so the agent
//! loop can enumerate available tools without a round-trip on every turn.
use async_trait::async_trait;
use gen_ui_types::CoreResult;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// The transport seam a registered MCP server speaks over. Object-safe so the
/// registry holds `Box<dyn McpTransport>`. The HTTP+SSE impl ([`super::SseTransport`])
/// is native-only (reqwest wasm response is !Send); a wasm build registers no
/// concrete transport (the browser drives MCP from JS).
#[async_trait]
pub trait McpTransport: Send + Sync {
    async fn request(
        &self,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> CoreResult<serde_json::Value>;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct McpTool {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    /// JSON-Schema of the tool's input (`inputSchema` in the MCP spec).
    #[serde(default, rename = "inputSchema")]
    pub input_schema: serde_json::Value,
}

/// A connected MCP server: the transport plus its last-known tool inventory.
pub struct McpServerHandle {
    pub name: String,
    transport: Box<dyn McpTransport>,
    tools: RwLock<Vec<McpTool>>,
}

impl McpServerHandle {
    pub fn new(name: impl Into<String>, transport: Box<dyn McpTransport>) -> Self {
        Self {
            name: name.into(),
            transport,
            tools: RwLock::new(Vec::new()),
        }
    }

    /// Refresh the cached tool inventory via `tools/list`.
    pub async fn refresh_tools(&self) -> CoreResult<Vec<McpTool>> {
        let value = self.transport.request("tools/list", None).await?;
        let tools: Vec<McpTool> = value
            .get("tools")
            .and_then(|t| serde_json::from_value(t.clone()).ok())
            .unwrap_or_default();
        *self.tools.write() = tools.clone();
        Ok(tools)
    }

    /// Invoke a tool via `tools/call`.
    pub async fn call_tool(
        &self,
        name: &str,
        arguments: serde_json::Value,
    ) -> CoreResult<serde_json::Value> {
        self.transport
            .request(
                "tools/call",
                Some(serde_json::json!({ "name": name, "arguments": arguments })),
            )
            .await
    }

    pub fn cached_tools(&self) -> Vec<McpTool> {
        self.tools.read().clone()
    }
}

#[derive(Default, Clone)]
pub struct McpRegistry {
    servers: Arc<RwLock<HashMap<String, Arc<McpServerHandle>>>>,
}

impl McpRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a server (e.g. flint-forge's `/mcp/v1/a2ui`) under its name.
    pub fn register(&self, handle: McpServerHandle) -> Arc<McpServerHandle> {
        let handle = Arc::new(handle);
        self.servers
            .write()
            .insert(handle.name.clone(), handle.clone());
        handle
    }

    pub fn get(&self, name: &str) -> Option<Arc<McpServerHandle>> {
        self.servers.read().get(name).cloned()
    }

    pub fn server_names(&self) -> Vec<String> {
        self.servers.read().keys().cloned().collect()
    }
}
