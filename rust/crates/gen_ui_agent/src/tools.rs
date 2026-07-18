// TJ-ARCH-MOB-001 compliant
//! MCP tools ↔ LLM tool-calling bridge (C-108).
//!
//! Two translations, both boring on purpose:
//!   - MCP `tools/list` inventory → liter-llm `ChatCompletionTool` definitions.
//!   - An LLM tool call → an MCP `tools/call`, whose result goes back as a tool message.
//!
//! Naming is the load-bearing detail. MCP tool names are only unique *within* a server,
//! and the model sees one flat list, so names are qualified `server__tool` on the way out
//! and split back on the way in. `__` rather than `/` or `.`: OpenAI-compatible providers
//! constrain function names to `[A-Za-z0-9_-]`, so a separator outside that set is
//! silently rejected or mangled by the provider rather than failing here.
use gen_ui_mcp::{McpRegistry, McpTool};
use liter_llm::types::common::{ChatCompletionTool, FunctionDefinition, ToolType};

use crate::error::AgentError;

/// Separator between server name and tool name in the flat, model-facing namespace.
const QUALIFIER: &str = "__";

/// Qualify an MCP tool name for the model's flat namespace.
fn qualify(server: &str, tool: &str) -> String {
    format!("{server}{QUALIFIER}{tool}")
}

/// Split a model-facing name back into (server, tool).
///
/// Splits on the FIRST separator: server names are ours (we register them), tool names
/// come from the server and may themselves contain `__`.
fn unqualify(qualified: &str) -> Option<(&str, &str)> {
    qualified.split_once(QUALIFIER)
}

/// Every registered server's tools, as LLM tool definitions.
///
/// Uses each server's cached inventory rather than re-listing: `refresh_tools` is a
/// network round-trip, and doing one per server on every chat turn would put MCP latency
/// on the critical path of a request that may not call a tool at all. Callers refresh at
/// registration time (see `register_server`).
pub fn tool_definitions(registry: &McpRegistry) -> Vec<ChatCompletionTool> {
    let mut defs = Vec::new();
    for server_name in registry.server_names() {
        let Some(server) = registry.get(&server_name) else {
            continue;
        };
        for tool in server.cached_tools() {
            defs.push(to_definition(&server_name, &tool));
        }
    }
    defs
}

fn to_definition(server: &str, tool: &McpTool) -> ChatCompletionTool {
    ChatCompletionTool {
        tool_type: ToolType::Function,
        function: FunctionDefinition {
            name: qualify(server, &tool.name),
            description: tool.description.clone(),
            // MCP's `inputSchema` IS a JSON Schema, which is exactly what the
            // `parameters` field wants — no translation, just a move.
            parameters: Some(tool.input_schema.clone()),
            strict: None,
        },
    }
}

/// Execute a model-issued tool call against the MCP server that owns it.
///
/// `arguments` arrives as a JSON *string* (the OpenAI wire format), so it is parsed here
/// rather than at the call site — a malformed one is the model's error, and it comes back
/// as a tool result the model can see and retry, not as a failed run.
pub async fn call_tool(
    registry: &McpRegistry,
    qualified_name: &str,
    arguments: &str,
) -> Result<serde_json::Value, AgentError> {
    let (server_name, tool_name) = unqualify(qualified_name).ok_or_else(|| {
        AgentError::Config(format!(
            "tool name '{qualified_name}' is not '{QUALIFIER}'-qualified — the model \
             invented a name, or a server was registered after the definitions were sent"
        ))
    })?;

    let server = registry.get(server_name).ok_or_else(|| {
        AgentError::Config(format!("no MCP server named '{server_name}' is registered"))
    })?;

    // Empty arguments are common when a tool takes none; `{}` is the honest translation
    // of "no arguments" for a JSON-object schema.
    let args: serde_json::Value = if arguments.trim().is_empty() {
        serde_json::json!({})
    } else {
        serde_json::from_str(arguments)
            .map_err(|e| AgentError::Config(format!("tool arguments are not valid JSON: {e}")))?
    };

    server
        .call_tool(tool_name, args)
        .await
        .map_err(|e| AgentError::Config(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn qualified_names_round_trip() {
        let q = qualify("forge", "list_views");
        assert_eq!(q, "forge__list_views");
        assert_eq!(unqualify(&q), Some(("forge", "list_views")));
    }

    #[test]
    fn unqualify_splits_on_the_first_separator_only() {
        // Tool names come from the server and may contain the separator themselves;
        // splitting on the last one would silently address the wrong server.
        assert_eq!(
            unqualify("forge__list__views"),
            Some(("forge", "list__views"))
        );
    }

    #[test]
    fn unqualified_names_are_rejected_rather_than_guessed() {
        assert_eq!(unqualify("list_views"), None);
    }
}
