// TJ-ARCH-MOB-001 compliant
//! gen_ui_protocol (L1) — A2UI/AG-UI adapters. Pure transformation over the L0
//! event enums; wasm-safe (no IO, no runtime dependency).
use gen_ui_types::content_block::ContentBlock;
use gen_ui_types::events::{A2uiEvent, AguiEvent, StreamEvent};
use gen_ui_types::inference::{
    GrammarConstraint, InferenceMessage, InferenceRequest, InferenceRole, InferenceTool,
    InferenceToolAnnotations, PromptPlanOrigin, SampleParams,
};
use gen_ui_types::lifecycle::{RunError, RunPhase};
use serde::{Deserialize, Serialize};

/// UAR-compatible provider payload derived from KnowMe's canonical local
/// inference request.
///
/// This intentionally avoids depending on the UAR crate: mobile can build this
/// bridge today, and the future embedded UAR provider can adapt the same
/// payload into `universal_agent_runtime::llm::LlmRequest` mechanically.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UarLlmRequestPayload {
    pub messages: Vec<serde_json::Value>,
    pub tools: Vec<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extra_params: Option<serde_json::Value>,
}

/// Convert a UAR `LlmRequest`-shaped payload into KnowMe's canonical local
/// provider request.
///
/// This is the inverse of [`inference_request_to_uar_llm_payload`] and exists
/// so the future mobile embedded UAR kernel can delegate to KnowMe local
/// providers without flattening roles or silently dropping tool declarations.
/// It stays JSON-level instead of depending on the UAR crate, which keeps this
/// protocol crate wasm/mobile-safe.
pub fn uar_llm_payload_to_inference_request(
    payload: &UarLlmRequestPayload,
) -> Result<InferenceRequest, String> {
    let messages = payload
        .messages
        .iter()
        .map(uar_message_to_inference_message)
        .collect::<Result<Vec<_>, _>>()?;
    let tools = payload
        .tools
        .iter()
        .map(uar_tool_to_inference_tool)
        .collect::<Result<Vec<_>, _>>()?;
    let generation_settings = payload
        .extra_params
        .as_ref()
        .map(sample_params_from_uar_extra)
        .transpose()?
        .unwrap_or_default();
    let grammar = payload
        .extra_params
        .as_ref()
        .and_then(|extra| extra.get("grammar"))
        .filter(|value| !value.is_null())
        .map(|value| {
            serde_json::from_value::<GrammarConstraint>(value.clone())
                .map_err(|error| format!("invalid UAR grammar constraint: {error}"))
        })
        .transpose()?;
    let originating_plan = payload
        .extra_params
        .as_ref()
        .and_then(|extra| extra.get("originating_plan"))
        .filter(|value| !value.is_null())
        .map(|value| {
            serde_json::from_value::<PromptPlanOrigin>(value.clone())
                .map_err(|error| format!("invalid UAR originating plan: {error}"))
        })
        .transpose()?;

    let prompt = messages
        .iter()
        .rev()
        .find(|message| message.role == InferenceRole::User)
        .map(|message| message.content.clone())
        .unwrap_or_default();

    Ok(InferenceRequest {
        prompt,
        messages,
        tools,
        images: Vec::new(),
        generation_settings,
        grammar,
        originating_plan,
    })
}

/// Convert KnowMe's structured local-provider request into the message/tool
/// shape UAR's `LlmDriver` expects.
pub fn inference_request_to_uar_llm_payload(request: &InferenceRequest) -> UarLlmRequestPayload {
    let messages = request
        .effective_messages()
        .into_iter()
        .map(|message| {
            let mut value = serde_json::json!({
                "role": message.role.as_str(),
                "content": message.content,
            });
            if let serde_json::Value::Object(ref mut object) = value {
                if let Some(name) = message.name {
                    object.insert("name".into(), serde_json::Value::String(name));
                }
                if let Some(tool_call_id) = message.tool_call_id {
                    object.insert(
                        "tool_call_id".into(),
                        serde_json::Value::String(tool_call_id),
                    );
                }
            }
            value
        })
        .collect();

    let tools = request
        .tools
        .iter()
        .map(inference_tool_to_uar_tool)
        .collect();
    let extra_params = Some(serde_json::json!({
        "temperature": request.generation_settings.temperature,
        "top_p": request.generation_settings.top_p,
        "top_k": request.generation_settings.top_k,
        "min_p": request.generation_settings.min_p,
        "presence_penalty": request.generation_settings.presence_penalty,
        "max_tokens": request.generation_settings.max_tokens,
        "grammar": request.grammar,
        "originating_plan": request.originating_plan,
    }))
    .filter(|value| {
        value
            .as_object()
            .map(|object| !object.is_empty())
            .unwrap_or(true)
    });

    UarLlmRequestPayload {
        messages,
        tools,
        extra_params,
    }
}

fn inference_tool_to_uar_tool(tool: &InferenceTool) -> serde_json::Value {
    serde_json::json!({
        "type": "function",
        "function": {
            "name": tool.name,
            "description": tool.description,
            "parameters": tool.parameters,
        },
        "annotations": tool.annotations,
    })
}

fn uar_message_to_inference_message(value: &serde_json::Value) -> Result<InferenceMessage, String> {
    let role = match json_string(value, &["role"]).unwrap_or("user") {
        "system" => InferenceRole::System,
        "user" => InferenceRole::User,
        "assistant" => InferenceRole::Assistant,
        "tool" => InferenceRole::Tool,
        other => {
            return Err(format!("unsupported UAR message role '{other}'"));
        }
    };
    Ok(InferenceMessage {
        role,
        content: flatten_uar_message_content(value.get("content")),
        name: json_string(value, &["name"]).map(str::to_owned),
        tool_call_id: first_json_string(value, &[&["tool_call_id"][..], &["toolCallId"][..]])
            .map(str::to_owned),
    })
}

fn flatten_uar_message_content(content: Option<&serde_json::Value>) -> String {
    match content {
        Some(serde_json::Value::String(text)) => text.clone(),
        Some(serde_json::Value::Array(parts)) => parts
            .iter()
            .filter_map(flatten_uar_content_part)
            .collect::<Vec<_>>()
            .join("\n"),
        Some(other) => other.to_string(),
        None => String::new(),
    }
}

fn flatten_uar_content_part(part: &serde_json::Value) -> Option<String> {
    let part_type = json_string(part, &["type"]).unwrap_or("text");
    match part_type {
        "text" => json_string(part, &["text"]).map(str::to_owned),
        "image_url" => {
            let url = json_string(part, &["image_url", "url"]).unwrap_or("<unknown>");
            Some(format!(
                "[image content supplied by UAR but not directly consumable by this text-only local provider: {url}]"
            ))
        }
        other => Some(format!(
            "[unsupported UAR content part preserved for local provider bridge: {other} {}]",
            part
        )),
    }
}

fn uar_tool_to_inference_tool(value: &serde_json::Value) -> Result<InferenceTool, String> {
    let function = value.get("function").unwrap_or(value);
    let name = json_string(function, &["name"])
        .ok_or_else(|| format!("UAR tool is missing function.name: {value}"))?
        .to_owned();
    let description = first_json_string(function, &[&["description"][..]])
        .or_else(|| json_string(value, &["description"]))
        .unwrap_or("")
        .to_owned();
    let parameters = function
        .get("parameters")
        .or_else(|| value.get("parameters"))
        .cloned()
        .unwrap_or_else(|| serde_json::json!({"type": "object", "properties": {}}));
    let annotations = value
        .get("annotations")
        .filter(|value| !value.is_null())
        .map(|value| {
            serde_json::from_value::<InferenceToolAnnotations>(value.clone())
                .map_err(|error| format!("invalid UAR tool annotations for {name}: {error}"))
        })
        .transpose()?;
    Ok(InferenceTool {
        name,
        description,
        parameters,
        annotations,
    })
}

fn sample_params_from_uar_extra(value: &serde_json::Value) -> Result<SampleParams, String> {
    let mut params = SampleParams::default();
    if let Some(temperature) = json_f32(value, &["temperature"])? {
        params.temperature = temperature;
    }
    if let Some(top_p) = json_f32(value, &["top_p"])? {
        params.top_p = top_p;
    }
    if let Some(top_k) = json_i32(value, &["top_k"])? {
        params.top_k = top_k;
    }
    if let Some(min_p) = json_f32(value, &["min_p"])? {
        params.min_p = min_p;
    }
    if let Some(presence_penalty) = json_f32(value, &["presence_penalty"])? {
        params.presence_penalty = presence_penalty;
    }
    if let Some(max_tokens) = json_u32(value, &["max_tokens"]) {
        params.max_tokens = max_tokens;
    }
    Ok(params)
}

/// Convert a local provider stream event into a UAR `NormalizedEvent`-shaped
/// JSON payload.
///
/// The future UAR `ExternalLlmDriver` host callback can deserialize this into
/// UAR's native `NormalizedEvent` enum. Keeping the transformation here avoids
/// a mobile dependency on the full runtime while preserving every local semantic
/// chunk for UAR orchestration.
pub fn stream_event_to_uar_normalized_payload(event: &StreamEvent) -> serde_json::Value {
    match event {
        StreamEvent::MessageStart => serde_json::json!({
            "type": "stream.start",
            "data": {"request_id": "knowme-local"}
        }),
        StreamEvent::TextDelta { delta, .. } => serde_json::json!({
            "type": "message.delta",
            "data": {"text": delta}
        }),
        StreamEvent::ThinkingDelta { delta, .. } => serde_json::json!({
            "type": "thinking.delta",
            "data": {"text": delta}
        }),
        StreamEvent::ToolCallStarted { id, name } => serde_json::json!({
            "type": "tool_call.delta",
            "data": {
                "call_index": 0,
                "id": id,
                "name": name,
                "arguments_delta": null
            }
        }),
        StreamEvent::ToolCallDelta { delta, .. } => serde_json::json!({
            "type": "tool_call.delta",
            "data": {
                "call_index": 0,
                "id": null,
                "name": null,
                "arguments_delta": delta
            }
        }),
        StreamEvent::ToolCallComplete { id } => serde_json::json!({
            "type": "tool_call.complete",
            "data": {
                "call_index": 0,
                "id": id,
                "name": "unknown_tool",
                "arguments_json": ""
            }
        }),
        StreamEvent::ToolCallReady {
            id,
            name,
            input_json,
        } => serde_json::json!({
            "type": "tool_call.complete",
            "data": {
                "call_index": 0,
                "id": id,
                "name": name,
                "arguments_json": input_json
            }
        }),
        StreamEvent::ToolResult {
            tool_use_id,
            output_json,
            is_error,
        } => serde_json::json!({
            "type": "tool_result",
            "data": {
                "id": tool_use_id,
                "name": "unknown_tool",
                "content": output_json,
                "success": !is_error
            }
        }),
        StreamEvent::SkillActivation { name, status } => serde_json::json!({
            "type": "skill.activation",
            "data": {
                "name": name,
                "status": status
            }
        }),
        StreamEvent::Citation { source, quote } => serde_json::json!({
            "type": "citation",
            "data": {
                "source": source,
                "quote": quote
            }
        }),
        StreamEvent::Memory {
            operation,
            key,
            value,
        } => serde_json::json!({
            "type": "memory",
            "data": {
                "operation": operation,
                "key": key,
                "value": value
            }
        }),
        StreamEvent::Custom {
            source,
            event_name,
            payload,
        } => serde_json::json!({
            "type": "custom",
            "data": {
                "source": source,
                "event_name": event_name,
                "payload": payload
            }
        }),
        StreamEvent::Error { message } => serde_json::json!({
            "type": "error",
            "data": {
                "message": message,
                "code": "knowme_local_provider_error"
            }
        }),
        StreamEvent::Cancelled { message } => serde_json::json!({
            "type": "cancelled",
            "data": {
                "message": message
            }
        }),
        StreamEvent::Done => serde_json::json!({
            "type": "done"
        }),
    }
}

/// Normalize a UAR `uar.agui/1` JSON event into KnowMe's provider-neutral
/// `StreamEvent` contract.
///
/// This is intentionally narrow and side-effect free: runtime execution still
/// belongs behind the future UAR facade, but the projection seam can be tested
/// offline now. Unknown UAR payloads are preserved as `StreamEvent::Custom`
/// rather than dropped.
pub fn normalize_uar_agui_event(payload: &serde_json::Value) -> StreamEvent {
    let event_type = json_string(payload, &["type"]).unwrap_or("CUSTOM");
    match event_type {
        "agui.stream.start" => StreamEvent::MessageStart,
        "RUN_STARTED" => StreamEvent::MessageStart,
        "agui.message.delta" => StreamEvent::TextDelta {
            index: json_u32(payload, &["index"])
                .or_else(|| json_u32(payload, &["message", "index"]))
                .unwrap_or(0),
            delta: first_json_string(
                payload,
                &[
                    &["delta", "text"][..],
                    &["delta"][..],
                    &["text"][..],
                    &["content"][..],
                    &["message", "content"][..],
                ],
            )
            .unwrap_or_default()
            .to_owned(),
        },
        "TEXT_MESSAGE_CONTENT" => StreamEvent::TextDelta {
            index: json_u32(payload, &["index"]).unwrap_or(0),
            delta: first_json_string(
                payload,
                &[
                    &["delta"][..],
                    &["content"][..],
                    &["text"][..],
                    &["message", "content"][..],
                ],
            )
            .unwrap_or_default()
            .to_owned(),
        },
        "agui.thinking.delta" | "agui.reasoning.delta" => StreamEvent::ThinkingDelta {
            index: json_u32(payload, &["index"]).unwrap_or(0),
            delta: first_json_string(
                payload,
                &[
                    &["delta", "text"][..],
                    &["delta"][..],
                    &["text"][..],
                    &["content"][..],
                    &["reasoning"][..],
                ],
            )
            .unwrap_or_default()
            .to_owned(),
        },
        "REASONING_MESSAGE_CONTENT" => StreamEvent::ThinkingDelta {
            index: json_u32(payload, &["index"]).unwrap_or(0),
            delta: first_json_string(
                payload,
                &[
                    &["delta"][..],
                    &["content"][..],
                    &["text"][..],
                    &["reasoning"][..],
                ],
            )
            .unwrap_or_default()
            .to_owned(),
        },
        "TOOL_CALL_START" => StreamEvent::ToolCallStarted {
            id: first_json_string(
                payload,
                &[&["toolCallId"][..], &["id"][..], &["tool_call_id"][..]],
            )
            .unwrap_or("unknown-tool-call")
            .to_owned(),
            name: first_json_string(
                payload,
                &[&["name"][..], &["toolName"][..], &["tool", "name"][..]],
            )
            .unwrap_or("unknown_tool")
            .to_owned(),
        },
        "agui.tool_call.delta" => StreamEvent::ToolCallDelta {
            id: first_json_string(
                payload,
                &[&["id"][..], &["toolCallId"][..], &["tool_call_id"][..]],
            )
            .unwrap_or("unknown-tool-call")
            .to_owned(),
            delta: first_json_string(
                payload,
                &[
                    &["delta", "arguments"][..],
                    &["arguments_delta"][..],
                    &["delta"][..],
                    &["args"][..],
                    &["arguments"][..],
                ],
            )
            .map(str::to_owned)
            .unwrap_or_default(),
        },
        "TOOL_CALL_ARGS" => StreamEvent::ToolCallDelta {
            id: first_json_string(
                payload,
                &[&["toolCallId"][..], &["id"][..], &["tool_call_id"][..]],
            )
            .unwrap_or("unknown-tool-call")
            .to_owned(),
            delta: first_json_string(
                payload,
                &[
                    &["delta"][..],
                    &["args"][..],
                    &["arguments"][..],
                    &["argumentsDelta"][..],
                ],
            )
            .map(str::to_owned)
            .unwrap_or_else(|| {
                payload
                    .get("arguments")
                    .map(json_payload_to_string)
                    .unwrap_or_default()
            }),
        },
        "TOOL_CALL_END" => StreamEvent::ToolCallComplete {
            id: first_json_string(
                payload,
                &[&["toolCallId"][..], &["id"][..], &["tool_call_id"][..]],
            )
            .unwrap_or("unknown-tool-call")
            .to_owned(),
        },
        "agui.tool_call.complete" => StreamEvent::ToolCallReady {
            id: first_json_string(
                payload,
                &[&["id"][..], &["toolCallId"][..], &["tool_call_id"][..]],
            )
            .unwrap_or("unknown-tool-call")
            .to_owned(),
            name: first_json_string(
                payload,
                &[&["name"][..], &["toolName"][..], &["tool", "name"][..]],
            )
            .unwrap_or("unknown_tool")
            .to_owned(),
            input_json: first_json_string(
                payload,
                &[
                    &["arguments_json"][..],
                    &["input_json"][..],
                    &["arguments"][..],
                    &["input"][..],
                ],
            )
            .map(str::to_owned)
            .unwrap_or_else(|| {
                payload
                    .get("arguments")
                    .or_else(|| payload.get("input"))
                    .map(json_payload_to_string)
                    .unwrap_or_default()
            }),
        },
        "TOOL_CALL_RESULT" => StreamEvent::ToolResult {
            tool_use_id: first_json_string(
                payload,
                &[&["toolCallId"][..], &["id"][..], &["tool_call_id"][..]],
            )
            .unwrap_or("unknown-tool-call")
            .to_owned(),
            output_json: payload
                .get("result")
                .or_else(|| payload.get("output"))
                .or_else(|| payload.get("content"))
                .map(json_payload_to_string)
                .unwrap_or_else(|| "{}".to_owned()),
            is_error: json_bool(payload, &["isError"])
                .or_else(|| json_bool(payload, &["is_error"]))
                .unwrap_or(false),
        },
        "agui.tool_result" => StreamEvent::ToolResult {
            tool_use_id: first_json_string(
                payload,
                &[&["id"][..], &["toolCallId"][..], &["tool_call_id"][..]],
            )
            .unwrap_or("unknown-tool-call")
            .to_owned(),
            output_json: payload
                .get("content")
                .or_else(|| payload.get("result"))
                .or_else(|| payload.get("output"))
                .map(json_payload_to_string)
                .unwrap_or_else(|| "{}".to_owned()),
            is_error: !json_bool(payload, &["success"]).unwrap_or(true),
        },
        "agui.citation.added" => StreamEvent::Citation {
            source: first_json_string(
                payload,
                &[
                    &["citation", "title"][..],
                    &["citation", "url"][..],
                    &["source"][..],
                    &["url"][..],
                ],
            )
            .unwrap_or("unknown")
            .to_owned(),
            quote: first_json_string(
                payload,
                &[
                    &["citation", "snippet"][..],
                    &["citation", "quote"][..],
                    &["quote"][..],
                    &["snippet"][..],
                    &["text"][..],
                ],
            )
            .unwrap_or_default()
            .to_owned(),
        },
        "agui.memory.update" => StreamEvent::Memory {
            operation: first_json_string(payload, &[&["operation"][..], &["phase"][..]])
                .unwrap_or("set")
                .to_owned(),
            key: first_json_string(payload, &[&["key"][..], &["memory", "key"][..]])
                .unwrap_or("unknown")
                .to_owned(),
            value: first_json_string(payload, &[&["value"][..], &["memory", "value"][..]])
                .map(str::to_owned),
        },
        "agui.memory.recall" => StreamEvent::Memory {
            operation: "recall".to_owned(),
            key: "memory_context".to_owned(),
            value: payload.get("items").map(json_payload_to_string),
        },
        "agui.skill.activated" => StreamEvent::SkillActivation {
            name: first_json_string(
                payload,
                &[
                    &["skill", "id"][..],
                    &["skill", "title"][..],
                    &["skill", "name"][..],
                    &["name"][..],
                ],
            )
            .unwrap_or("unknown_skill")
            .to_owned(),
            status: first_json_string(payload, &[&["phase"][..], &["status"][..]])
                .unwrap_or("activated")
                .to_owned(),
        },
        "CUSTOM" => normalize_uar_custom_event(payload),
        "agui.done" => StreamEvent::Done,
        "RUN_FINISHED" => StreamEvent::Done,
        "agui.cancelled" => StreamEvent::Cancelled {
            message: first_json_string(payload, &[&["message"][..], &["reason"][..]])
                .unwrap_or("Run cancelled")
                .to_owned(),
        },
        "agui.error" => StreamEvent::Error {
            message: first_json_string(
                payload,
                &[
                    &["message"][..],
                    &["error", "message"][..],
                    &["value", "message"][..],
                ],
            )
            .unwrap_or("UAR run failed")
            .to_owned(),
        },
        "RUN_ERROR" => StreamEvent::Error {
            message: first_json_string(
                payload,
                &[
                    &["message"][..],
                    &["error", "message"][..],
                    &["value", "message"][..],
                ],
            )
            .unwrap_or("UAR run failed")
            .to_owned(),
        },
        _ => StreamEvent::Custom {
            source: "uar".into(),
            event_name: event_type.to_owned(),
            payload: payload.clone(),
        },
    }
}

fn normalize_uar_custom_event(payload: &serde_json::Value) -> StreamEvent {
    let name = json_string(payload, &["name"]).unwrap_or("CUSTOM");
    let value = payload.get("value").unwrap_or(payload);
    if name == "uar.skill.activated" {
        return StreamEvent::SkillActivation {
            name: first_json_string(
                value,
                &[
                    &["skill", "id"][..],
                    &["skill", "name"][..],
                    &["skill"][..],
                    &["name"][..],
                ],
            )
            .unwrap_or("unknown_skill")
            .to_owned(),
            status: first_json_string(value, &[&["status"][..], &["phase"][..]])
                .unwrap_or("activated")
                .to_owned(),
        };
    }
    if matches!(
        name,
        "uar.citation" | "uar.citation.added" | "citation" | "citation.added"
    ) {
        return StreamEvent::Citation {
            source: first_json_string(
                value,
                &[
                    &["source"][..],
                    &["citation", "source"][..],
                    &["document", "title"][..],
                    &["uri"][..],
                    &["url"][..],
                ],
            )
            .unwrap_or("unknown")
            .to_owned(),
            quote: first_json_string(
                value,
                &[
                    &["quote"][..],
                    &["text"][..],
                    &["excerpt"][..],
                    &["citation", "quote"][..],
                    &["citation", "text"][..],
                ],
            )
            .unwrap_or_default()
            .to_owned(),
        };
    }
    if matches!(
        name,
        "uar.memory" | "uar.memory.updated" | "memory" | "memory.updated"
    ) {
        return StreamEvent::Memory {
            operation: first_json_string(
                value,
                &[
                    &["operation"][..],
                    &["op"][..],
                    &["action"][..],
                    &["status"][..],
                ],
            )
            .unwrap_or("recorded")
            .to_owned(),
            key: first_json_string(
                value,
                &[
                    &["key"][..],
                    &["memory", "key"][..],
                    &["id"][..],
                    &["memory", "id"][..],
                    &["name"][..],
                ],
            )
            .unwrap_or("unknown")
            .to_owned(),
            value: first_json_string(
                value,
                &[
                    &["value"][..],
                    &["text"][..],
                    &["memory", "value"][..],
                    &["memory", "text"][..],
                ],
            )
            .map(str::to_owned),
        };
    }

    StreamEvent::Custom {
        source: "uar".into(),
        event_name: name.to_owned(),
        payload: payload.clone(),
    }
}

fn json_string<'a>(value: &'a serde_json::Value, path: &[&str]) -> Option<&'a str> {
    let mut cursor = value;
    for segment in path {
        cursor = cursor.get(*segment)?;
    }
    cursor.as_str()
}

fn first_json_string<'a>(value: &'a serde_json::Value, paths: &[&[&str]]) -> Option<&'a str> {
    paths.iter().find_map(|path| json_string(value, path))
}

fn json_bool(value: &serde_json::Value, path: &[&str]) -> Option<bool> {
    let mut cursor = value;
    for segment in path {
        cursor = cursor.get(*segment)?;
    }
    cursor.as_bool()
}

fn json_u32(value: &serde_json::Value, path: &[&str]) -> Option<u32> {
    let mut cursor = value;
    for segment in path {
        cursor = cursor.get(*segment)?;
    }
    cursor.as_u64().and_then(|value| u32::try_from(value).ok())
}

fn json_i32(value: &serde_json::Value, path: &[&str]) -> Result<Option<i32>, String> {
    let mut cursor = value;
    for segment in path {
        let Some(next) = cursor.get(*segment) else {
            return Ok(None);
        };
        cursor = next;
    }
    cursor
        .as_i64()
        .map(|value| {
            i32::try_from(value).map_err(|_| format!("integer at {path:?} is outside i32 range"))
        })
        .transpose()
}

fn json_f32(value: &serde_json::Value, path: &[&str]) -> Result<Option<f32>, String> {
    let mut cursor = value;
    for segment in path {
        let Some(next) = cursor.get(*segment) else {
            return Ok(None);
        };
        cursor = next;
    }
    cursor
        .as_f64()
        .map(|value| {
            if value.is_finite() && value >= f32::MIN as f64 && value <= f32::MAX as f64 {
                Ok(value as f32)
            } else {
                Err(format!("float at {path:?} is outside f32 range"))
            }
        })
        .transpose()
}

fn json_payload_to_string(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(value) => value.clone(),
        other => other.to_string(),
    }
}

/// Stateful local-provider → UAR normalized-event adapter.
///
/// UAR has no distinct `ToolCallStarted` event; it receives deltas and a final
/// complete event with the full name and arguments. KnowMe's provider stream
/// separates start/delta/complete, so this adapter must retain pending tool
/// state to avoid losing the exact executable arguments.
#[derive(Default)]
pub struct UarNormalizedEventAdapter {
    pending_tools: Vec<PendingTool>,
}

impl UarNormalizedEventAdapter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn ingest(&mut self, event: &StreamEvent) -> Vec<serde_json::Value> {
        match event {
            StreamEvent::ToolCallStarted { id, name } => {
                self.pending_tools.push(PendingTool {
                    id: id.clone(),
                    name: name.clone(),
                    args: String::new(),
                });
                vec![serde_json::json!({
                    "type": "tool_call.delta",
                    "data": {
                        "call_index": self.pending_tools.len().saturating_sub(1),
                        "id": id,
                        "name": name,
                        "arguments_delta": null
                    }
                })]
            }
            StreamEvent::ToolCallDelta { id, delta } => {
                let call_index = self
                    .pending_tools
                    .iter()
                    .position(|tool| &tool.id == id)
                    .unwrap_or(0);
                if let Some(tool) = self.pending_tools.iter_mut().find(|tool| &tool.id == id) {
                    tool.args.push_str(delta);
                }
                vec![serde_json::json!({
                    "type": "tool_call.delta",
                    "data": {
                        "call_index": call_index,
                        "id": id,
                        "name": null,
                        "arguments_delta": delta
                    }
                })]
            }
            StreamEvent::ToolCallComplete { id } => {
                let Some(index) = self.pending_tools.iter().position(|tool| &tool.id == id) else {
                    return vec![stream_event_to_uar_normalized_payload(event)];
                };
                let tool = self.pending_tools.remove(index);
                vec![serde_json::json!({
                    "type": "tool_call.complete",
                    "data": {
                        "call_index": index,
                        "id": tool.id,
                        "name": tool.name,
                        "arguments_json": tool.args
                    }
                })]
            }
            StreamEvent::ToolCallReady { .. } => {
                vec![stream_event_to_uar_normalized_payload(event)]
            }
            other => vec![stream_event_to_uar_normalized_payload(other)],
        }
    }
}

/// StreamEvent -> A2uiEvent(s). Feature-complete adapter to be filled per the
/// ContentBlock contract; C-001 lands the seam + a working text path.
#[derive(Default)]
pub struct A2uiAdapter {
    run_id: String,
    reasoning: ReasoningParser,
    /// In-flight tool calls, keyed by call id (C-108).
    ///
    /// Tool arguments arrive as a *stream of string fragments* — the model emits
    /// `{"pat`, `h": "/tm`, `p"}` across chunks — so a ToolUse block can only be emitted
    /// once the arguments are complete. Holding the name here is what makes
    /// ToolCallComplete (which carries only an id) able to name its own block.
    pending_tools: Vec<PendingTool>,
}

struct PendingTool {
    id: String,
    name: String,
    /// Accumulated argument fragments. Not parsed as JSON until complete — a partial
    /// fragment is not valid JSON, and treating it as such would fail every time.
    args: String,
}

#[derive(Default)]
struct ReasoningParser {
    mode: ReasoningMode,
    pending: String,
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum ReasoningMode {
    #[default]
    Answer,
    Thinking,
}

impl ReasoningParser {
    fn ingest(&mut self, delta: &str) -> Vec<ContentBlock> {
        self.pending.push_str(delta);
        let mut blocks = Vec::new();
        loop {
            let marker = match self.mode {
                ReasoningMode::Answer => "<think>",
                ReasoningMode::Thinking => "</think>",
            };
            if let Some(index) = self.pending.find(marker) {
                let text = self.pending[..index].to_owned();
                self.push_current(text, &mut blocks);
                self.pending.drain(..index + marker.len());
                self.mode = match self.mode {
                    ReasoningMode::Answer => ReasoningMode::Thinking,
                    ReasoningMode::Thinking => ReasoningMode::Answer,
                };
                continue;
            }

            // Keep only a suffix that could still become the delimiter after
            // the next token. Everything before it is provably ordinary text.
            let retained = longest_marker_prefix_suffix(&self.pending, marker);
            let emit_len = self.pending.len() - retained;
            if emit_len > 0 {
                let text = self.pending[..emit_len].to_owned();
                self.pending.drain(..emit_len);
                self.push_current(text, &mut blocks);
            }
            break;
        }
        blocks
    }

    fn flush(&mut self) -> Vec<ContentBlock> {
        let text = std::mem::take(&mut self.pending);
        let mut blocks = Vec::new();
        self.push_current(text, &mut blocks);
        blocks
    }

    fn is_thinking(&self) -> bool {
        self.mode == ReasoningMode::Thinking
    }

    fn push_current(&self, text: String, output: &mut Vec<ContentBlock>) {
        if text.is_empty() {
            return;
        }
        output.push(match self.mode {
            ReasoningMode::Answer => ContentBlock::Text { text },
            ReasoningMode::Thinking => ContentBlock::Thinking { text },
        });
    }
}

fn longest_marker_prefix_suffix(value: &str, marker: &str) -> usize {
    (1..marker.len())
        .rev()
        .find(|&length| value.ends_with(&marker[..length]))
        .unwrap_or(0)
}

impl A2uiAdapter {
    pub fn new(run_id: impl Into<String>) -> Self {
        Self {
            run_id: run_id.into(),
            reasoning: ReasoningParser::default(),
            pending_tools: Vec::new(),
        }
    }

    fn block_events(&self, blocks: Vec<ContentBlock>) -> Vec<A2uiEvent> {
        blocks
            .into_iter()
            .map(|block| A2uiEvent::Block {
                run_id: self.run_id.clone(),
                block,
            })
            .collect()
    }

    pub fn ingest(&mut self, ev: &StreamEvent) -> Vec<A2uiEvent> {
        match ev {
            StreamEvent::MessageStart => vec![A2uiEvent::RunStarted {
                run_id: self.run_id.clone(),
            }],
            StreamEvent::TextDelta { delta, .. } => {
                let blocks = self.reasoning.ingest(delta);
                self.block_events(blocks)
            }
            StreamEvent::ThinkingDelta { delta, .. } => {
                let pending = self.reasoning.flush();
                let mut events = self.block_events(pending);
                events.push(A2uiEvent::Block {
                    run_id: self.run_id.clone(),
                    block: ContentBlock::Thinking {
                        text: delta.clone(),
                    },
                });
                events
            }

            // A tool call announces itself, then dribbles its arguments in. Nothing is
            // emitted until Complete: a half-built ToolUse block would render arguments
            // that are not yet valid JSON.
            StreamEvent::ToolCallStarted { id, name } => {
                self.pending_tools.push(PendingTool {
                    id: id.clone(),
                    name: name.clone(),
                    args: String::new(),
                });
                vec![]
            }
            StreamEvent::ToolCallDelta { id, delta } => {
                if let Some(t) = self.pending_tools.iter_mut().find(|t| &t.id == id) {
                    t.args.push_str(delta);
                }
                // A delta for an unknown id means Started was dropped or reordered.
                // Silently ignoring it is right: the alternative is inventing a tool
                // call whose name we do not know.
                vec![]
            }
            StreamEvent::ToolCallComplete { id } => {
                match self.pending_tools.iter().position(|t| &t.id == id) {
                    Some(i) => {
                        let t = self.pending_tools.remove(i);
                        vec![A2uiEvent::Block {
                            run_id: self.run_id.clone(),
                            block: ContentBlock::ToolUse {
                                id: t.id,
                                name: t.name,
                                input_json: t.args,
                            },
                        }]
                    }
                    None => vec![],
                }
            }
            StreamEvent::ToolCallReady {
                id,
                name,
                input_json,
            } => vec![A2uiEvent::Block {
                run_id: self.run_id.clone(),
                block: ContentBlock::ToolUse {
                    id: id.clone(),
                    name: name.clone(),
                    input_json: input_json.clone(),
                },
            }],
            StreamEvent::ToolResult {
                tool_use_id,
                output_json,
                is_error,
            } => vec![A2uiEvent::Block {
                run_id: self.run_id.clone(),
                block: ContentBlock::ToolResult {
                    tool_use_id: tool_use_id.clone(),
                    output_json: output_json.clone(),
                    is_error: *is_error,
                },
            }],
            StreamEvent::SkillActivation { name, status } => vec![A2uiEvent::Block {
                run_id: self.run_id.clone(),
                block: ContentBlock::Skill {
                    name: name.clone(),
                    status: status.clone(),
                },
            }],
            StreamEvent::Citation { source, quote } => vec![A2uiEvent::Block {
                run_id: self.run_id.clone(),
                block: ContentBlock::Citation {
                    source: source.clone(),
                    quote: quote.clone(),
                },
            }],
            StreamEvent::Memory {
                operation,
                key,
                value,
            } => vec![A2uiEvent::Block {
                run_id: self.run_id.clone(),
                block: ContentBlock::Memory {
                    operation: operation.clone(),
                    key: key.clone(),
                    value: value.clone(),
                },
            }],

            StreamEvent::Custom {
                source,
                event_name,
                payload,
            } => vec![A2uiEvent::Block {
                run_id: self.run_id.clone(),
                block: ContentBlock::Artifact {
                    id: format!("{source}:{event_name}"),
                    kind: artifact_kind_for_custom_event(source, event_name).into(),
                    content: serde_json::json!({
                        "source": source,
                        "eventName": event_name,
                        "payload": payload,
                    })
                    .to_string(),
                },
            }],
            StreamEvent::Done => {
                let unclosed = self.reasoning.is_thinking();
                let pending = self.reasoning.flush();
                let mut events = self.block_events(pending);
                if unclosed {
                    events.push(A2uiEvent::RunError {
                        run_id: self.run_id.clone(),
                        error: RunError::protocol(
                            "The model stream ended before its reasoning section closed.",
                        ),
                    });
                } else {
                    events.push(A2uiEvent::RunFinished {
                        run_id: self.run_id.clone(),
                    });
                }
                events
            }
            StreamEvent::Error { message } => {
                let pending = self.reasoning.flush();
                let mut events = self.block_events(pending);
                events.push(A2uiEvent::RunError {
                    run_id: self.run_id.clone(),
                    error: RunError {
                        code: "inference_error".into(),
                        phase: RunPhase::Generation,
                        message: message.clone(),
                        retryable: true,
                        diagnostics: None,
                    },
                });
                events
            }
            StreamEvent::Cancelled { message } => {
                let pending = self.reasoning.flush();
                let mut events = self.block_events(pending);
                events.push(A2uiEvent::RunCancelled {
                    run_id: self.run_id.clone(),
                    phase: RunPhase::Generation,
                    message: message.clone(),
                });
                events
            }
        }
    }
}

fn artifact_kind_for_custom_event(source: &str, event_name: &str) -> &'static str {
    if source == "uar"
        && matches!(
            event_name,
            "agui.artifact"
                | "agui.artifact_input_request"
                | "agui.state.patch"
                | "STATE_DELTA"
                | "uar.artifact.available"
                | "uar.artifact.input_required"
        )
    {
        "a2ui"
    } else if source == "uar"
        && matches!(
            event_name,
            "agui.quality.sycophancy"
                | "agui.quality.sycophancy_corrected"
                | "agui.guardrail"
                | "agui.context.update"
                | "agui.tool_call.approval_required"
                | "agui.tool_call.denied"
                | "agui.memory.mutation"
                | "agui.budget.alert"
                | "runtime.step"
                | "STEP_STARTED"
                | "STEP_FINISHED"
                | "uar.quality.sycophancy_flagged"
                | "uar.quality.sycophancy_corrected"
                | "uar.guardrail.flagged"
                | "uar.context.updated"
        )
    {
        "lifecycle"
    } else {
        "provider_event"
    }
}

/// A2uiEvent -> AguiEvent(s), bidirectional-capable.
pub struct AguiAdapter {
    thread_id: String,
    /// Retained for the bidirectional path (client→agent) filled in a later lane.
    #[allow(dead_code)]
    run_id: String,
}
impl AguiAdapter {
    pub fn new(thread_id: impl Into<String>, run_id: impl Into<String>) -> Self {
        Self {
            thread_id: thread_id.into(),
            run_id: run_id.into(),
        }
    }
    pub fn translate(&mut self, ev: &A2uiEvent) -> Vec<AguiEvent> {
        match ev {
            A2uiEvent::RunStarted { run_id } => vec![AguiEvent::RunStarted {
                thread_id: self.thread_id.clone(),
                run_id: run_id.clone(),
            }],
            A2uiEvent::Block {
                run_id: _,
                block: ContentBlock::Text { text },
            } => vec![AguiEvent::TextMessageContent {
                delta: text.clone(),
            }],
            A2uiEvent::Block {
                run_id: _,
                block: ContentBlock::Thinking { text },
            } => vec![AguiEvent::ReasoningMessageContent {
                delta: text.clone(),
            }],
            A2uiEvent::Block {
                run_id: _,
                block:
                    ContentBlock::ToolUse {
                        id,
                        name,
                        input_json,
                    },
            } => vec![
                AguiEvent::ToolCallStart {
                    id: id.clone(),
                    name: name.clone(),
                },
                AguiEvent::ToolCallArgs {
                    id: id.clone(),
                    delta: input_json.clone(),
                },
                AguiEvent::ToolCallEnd { id: id.clone() },
            ],
            A2uiEvent::Block {
                run_id: _,
                block:
                    ContentBlock::ToolResult {
                        tool_use_id,
                        output_json,
                        is_error,
                    },
            } => vec![AguiEvent::ToolCallResult {
                id: tool_use_id.clone(),
                output_json: output_json.clone(),
                is_error: *is_error,
            }],
            A2uiEvent::Block {
                run_id: _,
                block: ContentBlock::Skill { name, status },
            } => vec![AguiEvent::Custom {
                name: "uar.skill.activated".into(),
                value: serde_json::json!({
                    "skill": {"id": name},
                    "status": status,
                    "source": "knowme",
                }),
            }],
            A2uiEvent::Block {
                run_id: _,
                block: ContentBlock::Artifact { id, kind, content },
            } => vec![AguiEvent::Custom {
                name: "knowme.artifact".into(),
                value: serde_json::json!({
                    "id": id,
                    "kind": kind,
                    "content": content,
                }),
            }],
            A2uiEvent::Block { run_id: _, block } => vec![AguiEvent::Custom {
                name: "knowme.content_block".into(),
                value: serde_json::to_value(block).unwrap_or_else(|error| {
                    serde_json::json!({
                        "serializationError": error.to_string(),
                    })
                }),
            }],
            A2uiEvent::RunPhaseChanged {
                run_id,
                phase,
                status,
                message,
            } => vec![AguiEvent::Custom {
                name: "knowme.run_phase_changed".into(),
                value: serde_json::json!({
                    "runId": run_id,
                    "phase": phase,
                    "status": status,
                    "message": message,
                }),
            }],
            A2uiEvent::ContextTelemetry { run_id, payload } => vec![AguiEvent::Custom {
                name: "knowme.context_telemetry".into(),
                value: serde_json::json!({
                    "runId": run_id,
                    "payload": payload,
                }),
            }],
            A2uiEvent::RunFinished { run_id } => vec![AguiEvent::RunFinished {
                run_id: run_id.clone(),
            }],
            A2uiEvent::RunCancelled {
                run_id, message, ..
            } => vec![AguiEvent::RunCancelled {
                run_id: run_id.clone(),
                message: message.clone(),
            }],
            A2uiEvent::RunError { run_id, error } => vec![AguiEvent::RunError {
                run_id: run_id.clone(),
                error_json: serde_json::to_string(error).unwrap_or_else(|serialization_error| {
                    serde_json::json!({
                        "code": "run_error_serialization_failed",
                        "message": serialization_error.to_string(),
                    })
                    .to_string()
                }),
            }],
        }
    }
}

/// Stateful local-provider bridge for the future embedded UAR provider lane.
///
/// Local engines already emit KnowMe `StreamEvent`s. UAR consumers need AG-UI
/// style events. This bridge composes the existing `A2uiAdapter` and
/// `AguiAdapter` so reasoning parsing, tool-argument accumulation, unclosed
/// reasoning errors, and unknown-event preservation stay identical to the UI
/// path.
pub struct UarLocalProviderEventBridge {
    a2ui: A2uiAdapter,
    agui: AguiAdapter,
}

impl UarLocalProviderEventBridge {
    pub fn new(thread_id: impl Into<String>, run_id: impl Into<String>) -> Self {
        let run_id = run_id.into();
        Self {
            a2ui: A2uiAdapter::new(run_id.clone()),
            agui: AguiAdapter::new(thread_id, run_id),
        }
    }

    pub fn ingest(&mut self, event: &StreamEvent) -> Vec<AguiEvent> {
        self.a2ui
            .ingest(event)
            .into_iter()
            .flat_map(|event| self.agui.translate(&event))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rendered(events: &[A2uiEvent]) -> (String, String) {
        let mut answer = String::new();
        let mut thinking = String::new();
        for event in events {
            if let A2uiEvent::Block { block, .. } = event {
                match block {
                    ContentBlock::Text { text } => answer.push_str(text),
                    ContentBlock::Thinking { text } => thinking.push_str(text),
                    _ => {}
                }
            }
        }
        (answer, thinking)
    }

    fn parse_chunks(chunks: &[&str]) -> Vec<A2uiEvent> {
        let mut adapter = A2uiAdapter::new("run-reasoning");
        let mut output = Vec::new();
        for (index, chunk) in chunks.iter().enumerate() {
            output.extend(adapter.ingest(&StreamEvent::TextDelta {
                index: index as u32,
                delta: (*chunk).into(),
            }));
        }
        output.extend(adapter.ingest(&StreamEvent::Done));
        output
    }

    #[test]
    fn uar_llm_payload_becomes_structured_inference_request() {
        let payload = UarLlmRequestPayload {
            messages: vec![
                serde_json::json!({"role": "system", "content": "Be precise."}),
                serde_json::json!({
                    "role": "user",
                    "content": [
                        {"type": "text", "text": "Summarize this."},
                        {"type": "image_url", "image_url": {"url": "file://photo.png"}}
                    ]
                }),
            ],
            tools: vec![serde_json::json!({
                "type": "function",
                "function": {
                    "name": "knowme_diagnostics__local_runtime_status",
                    "description": "Return local status.",
                    "parameters": {
                        "type": "object",
                        "properties": {},
                        "additionalProperties": false
                    }
                },
                "annotations": {
                    "readOnlyHint": true,
                    "destructiveHint": false,
                    "openWorldHint": false
                }
            })],
            extra_params: Some(serde_json::json!({
                "temperature": 0.3,
                "top_p": 0.9,
                "top_k": 40,
                "min_p": 0.05,
                "presence_penalty": 0.1,
                "max_tokens": 333,
                "grammar": {"kind": "json_schema", "data": "{}"},
                "originating_plan": {
                    "version": "ctx-1",
                    "estimatedTokens": 123,
                    "estimator": "exact"
                }
            })),
        };

        let request = uar_llm_payload_to_inference_request(&payload).unwrap();
        assert_eq!(request.messages[0].role, InferenceRole::System);
        assert_eq!(request.messages[1].role, InferenceRole::User);
        assert!(request.messages[1].content.contains("Summarize this."));
        assert!(request.messages[1].content.contains("file://photo.png"));
        assert_eq!(request.prompt, request.messages[1].content);
        assert_eq!(
            request.tools[0].name,
            "knowme_diagnostics__local_runtime_status"
        );
        assert_eq!(
            request.tools[0]
                .annotations
                .as_ref()
                .unwrap()
                .read_only_hint,
            Some(true)
        );
        assert_eq!(request.generation_settings.temperature, 0.3);
        assert_eq!(request.generation_settings.top_p, 0.9);
        assert_eq!(request.generation_settings.top_k, 40);
        assert_eq!(request.generation_settings.max_tokens, 333);
        assert_eq!(request.grammar.as_ref().unwrap().kind, "json_schema");
        assert_eq!(request.originating_plan.as_ref().unwrap().version, "ctx-1");
    }

    #[test]
    fn uar_normalized_event_adapter_preserves_completed_tool_arguments() {
        let mut adapter = UarNormalizedEventAdapter::new();
        let mut output = Vec::new();
        output.extend(adapter.ingest(&StreamEvent::ToolCallStarted {
            id: "call-1".into(),
            name: "docs_search".into(),
        }));
        output.extend(adapter.ingest(&StreamEvent::ToolCallDelta {
            id: "call-1".into(),
            delta: r#"{"q":"#.into(),
        }));
        output.extend(adapter.ingest(&StreamEvent::ToolCallDelta {
            id: "call-1".into(),
            delta: r#""uar"}"#.into(),
        }));
        output.extend(adapter.ingest(&StreamEvent::ToolCallComplete {
            id: "call-1".into(),
        }));

        let complete = output
            .iter()
            .find(|event| {
                event.get("type").and_then(serde_json::Value::as_str) == Some("tool_call.complete")
            })
            .expect("complete event exists");
        assert_eq!(
            complete
                .pointer("/data/name")
                .and_then(serde_json::Value::as_str),
            Some("docs_search")
        );
        assert_eq!(
            complete
                .pointer("/data/arguments_json")
                .and_then(serde_json::Value::as_str),
            Some(r#"{"q":"uar"}"#)
        );
    }

    #[test]
    fn stream_events_become_uar_normalized_payloads() {
        let text = stream_event_to_uar_normalized_payload(&StreamEvent::TextDelta {
            index: 0,
            delta: "hello".into(),
        });
        assert_eq!(
            text.get("type").and_then(serde_json::Value::as_str),
            Some("message.delta")
        );
        assert_eq!(
            text.pointer("/data/text")
                .and_then(serde_json::Value::as_str),
            Some("hello")
        );

        let skill = stream_event_to_uar_normalized_payload(&StreamEvent::SkillActivation {
            name: "meeting_summary".into(),
            status: "running".into(),
        });
        assert_eq!(
            skill.get("type").and_then(serde_json::Value::as_str),
            Some("skill.activation")
        );
        assert_eq!(
            skill
                .pointer("/data/name")
                .and_then(serde_json::Value::as_str),
            Some("meeting_summary")
        );
    }

    /// Tool arguments arrive as a stream of fragments that are individually invalid
    /// JSON. The adapter must hold them until complete, then emit ONE ToolUse block —
    /// not one per fragment, and not a block carrying half an object.
    #[test]
    fn tool_call_fragments_accumulate_into_one_block() {
        let mut a = A2uiAdapter::new("run-1");

        assert!(
            a.ingest(&StreamEvent::ToolCallStarted {
                id: "c1".into(),
                name: "forge__read".into()
            })
            .is_empty(),
            "a started call emits nothing until its arguments land"
        );

        for frag in [r#"{"pa"#, r#"th":"/t"#, r#"mp"}"#] {
            assert!(
                a.ingest(&StreamEvent::ToolCallDelta {
                    id: "c1".into(),
                    delta: frag.into()
                })
                .is_empty(),
                "argument fragments must not emit blocks — {frag:?} is not valid JSON"
            );
        }

        let out = a.ingest(&StreamEvent::ToolCallComplete { id: "c1".into() });
        assert_eq!(out.len(), 1, "completion emits exactly one block");
        match &out[0] {
            A2uiEvent::Block {
                block:
                    ContentBlock::ToolUse {
                        id,
                        name,
                        input_json,
                    },
                ..
            } => {
                assert_eq!(id, "c1");
                assert_eq!(name, "forge__read");
                assert_eq!(
                    input_json, r#"{"path":"/tmp"}"#,
                    "fragments must reassemble in order"
                );
            }
            other => panic!("expected a ToolUse block, got {other:?}"),
        }
    }

    /// Two calls in one turn must not interleave their arguments.
    #[test]
    fn concurrent_tool_calls_keep_their_arguments_separate() {
        let mut a = A2uiAdapter::new("run-1");
        a.ingest(&StreamEvent::ToolCallStarted {
            id: "c1".into(),
            name: "s__a".into(),
        });
        a.ingest(&StreamEvent::ToolCallStarted {
            id: "c2".into(),
            name: "s__b".into(),
        });
        a.ingest(&StreamEvent::ToolCallDelta {
            id: "c1".into(),
            delta: r#"{"x":1"#.into(),
        });
        a.ingest(&StreamEvent::ToolCallDelta {
            id: "c2".into(),
            delta: r#"{"y":2"#.into(),
        });
        a.ingest(&StreamEvent::ToolCallDelta {
            id: "c1".into(),
            delta: "}".into(),
        });
        a.ingest(&StreamEvent::ToolCallDelta {
            id: "c2".into(),
            delta: "}".into(),
        });

        let o1 = a.ingest(&StreamEvent::ToolCallComplete { id: "c1".into() });
        let o2 = a.ingest(&StreamEvent::ToolCallComplete { id: "c2".into() });

        for (out, want_id, want_args) in [(o1, "c1", r#"{"x":1}"#), (o2, "c2", r#"{"y":2}"#)] {
            match &out[0] {
                A2uiEvent::Block {
                    block: ContentBlock::ToolUse { id, input_json, .. },
                    ..
                } => {
                    assert_eq!(id, want_id);
                    assert_eq!(input_json, want_args, "calls must not share a buffer");
                }
                other => panic!("expected ToolUse, got {other:?}"),
            }
        }
    }

    #[test]
    fn tool_results_emit_tool_result_blocks() {
        let mut a = A2uiAdapter::new("run-1");
        let out = a.ingest(&StreamEvent::ToolResult {
            tool_use_id: "c1".into(),
            output_json: r#"{"ok":false}"#.into(),
            is_error: true,
        });

        assert_eq!(out.len(), 1);
        match &out[0] {
            A2uiEvent::Block {
                block:
                    ContentBlock::ToolResult {
                        tool_use_id,
                        output_json,
                        is_error,
                    },
                ..
            } => {
                assert_eq!(tool_use_id, "c1");
                assert_eq!(output_json, r#"{"ok":false}"#);
                assert!(*is_error);
            }
            other => panic!("expected ToolResult block, got {other:?}"),
        }
    }

    /// A delta for an id we never saw started means Started was dropped or reordered.
    /// Dropping it is right — the alternative is inventing a call whose name is unknown.
    #[test]
    fn orphan_tool_deltas_are_dropped_not_guessed() {
        let mut a = A2uiAdapter::new("run-1");
        assert!(a
            .ingest(&StreamEvent::ToolCallDelta {
                id: "ghost".into(),
                delta: "{}".into()
            })
            .is_empty());
        assert!(a
            .ingest(&StreamEvent::ToolCallComplete { id: "ghost".into() })
            .is_empty());
    }

    /// The text path must be unchanged by the tool work.
    #[test]
    fn text_deltas_still_emit_text_blocks() {
        let mut a = A2uiAdapter::new("run-1");
        let out = a.ingest(&StreamEvent::TextDelta {
            index: 0,
            delta: "hi".into(),
        });
        assert!(matches!(
            &out[0],
            A2uiEvent::Block { block: ContentBlock::Text { text }, .. } if text == "hi"
        ));
    }

    #[test]
    fn reasoning_delimiters_survive_every_stream_boundary() {
        let input = "before<think>reasoning ✓</think>after";
        for split in 0..=input.len() {
            if !input.is_char_boundary(split) {
                continue;
            }
            let output = parse_chunks(&[&input[..split], &input[split..]]);
            assert_eq!(
                rendered(&output),
                ("beforeafter".into(), "reasoning ✓".into()),
                "split at byte {split}"
            );
            assert!(matches!(output.last(), Some(A2uiEvent::RunFinished { .. })));
        }
    }

    #[test]
    fn multiple_reasoning_sections_remain_separate_from_answer() {
        let output = parse_chunks(&["a<th", "ink>one</th", "ink>b<think>two</think>c"]);
        assert_eq!(rendered(&output), ("abc".into(), "onetwo".into()));
    }

    #[test]
    fn dedicated_reasoning_fields_emit_thinking_blocks() {
        let mut adapter = A2uiAdapter::new("run-1");
        let output = adapter.ingest(&StreamEvent::ThinkingDelta {
            index: 0,
            delta: "provider reasoning".into(),
        });
        assert!(matches!(
            &output[0],
            A2uiEvent::Block { block: ContentBlock::Thinking { text }, .. }
                if text == "provider reasoning"
        ));
    }

    #[test]
    fn skill_activations_emit_skill_blocks() {
        let mut adapter = A2uiAdapter::new("run-1");
        let output = adapter.ingest(&StreamEvent::SkillActivation {
            name: "document_ingest".into(),
            status: "running".into(),
        });
        assert!(matches!(
            &output[0],
            A2uiEvent::Block { block: ContentBlock::Skill { name, status }, .. }
                if name == "document_ingest" && status == "running"
        ));
    }

    #[test]
    fn unclosed_reasoning_is_preserved_and_fails_the_run() {
        let output = parse_chunks(&["answer<think>unfinished"]);
        assert_eq!(rendered(&output), ("answer".into(), "unfinished".into()));
        assert!(matches!(
            output.last(),
            Some(A2uiEvent::RunError { error, .. }) if error.code == "protocol_error"
        ));
    }

    #[test]
    fn unknown_provider_events_are_preserved_as_artifacts() {
        let mut adapter = A2uiAdapter::new("run-1");
        let output = adapter.ingest(&StreamEvent::Custom {
            source: "provider-x".into(),
            event_name: "new_chunk".into(),
            payload: serde_json::json!({"opaque": true}),
        });
        assert!(matches!(
            &output[0],
            A2uiEvent::Block {
                block: ContentBlock::Artifact { kind, content, .. }, ..
            } if kind == "provider_event" && content.contains("new_chunk")
        ));
    }

    #[test]
    fn agui_adapter_round_trips_reasoning_through_uar_normalizer() {
        let mut agui = AguiAdapter::new("thread-1", "run-1");
        let events = agui.translate(&A2uiEvent::Block {
            run_id: "run-1".into(),
            block: ContentBlock::Thinking {
                text: "plain reasoning".into(),
            },
        });

        assert_eq!(
            events,
            vec![AguiEvent::ReasoningMessageContent {
                delta: "plain reasoning".into()
            }]
        );

        let stream = normalize_uar_agui_event(&serde_json::to_value(&events[0]).unwrap());
        assert!(matches!(
            stream,
            StreamEvent::ThinkingDelta { delta, .. } if delta == "plain reasoning"
        ));
    }

    #[test]
    fn agui_adapter_round_trips_tool_use_lifecycle() {
        let mut agui = AguiAdapter::new("thread-1", "run-1");
        let events = agui.translate(&A2uiEvent::Block {
            run_id: "run-1".into(),
            block: ContentBlock::ToolUse {
                id: "tool-1".into(),
                name: "calendar.find".into(),
                input_json: r#"{"query":"tomorrow"}"#.into(),
            },
        });

        assert_eq!(
            events,
            vec![
                AguiEvent::ToolCallStart {
                    id: "tool-1".into(),
                    name: "calendar.find".into(),
                },
                AguiEvent::ToolCallArgs {
                    id: "tool-1".into(),
                    delta: r#"{"query":"tomorrow"}"#.into(),
                },
                AguiEvent::ToolCallEnd {
                    id: "tool-1".into(),
                },
            ]
        );

        let normalized: Vec<StreamEvent> = events
            .iter()
            .map(|event| normalize_uar_agui_event(&serde_json::to_value(event).unwrap()))
            .collect();
        assert!(matches!(
            &normalized[0],
            StreamEvent::ToolCallStarted { id, name }
                if id == "tool-1" && name == "calendar.find"
        ));
        assert!(matches!(
            &normalized[1],
            StreamEvent::ToolCallDelta { id, delta }
                if id == "tool-1" && delta == r#"{"query":"tomorrow"}"#
        ));
        assert!(matches!(
            &normalized[2],
            StreamEvent::ToolCallComplete { id } if id == "tool-1"
        ));
    }

    #[test]
    fn agui_adapter_preserves_skill_activation_as_custom_uar_event() {
        let mut agui = AguiAdapter::new("thread-1", "run-1");
        let events = agui.translate(&A2uiEvent::Block {
            run_id: "run-1".into(),
            block: ContentBlock::Skill {
                name: "office_writer".into(),
                status: "activated".into(),
            },
        });

        let stream = normalize_uar_agui_event(&serde_json::to_value(&events[0]).unwrap());
        assert!(matches!(
            stream,
            StreamEvent::SkillActivation { name, status }
                if name == "office_writer" && status == "activated"
        ));
    }

    #[test]
    fn agui_adapter_preserves_errors_and_unknown_blocks() {
        let mut agui = AguiAdapter::new("thread-1", "run-1");
        let error_events = agui.translate(&A2uiEvent::RunError {
            run_id: "run-1".into(),
            error: RunError::protocol("bad stream"),
        });
        assert!(matches!(
            &error_events[0],
            AguiEvent::RunError { run_id, error_json }
                if run_id == "run-1" && error_json.contains("bad stream")
        ));

        let artifact_events = agui.translate(&A2uiEvent::Block {
            run_id: "run-1".into(),
            block: ContentBlock::Artifact {
                id: "future-1".into(),
                kind: "provider_event".into(),
                content: r#"{"opaque":true}"#.into(),
            },
        });
        assert!(matches!(
            &artifact_events[0],
            AguiEvent::Custom { name, value }
                if name == "knowme.artifact" && value["id"] == "future-1"
        ));
    }

    #[test]
    fn inference_request_becomes_uar_driver_payload_without_flattening_roles() {
        let request = InferenceRequest {
            prompt: String::new(),
            messages: vec![
                gen_ui_types::inference::InferenceMessage {
                    role: gen_ui_types::inference::InferenceRole::System,
                    content: "Be concise".into(),
                    name: None,
                    tool_call_id: None,
                },
                gen_ui_types::inference::InferenceMessage {
                    role: gen_ui_types::inference::InferenceRole::User,
                    content: "Find tomorrow's meetings".into(),
                    name: Some("traveler".into()),
                    tool_call_id: None,
                },
                gen_ui_types::inference::InferenceMessage {
                    role: gen_ui_types::inference::InferenceRole::Tool,
                    content: r#"{"count":2}"#.into(),
                    name: None,
                    tool_call_id: Some("tool-1".into()),
                },
            ],
            tools: vec![InferenceTool {
                name: "calendar.find".into(),
                description: "Find calendar events".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {"query": {"type": "string"}},
                    "required": ["query"]
                }),
                annotations: Some(gen_ui_types::inference::InferenceToolAnnotations {
                    read_only_hint: Some(true),
                    ..Default::default()
                }),
            }],
            images: Vec::new(),
            generation_settings: gen_ui_types::inference::SampleParams {
                temperature: 0.2,
                max_tokens: 256,
                ..Default::default()
            },
            grammar: None,
            originating_plan: None,
        };

        let payload = inference_request_to_uar_llm_payload(&request);
        assert_eq!(payload.messages[0]["role"], "system");
        assert_eq!(payload.messages[1]["role"], "user");
        assert_eq!(payload.messages[1]["name"], "traveler");
        assert_eq!(payload.messages[2]["role"], "tool");
        assert_eq!(payload.messages[2]["tool_call_id"], "tool-1");
        assert_eq!(payload.tools[0]["type"], "function");
        assert_eq!(payload.tools[0]["function"]["name"], "calendar.find");
        assert_eq!(
            payload.tools[0]["function"]["parameters"]["required"][0],
            "query"
        );
        assert_eq!(
            payload.tools[0]["annotations"]["readOnlyHint"],
            serde_json::Value::Bool(true)
        );
        assert!(
            (payload.extra_params.as_ref().unwrap()["temperature"]
                .as_f64()
                .unwrap()
                - 0.2)
                .abs()
                < 0.000_001
        );
        assert_eq!(payload.extra_params.as_ref().unwrap()["max_tokens"], 256);
    }

    #[test]
    fn local_provider_event_bridge_uses_same_reasoning_and_tool_semantics() {
        let mut bridge = UarLocalProviderEventBridge::new("thread-1", "run-1");
        let mut output = Vec::new();
        for event in [
            StreamEvent::MessageStart,
            StreamEvent::TextDelta {
                index: 0,
                delta: "answer<think>private".into(),
            },
            StreamEvent::TextDelta {
                index: 1,
                delta: "</think>done".into(),
            },
            StreamEvent::ToolCallStarted {
                id: "tool-1".into(),
                name: "docs.search".into(),
            },
            StreamEvent::ToolCallDelta {
                id: "tool-1".into(),
                delta: r#"{"q":"uar"}"#.into(),
            },
            StreamEvent::ToolCallComplete {
                id: "tool-1".into(),
            },
            StreamEvent::Done,
        ] {
            output.extend(bridge.ingest(&event));
        }

        assert!(matches!(
            &output[0],
            AguiEvent::RunStarted { thread_id, run_id }
                if thread_id == "thread-1" && run_id == "run-1"
        ));
        assert!(output.iter().any(|event| matches!(
            event,
            AguiEvent::ReasoningMessageContent { delta } if delta == "private"
        )));
        assert!(output.iter().any(|event| matches!(
            event,
            AguiEvent::ToolCallStart { id, name } if id == "tool-1" && name == "docs.search"
        )));
        assert!(
            matches!(output.last(), Some(AguiEvent::RunFinished { run_id }) if run_id == "run-1")
        );
    }

    #[test]
    fn uar_agui_reasoning_maps_to_thinking_block() {
        let event = normalize_uar_agui_event(&serde_json::json!({
            "type": "REASONING_MESSAGE_CONTENT",
            "profile": "uar.agui/1",
            "runId": "run-1",
            "sequence": 2,
            "delta": "visible reasoning"
        }));
        let mut adapter = A2uiAdapter::new("run-1");
        let output = adapter.ingest(&event);
        assert!(matches!(
            &output[0],
            A2uiEvent::Block { block: ContentBlock::Thinking { text }, .. }
                if text == "visible reasoning"
        ));
    }

    #[test]
    fn uar_skill_activation_maps_to_skill_block() {
        let event = normalize_uar_agui_event(&serde_json::json!({
            "type": "CUSTOM",
            "profile": "uar.agui/1",
            "name": "uar.skill.activated",
            "value": {
                "skill": {"id": "office_writer", "name": "Office Writer"},
                "selectionMethod": "routing"
            },
            "runId": "run-1",
            "sequence": 4
        }));
        let mut adapter = A2uiAdapter::new("run-1");
        let output = adapter.ingest(&event);
        assert!(matches!(
            &output[0],
            A2uiEvent::Block { block: ContentBlock::Skill { name, status }, .. }
                if name == "office_writer" && status == "activated"
        ));
    }

    #[test]
    fn uar_citation_custom_event_maps_to_citation_block() {
        let event = normalize_uar_agui_event(&serde_json::json!({
            "type": "CUSTOM",
            "profile": "uar.agui/1",
            "name": "uar.citation.added",
            "value": {
                "source": "handbook.pdf",
                "quote": "Submit receipts within 30 days."
            },
            "runId": "run-1",
            "sequence": 5
        }));
        let mut adapter = A2uiAdapter::new("run-1");
        let output = adapter.ingest(&event);
        assert!(matches!(
            &output[0],
            A2uiEvent::Block {
                block: ContentBlock::Citation { source, quote },
                ..
            } if source == "handbook.pdf" && quote == "Submit receipts within 30 days."
        ));
    }

    #[test]
    fn uar_memory_custom_event_maps_to_memory_block() {
        let event = normalize_uar_agui_event(&serde_json::json!({
            "type": "CUSTOM",
            "profile": "uar.agui/1",
            "name": "uar.memory.updated",
            "value": {
                "operation": "retrieved",
                "key": "travel.preference",
                "value": "aisle seat"
            },
            "runId": "run-1",
            "sequence": 6
        }));
        let mut adapter = A2uiAdapter::new("run-1");
        let output = adapter.ingest(&event);
        assert!(matches!(
            &output[0],
            A2uiEvent::Block {
                block: ContentBlock::Memory {
                    operation,
                    key,
                    value
                },
                ..
            } if operation == "retrieved"
                && key == "travel.preference"
                && value.as_deref() == Some("aisle seat")
        ));
    }

    #[test]
    fn uar_artifact_custom_event_maps_to_a2ui_artifact_block() {
        let event = normalize_uar_agui_event(&serde_json::json!({
            "type": "CUSTOM",
            "profile": "uar.agui/1",
            "name": "uar.artifact.input_required",
            "value": {
                "artifactId": "artifact-1",
                "artifactType": "a2ui/confirm",
                "title": "Approve calendar change",
                "content": "{\"type\":\"object\"}",
                "metadata": {"catalogId": "urn:uar:a2ui:catalog:1"}
            },
            "runId": "run-1",
            "sequence": 7
        }));
        let mut adapter = A2uiAdapter::new("run-1");
        let output = adapter.ingest(&event);
        assert!(matches!(
            &output[0],
            A2uiEvent::Block {
                block: ContentBlock::Artifact { kind, content, .. },
                ..
            } if kind == "a2ui"
                && content.contains("uar.artifact.input_required")
                && content.contains("artifact-1")
                && content.contains("a2ui/confirm")
        ));
    }

    #[test]
    fn uar_agui_state_patch_maps_to_a2ui_artifact_block() {
        let event = normalize_uar_agui_event(&serde_json::json!({
            "type": "agui.state.patch",
            "kind": "state",
            "phase": "patch",
            "request_id": "run-1",
            "patch": [{
                "op": "add",
                "path": "/a2ui/surfaces/surface-1",
                "value": {"catalogId": "urn:uar:a2ui:catalog:1"}
            }]
        }));
        let mut adapter = A2uiAdapter::new("run-1");
        let output = adapter.ingest(&event);
        assert!(matches!(
            &output[0],
            A2uiEvent::Block {
                block: ContentBlock::Artifact { kind, content, .. },
                ..
            } if kind == "a2ui"
                && content.contains("agui.state.patch")
                && content.contains("/a2ui/surfaces/surface-1")
        ));
    }

    #[test]
    fn direct_uar_agui_service_events_map_to_semantic_blocks() {
        let events = [
            normalize_uar_agui_event(&serde_json::json!({
                "type": "agui.message.delta",
                "kind": "message",
                "phase": "delta",
                "delta": {"text": "hello"}
            })),
            normalize_uar_agui_event(&serde_json::json!({
                "type": "agui.thinking.delta",
                "kind": "thinking",
                "phase": "delta",
                "delta": {"text": "reason"}
            })),
            normalize_uar_agui_event(&serde_json::json!({
                "type": "agui.citation.added",
                "kind": "citation",
                "phase": "added",
                "citation": {
                    "url": "https://example.test/policy",
                    "title": "Policy",
                    "snippet": "Use approval gates."
                }
            })),
            normalize_uar_agui_event(&serde_json::json!({
                "type": "agui.memory.update",
                "kind": "memory",
                "phase": "update",
                "key": "preference",
                "value": "markdown",
                "operation": "set"
            })),
            normalize_uar_agui_event(&serde_json::json!({
                "type": "agui.skill.activated",
                "kind": "skill",
                "phase": "activated",
                "skill": {"id": "office_writer", "title": "Office Writer"}
            })),
            normalize_uar_agui_event(&serde_json::json!({
                "type": "agui.tool_call.complete",
                "kind": "tool_call",
                "phase": "complete",
                "id": "call-1",
                "name": "calendar.create",
                "arguments_json": "{\"title\":\"Planning\"}"
            })),
            normalize_uar_agui_event(&serde_json::json!({
                "type": "agui.tool_result",
                "kind": "tool_result",
                "id": "call-1",
                "content": "{\"ok\":true}",
                "success": true
            })),
        ];

        let mut adapter = A2uiAdapter::new("run-1");
        let output = events
            .iter()
            .flat_map(|event| adapter.ingest(event))
            .collect::<Vec<_>>();

        assert!(output.iter().any(|event| matches!(
            event,
            A2uiEvent::Block { block: ContentBlock::Text { text }, .. } if text == "hello"
        )));
        assert!(output.iter().any(|event| matches!(
            event,
            A2uiEvent::Block { block: ContentBlock::Thinking { text }, .. } if text == "reason"
        )));
        assert!(output.iter().any(|event| matches!(
            event,
            A2uiEvent::Block {
                block: ContentBlock::Citation { source, quote },
                ..
            } if source == "Policy" && quote == "Use approval gates."
        )));
        assert!(output.iter().any(|event| matches!(
            event,
            A2uiEvent::Block {
                block: ContentBlock::Memory {
                    operation,
                    key,
                    value
                },
                ..
            } if operation == "set" && key == "preference" && value.as_deref() == Some("markdown")
        )));
        assert!(output.iter().any(|event| matches!(
            event,
            A2uiEvent::Block { block: ContentBlock::Skill { name, status }, .. }
                if name == "office_writer" && status == "activated"
        )));
        assert!(output.iter().any(|event| matches!(
            event,
            A2uiEvent::Block {
                block: ContentBlock::ToolUse { id, name, input_json },
                ..
            } if id == "call-1"
                && name == "calendar.create"
                && input_json == "{\"title\":\"Planning\"}"
        )));
        assert!(output.iter().any(|event| matches!(
            event,
            A2uiEvent::Block {
                block: ContentBlock::ToolResult {
                    tool_use_id,
                    output_json,
                    is_error
                },
                ..
            } if tool_use_id == "call-1" && output_json == "{\"ok\":true}" && !is_error
        )));
    }

    #[test]
    fn direct_uar_cancelled_event_maps_to_run_cancelled() {
        let event = normalize_uar_agui_event(&serde_json::json!({
            "type": "agui.cancelled",
            "kind": "cancelled",
            "request_id": "run-1",
            "message": "User stopped the run"
        }));
        let mut adapter = A2uiAdapter::new("run-1");
        let output = adapter.ingest(&event);
        assert!(matches!(
            &output[0],
            A2uiEvent::RunCancelled { message, .. } if message == "User stopped the run"
        ));
    }

    #[test]
    fn direct_uar_lifecycle_events_map_to_lifecycle_artifacts() {
        let events = [
            normalize_uar_agui_event(&serde_json::json!({
                "type": "agui.guardrail",
                "kind": "guardrail",
                "category": "injection",
                "reason": "prompt_injection"
            })),
            normalize_uar_agui_event(&serde_json::json!({
                "type": "agui.tool_call.approval_required",
                "kind": "tool_call",
                "phase": "approval_required",
                "id": "call-1",
                "name": "calendar.create"
            })),
            normalize_uar_agui_event(&serde_json::json!({
                "type": "agui.budget.alert",
                "kind": "budget",
                "scope": "run",
                "exceeded": false
            })),
        ];
        let mut adapter = A2uiAdapter::new("run-1");
        let output = events
            .iter()
            .flat_map(|event| adapter.ingest(event))
            .collect::<Vec<_>>();

        assert_eq!(output.len(), 3);
        assert!(output.iter().all(|event| matches!(
            event,
            A2uiEvent::Block {
                block: ContentBlock::Artifact { kind, .. },
                ..
            } if kind == "lifecycle"
        )));
    }

    #[test]
    fn uar_tool_lifecycle_maps_to_tool_use_block() {
        let events = [
            normalize_uar_agui_event(&serde_json::json!({
                "type": "TOOL_CALL_START",
                "toolCallId": "call-1",
                "name": "calendar.find"
            })),
            normalize_uar_agui_event(&serde_json::json!({
                "type": "TOOL_CALL_ARGS",
                "toolCallId": "call-1",
                "delta": "{\"query\":\"tomorrow\"}"
            })),
            normalize_uar_agui_event(&serde_json::json!({
                "type": "TOOL_CALL_END",
                "toolCallId": "call-1"
            })),
        ];
        let mut adapter = A2uiAdapter::new("run-1");
        let mut output = Vec::new();
        for event in events {
            output.extend(adapter.ingest(&event));
        }
        assert!(matches!(
            &output[0],
            A2uiEvent::Block {
                block: ContentBlock::ToolUse { id, name, input_json }, ..
            } if id == "call-1" && name == "calendar.find" && input_json == "{\"query\":\"tomorrow\"}"
        ));
    }

    #[test]
    fn unknown_uar_custom_events_are_preserved_as_artifacts() {
        let event = normalize_uar_agui_event(&serde_json::json!({
            "type": "CUSTOM",
            "profile": "uar.agui/1",
            "name": "uar.future.event",
            "value": {"still": "kept"}
        }));
        let mut adapter = A2uiAdapter::new("run-1");
        let output = adapter.ingest(&event);
        assert!(matches!(
            &output[0],
            A2uiEvent::Block {
                block: ContentBlock::Artifact { kind, content, .. }, ..
            } if kind == "provider_event"
                && content.contains("uar.future.event")
                && content.contains("still")
        ));
    }
}
