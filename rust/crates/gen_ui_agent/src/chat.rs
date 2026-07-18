// TJ-ARCH-MOB-001 compliant
//! Shared chat-send implementation. Called identically from gen_ui_ffi (mobile,
//! via frb) and tauri-plugin-gen-ui (desktop, via Tauri IPC) — the only
//! platform-specific code lives in how each leaf forwards A2uiEvents to its own
//! transport (StreamSink vs Tauri emit); this module owns everything else:
//! provider/model selection from the config DB, the liter-llm call, and
//! adapting the response stream into ContentBlocks.
use gen_ui_protocol::A2uiAdapter;
use gen_ui_types::events::StreamEvent;
use gen_ui_types::inference::{LocalModelSpec, SampleParams};
use liter_llm::client::ClientBuilder;
use liter_llm::types::chat::ChatCompletionRequest;
use liter_llm::types::common::{Message, UserContent, UserMessage};
use liter_llm::LlmClient;

use crate::error::AgentError;
use crate::{secrets, state};

const SURFACE_CHAT: &str = "chat";
const LANE_CLOUD: &str = "cloud";
/// On-device inference. A `model_pref` row with this lane carries the local
/// model id in `model_id` and no `provider_id` (there is no remote provider).
const LANE_LOCAL: &str = "local";

/// Which lane a chat turn should run on. An unconfigured install prefers the
/// bundled local engine; cloud is BYOK and therefore cannot be the zero-config
/// default. Builds without a local engine still fall back to cloud visibly.
const SETTING_ACTIVE_LANE: &str = "active_lane";
const DEFAULT_LOCAL_MODEL: &str = "qwen2.5-0.5b-instruct-q4";

/// Sampling defaults for the local lane, used when a model_pref's `params`
/// object omits them. Mirrors the middle-of-the-road values the cloud providers
/// apply server-side, so switching lanes doesn't silently change answer style.
const DEFAULT_TEMPERATURE: f32 = 0.7;
const DEFAULT_TOP_P: f32 = 0.95;
const DEFAULT_MAX_TOKENS: u32 = 128;

/// Request-scoped cloud credentials for anonymous hosted-web BYOK. The Axum
/// leaf owns the lifetime: this value is never persisted or placed in global
/// agent state, and is dropped as soon as the provider client is constructed.
pub struct EphemeralCloudConfig {
    pub provider: String,
    pub base_url: Option<String>,
    pub api_key: String,
    pub model: String,
}

/// Start a chat turn. Returns the run_id whose events are published on the
/// shared A2uiEvent broadcast (see `state::subscribe`); the actual generation
/// runs on a spawned task so `send` returns as soon as the provider/model are
/// resolved, matching the frb/Tauri contract's fire-and-subscribe shape.
pub async fn send(user_message: String, history: Vec<String>) -> Result<String, AgentError> {
    let run_id = uuid::Uuid::new_v4().to_string();

    // Which lane? Unset → local when this surface ships an engine. This makes
    // first-run chat functional without a provider account or API key.
    let lane = state::config()?
        .get_setting(SETTING_ACTIVE_LANE)
        .await?
        .and_then(|v| v.as_str().map(str::to_owned))
        .unwrap_or_else(default_lane);

    if lane == LANE_LOCAL {
        return send_local(user_message, history, run_id).await;
    }

    let pref = state::config()?
        .get_model_pref(SURFACE_CHAT, LANE_CLOUD)
        .await?
        .ok_or(AgentError::NoProvider)?;
    let provider_id = pref.provider_id.as_deref().ok_or(AgentError::NoProvider)?;
    let provider = state::config()?
        .get_provider(provider_id)
        .await?
        .ok_or_else(|| AgentError::DanglingProvider(provider_id.to_string()))?;
    if !provider.enabled {
        return Err(AgentError::NoProvider);
    }
    let api_key = match &provider.api_key_ref {
        Some(key_ref) => secrets::resolve_api_key(key_ref)?,
        None => return Err(AgentError::NoProvider),
    };

    start_cloud_run(
        user_message,
        history,
        run_id,
        provider.kind,
        provider.base_url,
        api_key,
        pref.model_id,
    )
}

/// Start a cloud run from request-scoped hosted-web credentials. This is the
/// anonymous BYOK path: unlike `send`, it never reads or writes the keychain or
/// config DB and therefore cannot retain a browser user's secret server-side.
pub fn send_cloud_ephemeral(
    user_message: String,
    history: Vec<String>,
    config: EphemeralCloudConfig,
) -> Result<String, AgentError> {
    let run_id = uuid::Uuid::new_v4().to_string();
    start_cloud_run(
        user_message,
        history,
        run_id,
        config.provider,
        config.base_url,
        config.api_key,
        config.model,
    )
}

fn start_cloud_run(
    user_message: String,
    history: Vec<String>,
    run_id: String,
    provider: String,
    base_url: Option<String>,
    api_key: String,
    model: String,
) -> Result<String, AgentError> {
    let mut builder = ClientBuilder::new().api_key(api_key).provider(provider);
    if let Some(base_url) = base_url {
        builder = builder.base_url(base_url);
    }
    let client = builder
        .build()
        .map_err(|e| AgentError::Client(e.to_string()))?;
    let messages = build_messages(&history, &user_message);
    let tools = crate::tools::tool_definitions(state::mcp()?);
    let tools = (!tools.is_empty()).then_some(tools);
    let request = ChatCompletionRequest {
        model,
        messages,
        tools,
        ..Default::default()
    };
    let task_run_id = run_id.clone();
    gen_ui_runtime::spawn(async move {
        run_stream(client, request, task_run_id).await;
    });
    Ok(run_id)
}

/// Which lane chat turns currently run on: `"cloud"` or `"local"`.
pub async fn active_lane() -> Result<String, AgentError> {
    Ok(state::config()?
        .get_setting(SETTING_ACTIVE_LANE)
        .await?
        .and_then(|v| v.as_str().map(str::to_owned))
        .unwrap_or_else(default_lane))
}

fn default_lane() -> String {
    if has_local_engine() {
        LANE_LOCAL.to_string()
    } else {
        LANE_CLOUD.to_string()
    }
}

/// Switch lanes. Rejects unknown values rather than persisting a string that
/// would silently fall back to cloud on the next turn — a user who asked for
/// on-device inference must not be quietly answered by a network provider.
///
/// Selecting `local` on a build with no engine fails here, at the toggle, rather
/// than at the next chat turn — the UI can then keep the switch off and say why.
pub async fn set_active_lane(lane: &str) -> Result<(), AgentError> {
    match lane {
        LANE_CLOUD => {}
        LANE_LOCAL => {
            state::inference()?;
        }
        other => {
            return Err(AgentError::Config(format!(
                "unknown lane '{other}' — expected '{LANE_CLOUD}' or '{LANE_LOCAL}'"
            )))
        }
    }
    state::config()?
        .set_setting(
            SETTING_ACTIVE_LANE,
            serde_json::Value::String(lane.to_string()),
        )
        .await
}

/// Whether this build/platform has a local-inference engine at all. The UI uses
/// this to show or hide the lane toggle instead of offering a switch that errors.
pub fn has_local_engine() -> bool {
    state::inference().is_ok()
}

/// Local-lane counterpart of `send`. Same contract: resolve the model, spawn the
/// generation, return the run_id immediately — events arrive on the shared A2UI
/// broadcast exactly as they do for cloud, so no caller can tell the lanes apart.
async fn send_local(
    user_message: String,
    history: Vec<String>,
    run_id: String,
) -> Result<String, AgentError> {
    let engine = state::inference()?;

    // A local preference is optional. The built-in catalog model is the
    // zero-configuration path; a stored preference overrides it.
    let pref = state::config()?
        .get_model_pref(SURFACE_CHAT, LANE_LOCAL)
        .await?;
    let default_params = serde_json::Value::Null;
    let json_params = pref
        .as_ref()
        .map(|value| &value.params)
        .unwrap_or(&default_params);
    let model_id = pref
        .as_ref()
        .map(|value| value.model_id.as_str())
        .unwrap_or(DEFAULT_LOCAL_MODEL);

    let params = SampleParams {
        temperature: param_f32(json_params, "temperature", DEFAULT_TEMPERATURE),
        top_p: param_f32(json_params, "top_p", DEFAULT_TOP_P),
        max_tokens: param_u32(json_params, "max_tokens", DEFAULT_MAX_TOKENS),
    };
    let spec = LocalModelSpec {
        model: model_id.to_string(),
        context_len: json_params
            .get("context_len")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32),
    };

    // Load before returning the run_id: a first-run model download is minutes
    // long, and reporting "started" before the model exists would leave the UI
    // streaming nothing. Idempotent, so subsequent turns fall straight through.
    engine
        .load(&spec)
        .await
        .map_err(AgentError::LocalInference)?;

    let prompt = build_local_prompt(&history, &user_message);
    let engine = engine.clone();
    let run_id_for_task = run_id.clone();
    gen_ui_runtime::spawn(async move {
        let mut adapter = A2uiAdapter::new(run_id_for_task);
        for event in adapter.ingest(&StreamEvent::MessageStart) {
            state::publish(event);
        }

        let stream = match engine.generate(&prompt, &params).await {
            Ok(s) => s,
            Err(e) => {
                for event in adapter.ingest(&StreamEvent::Error {
                    message: e.to_string(),
                }) {
                    state::publish(event);
                }
                return;
            }
        };

        // The engine already emits StreamEvents (the seam's whole point), so the
        // adapter ingests them verbatim — identical handling to the cloud lane.
        use futures::StreamExt;
        let mut stream = std::pin::pin!(stream);
        while let Some(event) = stream.next().await {
            let terminal = matches!(event, StreamEvent::Done | StreamEvent::Error { .. });
            for a2ui in adapter.ingest(&event) {
                state::publish(a2ui);
            }
            if terminal {
                return;
            }
        }
        // Stream ended without Done (engine dropped the sender) — still close the
        // run so the UI stops spinning.
        for event in adapter.ingest(&StreamEvent::Done) {
            state::publish(event);
        }
    });

    Ok(run_id)
}

fn param_f32(params: &serde_json::Value, key: &str, default: f32) -> f32 {
    params
        .get(key)
        .and_then(|v| v.as_f64())
        .map(|v| v as f32)
        .unwrap_or(default)
}

fn param_u32(params: &serde_json::Value, key: &str, default: u32) -> u32 {
    params
        .get(key)
        .and_then(|v| v.as_u64())
        .map(|v| v as u32)
        .unwrap_or(default)
}

/// Flatten history + the new turn into a single prompt string.
///
/// Unlike the cloud lane (structured Messages), the InferenceProvider seam takes
/// one `&str` — the engine applies the model's own chat template underneath, so
/// roles are marked in plain text and the template does the rest.
fn build_local_prompt(history: &[String], user_message: &str) -> String {
    let mut prompt = String::new();
    for pair in history.chunks_exact(2) {
        let [role, text] = pair else { continue };
        if matches!(role.as_str(), "user" | "assistant") {
            prompt.push_str(role);
            prompt.push_str(": ");
            prompt.push_str(text);
            prompt.push('\n');
        }
    }
    prompt.push_str(user_message);
    prompt
}

/// User + prior turns (role/text pairs, oldest first) into liter-llm Messages.
fn build_messages(history: &[String], user_message: &str) -> Vec<Message> {
    use liter_llm::types::common::{AssistantContent, AssistantMessage};

    let mut messages: Vec<Message> = history
        .chunks_exact(2)
        .filter_map(|pair| {
            let [role, text] = pair else { return None };
            match role.as_str() {
                "user" => Some(Message::User(UserMessage {
                    content: UserContent::Text(text.clone()),
                    name: None,
                })),
                "assistant" => Some(Message::Assistant(AssistantMessage {
                    content: Some(AssistantContent::Text(text.clone())),
                    ..Default::default()
                })),
                _ => None,
            }
        })
        .collect();
    messages.push(Message::User(UserMessage {
        content: UserContent::Text(user_message.to_string()),
        name: None,
    }));
    messages
}

async fn run_stream(client: impl LlmClient, request: ChatCompletionRequest, run_id: String) {
    let mut adapter = A2uiAdapter::new(run_id.clone());
    for event in adapter.ingest(&StreamEvent::MessageStart) {
        state::publish(event);
    }

    let stream = match client.chat_stream(request).await {
        Ok(s) => s,
        Err(e) => {
            for event in adapter.ingest(&StreamEvent::Error {
                message: e.to_string(),
            }) {
                state::publish(event);
            }
            return;
        }
    };

    use futures::StreamExt;
    let mut stream = std::pin::pin!(stream);
    let mut index: u32 = 0;
    // Maps a tool call's stream index -> its id. Argument fragments carry only the
    // index; the id arrives once, in the call's opening chunk.
    let mut tool_call_ids: std::collections::BTreeMap<u32, String> = Default::default();
    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(chunk) => {
                for choice in &chunk.choices {
                    if let Some(delta) = &choice.delta.content {
                        for event in adapter.ingest(&StreamEvent::TextDelta {
                            index,
                            delta: delta.clone(),
                        }) {
                            state::publish(event);
                        }
                        index += 1;
                    }

                    // Tool calls stream in fragments (C-108): the id + name arrive in the
                    // first chunk for a call, then the arguments dribble in across later
                    // ones. The adapter accumulates; we just translate the wire shape.
                    for tc in choice.delta.tool_calls.iter().flatten() {
                        // `id` present = this is the opening chunk for the call.
                        if let (Some(id), Some(name)) =
                            (&tc.id, tc.function.as_ref().and_then(|f| f.name.as_ref()))
                        {
                            for event in adapter.ingest(&StreamEvent::ToolCallStarted {
                                id: id.clone(),
                                name: name.clone(),
                            }) {
                                state::publish(event);
                            }
                            tool_call_ids.insert(tc.index, id.clone());
                        }
                        // Argument fragments carry only the index, so the id comes from
                        // the opening chunk we recorded above.
                        if let Some(args) = tc.function.as_ref().and_then(|f| f.arguments.as_ref())
                        {
                            if let Some(id) = tool_call_ids.get(&tc.index) {
                                for event in adapter.ingest(&StreamEvent::ToolCallDelta {
                                    id: id.clone(),
                                    delta: args.clone(),
                                }) {
                                    state::publish(event);
                                }
                            }
                        }
                    }

                    // `tool_calls` as the finish reason means every call is complete —
                    // close them out (which is what emits the ToolUse blocks) before the
                    // run ends, or the transcript shows a turn that called nothing.
                    if choice.finish_reason.is_some() {
                        for id in tool_call_ids.values() {
                            for event in
                                adapter.ingest(&StreamEvent::ToolCallComplete { id: id.clone() })
                            {
                                state::publish(event);
                            }
                        }
                        for event in adapter.ingest(&StreamEvent::Done) {
                            state::publish(event);
                        }
                        return;
                    }
                }
            }
            Err(e) => {
                for event in adapter.ingest(&StreamEvent::Error {
                    message: e.to_string(),
                }) {
                    state::publish(event);
                }
                return;
            }
        }
    }
    // Stream ended without an explicit finish_reason (some providers omit it on
    // the final chunk) — still signal completion so the UI stops its spinner.
    for event in adapter.ingest(&StreamEvent::Done) {
        state::publish(event);
    }
}
