// TJ-ARCH-MOB-001 compliant
//! HTTP+SSE implementation of [`crate::registry::McpTransport`]. The SSE channel
//! carries server→client notifications; requests are JSON-RPC 2.0 over HTTP POST.
//! Bearer auth is supplied by the caller (the flint client injects the gate JWT).
//! Native-only: reqwest's wasm `Response` future is not `Send`.
use crate::jsonrpc::{JsonRpcRequest, JsonRpcResponse};
use crate::registry::McpTransport;
use async_trait::async_trait;
use gen_ui_types::{CoreError, CoreResult};
use std::sync::atomic::{AtomicU64, Ordering};

pub struct SseTransport {
    http: reqwest::Client,
    endpoint: String,
    bearer: Option<String>,
    next_id: AtomicU64,
}

impl SseTransport {
    /// `endpoint` is the JSON-RPC POST URL (e.g. `https://forge/mcp/v1/a2ui`).
    pub fn new(http: reqwest::Client, endpoint: impl Into<String>, bearer: Option<String>) -> Self {
        Self {
            http,
            endpoint: endpoint.into(),
            bearer,
            next_id: AtomicU64::new(1),
        }
    }
}

#[async_trait]
impl McpTransport for SseTransport {
    async fn request(
        &self,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> CoreResult<serde_json::Value> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let body = JsonRpcRequest::new(id, method, params);
        let mut req = self.http.post(&self.endpoint).json(&body);
        if let Some(token) = &self.bearer {
            req = req.bearer_auth(token);
        }
        let resp = req
            .send()
            .await
            .map_err(|e| CoreError::Transient(e.to_string()))?;
        if !resp.status().is_success() {
            return Err(CoreError::Terminal(format!("mcp http {}", resp.status())));
        }
        let parsed: JsonRpcResponse = resp
            .json()
            .await
            .map_err(|e| CoreError::Serde(e.to_string()))?;
        if let Some(err) = parsed.error {
            return Err(CoreError::Terminal(format!(
                "jsonrpc {}: {}",
                err.code, err.message
            )));
        }
        parsed
            .result
            .ok_or_else(|| CoreError::Terminal("jsonrpc: empty result".into()))
    }
}
