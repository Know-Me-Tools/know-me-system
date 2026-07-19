// TJ-ARCH-MOB-001 compliant
//! Shared chat-send implementation. Called identically from gen_ui_ffi (mobile,
//! via frb) and tauri-plugin-gen-ui (desktop, via Tauri IPC) — the only
//! platform-specific code lives in how each leaf forwards A2uiEvents to its own
//! transport (StreamSink vs Tauri emit); this module owns everything else:
//! provider/model selection from the config DB, the liter-llm call, and
//! adapting the response stream into ContentBlocks.
use gen_ui_context::{
    presets, resolve_capabilities, AssembleRequest, CharsHeuristic, ContextBuilder, ContextError,
    ContextSources, EstimatorKind, RankedItem, SemanticKind, SessionEnvelope, StrategyConfig,
    ToolCandidate, Turn,
};
use gen_ui_protocol::A2uiAdapter;
use gen_ui_types::events::{A2uiEvent, StreamEvent};
use gen_ui_types::inference::{
    ImageInput, InferenceMessage, InferenceRequest, InferenceRole, InferenceTool,
    InferenceToolAnnotations, InferenceToolResult, LocalModelSpec, PromptPlanOrigin, SampleParams,
};
use gen_ui_types::lifecycle::{
    ModelOperationState, RunError, RunPhase, RunStatus, RuntimeDiagnostics,
};
use liter_llm::client::ClientBuilder;
use liter_llm::types::chat::ChatCompletionRequest;
use liter_llm::types::common::{
    ChatCompletionTool, ContentPart, DocumentContent, ImageDetail, ImageUrl, Message,
    SystemMessage, ToolMessage, UserContent, UserMessage,
};
use liter_llm::LlmClient;
use std::collections::BTreeMap;
use std::sync::atomic::Ordering;

use crate::error::AgentError;
use crate::state::PendingToolConfirmation;
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
const SETTING_THEME_MODE: &str = "theme_mode";
#[cfg(target_os = "ios")]
const DEFAULT_LOCAL_MODEL: &str = "qwen3.5-9b-q4-k-m";
#[cfg(target_os = "android")]
const DEFAULT_LOCAL_MODEL: &str = "gemma-4-e2b-it-litertlm";
#[cfg(not(any(target_os = "ios", target_os = "android")))]
const DEFAULT_LOCAL_MODEL: &str = "qwen3-4b-instruct-2507-q5-k-m";

/// Sampling defaults for the local lane, used when a model_pref's `params`
/// object omits them. Mirrors the middle-of-the-road values the cloud providers
/// apply server-side, so switching lanes doesn't silently change answer style.
const DEFAULT_TEMPERATURE: f32 = 0.7;
const DEFAULT_TOP_P: f32 = 0.8;
const DEFAULT_TOP_K: i32 = 20;
const DEFAULT_MIN_P: f32 = 0.0;
const DEFAULT_MAX_TOKENS: u32 = 1024;
const MAX_LOCAL_TOOL_REPAIR_ATTEMPTS: u8 = 2;

/// Request-scoped cloud credentials for anonymous hosted-web BYOK. The Axum
/// leaf owns the lifetime: this value is never persisted or placed in global
/// agent state, and is dropped as soon as the provider client is constructed.
pub struct EphemeralCloudConfig {
    pub provider: String,
    pub base_url: Option<String>,
    pub api_key: String,
    pub model: String,
}

struct ActiveRun {
    run_id: String,
    cancellation: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl ActiveRun {
    fn new(run_id: &str) -> Self {
        Self {
            run_id: run_id.to_owned(),
            cancellation: state::register_run(run_id),
        }
    }

    fn cancelled(&self) -> bool {
        self.cancellation.load(Ordering::Acquire)
    }

    fn publish_cancelled(&self, phase: RunPhase) {
        state::publish(A2uiEvent::RunCancelled {
            run_id: self.run_id.clone(),
            phase,
            message: "Cancelled by user".into(),
        });
    }
}

impl Drop for ActiveRun {
    fn drop(&mut self) {
        state::finish_run(&self.run_id);
    }
}

fn publish_run_error(
    run_id: &str,
    code: &str,
    phase: RunPhase,
    message: impl Into<String>,
    retryable: bool,
) {
    state::publish(A2uiEvent::RunError {
        run_id: run_id.to_owned(),
        error: RunError {
            code: code.into(),
            phase,
            message: message.into(),
            retryable,
            diagnostics: None,
        },
    });
}

/// Start a chat turn. Returns the run_id whose events are published on the
/// shared A2uiEvent broadcast (see `state::subscribe`); the actual generation
/// runs on a spawned task so `send` returns as soon as the provider/model are
/// resolved, matching the frb/Tauri contract's fire-and-subscribe shape.
pub async fn send(
    user_message: String,
    history: Vec<InferenceMessage>,
    conversation_id: String,
    branch_id: String,
) -> Result<String, AgentError> {
    let run_id = uuid::Uuid::new_v4().to_string();

    // Which lane? Unset → local when this surface ships an engine. This makes
    // first-run chat functional without a provider account or API key.
    let lane = state::config()?
        .get_setting(SETTING_ACTIVE_LANE)
        .await?
        .and_then(|v| v.as_str().map(str::to_owned))
        .unwrap_or_else(default_lane);

    if lane == LANE_LOCAL {
        return send_local(
            user_message,
            history,
            conversation_id,
            branch_id,
            run_id,
            Vec::new(),
        )
        .await;
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
        conversation_id,
        branch_id,
    )
}

/// Validate user-selected files in Rust and build real multimodal provider
/// parts. Local models receive bounded text extraction and trusted metadata;
/// cloud vision/document models receive base64-backed image/document parts.
pub async fn send_with_attachments(
    user_message: String,
    history: Vec<InferenceMessage>,
    attachment_paths: Vec<String>,
    conversation_id: String,
    branch_id: String,
    user_message_id: String,
) -> Result<String, AgentError> {
    if attachment_paths.is_empty() {
        return send(user_message, history, conversation_id, branch_id).await;
    }
    use base64::Engine as _;
    let run_id = uuid::Uuid::new_v4().to_string();

    let lane = state::config()?
        .get_setting(SETTING_ACTIVE_LANE)
        .await?
        .and_then(|value| value.as_str().map(str::to_owned))
        .unwrap_or_else(default_lane);
    if lane == LANE_LOCAL {
        let mut local_prompt = user_message.clone();
        local_prompt.push_str("\n\nAttached files (processed by xberg):\n");
        let mut images = Vec::new();
        for path in attachment_paths.into_iter().take(6) {
            let attachment = read_attachment(&path).await?;
            let processed = gen_ui_document::process_bytes(
                attachment.file_name.clone(),
                attachment.bytes.clone(),
            )
            .await
            .map_err(|error| {
                AgentError::Attachment(format!(
                    "xberg could not process '{}': {error}",
                    attachment.file_name
                ))
            })?;
            crate::memory::ingest_conversation_document(
                &processed,
                &conversation_id,
                &branch_id,
                &user_message_id,
                &run_id,
            )
            .await?;
            local_prompt.push_str(&format!(
                "\n### {} ({})\n{}\n",
                processed.file_name, processed.mime_type, processed.text
            ));

            // iOS and desktop retain the existing multimodal image path, but Xberg
            // still runs first so OCR/metadata is available in the text context.
            // Android's default Gemma 4 E2B LiteRT-LM lane is text-only and never
            // receives image tensors.
            if attachment.is_image && attachment.extension != "svg" && !cfg!(target_os = "android")
            {
                images.push(ImageInput {
                    file_name: attachment.file_name,
                    mime_type: attachment.mime,
                    bytes: attachment.bytes,
                });
            }
        }
        return send_local(
            local_prompt,
            history,
            conversation_id,
            branch_id,
            run_id,
            images,
        )
        .await;
    }

    let mut local_fallback = user_message.clone();
    local_fallback.push_str("\n\nAttached files (xberg extraction):\n");
    let mut parts = vec![ContentPart::Text { text: user_message }];
    for path in attachment_paths.into_iter().take(6) {
        let attachment = read_attachment(&path).await?;
        local_fallback.push_str(&format!(
            "\n- {} ({} bytes)\n",
            attachment.file_name,
            attachment.bytes.len()
        ));
        match gen_ui_document::process_bytes(attachment.file_name.clone(), attachment.bytes.clone())
            .await
        {
            Ok(processed) => {
                crate::memory::ingest_conversation_document(
                    &processed,
                    &conversation_id,
                    &branch_id,
                    &user_message_id,
                    &run_id,
                )
                .await?;
                local_fallback.push_str(&format!("```text\n{}\n```\n", processed.text));
                parts.push(ContentPart::Text {
                    text: format!(
                        "\nExtracted content from {}:\n{}",
                        processed.file_name, processed.text
                    ),
                });
            }
            Err(error) => {
                // Cloud vision/document providers may understand a binary even
                // when the on-device xberg feature set does not. Preserve the
                // original part and make the degradation visible in the prompt.
                local_fallback.push_str(&format!("[xberg extraction unavailable: {error}]\n"));
            }
        }
        let encoded = base64::engine::general_purpose::STANDARD.encode(&attachment.bytes);
        if attachment.is_image {
            parts.push(ContentPart::ImageUrl {
                image_url: ImageUrl {
                    url: format!("data:{};base64,{encoded}", attachment.mime),
                    detail: Some(ImageDetail::Auto),
                },
            });
        } else if attachment.is_document {
            parts.push(ContentPart::Document {
                document: DocumentContent {
                    data: encoded,
                    media_type: attachment.mime.into(),
                },
            });
        } else {
            parts.push(ContentPart::Text {
                text: format!(
                    "\nAttachment {} has MIME type {}; the selected model does not accept this binary format directly.",
                    attachment.file_name, attachment.mime
                ),
            });
        }
    }
    send_content(
        UserContent::Parts(parts),
        local_fallback,
        history,
        conversation_id,
        branch_id,
        run_id,
    )
    .await
}

struct AttachmentData {
    file_name: String,
    extension: String,
    mime: String,
    is_image: bool,
    is_document: bool,
    bytes: Vec<u8>,
}

async fn read_attachment(path: &str) -> Result<AttachmentData, AgentError> {
    let metadata = tokio::fs::metadata(path)
        .await
        .map_err(|error| AgentError::Attachment(format!("cannot open selected file: {error}")))?;
    if !metadata.is_file() || metadata.len() > gen_ui_document::MAX_INPUT_BYTES {
        return Err(AgentError::Attachment(
            "selected file is not a regular file or exceeds 25 MB".into(),
        ));
    }
    let path_ref = std::path::Path::new(path);
    let file_name = path_ref
        .file_name()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or("attachment")
        .to_owned();
    let extension = path_ref
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or_default()
        .to_ascii_lowercase();
    let bytes = tokio::fs::read(path)
        .await
        .map_err(|error| AgentError::Attachment(format!("cannot read selected file: {error}")))?;
    let (mime, is_image, is_document) = attachment_kind(&extension);
    Ok(AttachmentData {
        file_name,
        extension,
        mime: mime.to_owned(),
        is_image,
        is_document,
        bytes,
    })
}

fn attachment_kind(extension: &str) -> (&'static str, bool, bool) {
    match extension {
        "png" => ("image/png", true, false),
        "jpg" | "jpeg" => ("image/jpeg", true, false),
        "gif" => ("image/gif", true, false),
        "webp" => ("image/webp", true, false),
        "svg" => ("image/svg+xml", true, false),
        "pdf" => ("application/pdf", false, true),
        "txt" => ("text/plain", false, true),
        "md" => ("text/markdown", false, true),
        "json" => ("application/json", false, true),
        "csv" => ("text/csv", false, true),
        "doc" => ("application/msword", false, true),
        "docx" => (
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            false,
            true,
        ),
        "xls" => ("application/vnd.ms-excel", false, true),
        "xlsx" => (
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            false,
            true,
        ),
        "ppt" => ("application/vnd.ms-powerpoint", false, true),
        "pptx" => (
            "application/vnd.openxmlformats-officedocument.presentationml.presentation",
            false,
            true,
        ),
        "html" | "htm" => ("text/html", false, true),
        "xml" => ("application/xml", false, true),
        "zip" => ("application/zip", false, true),
        "mp4" => ("video/mp4", false, false),
        "webm" => ("video/webm", false, false),
        _ => ("application/octet-stream", false, false),
    }
}

async fn send_content(
    user_content: UserContent,
    local_fallback: String,
    history: Vec<InferenceMessage>,
    conversation_id: String,
    branch_id: String,
    run_id: String,
) -> Result<String, AgentError> {
    let lane = state::config()?
        .get_setting(SETTING_ACTIVE_LANE)
        .await?
        .and_then(|value| value.as_str().map(str::to_owned))
        .unwrap_or_else(default_lane);
    if lane == LANE_LOCAL {
        return send_local(
            local_fallback,
            history,
            conversation_id,
            branch_id,
            run_id,
            Vec::new(),
        )
        .await;
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
    let api_key = provider
        .api_key_ref
        .as_deref()
        .ok_or(AgentError::NoProvider)
        .and_then(secrets::resolve_api_key)?;
    start_cloud_run_with_content(
        user_content,
        local_fallback,
        history,
        run_id,
        provider.kind,
        provider.base_url,
        api_key,
        pref.model_id,
        conversation_id,
        branch_id,
    )
}

/// Continue a conversation from a structured client-side UI interaction.
/// Parsing here rejects malformed or non-object payloads before they reach a
/// provider. The provider receives explicit context rather than UI prose.
pub async fn send_interaction(
    interaction_json: String,
    history: Vec<InferenceMessage>,
    conversation_id: String,
    branch_id: String,
) -> Result<String, AgentError> {
    let value: serde_json::Value = serde_json::from_str(&interaction_json)
        .map_err(|error| AgentError::Interaction(format!("invalid JSON: {error}")))?;
    if !value.is_object() {
        return Err(AgentError::Interaction(
            "interaction payload must be a JSON object".into(),
        ));
    }
    send(
        format!(
            "The user submitted this structured client interaction. Continue the conversation and honor its surface/action context:\n{}",
            value
        ),
        history,
        conversation_id,
        branch_id,
    )
    .await
}

/// Execute a host-owned pending local tool confirmation and resume the active
/// local conversation with the exact stored tool result. This deliberately does
/// not route through `send_interaction`: approval is a privileged host action,
/// not model-visible prose that could be spoofed by arbitrary UI JSON.
pub async fn approve_tool_confirmation(approval_id: String) -> Result<String, AgentError> {
    let run_id = uuid::Uuid::new_v4().to_string();
    let pending = match state::take_pending_tool_confirmation(&approval_id) {
        Ok(pending) => pending,
        Err(error) => {
            let run_id_for_task = run_id.clone();
            gen_ui_runtime::spawn(async move {
                let mut adapter = A2uiAdapter::new(run_id_for_task.clone());
                for event in adapter.ingest(&StreamEvent::MessageStart) {
                    state::publish(event);
                }
                publish_run_error(
                    &run_id_for_task,
                    "tool_approval_not_found",
                    RunPhase::ToolExecution,
                    format!(
                        "This tool approval is no longer active. Retry the prompt to create a fresh approval request. Details: {error}"
                    ),
                    false,
                );
            });
            return Ok(run_id);
        }
    };
    let engine = state::inference()?.clone();
    let registry = state::mcp()?.clone();
    let run_id_for_task = run_id.clone();
    let active_run = ActiveRun::new(&run_id);

    gen_ui_runtime::spawn(async move {
        use futures::StreamExt;

        let mut adapter = A2uiAdapter::new(run_id_for_task.clone());
        for event in adapter.ingest(&StreamEvent::MessageStart) {
            state::publish(event);
        }
        if active_run.cancelled() {
            active_run.publish_cancelled(RunPhase::Queued);
            return;
        }
        state::publish(A2uiEvent::RunPhaseChanged {
            run_id: run_id_for_task.clone(),
            phase: RunPhase::ToolExecution,
            status: RunStatus::Preparing,
            message: format!(
                "Executing approved local tool call {} from run {}",
                pending.tool_name, pending.source_run_id
            ),
        });

        for event in adapter.ingest(&StreamEvent::ToolCallStarted {
            id: pending.tool_use_id.clone(),
            name: pending.tool_name.clone(),
        }) {
            state::publish(event);
        }
        let arguments_json = pending.arguments.to_string();
        for event in adapter.ingest(&StreamEvent::ToolCallDelta {
            id: pending.tool_use_id.clone(),
            delta: arguments_json.clone(),
        }) {
            state::publish(event);
        }
        for event in adapter.ingest(&StreamEvent::ToolCallComplete {
            id: pending.tool_use_id.clone(),
        }) {
            state::publish(event);
        }

        if active_run.cancelled() {
            active_run.publish_cancelled(RunPhase::ToolExecution);
            return;
        }

        let (output_json, is_error) =
            call_approved_mcp_tool(&registry, &pending.tool_name, &pending.arguments).await;
        for event in adapter.ingest(&StreamEvent::ToolResult {
            tool_use_id: pending.tool_use_id.clone(),
            output_json: output_json.clone(),
            is_error,
        }) {
            state::publish(event);
        }

        if active_run.cancelled() {
            active_run.publish_cancelled(RunPhase::ToolExecution);
            return;
        }
        state::publish(A2uiEvent::RunPhaseChanged {
            run_id: run_id_for_task.clone(),
            phase: RunPhase::Generation,
            status: RunStatus::Generating,
            message: "Continuing response with approved tool result".into(),
        });

        let continuation = InferenceToolResult {
            tool_name: pending.tool_name,
            output_json,
            is_error,
        };
        let continuation_stream = match engine.continue_with_tool_results(&[continuation]).await {
            Ok(stream) => stream,
            Err(error) => {
                publish_run_error(
                    &run_id_for_task,
                    "approved_tool_continuation_failed",
                    RunPhase::Generation,
                    error.to_string(),
                    true,
                );
                return;
            }
        };
        let mut continuation_stream = std::pin::pin!(continuation_stream);
        while let Some(event) = continuation_stream.next().await {
            if active_run.cancelled() {
                active_run.publish_cancelled(RunPhase::Generation);
                return;
            }
            let terminal = matches!(event, StreamEvent::Done | StreamEvent::Error { .. });
            for a2ui in adapter.ingest(&event) {
                state::publish(a2ui);
            }
            if terminal {
                return;
            }
        }

        for event in adapter.ingest(&StreamEvent::Done) {
            state::publish(event);
        }
    });

    Ok(run_id)
}

/// Start a cloud run from request-scoped hosted-web credentials. This is the
/// anonymous BYOK path: unlike `send`, it never reads or writes the keychain or
/// config DB and therefore cannot retain a browser user's secret server-side.
pub fn send_cloud_ephemeral(
    user_message: String,
    history: Vec<InferenceMessage>,
    config: EphemeralCloudConfig,
    conversation_id: String,
    branch_id: String,
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
        conversation_id,
        branch_id,
    )
}

fn start_cloud_run(
    user_message: String,
    history: Vec<InferenceMessage>,
    run_id: String,
    provider: String,
    base_url: Option<String>,
    api_key: String,
    model: String,
    conversation_id: String,
    branch_id: String,
) -> Result<String, AgentError> {
    start_cloud_run_with_content(
        UserContent::Text(user_message.clone()),
        user_message,
        history,
        run_id,
        provider,
        base_url,
        api_key,
        model,
        conversation_id,
        branch_id,
    )
}

fn start_cloud_run_with_content(
    user_content: UserContent,
    context_text: String,
    history: Vec<InferenceMessage>,
    run_id: String,
    provider: String,
    base_url: Option<String>,
    api_key: String,
    model: String,
    conversation_id: String,
    branch_id: String,
) -> Result<String, AgentError> {
    let mut builder = ClientBuilder::new().api_key(api_key).provider(provider);
    if let Some(base_url) = base_url {
        builder = builder.base_url(base_url);
    }
    let client = builder
        .build()
        .map_err(|e| AgentError::Client(e.to_string()))?;
    let mcp_registry = state::mcp()?;
    let tools = crate::tools::tool_definitions(mcp_registry);
    let context_tools = local_tool_definitions(mcp_registry);
    let tools = (!tools.is_empty()).then_some(tools);
    let task_run_id = run_id.clone();
    let active_run = ActiveRun::new(&run_id);
    gen_ui_runtime::spawn(async move {
        run_stream(
            client,
            model,
            user_content,
            context_text,
            history,
            tools,
            context_tools,
            conversation_id,
            branch_id,
            task_run_id,
            active_run,
        )
        .await;
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

/// Persisted UI theme preference shared by all native clients.
pub async fn theme_mode() -> Result<String, AgentError> {
    let value = state::config()?
        .get_setting(SETTING_THEME_MODE)
        .await?
        .and_then(|value| value.as_str().map(str::to_owned))
        .unwrap_or_else(|| "system".into());
    match value.as_str() {
        "system" | "light" | "dark" => Ok(value),
        _ => Ok("system".into()),
    }
}

pub async fn set_theme_mode(mode: &str) -> Result<(), AgentError> {
    if !matches!(mode, "system" | "light" | "dark") {
        return Err(AgentError::Config(format!("unknown theme mode '{mode}'")));
    }
    state::config()?
        .set_setting(
            SETTING_THEME_MODE,
            serde_json::Value::String(mode.to_owned()),
        )
        .await
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

pub fn local_model_status() -> Result<ModelOperationState, AgentError> {
    Ok(state::inference()?.model_operation_state())
}

pub fn watch_local_model_status(
) -> Result<futures::stream::BoxStream<'static, ModelOperationState>, AgentError> {
    Ok(state::inference()?.watch_model_operation())
}

pub fn cancel_model_operation() -> Result<(), AgentError> {
    state::inference()?.cancel_model_operation();
    Ok(())
}

pub async fn local_runtime_diagnostics() -> Result<Option<RuntimeDiagnostics>, AgentError> {
    Ok(state::inference()?.runtime_diagnostics().await)
}

pub async fn prepare_local_model() -> Result<(), AgentError> {
    let pref = state::config()?
        .get_model_pref(SURFACE_CHAT, LANE_LOCAL)
        .await?;
    let default_params = serde_json::Value::Null;
    let json_params = pref
        .as_ref()
        .map(|value| &value.params)
        .unwrap_or(&default_params);
    let model = pref
        .as_ref()
        .map(|value| value.model_id.clone())
        .unwrap_or_else(|| DEFAULT_LOCAL_MODEL.into());
    state::inference()?
        .load(&LocalModelSpec {
            model,
            context_len: json_params
                .get("context_len")
                .and_then(serde_json::Value::as_u64)
                .and_then(|value| u32::try_from(value).ok()),
        })
        .await
        .map_err(AgentError::from)
}

/// Local-lane counterpart of `send`. Same contract: resolve the model, spawn the
/// generation, return the run_id immediately — events arrive on the shared A2UI
/// broadcast exactly as they do for cloud, so no caller can tell the lanes apart.
async fn send_local(
    user_message: String,
    history: Vec<InferenceMessage>,
    conversation_id: String,
    branch_id: String,
    run_id: String,
    images: Vec<ImageInput>,
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
        .unwrap_or(DEFAULT_LOCAL_MODEL)
        .to_owned();

    let params = SampleParams {
        temperature: param_f32(json_params, "temperature", DEFAULT_TEMPERATURE),
        top_p: param_f32(json_params, "top_p", DEFAULT_TOP_P),
        top_k: param_i32(json_params, "top_k", DEFAULT_TOP_K),
        min_p: param_f32(json_params, "min_p", DEFAULT_MIN_P),
        presence_penalty: param_f32(json_params, "presence_penalty", 0.0),
        max_tokens: param_u32(json_params, "max_tokens", DEFAULT_MAX_TOKENS),
    };
    let spec = LocalModelSpec {
        model: model_id.clone(),
        context_len: json_params
            .get("context_len")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32),
    };

    let mcp_registry = state::mcp()?.clone();
    let available_tools = local_tool_definitions(&mcp_registry);
    let tool_policies = local_tool_policies(&mcp_registry);
    let engine = engine.clone();
    let run_id_for_task = run_id.clone();
    let active_run = ActiveRun::new(&run_id);
    gen_ui_runtime::spawn(async move {
        let mut adapter = A2uiAdapter::new(run_id_for_task.clone());
        for event in adapter.ingest(&StreamEvent::MessageStart) {
            state::publish(event);
        }
        if active_run.cancelled() {
            active_run.publish_cancelled(RunPhase::Queued);
            return;
        }
        state::publish(A2uiEvent::RunPhaseChanged {
            run_id: run_id_for_task.clone(),
            phase: RunPhase::ContextAssembly,
            status: RunStatus::Preparing,
            message: "Assembling governed context".into(),
        });

        let assembled = match assemble_context(
            &model_id,
            &history,
            &user_message,
            available_tools,
            false,
            &conversation_id,
            &branch_id,
        )
        .await
        {
            Ok(context) => context,
            Err(error) => {
                publish_run_error(
                    &run_id_for_task,
                    "context_assembly_failed",
                    RunPhase::ContextAssembly,
                    error.to_string(),
                    true,
                );
                return;
            }
        };
        state::publish(A2uiEvent::ContextTelemetry {
            run_id: run_id_for_task.clone(),
            payload: serde_json::to_value(&assembled.telemetry)
                .unwrap_or_else(|_| serde_json::json!({"error": "telemetry serialization failed"})),
        });
        if active_run.cancelled() {
            active_run.publish_cancelled(RunPhase::ContextAssembly);
            return;
        }
        let request = InferenceRequest {
            prompt: String::new(),
            messages: assembled.prompt.messages,
            tools: assembled.tools,
            images,
            generation_settings: params,
            grammar: None,
            originating_plan: Some(PromptPlanOrigin {
                version: assembled.prompt.version,
                estimated_tokens: assembled.prompt.estimated_tokens,
                estimator: serde_json::to_value(&assembled.prompt.estimator)
                    .ok()
                    .and_then(|value| value.as_str().map(str::to_owned))
                    .unwrap_or_else(|| "unknown".into()),
            }),
        };
        state::publish(A2uiEvent::RunPhaseChanged {
            run_id: run_id_for_task.clone(),
            phase: RunPhase::ModelLoading,
            status: RunStatus::Preparing,
            message: "Preparing local model".into(),
        });

        if let Err(error) = engine.load(&spec).await {
            publish_run_error(
                &run_id_for_task,
                "model_load_failed",
                RunPhase::ModelLoading,
                error.to_string(),
                true,
            );
            return;
        }
        if active_run.cancelled() {
            // Some llama.cpp backends cannot interrupt a model load. Do not
            // retain a model whose requesting run was cancelled while loading.
            let _ = engine.unload().await;
            active_run.publish_cancelled(RunPhase::ModelLoading);
            return;
        }
        state::publish(A2uiEvent::RunPhaseChanged {
            run_id: run_id_for_task.clone(),
            phase: RunPhase::Generation,
            status: RunStatus::Generating,
            message: "Generating response".into(),
        });

        let stream = match engine.generate_request(&request).await {
            Ok(s) => s,
            Err(e) => {
                publish_run_error(
                    &run_id_for_task,
                    "generation_start_failed",
                    RunPhase::Generation,
                    e.to_string(),
                    true,
                );
                return;
            }
        };

        // The engine already emits StreamEvents (the seam's whole point), so the
        // adapter ingests them verbatim — identical handling to the cloud lane.
        use futures::StreamExt;
        let mut stream = std::pin::pin!(stream);
        let mut tool_gate = LocalToolCallGate::new(request.tools.clone(), tool_policies);
        let mut pending_continuation: Option<InferenceToolResult> = None;
        while let Some(event) = stream.next().await {
            if active_run.cancelled() {
                active_run.publish_cancelled(RunPhase::Generation);
                return;
            }
            let terminal = matches!(event, StreamEvent::Done | StreamEvent::Error { .. });
            let completed_tool_call = tool_gate.observe(&event);
            for a2ui in adapter.ingest(&event) {
                state::publish(a2ui);
            }
            if let Some(call) = completed_tool_call {
                state::publish(A2uiEvent::RunPhaseChanged {
                    run_id: run_id_for_task.clone(),
                    phase: RunPhase::ToolExecution,
                    status: RunStatus::Preparing,
                    message: format!("Handling local tool call {}", call.name),
                });
                let (follow_up, continuation) = tool_gate
                    .execute_or_reject(&run_id_for_task, &mcp_registry, call)
                    .await;
                for a2ui in adapter.ingest(&follow_up) {
                    state::publish(a2ui);
                }
                if let Some(result) = continuation {
                    pending_continuation = Some(result);
                    break;
                }
            }
            if terminal {
                return;
            }
        }

        while let Some(result) = pending_continuation.take() {
            if active_run.cancelled() {
                active_run.publish_cancelled(RunPhase::ToolExecution);
                return;
            }
            state::publish(A2uiEvent::RunPhaseChanged {
                run_id: run_id_for_task.clone(),
                phase: RunPhase::Generation,
                status: RunStatus::Generating,
                message: if result.is_error {
                    "Requesting corrected local tool arguments".into()
                } else {
                    "Continuing response with tool result".into()
                },
            });
            let continuation_stream = match engine.continue_with_tool_results(&[result]).await {
                Ok(stream) => stream,
                Err(error) => {
                    publish_run_error(
                        &run_id_for_task,
                        "tool_result_continuation_failed",
                        RunPhase::Generation,
                        error.to_string(),
                        true,
                    );
                    return;
                }
            };
            let mut continuation_stream = std::pin::pin!(continuation_stream);
            while let Some(continuation_event) = continuation_stream.next().await {
                if active_run.cancelled() {
                    active_run.publish_cancelled(RunPhase::Generation);
                    return;
                }
                let terminal = matches!(
                    continuation_event,
                    StreamEvent::Done | StreamEvent::Error { .. }
                );
                let completed_tool_call = tool_gate.observe(&continuation_event);
                for a2ui in adapter.ingest(&continuation_event) {
                    state::publish(a2ui);
                }
                if let Some(call) = completed_tool_call {
                    state::publish(A2uiEvent::RunPhaseChanged {
                        run_id: run_id_for_task.clone(),
                        phase: RunPhase::ToolExecution,
                        status: RunStatus::Preparing,
                        message: format!("Handling local tool call {}", call.name),
                    });
                    let (follow_up, continuation) = tool_gate
                        .execute_or_reject(&run_id_for_task, &mcp_registry, call)
                        .await;
                    for a2ui in adapter.ingest(&follow_up) {
                        state::publish(a2ui);
                    }
                    pending_continuation = continuation;
                    break;
                }
                if terminal {
                    return;
                }
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

struct InlineContextSources {
    history: Vec<Turn>,
    tools: Vec<InferenceTool>,
}

#[async_trait::async_trait]
impl ContextSources for InlineContextSources {
    async fn history(
        &self,
        _conversation_id: &str,
        _branch_id: &str,
    ) -> Result<Vec<Turn>, ContextError> {
        Ok(self.history.clone())
    }

    async fn memories(&self, query: &str, limit: usize) -> Result<Vec<RankedItem>, ContextError> {
        let hits = state::memory()
            .map_err(|error| ContextError::Source(error.to_string()))?
            .memory_search(query, limit.saturating_mul(3).max(1))
            .await
            .map_err(|error| ContextError::Source(error.to_string()))?;
        Ok(hits
            .into_iter()
            .filter(|hit| {
                hit.kind != "document"
                    && hit.kind != "document_chunk"
                    && hit
                        .provenance
                        .as_ref()
                        .is_none_or(|provenance| provenance.durable)
            })
            .take(limit)
            .map(|hit| RankedItem {
                id: hit.id.clone(),
                text: hit.text,
                score_micros: (hit.score * 1_000_000.0).round() as i64,
                source: format!("memory:{}", hit.id),
            })
            .collect())
    }

    async fn knowledge(
        &self,
        conversation_id: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<RankedItem>, ContextError> {
        let hits = state::memory()
            .map_err(|error| ContextError::Source(error.to_string()))?
            .memory_search(query, limit.saturating_mul(3).max(1))
            .await
            .map_err(|error| ContextError::Source(error.to_string()))?;
        Ok(hits
            .into_iter()
            .filter(|hit| {
                (hit.kind == "document" || hit.kind == "document_chunk")
                    && hit.provenance.as_ref().is_some_and(|provenance| {
                        provenance.durable
                            || provenance.conversation_id.as_deref() == Some(conversation_id)
                    })
            })
            .take(limit)
            .map(|hit| RankedItem {
                id: hit.id.clone(),
                text: hit.text,
                score_micros: (hit.score * 1_000_000.0).round() as i64,
                source: format!("conversation:{conversation_id}:{}", hit.id),
            })
            .collect())
    }

    async fn tools(&self, _query: &str) -> Result<Vec<ToolCandidate>, ContextError> {
        Ok(self
            .tools
            .iter()
            .cloned()
            .enumerate()
            .map(|(index, tool)| ToolCandidate {
                tool,
                score: 1.0 - (index as f32 * 0.001),
            })
            .collect())
    }
}

async fn assemble_context(
    model_id: &str,
    history: &[InferenceMessage],
    user_message: &str,
    tools: Vec<InferenceTool>,
    cloud: bool,
    conversation_id: &str,
    branch_id: &str,
) -> Result<gen_ui_context::AssembledContext, ContextError> {
    let (device, model, mut runtime) = if cloud {
        presets::cloud(model_id)
    } else if model_id == "qwen3.5-9b-q4-k-m" {
        presets::ios_qwen35()
    } else if model_id == "gemma-4-e2b-it-litertlm-sm8750" {
        presets::android_litert_gemma4_e2b_sm8750()
    } else if model_id == "gemma-4-e2b-it-litertlm" {
        presets::android_litert_gemma4_e2b()
    } else {
        presets::android_qwen3()
    };
    // The context contract supports an exact tokenizer adapter. Until the
    // provider exposes its tokenizer through the shared trait, report and use
    // the conservative chars/4 estimator with the larger safety margin.
    runtime.estimator = EstimatorKind::CharsHeuristic;
    let capabilities =
        resolve_capabilities(device, model, runtime, &SessionEnvelope::default(), false)?;
    let history = history
        .iter()
        .enumerate()
        .map(|(index, message)| {
            let semantic_kind = match message.role {
                InferenceRole::User => SemanticKind::User,
                InferenceRole::Tool => SemanticKind::ToolResult,
                InferenceRole::System | InferenceRole::Assistant => SemanticKind::AssistantAnswer,
            };
            Turn {
                id: format!("history-{index}"),
                role: message.role.clone(),
                text: message.content.clone(),
                semantic_kind,
                tool_group_id: message.tool_call_id.clone(),
            }
        })
        .collect();
    let sources = InlineContextSources { history, tools };
    ContextBuilder {
        capabilities: &capabilities,
        estimator: &CharsHeuristic,
        sources: &sources,
        summarizer: None,
    }
    .assemble(&AssembleRequest {
        conversation_id: conversation_id.into(),
        branch_id: branch_id.into(),
        assistant_id: "local-assistant".into(),
        system_prompt: "You are KnowMe, a helpful private assistant. Follow the user's instructions, use available tools when useful, and answer in the user's language.".into(),
        user_text: user_message.into(),
        thinking_requested: cloud || model_id == "qwen3.5-9b-q4-k-m",
        strategy: StrategyConfig::default(),
    })
    .await
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

fn param_i32(params: &serde_json::Value, key: &str, default: i32) -> i32 {
    params
        .get(key)
        .and_then(|value| value.as_i64())
        .and_then(|value| i32::try_from(value).ok())
        .unwrap_or(default)
}

fn local_tool_definitions(registry: &gen_ui_mcp::McpRegistry) -> Vec<InferenceTool> {
    let mut definitions = Vec::new();
    for server_name in registry.server_names() {
        let Some(server) = registry.get(&server_name) else {
            continue;
        };
        for tool in server.cached_tools() {
            definitions.push(inference_tool_from_mcp(&server_name, tool));
        }
    }
    definitions
}

fn local_tool_policies(registry: &gen_ui_mcp::McpRegistry) -> BTreeMap<String, LocalToolPolicy> {
    let mut policies = BTreeMap::new();
    for server_name in registry.server_names() {
        let Some(server) = registry.get(&server_name) else {
            continue;
        };
        let trusted = server.trust() == gen_ui_mcp::McpServerTrust::TrustedLocal;
        for tool in server.cached_tools() {
            let name = crate::tools::qualify(&server_name, &tool.name);
            let annotations = tool.annotations.unwrap_or_default();
            let mut confirmation_reasons = Vec::new();
            if annotations.read_only_hint != Some(true) {
                confirmation_reasons.push("tool_is_not_declared_read_only".to_string());
            }
            if annotations.destructive_hint != Some(false) {
                confirmation_reasons.push("tool_may_be_destructive".to_string());
            }
            if annotations.open_world_hint != Some(false) {
                confirmation_reasons.push("tool_uses_open_world_context".to_string());
            }
            let auto_execute = trusted
                && annotations.read_only_hint == Some(true)
                && annotations.destructive_hint == Some(false)
                && annotations.open_world_hint == Some(false);
            policies.insert(
                name,
                LocalToolPolicy {
                    trusted_server: trusted,
                    auto_execute,
                    confirmation_reasons,
                },
            );
        }
    }
    policies
}

fn inference_tool_from_mcp(server_name: &str, tool: gen_ui_mcp::McpTool) -> InferenceTool {
    InferenceTool {
        name: crate::tools::qualify(server_name, &tool.name),
        description: tool.description.unwrap_or_default(),
        parameters: tool.input_schema,
        annotations: tool
            .annotations
            .map(|annotations| InferenceToolAnnotations {
                title: annotations.title,
                read_only_hint: annotations.read_only_hint,
                destructive_hint: annotations.destructive_hint,
                idempotent_hint: annotations.idempotent_hint,
                open_world_hint: annotations.open_world_hint,
            }),
    }
}

#[derive(Debug)]
struct PendingLocalToolCall {
    id: String,
    name: String,
    arguments: String,
}

#[derive(Debug)]
struct LocalToolCallGate {
    tools: BTreeMap<String, InferenceTool>,
    policies: BTreeMap<String, LocalToolPolicy>,
    pending: BTreeMap<String, PendingLocalToolCall>,
    repair_attempts: u8,
}

#[derive(Debug, Clone, Default)]
struct LocalToolPolicy {
    trusted_server: bool,
    auto_execute: bool,
    confirmation_reasons: Vec<String>,
}

impl LocalToolCallGate {
    fn new(tools: Vec<InferenceTool>, policies: BTreeMap<String, LocalToolPolicy>) -> Self {
        Self {
            tools: tools
                .into_iter()
                .map(|tool| (tool.name.clone(), tool))
                .collect(),
            policies,
            pending: Default::default(),
            repair_attempts: 0,
        }
    }

    fn observe(&mut self, event: &StreamEvent) -> Option<PendingLocalToolCall> {
        match event {
            StreamEvent::ToolCallStarted { id, name } => {
                self.pending.insert(
                    id.clone(),
                    PendingLocalToolCall {
                        id: id.clone(),
                        name: name.clone(),
                        arguments: String::new(),
                    },
                );
                None
            }
            StreamEvent::ToolCallDelta { id, delta } => {
                if let Some(call) = self.pending.get_mut(id) {
                    call.arguments.push_str(delta);
                }
                None
            }
            StreamEvent::ToolCallComplete { id } => self.pending.remove(id),
            _ => None,
        }
    }

    async fn execute_or_reject(
        &mut self,
        run_id: &str,
        registry: &gen_ui_mcp::McpRegistry,
        call: PendingLocalToolCall,
    ) -> (StreamEvent, Option<InferenceToolResult>) {
        let tool_name = call.name.clone();
        let result = match self.validate_call(&call) {
            Ok(arguments) => {
                self.repair_attempts = 0;
                let policy = self.policies.get(&call.name).cloned().unwrap_or_default();
                if !policy.trusted_server {
                    Err(serde_json::json!({
                        "ok": false,
                        "code": "local_tool_server_not_trusted",
                        "message": "The local model requested a tool from an MCP server that is not trusted for automatic local execution.",
                        "tool": call.name,
                    }))
                } else if !policy.auto_execute {
                    let approval_id = uuid::Uuid::new_v4().to_string();
                    state::register_pending_tool_confirmation(PendingToolConfirmation {
                        approval_id: approval_id.clone(),
                        source_run_id: run_id.to_owned(),
                        tool_use_id: call.id.clone(),
                        tool_name: call.name.clone(),
                        arguments: arguments.clone(),
                    });
                    Err(serde_json::json!({
                        "ok": false,
                        "code": "local_tool_requires_confirmation",
                        "message": "The local model produced a valid tool-call candidate, but KnowMe did not execute it because the tool is not explicitly read-only, non-destructive, and closed-world.",
                        "tool": call.name,
                        "approvalId": approval_id.clone(),
                        "requiresConfirmation": true,
                        "executionStatus": "not_executed",
                        "reasons": policy.confirmation_reasons,
                        "confirmation": {
                            "status": "required",
                            "approvalId": approval_id,
                            "executionStatus": "not_executed",
                            "policy": "manual_confirmation_required_for_mutating_destructive_or_open_world_tools"
                        }
                    }))
                } else {
                    let (output_json, is_error) =
                        self.call_mcp_tool(registry, &call, &arguments).await;
                    Ok((
                        output_json.clone(),
                        is_error,
                        Some(InferenceToolResult {
                            tool_name,
                            output_json,
                            is_error,
                        }),
                    ))
                }
            }
            Err(error) => Err(error),
        };
        match result {
            Ok((output_json, is_error, continuation)) => (
                StreamEvent::ToolResult {
                    tool_use_id: call.id,
                    output_json,
                    is_error,
                },
                continuation,
            ),
            Err(output) => {
                let output_json = serde_json::to_string(&output).unwrap_or_else(|_| {
                    r#"{"ok":false,"code":"tool_result_serialization_failed"}"#.into()
                });
                let continuation = self.repair_continuation(&call.name, &output, &output_json);
                (
                    StreamEvent::ToolResult {
                        tool_use_id: call.id,
                        output_json,
                        is_error: true,
                    },
                    continuation,
                )
            }
        }
    }

    fn repair_continuation(
        &mut self,
        tool_name: &str,
        output: &serde_json::Value,
        output_json: &str,
    ) -> Option<InferenceToolResult> {
        let code = output
            .get("code")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        if !matches!(
            code,
            "invalid_tool_arguments_json" | "tool_arguments_schema_mismatch"
        ) {
            return None;
        }
        if self.repair_attempts >= MAX_LOCAL_TOOL_REPAIR_ATTEMPTS {
            return None;
        }
        self.repair_attempts += 1;
        Some(InferenceToolResult {
            tool_name: tool_name.to_owned(),
            output_json: output_json.to_owned(),
            is_error: true,
        })
    }

    async fn call_mcp_tool(
        &self,
        registry: &gen_ui_mcp::McpRegistry,
        call: &PendingLocalToolCall,
        arguments: &serde_json::Value,
    ) -> (String, bool) {
        call_approved_mcp_tool(registry, &call.name, arguments).await
    }

    fn validate_call(
        &self,
        call: &PendingLocalToolCall,
    ) -> Result<serde_json::Value, serde_json::Value> {
        let Some(tool) = self.tools.get(&call.name) else {
            return Err(serde_json::json!({
                "ok": false,
                "code": "unknown_tool",
                "message": "The local model requested a tool that was not in the shortlisted tool set for this turn.",
                "tool": call.name,
            }));
        };
        let arguments: serde_json::Value =
            serde_json::from_str(&call.arguments).map_err(|error| {
                serde_json::json!({
                    "ok": false,
                    "code": "invalid_tool_arguments_json",
                    "message": format!("Tool arguments are not valid JSON: {error}"),
                    "tool": call.name,
                })
            })?;
        if let Err(error) = validate_tool_arguments(&tool.parameters, &arguments) {
            return Err(error_result(
                "tool_arguments_schema_mismatch",
                error,
                &call.name,
            ));
        }
        Ok(arguments)
    }
}

async fn call_approved_mcp_tool(
    registry: &gen_ui_mcp::McpRegistry,
    tool_name: &str,
    arguments: &serde_json::Value,
) -> (String, bool) {
    match crate::tools::call_tool(registry, tool_name, &arguments.to_string()).await {
        Ok(value) => {
            let is_error = value
                .get("isError")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false);
            (
                serde_json::to_string(&value)
                    .unwrap_or_else(|_| r#"{"ok":false,"code":"tool_result_invalid"}"#.into()),
                is_error,
            )
        }
        Err(error) => (
            serde_json::json!({
                "ok": false,
                "code": "local_tool_execution_failed",
                "message": error.to_string(),
                "tool": tool_name,
            })
            .to_string(),
            true,
        ),
    }
}

fn validate_tool_arguments(
    schema: &serde_json::Value,
    arguments: &serde_json::Value,
) -> Result<(), String> {
    let validator = jsonschema::validator_for(schema)
        .map_err(|error| format!("Tool argument schema is invalid: {error}"))?;
    let errors: Vec<String> = validator
        .iter_errors(arguments)
        .take(5)
        .map(|error| error.to_string())
        .collect();
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}

fn error_result(code: &str, message: impl Into<String>, tool: &str) -> serde_json::Value {
    serde_json::json!({
        "ok": false,
        "code": code,
        "message": message.into(),
        "tool": tool,
    })
}

/// Provider-neutral planned messages into liter-llm messages. The final user
/// message retains its typed image/document parts while context selection stays
/// entirely inside `gen_ui_context`.
fn build_messages_with_content(
    planned: &[gen_ui_types::inference::InferenceMessage],
    user_content: UserContent,
) -> Vec<Message> {
    use liter_llm::types::common::{AssistantContent, AssistantMessage};

    let final_user = planned
        .iter()
        .rposition(|message| message.role == InferenceRole::User);
    planned
        .iter()
        .enumerate()
        .map(|(index, message)| match message.role {
            InferenceRole::System => Message::System(SystemMessage {
                content: UserContent::Text(message.content.clone()),
                name: message.name.clone(),
            }),
            InferenceRole::User => Message::User(UserMessage {
                content: if Some(index) == final_user {
                    user_content.clone()
                } else {
                    UserContent::Text(message.content.clone())
                },
                name: message.name.clone(),
            }),
            InferenceRole::Assistant => Message::Assistant(AssistantMessage {
                content: Some(AssistantContent::Text(message.content.clone())),
                ..Default::default()
            }),
            InferenceRole::Tool => Message::Tool(ToolMessage {
                content: message.content.clone(),
                tool_call_id: message.tool_call_id.clone().unwrap_or_default(),
                name: message.name.clone(),
            }),
        })
        .collect()
}

async fn run_stream(
    client: impl LlmClient,
    model: String,
    user_content: UserContent,
    context_text: String,
    history: Vec<InferenceMessage>,
    tools: Option<Vec<ChatCompletionTool>>,
    context_tools: Vec<InferenceTool>,
    conversation_id: String,
    branch_id: String,
    run_id: String,
    active_run: ActiveRun,
) {
    let mut adapter = A2uiAdapter::new(run_id.clone());
    for event in adapter.ingest(&StreamEvent::MessageStart) {
        state::publish(event);
    }
    if active_run.cancelled() {
        active_run.publish_cancelled(RunPhase::Queued);
        return;
    }

    state::publish(A2uiEvent::RunPhaseChanged {
        run_id: run_id.clone(),
        phase: RunPhase::ContextAssembly,
        status: RunStatus::Preparing,
        message: "Assembling governed context".into(),
    });
    let assembled = match assemble_context(
        &model,
        &history,
        &context_text,
        context_tools,
        true,
        &conversation_id,
        &branch_id,
    )
    .await
    {
        Ok(context) => context,
        Err(error) => {
            publish_run_error(
                &run_id,
                "context_assembly_failed",
                RunPhase::ContextAssembly,
                error.to_string(),
                true,
            );
            return;
        }
    };
    state::publish(A2uiEvent::ContextTelemetry {
        run_id: run_id.clone(),
        payload: serde_json::to_value(&assembled.telemetry)
            .unwrap_or_else(|_| serde_json::json!({"error": "telemetry serialization failed"})),
    });
    let request = ChatCompletionRequest {
        model,
        messages: build_messages_with_content(&assembled.prompt.messages, user_content),
        tools,
        ..Default::default()
    };
    state::publish(A2uiEvent::RunPhaseChanged {
        run_id: run_id.clone(),
        phase: RunPhase::Generation,
        status: RunStatus::Generating,
        message: "Generating response".into(),
    });

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
        if active_run.cancelled() {
            active_run.publish_cancelled(RunPhase::Generation);
            return;
        }
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

#[cfg(test)]
mod attachment_tests {
    use super::*;
    use std::sync::Arc;

    use async_trait::async_trait;
    use gen_ui_db::relational::{
        ConfigStore, ModelPref, Provider, RelationalError, RelationalResult,
    };
    use gen_ui_db_graph::{
        Embedder, EmbeddingModelInfo, GraphError, GraphStore, GraphStoreConfig, EMBED_DIM,
    };

    struct ZeroEmbedder;
    impl Embedder for ZeroEmbedder {
        fn model_info(&self) -> EmbeddingModelInfo {
            EmbeddingModelInfo {
                name: "zero-fake".into(),
                dim: EMBED_DIM,
            }
        }

        fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, GraphError> {
            Ok(texts.iter().map(|_| vec![0.0; EMBED_DIM]).collect())
        }
    }

    struct EmptyConfigStore;

    #[async_trait]
    impl ConfigStore for EmptyConfigStore {
        async fn list_providers(&self) -> RelationalResult<Vec<Provider>> {
            Ok(vec![])
        }

        async fn upsert_provider(&self, _: &Provider) -> RelationalResult<()> {
            Err(RelationalError::Sync("read-only test store".into()))
        }

        async fn delete_provider(&self, _: &str) -> RelationalResult<()> {
            Err(RelationalError::Sync("read-only test store".into()))
        }

        async fn get_model_pref(&self, _: &str, _: &str) -> RelationalResult<Option<ModelPref>> {
            Ok(None)
        }

        async fn upsert_model_pref(&self, _: &ModelPref) -> RelationalResult<()> {
            Err(RelationalError::Sync("read-only test store".into()))
        }

        async fn get_setting(&self, _: &str) -> RelationalResult<Option<serde_json::Value>> {
            Ok(None)
        }

        async fn set_setting(&self, _: &str, _: serde_json::Value) -> RelationalResult<()> {
            Err(RelationalError::Sync("read-only test store".into()))
        }
    }

    async fn init_agent_state_for_tests(database: &str) {
        gen_ui_runtime::init(Some(2));
        let memory = Arc::new(
            GraphStore::open(GraphStoreConfig {
                endpoint: "memory".to_string(),
                namespace: "test".to_string(),
                database: database.to_string(),
                embedder: Arc::new(ZeroEmbedder),
            })
            .await
            .expect("ephemeral graph store opens"),
        );
        state::init(
            crate::ConfigBackend::Postgres(Arc::new(EmptyConfigStore)),
            memory,
        );
    }

    #[tokio::test]
    async fn expired_tool_approval_publishes_typed_run_error() {
        init_agent_state_for_tests("expired_tool_approval").await;

        let run_id = approve_tool_confirmation("missing-approval".into())
            .await
            .expect("expired approval still returns a run id");
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        let (backlog, _) = state::subscribe_run(&run_id).expect("run subscription");

        assert!(backlog
            .iter()
            .any(|event| matches!(event, A2uiEvent::RunStarted { run_id: id } if id == &run_id)));
        assert!(backlog.iter().any(|event| matches!(
            event,
            A2uiEvent::RunError { run_id: id, error }
                if id == &run_id
                    && error.code == "tool_approval_not_found"
                    && error.message.contains("Retry the prompt")
        )));
    }

    #[test]
    fn attachment_types_route_images_documents_and_video_safely() {
        assert_eq!(attachment_kind("png"), ("image/png", true, false));
        assert_eq!(attachment_kind("pdf"), ("application/pdf", false, true));
        assert_eq!(attachment_kind("mp4"), ("video/mp4", false, false));
    }

    #[tokio::test]
    async fn local_tool_gate_rejects_untrusted_server() {
        let mut gate = LocalToolCallGate::new(
            vec![InferenceTool {
                name: "memory__search".into(),
                description: "Search memory".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "required": ["query"],
                    "properties": {
                        "query": {"type": "string"},
                        "limit": {"type": "integer"}
                    }
                }),
                annotations: Some(InferenceToolAnnotations {
                    read_only_hint: Some(true),
                    destructive_hint: Some(false),
                    open_world_hint: Some(false),
                    ..Default::default()
                }),
            }],
            BTreeMap::from([(
                "memory__search".into(),
                LocalToolPolicy {
                    trusted_server: false,
                    auto_execute: false,
                    ..Default::default()
                },
            )]),
        );

        assert!(gate
            .observe(&StreamEvent::ToolCallStarted {
                id: "call-1".into(),
                name: "memory__search".into(),
            })
            .is_none());
        assert!(gate
            .observe(&StreamEvent::ToolCallDelta {
                id: "call-1".into(),
                delta: r#"{"query":"roadmap","limit":3}"#.into(),
            })
            .is_none());
        let call = gate
            .observe(&StreamEvent::ToolCallComplete {
                id: "call-1".into(),
            })
            .expect("call completes");

        let (out, continuation) = gate
            .execute_or_reject("test-run", &gen_ui_mcp::McpRegistry::new(), call)
            .await;
        assert!(continuation.is_none());
        match out {
            StreamEvent::ToolResult {
                tool_use_id,
                output_json,
                is_error,
            } => {
                assert_eq!(tool_use_id, "call-1");
                assert!(is_error);
                let parsed: serde_json::Value =
                    serde_json::from_str(&output_json).expect("tool result is JSON");
                assert_eq!(parsed["code"], "local_tool_server_not_trusted");
            }
            other => panic!("expected ToolResult, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn local_tool_gate_executes_trusted_read_only_closed_world_tool() {
        struct StaticTransport;

        #[async_trait::async_trait]
        impl gen_ui_mcp::McpTransport for StaticTransport {
            async fn request(
                &self,
                method: &str,
                params: Option<serde_json::Value>,
            ) -> gen_ui_types::CoreResult<serde_json::Value> {
                match method {
                    "tools/list" => Ok(serde_json::json!({
                        "tools": [{
                            "name": "search",
                            "description": "Search memory",
                            "inputSchema": {
                                "type": "object",
                                "required": ["query"],
                                "properties": {
                                    "query": {"type": "string"}
                                }
                            },
                            "annotations": {
                                "readOnlyHint": true,
                                "destructiveHint": false,
                                "openWorldHint": false
                            }
                        }]
                    })),
                    "tools/call" => {
                        assert_eq!(
                            params
                                .as_ref()
                                .and_then(|value| value.pointer("/arguments/query"))
                                .and_then(serde_json::Value::as_str),
                            Some("roadmap")
                        );
                        Ok(serde_json::json!({
                            "content": [{"type": "text", "text": "roadmap result"}],
                            "structuredContent": {"hits": 1},
                            "isError": false
                        }))
                    }
                    other => Err(gen_ui_types::CoreError::Terminal(format!(
                        "unexpected MCP method {other}"
                    ))),
                }
            }
        }

        let registry = gen_ui_mcp::McpRegistry::new();
        let handle = registry.register(gen_ui_mcp::McpServerHandle::new_trusted_local(
            "memory",
            Box::new(StaticTransport),
        ));
        handle.refresh_tools().await.expect("tools refresh");
        let tools = local_tool_definitions(&registry);
        let policies = local_tool_policies(&registry);
        let mut gate = LocalToolCallGate::new(tools, policies);

        gate.observe(&StreamEvent::ToolCallStarted {
            id: "call-1".into(),
            name: "memory__search".into(),
        });
        gate.observe(&StreamEvent::ToolCallDelta {
            id: "call-1".into(),
            delta: r#"{"query":"roadmap"}"#.into(),
        });
        let call = gate
            .observe(&StreamEvent::ToolCallComplete {
                id: "call-1".into(),
            })
            .expect("call completes");
        let (out, continuation) = gate.execute_or_reject("test-run", &registry, call).await;

        match out {
            StreamEvent::ToolResult {
                tool_use_id,
                output_json,
                is_error,
            } => {
                assert_eq!(tool_use_id, "call-1");
                assert!(!is_error);
                let parsed: serde_json::Value =
                    serde_json::from_str(&output_json).expect("tool result is JSON");
                assert_eq!(parsed["structuredContent"]["hits"], 1);
                let continuation = continuation.expect("safe execution continues the model");
                assert_eq!(continuation.tool_name, "memory__search");
                assert_eq!(continuation.output_json, output_json);
                assert!(!continuation.is_error);
            }
            other => panic!("expected ToolResult, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn local_tool_gate_requires_confirmation_for_non_closed_world_tool() {
        let mut gate = LocalToolCallGate::new(
            vec![InferenceTool {
                name: "web__search".into(),
                description: "Search the web".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "required": ["query"],
                    "properties": {
                        "query": {"type": "string"}
                    }
                }),
                annotations: Some(InferenceToolAnnotations {
                    read_only_hint: Some(true),
                    destructive_hint: Some(false),
                    open_world_hint: Some(true),
                    ..Default::default()
                }),
            }],
            BTreeMap::from([(
                "web__search".into(),
                LocalToolPolicy {
                    trusted_server: true,
                    auto_execute: false,
                    confirmation_reasons: vec!["tool_uses_open_world_context".into()],
                },
            )]),
        );
        gate.observe(&StreamEvent::ToolCallStarted {
            id: "call-1".into(),
            name: "web__search".into(),
        });
        gate.observe(&StreamEvent::ToolCallDelta {
            id: "call-1".into(),
            delta: r#"{"query":"roadmap"}"#.into(),
        });
        let call = gate
            .observe(&StreamEvent::ToolCallComplete {
                id: "call-1".into(),
            })
            .expect("call completes");
        let (out, continuation) = gate
            .execute_or_reject("test-run", &gen_ui_mcp::McpRegistry::new(), call)
            .await;
        assert!(continuation.is_none());

        match out {
            StreamEvent::ToolResult { output_json, .. } => {
                let parsed: serde_json::Value =
                    serde_json::from_str(&output_json).expect("tool result is JSON");
                assert_eq!(parsed["code"], "local_tool_requires_confirmation");
                assert!(
                    parsed["approvalId"]
                        .as_str()
                        .is_some_and(|value| !value.is_empty()),
                    "confirmation-required tool results must carry a host approval id"
                );
                assert_eq!(parsed["requiresConfirmation"], true);
                assert_eq!(parsed["executionStatus"], "not_executed");
                assert_eq!(
                    parsed["reasons"]
                        .as_array()
                        .expect("confirmation reasons")
                        .as_slice(),
                    [serde_json::json!("tool_uses_open_world_context")]
                );
            }
            other => panic!("expected ToolResult, got {other:?}"),
        }
    }

    #[test]
    fn local_tool_gate_rejects_schema_mismatch() {
        let gate = LocalToolCallGate::new(
            vec![InferenceTool {
                name: "memory__search".into(),
                description: "Search memory".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "required": ["query"],
                    "properties": {
                        "query": {"type": "string"},
                        "limit": {"type": "integer"}
                    }
                }),
                annotations: Some(InferenceToolAnnotations {
                    read_only_hint: Some(true),
                    destructive_hint: Some(false),
                    open_world_hint: Some(false),
                    ..Default::default()
                }),
            }],
            BTreeMap::new(),
        );
        let call = PendingLocalToolCall {
            id: "call-1".into(),
            name: "memory__search".into(),
            arguments: r#"{"query":42}"#.into(),
        };
        let error = gate.validate_call(&call).expect_err("schema mismatch");
        assert_eq!(error["code"], "tool_arguments_schema_mismatch");
    }

    #[tokio::test]
    async fn local_tool_gate_repairs_validation_errors_at_most_twice() {
        let mut gate = LocalToolCallGate::new(
            vec![InferenceTool {
                name: "memory__search".into(),
                description: "Search memory".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "required": ["query"],
                    "properties": {
                        "query": {"type": "string"}
                    }
                }),
                annotations: Some(InferenceToolAnnotations {
                    read_only_hint: Some(true),
                    destructive_hint: Some(false),
                    open_world_hint: Some(false),
                    ..Default::default()
                }),
            }],
            BTreeMap::from([(
                "memory__search".into(),
                LocalToolPolicy {
                    trusted_server: true,
                    auto_execute: true,
                    ..Default::default()
                },
            )]),
        );

        for attempt in 1..=3 {
            let call = PendingLocalToolCall {
                id: format!("call-{attempt}"),
                name: "memory__search".into(),
                arguments: r#"{"query":42}"#.into(),
            };
            let (out, continuation) = gate
                .execute_or_reject("test-run", &gen_ui_mcp::McpRegistry::new(), call)
                .await;
            match out {
                StreamEvent::ToolResult {
                    output_json,
                    is_error,
                    ..
                } => {
                    assert!(is_error);
                    let parsed: serde_json::Value =
                        serde_json::from_str(&output_json).expect("tool result is JSON");
                    assert_eq!(parsed["code"], "tool_arguments_schema_mismatch");
                }
                other => panic!("expected ToolResult, got {other:?}"),
            }
            if attempt <= 2 {
                let continuation = continuation.expect("validation repair continues the model");
                assert_eq!(continuation.tool_name, "memory__search");
                assert!(continuation.is_error);
                assert!(continuation
                    .output_json
                    .contains("tool_arguments_schema_mismatch"));
            } else {
                assert!(
                    continuation.is_none(),
                    "third validation failure must not keep repairing"
                );
            }
        }
    }

    #[tokio::test]
    async fn local_tool_gate_does_not_repair_confirmation_required_calls() {
        let mut gate = LocalToolCallGate::new(
            vec![InferenceTool {
                name: "files__delete".into(),
                description: "Delete a file".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "required": ["path"],
                    "properties": {
                        "path": {"type": "string"}
                    }
                }),
                annotations: Some(InferenceToolAnnotations {
                    read_only_hint: Some(false),
                    destructive_hint: Some(true),
                    open_world_hint: Some(false),
                    ..Default::default()
                }),
            }],
            BTreeMap::from([(
                "files__delete".into(),
                LocalToolPolicy {
                    trusted_server: true,
                    auto_execute: false,
                    confirmation_reasons: vec!["tool_may_be_destructive".into()],
                },
            )]),
        );
        let call = PendingLocalToolCall {
            id: "call-1".into(),
            name: "files__delete".into(),
            arguments: r#"{"path":"/tmp/example"}"#.into(),
        };
        let (out, continuation) = gate
            .execute_or_reject("test-run", &gen_ui_mcp::McpRegistry::new(), call)
            .await;
        assert!(continuation.is_none());
        match out {
            StreamEvent::ToolResult { output_json, .. } => {
                let parsed: serde_json::Value =
                    serde_json::from_str(&output_json).expect("tool result is JSON");
                assert_eq!(parsed["code"], "local_tool_requires_confirmation");
                assert!(
                    parsed["approvalId"]
                        .as_str()
                        .is_some_and(|value| !value.is_empty()),
                    "confirmation-required tool results must carry a host approval id"
                );
                assert_eq!(parsed["requiresConfirmation"], true);
                assert_eq!(parsed["executionStatus"], "not_executed");
                assert_eq!(
                    parsed["reasons"]
                        .as_array()
                        .expect("confirmation reasons")
                        .as_slice(),
                    [serde_json::json!("tool_may_be_destructive")]
                );
            }
            other => panic!("expected ToolResult, got {other:?}"),
        }
    }

    #[test]
    fn local_tool_schema_validation_handles_nested_required_enum_and_extra_fields() {
        let schema = serde_json::json!({
            "type": "object",
            "required": ["query", "filters"],
            "additionalProperties": false,
            "properties": {
                "query": {"type": "string", "minLength": 3},
                "filters": {
                    "type": "object",
                    "required": ["kind"],
                    "additionalProperties": false,
                    "properties": {
                        "kind": {"type": "string", "enum": ["memory", "knowledge"]},
                        "limit": {"type": "integer", "minimum": 1, "maximum": 10}
                    }
                }
            }
        });

        validate_tool_arguments(
            &schema,
            &serde_json::json!({
                "query": "roadmap",
                "filters": {"kind": "memory", "limit": 3}
            }),
        )
        .expect("valid nested schema arguments pass");

        let error = validate_tool_arguments(
            &schema,
            &serde_json::json!({
                "query": "roadmap",
                "filters": {"kind": "web", "limit": 3},
                "unexpected": true
            }),
        )
        .expect_err("enum and additionalProperties violations fail");
        assert!(
            error.contains("web") || error.contains("unexpected"),
            "validator should report a real schema violation, got: {error}"
        );
    }

    #[test]
    fn local_tool_schema_validation_fails_closed_for_invalid_schema() {
        let error = validate_tool_arguments(
            &serde_json::json!({"type": "definitely-not-a-json-schema-type"}),
            &serde_json::json!({}),
        )
        .expect_err("invalid schemas fail closed");
        assert!(error.contains("schema is invalid"));
    }

    #[test]
    fn mcp_tool_annotations_survive_local_inference_mapping() {
        let tool = inference_tool_from_mcp(
            "memory",
            gen_ui_mcp::McpTool {
                name: "search".into(),
                title: Some("Search".into()),
                description: Some("Search memory".into()),
                input_schema: serde_json::json!({"type": "object"}),
                annotations: Some(gen_ui_mcp::McpToolAnnotations {
                    title: Some("Search Memory".into()),
                    read_only_hint: Some(true),
                    destructive_hint: Some(false),
                    idempotent_hint: Some(true),
                    open_world_hint: Some(false),
                }),
                execution: None,
                output_schema: None,
            },
        );

        assert_eq!(tool.name, "memory__search");
        let annotations = tool.annotations.expect("annotations are preserved");
        assert_eq!(annotations.read_only_hint, Some(true));
        assert_eq!(annotations.destructive_hint, Some(false));
        assert_eq!(annotations.idempotent_hint, Some(true));
        assert_eq!(annotations.open_world_hint, Some(false));
    }
}
