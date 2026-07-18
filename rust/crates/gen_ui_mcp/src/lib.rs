// TJ-ARCH-MOB-001 compliant
//! gen_ui_mcp (L2) — MCP (Model Context Protocol) client registry.
//! JSON-RPC 2.0 over HTTP POST + an SSE event channel (open standard, Rule 12).
//! flint-forge exposes its A2UI registry AS an MCP server at `/mcp/v1/a2ui`;
//! the flint client (gen_ui_client) registers that server into [`McpRegistry`].
//!
//! The registry + JSON-RPC envelopes are pure and cross-target. The HTTP+SSE
//! transport uses reqwest, whose wasm `Response` future is NOT `Send`; because the
//! MCP transport seam is object-safe with `Send` futures (registry holds
//! `Box<dyn McpTransport>` across threads on native), the concrete [`SseTransport`]
//! is native-only. On the browser the A2UI/MCP surface is driven from JS
//! (Connect-web / `@flint/react`) per the layer contract, not this crate.
#![cfg_attr(target_arch = "wasm32", allow(dead_code))]

pub mod jsonrpc;
pub mod registry;
#[cfg(not(target_arch = "wasm32"))]
pub mod sse_transport;

pub use registry::{McpRegistry, McpServerHandle, McpTool, McpTransport};
#[cfg(not(target_arch = "wasm32"))]
pub use sse_transport::SseTransport;
