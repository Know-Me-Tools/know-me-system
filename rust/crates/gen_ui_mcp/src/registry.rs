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
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    /// JSON-Schema of the tool's input (`inputSchema` in the MCP spec).
    #[serde(default, rename = "inputSchema")]
    pub input_schema: serde_json::Value,
    #[serde(default)]
    pub annotations: Option<McpToolAnnotations>,
    #[serde(default)]
    pub execution: Option<McpToolExecution>,
    #[serde(default, rename = "outputSchema")]
    pub output_schema: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct McpToolAnnotations {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub read_only_hint: Option<bool>,
    #[serde(default)]
    pub destructive_hint: Option<bool>,
    #[serde(default)]
    pub idempotent_hint: Option<bool>,
    #[serde(default)]
    pub open_world_hint: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct McpToolExecution {
    #[serde(default, rename = "taskSupport")]
    pub task_support: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum McpServerTrust {
    /// Default for externally registered MCP servers. Tool annotations from
    /// these servers are still preserved, but they are not enough to permit
    /// local automatic execution.
    Untrusted,
    /// The host registered this server as part of KnowMe's own trusted runtime
    /// boundary. Individual tools still need read-only/non-destructive/
    /// closed-world annotations before local automatic execution is allowed.
    TrustedLocal,
}

impl Default for McpServerTrust {
    fn default() -> Self {
        Self::Untrusted
    }
}

/// A connected MCP server: the transport plus its last-known tool inventory.
pub struct McpServerHandle {
    pub name: String,
    transport: Box<dyn McpTransport>,
    tools: RwLock<Vec<McpTool>>,
    trust: McpServerTrust,
}

impl McpServerHandle {
    pub fn new(name: impl Into<String>, transport: Box<dyn McpTransport>) -> Self {
        Self {
            name: name.into(),
            transport,
            tools: RwLock::new(Vec::new()),
            trust: McpServerTrust::Untrusted,
        }
    }

    pub fn new_trusted_local(name: impl Into<String>, transport: Box<dyn McpTransport>) -> Self {
        Self {
            name: name.into(),
            transport,
            tools: RwLock::new(Vec::new()),
            trust: McpServerTrust::TrustedLocal,
        }
    }

    pub fn trust(&self) -> McpServerTrust {
        self.trust
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
    ///
    /// This seam is compiled only for the UAR/agent runtime. Application-facing
    /// crates intentionally receive no raw execution API.
    #[cfg(feature = "runtime-internal")]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mcp_tool_annotations_deserialize_from_tools_list_shape() {
        let tool: McpTool = serde_json::from_value(serde_json::json!({
            "name": "search_memory",
            "title": "Search Memory",
            "description": "Search governed memory",
            "inputSchema": {
                "type": "object",
                "required": ["query"],
                "properties": {"query": {"type": "string"}}
            },
            "annotations": {
                "readOnlyHint": true,
                "destructiveHint": false,
                "idempotentHint": true,
                "openWorldHint": false
            },
            "execution": {
                "taskSupport": "optional"
            },
            "outputSchema": {
                "type": "object"
            }
        }))
        .expect("MCP tool parses");

        assert_eq!(tool.title.as_deref(), Some("Search Memory"));
        let annotations = tool.annotations.expect("annotations parse");
        assert_eq!(annotations.read_only_hint, Some(true));
        assert_eq!(annotations.destructive_hint, Some(false));
        assert_eq!(annotations.idempotent_hint, Some(true));
        assert_eq!(annotations.open_world_hint, Some(false));
        assert_eq!(
            tool.execution.and_then(|value| value.task_support),
            Some("optional".into())
        );
        assert!(tool.output_schema.is_some());
    }
}
