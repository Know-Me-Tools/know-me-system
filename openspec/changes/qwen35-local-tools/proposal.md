# Qwen3.5 local tool execution

## Why

The selected models support structured tool calls, but KnowMe's local lane currently sends a flat text prompt and cannot execute or display them.

## What changes

- Render permitted tool schemas in the Qwen chat template.
- Parse structured tool calls without treating arbitrary prose as executable intent.
- Execute calls through the existing permission-aware MCP registry.
- Publish tool use and results through existing AG-UI/A2UI content events.
- Bound iterations, arguments, output sizes, and failure behavior.

## Success criteria

- A deterministic safe request produces a structured local tool call.
- Only registered and permitted tools execute.
- Tool use/result blocks render through the existing Flutter event flow.
- The model can consume a tool result and provide a final response.
