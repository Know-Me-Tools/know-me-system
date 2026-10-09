// TJ-ARCH-MOB-001 compliant
//! Shared chat-send implementation. Called identically from gen_ui_ffi (mobile,
//! via frb) and tauri-plugin-gen-ui (desktop, via Tauri IPC) — the only
//! platform-specific code lives in how each leaf forwards A2uiEvents to its own
//! transport (StreamSink vs Tauri emit); this module owns everything else:
//! provider/model selection from the config DB, the liter-llm call, and
//! adapting the response stream into ContentBlocks.
use futures::StreamExt;
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
/// Universal Agent Runtime execution lane. On desktop/web this is HTTP/SSE
/// service access; mobile must use embedded-library mode once that facade is
/// implemented, never a local sidecar.
const LANE_UAR: &str = "uar";

/// Which lane a chat turn should run on. An unconfigured install prefers the
/// bundled local engine; cloud is BYOK and therefore cannot be the zero-config
/// default. Builds without a local engine still fall back to cloud visibly.
const SETTING_ACTIVE_LANE: &str = "active_lane";
const SETTING_LOCAL_SYSTEM_PROMPT: &str = "local_system_prompt";

/// Shipped default for the local lane's system prompt.
///
/// Stored in the local settings database on first read so the user can edit it
/// in Settings; this constant is only the seed and the reset target, never a
/// silent override of a saved value.
pub const DEFAULT_LOCAL_SYSTEM_PROMPT: &str = "You are KnowMe, a helpful private assistant. Follow the user's instructions, use available tools when useful, and answer in the user's language.";
const SETTING_THEME_MODE: &str = "theme_mode";
const SETTING_UAR_SERVICE_URL: &str = "uar_service_url";
const SETTING_UAR_SERVICE_API_KEY_REF: &str = "uar_service_api_key_ref";
const SETTING_UAR_AGENT_ARTIFACT: &str = "uar_agent_artifact";
const SETTING_UAR_SESSION_ID: &str = "uar_session_id";
const SETTING_UAR_DEPLOYMENT_MODE: &str = "uar_deployment_mode";
#[cfg(target_os = "ios")]
const DEFAULT_LOCAL_MODEL: &str = "qwen3.5-4b-mlx-4bit";
#[cfg(target_os = "android")]
const DEFAULT_LOCAL_MODEL: &str = "gemma-4-e2b-it-litertlm";
// macOS desktop runs the MLX (mlex) engine, so its default must be an MLX
// catalog id (the same Qwen3-4B MLX set iOS uses) — a GGUF id would not resolve
// against `mlx_artifact_set_for_id` and would fail to prepare local inference.
#[cfg(target_os = "macos")]
const DEFAULT_LOCAL_MODEL: &str = "qwen3.5-4b-mlx-4bit";
// Windows/Linux desktop runs llama.cpp, so its default is the pinned GGUF model.
#[cfg(not(any(target_os = "ios", target_os = "android", target_os = "macos")))]
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
/// Upper bound on tool declarations sent to a local model in one turn. Local
/// models (notably the Android LiteRT-LM Gemma lane) have a small context
/// window, so an unbounded registry would crowd out the conversation itself.
const MAX_LOCAL_TOOL_DECLARATIONS: usize = 16;

/// Request-scoped cloud credentials for anonymous hosted-web BYOK. The Axum
/// leaf owns the lifetime: this value is never persisted or placed in global
/// agent state, and is dropped as soon as the provider client is constructed.
pub struct EphemeralCloudConfig {
    pub provider: String,
    pub base_url: Option<String>,
    pub api_key: String,
    pub model: String,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfiguredUarService {
    pub service_url: Option<String>,
    pub has_api_key: bool,
    pub has_agent_artifact: bool,
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveUarServiceRequest {
    pub service_url: String,
    pub api_key: Option<String>,
    pub agent_artifact_json: String,
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UarServiceProbeResult {
    pub service_url: String,
    pub ok: bool,
    pub http_status: Option<u16>,
    pub runtime_status: Option<String>,
    pub message: String,
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

    let lane = resolve_lane_for_conversation(&conversation_id).await?;

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
    if lane == LANE_UAR {
        return send_uar(user_message, history, conversation_id, run_id).await;
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

    let lane = resolve_lane_for_conversation(&conversation_id).await?;
    if lane == LANE_LOCAL {
        let mut local_prompt = user_message.clone();
        local_prompt.push_str("\n\nAttached files (processed by xberg):\n");
        let mut images = Vec::new();
        for path in attachment_paths.into_iter().take(6) {
            let attachment = read_attachment(&path, &conversation_id).await?;
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
        let attachment = read_attachment(&path, &conversation_id).await?;
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
                    media_type: attachment.mime,
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
    is_video: bool,
    bytes: Vec<u8>,
    /// Path inside app-owned storage — NOT the path the picker returned.
    ///
    /// The picked path is not safe to keep: on Android `file_picker` hands back
    /// a purgeable cache path, so a transcript pointing at it renders during
    /// testing and breaks after the cache is cleared. Blocks carry this path.
    stored_path: std::path::PathBuf,
}

async fn read_attachment(path: &str, conversation_id: &str) -> Result<AttachmentData, AgentError> {
    let metadata = tokio::fs::metadata(path)
        .await
        .map_err(|error| AgentError::Attachment(format!("cannot open selected file: {error}")))?;
    // Checked BEFORE reading or copying: an over-cap file must never be pulled
    // into memory or written into app storage first and rejected after.
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
    let (mime, is_image, is_document, is_video) = attachment_kind(&extension);
    let stored_path = crate::attachment_store::store(conversation_id, &extension, &bytes).await?;
    Ok(AttachmentData {
        file_name,
        extension,
        mime: mime.to_owned(),
        is_image,
        is_document,
        is_video,
        bytes,
        stored_path,
    })
}

/// `(mime, is_image, is_document, is_video)`.
///
/// Video is its own flag rather than something each renderer re-derives by
/// string-matching the mime: three surfaces guessing at `mime.startsWith("video/")`
/// is how the kind hardcodes got out of step in the first place.
pub(crate) fn attachment_kind(extension: &str) -> (&'static str, bool, bool, bool) {
    match extension {
        "png" => ("image/png", true, false, false),
        "jpg" | "jpeg" => ("image/jpeg", true, false, false),
        "gif" => ("image/gif", true, false, false),
        "webp" => ("image/webp", true, false, false),
        "svg" => ("image/svg+xml", true, false, false),
        "pdf" => ("application/pdf", false, true, false),
        "txt" => ("text/plain", false, true, false),
        "md" => ("text/markdown", false, true, false),
        "json" => ("application/json", false, true, false),
        "csv" => ("text/csv", false, true, false),
        "doc" => ("application/msword", false, true, false),
        "docx" => (
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            false,
            true,
            false,
        ),
        "xls" => ("application/vnd.ms-excel", false, true, false),
        "xlsx" => (
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            false,
            true,
            false,
        ),
        "ppt" => ("application/vnd.ms-powerpoint", false, true, false),
        "pptx" => (
            "application/vnd.openxmlformats-officedocument.presentationml.presentation",
            false,
            true,
            false,
        ),
        "html" | "htm" => ("text/html", false, true, false),
        "xml" => ("application/xml", false, true, false),
        "zip" => ("application/zip", false, true, false),
        "mp4" => ("video/mp4", false, false, true),
        "webm" => ("video/webm", false, false, true),
        "mov" => ("video/quicktime", false, false, true),
        _ => ("application/octet-stream", false, false, false),
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
    let lane = resolve_lane_for_conversation(&conversation_id).await?;
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
    if lane == LANE_UAR {
        return send_uar(local_fallback, history, conversation_id, run_id).await;
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
    let lane = resolve_lane_for_conversation(&conversation_id).await?;
    if lane == LANE_UAR {
        let run_id = uuid::Uuid::new_v4().to_string();
        return submit_uar_interaction(run_id, conversation_id, value).await;
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

async fn submit_uar_interaction(
    run_id: String,
    conversation_id: String,
    interaction: serde_json::Value,
) -> Result<String, AgentError> {
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
            phase: RunPhase::ToolExecution,
            status: RunStatus::Preparing,
            message: "Submitting A2UI response to Universal Agent Runtime".into(),
        });

        let (runtime, _runtime_config, runtime_mode) = match selected_uar_runtime().await {
            Ok(runtime) => runtime,
            Err(error) => {
                publish_run_error(
                    &run_id_for_task,
                    "uar_runtime_not_configured",
                    RunPhase::ToolExecution,
                    error.to_string(),
                    true,
                );
                return;
            }
        };

        if let Some((title, source, description)) =
            uar_a2ui_promotion_from_interaction(&interaction)
        {
            match runtime
                .promote_a2ui_component(&title, &source, description.as_deref())
                .await
            {
                Ok(component) => {
                    state::publish(A2uiEvent::ContextTelemetry {
                        run_id: run_id_for_task.clone(),
                        payload: serde_json::json!({
                            "runtime": "universal-agent-runtime",
                            "mode": runtime_mode,
                            "operation": "a2ui_component_promoted",
                            "component": component,
                        }),
                    });
                    for event in adapter.ingest(&StreamEvent::Done) {
                        state::publish(event);
                    }
                }
                Err(error) => publish_run_error(
                    &run_id_for_task,
                    "uar_a2ui_promotion_failed",
                    RunPhase::Persistence,
                    error.to_string(),
                    true,
                ),
            }
            return;
        }

        let action = uar_a2ui_action_from_interaction(&interaction);
        let response = if action.is_none() {
            match uar_artifact_response_from_interaction(&interaction) {
                Some(response) => Some(response),
                None => {
                    publish_run_error(
                        &run_id_for_task,
                        "uar_interaction_not_a2ui_response",
                        RunPhase::ToolExecution,
                        "The UAR lane only accepts typed A2UI actions or artifact responses through chat_action.",
                        false,
                    );
                    return;
                }
            }
        } else {
            None
        };
        let explicit_source_run_id = uar_source_run_id_from_interaction(&interaction);
        let mapping = match (
            explicit_source_run_id,
            state::uar_run_for_conversation(&conversation_id),
        ) {
            (Some(uar_run_id), _) => state::UarRunMapping {
                knowme_run_id: "persisted-a2ui-surface".to_owned(),
                uar_run_id,
            },
            (None, Ok(Some(mapping))) => mapping,
            (None, Ok(None)) => {
                publish_run_error(
                    &run_id_for_task,
                    "uar_run_not_found",
                    RunPhase::ToolExecution,
                    "No active UAR run is registered for this conversation. Retry the prompt to create a fresh UAR run.",
                    false,
                );
                return;
            }
            (None, Err(error)) => {
                publish_run_error(
                    &run_id_for_task,
                    "uar_run_lookup_failed",
                    RunPhase::ToolExecution,
                    error.to_string(),
                    true,
                );
                return;
            }
        };

        if active_run.cancelled() {
            active_run.publish_cancelled(RunPhase::ToolExecution);
            return;
        }
        let interaction_id = action
            .as_ref()
            .map(|value| value.surface_id.clone())
            .or_else(|| response.as_ref().map(|value| value.artifact_id.clone()))
            .unwrap_or_else(|| "a2ui".to_string());
        let continuation = match action {
            Some(action) => {
                runtime
                    .submit_a2ui_action(&mapping.uar_run_id, action)
                    .await
            }
            None => {
                runtime
                    .submit_artifact_response(
                        &mapping.uar_run_id,
                        response.expect("response checked when action is absent"),
                    )
                    .await
            }
        };
        match continuation {
            Ok(projected) => {
                let continuation_run_id = projected.run_id.clone();
                state::register_uar_run(&conversation_id, &run_id_for_task, &continuation_run_id);
                state::publish(A2uiEvent::ContextTelemetry {
                    run_id: run_id_for_task.clone(),
                    payload: serde_json::json!({
                        "runtime": "universal-agent-runtime",
                        "mode": runtime_mode,
                        "uarRunId": mapping.uar_run_id,
                        "sourceKnowmeRunId": mapping.knowme_run_id,
                        "continuationRunId": continuation_run_id,
                        "interactionId": interaction_id,
                        "operation": "artifact_response_submitted",
                    }),
                });
                let mut events = std::pin::pin!(projected.events);
                while let Some(event) = events.next().await {
                    if active_run.cancelled() {
                        let _ = runtime.cancel_run(&continuation_run_id).await;
                        active_run.publish_cancelled(RunPhase::Generation);
                        return;
                    }
                    match event {
                        Ok(event) => state::publish(event_with_run_id(event, &run_id_for_task)),
                        Err(error) => {
                            publish_run_error(
                                &run_id_for_task,
                                "uar_interaction_stream_failed",
                                RunPhase::Generation,
                                error.to_string(),
                                true,
                            );
                            return;
                        }
                    }
                }
                for event in adapter.ingest(&StreamEvent::Done) {
                    state::publish(event);
                }
            }
            Err(error) => {
                publish_run_error(
                    &run_id_for_task,
                    "uar_artifact_response_failed",
                    RunPhase::ToolExecution,
                    error.to_string(),
                    true,
                );
            }
        }
    });
    Ok(run_id)
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

/// The local lane's system prompt, falling back to the shipped default.
///
/// A blank stored value also falls back: an empty system prompt would strip the
/// assistant's entire framing, and the context assembler requires a leading
/// system message, so "cleared" means "use the default" rather than "send
/// nothing".
pub async fn local_system_prompt() -> Result<String, AgentError> {
    Ok(state::config()?
        .get_setting(SETTING_LOCAL_SYSTEM_PROMPT)
        .await?
        .and_then(|value| value.as_str().map(str::to_owned))
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_LOCAL_SYSTEM_PROMPT.to_string()))
}

/// Persist the local lane's system prompt. A blank value clears the override,
/// restoring [`DEFAULT_LOCAL_SYSTEM_PROMPT`] on the next read.
pub async fn set_local_system_prompt(prompt: &str) -> Result<(), AgentError> {
    let trimmed = prompt.trim();
    state::config()?
        .set_setting(
            SETTING_LOCAL_SYSTEM_PROMPT,
            serde_json::Value::String(trimmed.to_string()),
        )
        .await
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

pub async fn uar_service_config() -> Result<ConfiguredUarService, AgentError> {
    Ok(ConfiguredUarService {
        service_url: string_setting(SETTING_UAR_SERVICE_URL).await?,
        has_api_key: string_setting(SETTING_UAR_SERVICE_API_KEY_REF)
            .await?
            .is_some(),
        has_agent_artifact: state::config()?
            .get_setting(SETTING_UAR_AGENT_ARTIFACT)
            .await?
            .is_some_and(|value| !value.is_null()),
        session_id: string_setting(SETTING_UAR_SESSION_ID).await?,
    })
}

pub async fn uar_runtime_status() -> Result<crate::uar::UarRuntimeStatus, AgentError> {
    use crate::uar::UarRuntimeFacade;

    match selected_uar_deployment_mode().await? {
        crate::uar::UarDeploymentMode::EmbeddedLibrary => Ok(state::embedded_uar()?.status()),
        crate::uar::UarDeploymentMode::HttpService
        | crate::uar::UarDeploymentMode::CloudService => {
            let config = load_uar_service_chat_config().await?;
            Ok(crate::uar::UarServiceRuntime::new(&config.service_url, config.api_key)?.status())
        }
    }
}

pub async fn save_uar_service_config(request: SaveUarServiceRequest) -> Result<(), AgentError> {
    let service_url = request.service_url.trim();
    if service_url.is_empty() {
        return Err(AgentError::Config("UAR service URL is required".into()));
    }
    crate::uar::UarServiceRuntime::new(service_url, None)?;

    let artifact = if request.agent_artifact_json.trim().is_empty() {
        None
    } else {
        Some(artifact_from_string_or_path(&request.agent_artifact_json).await?)
    };
    let existing_key_ref = string_setting(SETTING_UAR_SERVICE_API_KEY_REF).await?;
    let api_key_ref = match request.api_key.as_deref().map(str::trim) {
        Some(api_key) if !api_key.is_empty() => {
            let reference = "uar:service";
            secrets::store_api_key(reference, api_key)?;
            Some(reference.to_owned())
        }
        _ => existing_key_ref,
    };

    let config = state::config()?;
    config
        .set_setting(
            SETTING_UAR_SERVICE_URL,
            serde_json::Value::String(service_url.to_owned()),
        )
        .await?;
    config
        .set_setting(
            SETTING_UAR_AGENT_ARTIFACT,
            artifact.unwrap_or(serde_json::Value::Null),
        )
        .await?;
    config
        .set_setting(
            SETTING_UAR_SESSION_ID,
            request
                .session_id
                .map(|value| value.trim().to_owned())
                .filter(|value| !value.is_empty())
                .map(serde_json::Value::String)
                .unwrap_or(serde_json::Value::Null),
        )
        .await?;
    config
        .set_setting(
            SETTING_UAR_SERVICE_API_KEY_REF,
            api_key_ref
                .map(serde_json::Value::String)
                .unwrap_or(serde_json::Value::Null),
        )
        .await?;
    // iOS/Android AND macOS host the UAR as the embedded in-process kernel — there
    // is no service URL and no remote deployment. Writing a remote deployment mode
    // here would flip them out of `EmbeddedLibrary` (the default from
    // `selected_uar_deployment_mode`) and make every subsequent chat turn demand
    // `uar_service_url`, which does not exist there. So only the Windows/Linux
    // service clients persist a remote deployment mode; embedded hosts keep their
    // embedded default untouched.
    #[cfg(not(any(target_os = "ios", target_os = "android", target_os = "macos")))]
    config
        .set_setting(
            SETTING_UAR_DEPLOYMENT_MODE,
            serde_json::Value::String("http_service".to_owned()),
        )
        .await?;

    Ok(())
}

pub async fn probe_uar_service_config() -> Result<UarServiceProbeResult, AgentError> {
    let config = load_uar_service_chat_config().await?;
    let base_url = reqwest::Url::parse(&config.service_url)
        .map_err(|error| AgentError::Client(format!("UAR service URL: {error}")))?;
    let health_url = base_url
        .join("/health")
        .map_err(|error| AgentError::Client(format!("UAR health URL: {error}")))?;
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|error| AgentError::Client(format!("UAR health client: {error}")))?;
    let mut request = client.get(health_url);
    if let Some(api_key) = config.api_key {
        request = request.bearer_auth(api_key);
    }
    let response = request
        .send()
        .await
        .map_err(|error| AgentError::Client(format!("UAR health request: {error}")))?;
    let http_status = response.status();
    let status_code = http_status.as_u16();
    let body = response.text().await.unwrap_or_default();
    let body_json = serde_json::from_str::<serde_json::Value>(&body).ok();
    let runtime_status = body_json
        .as_ref()
        .and_then(|value| value.get("status"))
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned);
    let ok = http_status.is_success()
        && runtime_status
            .as_deref()
            .map(|status| matches!(status, "ok" | "ready"))
            .unwrap_or(true);
    let message = if ok {
        format!(
            "UAR service reachable{}",
            runtime_status
                .as_deref()
                .map(|status| format!(" · status {status}"))
                .unwrap_or_default()
        )
    } else if let Some(status) = runtime_status.as_deref() {
        format!("UAR service returned HTTP {status_code} · status {status}")
    } else if body.trim().is_empty() {
        format!("UAR service returned HTTP {status_code}")
    } else {
        format!("UAR service returned HTTP {status_code}: {}", body.trim())
    };
    Ok(UarServiceProbeResult {
        service_url: config.service_url,
        ok,
        http_status: Some(status_code),
        runtime_status,
        message,
    })
}

fn default_lane() -> String {
    #[cfg(any(target_os = "ios", target_os = "android"))]
    if state::embedded_uar().is_ok() {
        return LANE_UAR.to_string();
    }
    if has_local_engine() {
        LANE_LOCAL.to_string()
    } else {
        LANE_CLOUD.to_string()
    }
}

/// Map a conversation policy `chat_mode` onto an execution lane.
///
/// The UI's mode vocabulary is user-facing (`local`, `uar`, `agent`); the lanes
/// are execution paths. `agent` and `uar` both run through the embedded UAR —
/// they differ only in which agent artifact the run resolves — so both map to
/// [`LANE_UAR`]. An unrecognized mode yields `None` so the caller falls back
/// rather than inventing a lane.
fn lane_for_chat_mode(mode: &str) -> Option<&'static str> {
    match mode {
        "local" => Some(LANE_LOCAL),
        "uar" | "agent" => Some(LANE_UAR),
        "cloud" => Some(LANE_CLOUD),
        _ => None,
    }
}

/// Resolve which lane a turn in this conversation runs on.
///
/// Precedence: the conversation's own `chat_mode` → the global
/// `active_lane` setting → [`default_lane`]. Conversation scope comes first so
/// one chat can be on-device while another uses the orchestrator; the global
/// setting remains the fallback so conversations saved before per-conversation
/// modes existed keep their previous behavior.
async fn resolve_lane_for_conversation(conversation_id: &str) -> Result<String, AgentError> {
    if !conversation_id.is_empty() {
        if let Ok(policy) = crate::control_plane::conversation_policy(conversation_id).await {
            if let Some(lane) = chat_mode_lane(&policy) {
                return Ok(lane.to_string());
            }
        }
    }

    Ok(state::config()?
        .get_setting(SETTING_ACTIVE_LANE)
        .await?
        .and_then(|value| value.as_str().map(str::to_owned))
        .unwrap_or_else(default_lane))
}

/// The execution lane a stored conversation policy selects, if any.
///
/// Reads `chat_mode` from the **bare** `RunPolicy` — the shape the control plane
/// returns and that `save_conversation_policy` accepts. Returns `None` when the
/// policy stores no recognized mode, so the caller falls back to the global
/// lane rather than inventing a route.
fn chat_mode_lane(policy: &serde_json::Value) -> Option<&'static str> {
    policy
        .get("chat_mode")
        .and_then(serde_json::Value::as_str)
        .and_then(lane_for_chat_mode)
}

/// Whether a lane can be selected right now, and if not, why.
///
/// The UI calls this to render unavailable modes as disabled with a reason
/// instead of letting the user tap a control that will fail. It performs the
/// same checks [`set_active_lane`] does, but is side-effect free: it never
/// registers, loads, or mutates anything.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LaneAvailability {
    pub lane: String,
    pub available: bool,
    /// Human-readable reason when `available` is false; `None` when usable.
    pub reason: Option<String>,
}

/// Report availability for every selectable lane in one call, so the mode
/// picker can render its whole list without a round-trip per entry.
pub async fn lane_availability() -> Result<Vec<LaneAvailability>, AgentError> {
    let local = match state::inference() {
        Ok(_) => LaneAvailability {
            lane: LANE_LOCAL.to_string(),
            available: true,
            reason: None,
        },
        Err(error) => LaneAvailability {
            lane: LANE_LOCAL.to_string(),
            available: false,
            reason: Some(error.to_string()),
        },
    };
    let uar = match ensure_uar_lane_supported().await {
        Ok(()) => LaneAvailability {
            lane: LANE_UAR.to_string(),
            available: true,
            reason: None,
        },
        Err(error) => LaneAvailability {
            lane: LANE_UAR.to_string(),
            available: false,
            reason: Some(error.to_string()),
        },
    };
    Ok(vec![local, uar])
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
        LANE_UAR => {
            ensure_uar_lane_supported().await?;
        }
        other => {
            return Err(AgentError::Config(format!(
                "unknown lane '{other}' — expected '{LANE_CLOUD}', '{LANE_LOCAL}', or '{LANE_UAR}'"
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

async fn ensure_uar_lane_supported() -> Result<(), AgentError> {
    selected_uar_runtime().await.map(|_| ())
}

#[derive(Clone)]
struct UarRunConfig {
    artifact: Option<serde_json::Value>,
    session_id: Option<String>,
}

pub(crate) struct UarServiceChatConfig {
    pub(crate) service_url: String,
    pub(crate) api_key: Option<String>,
    pub(crate) artifact: Option<serde_json::Value>,
    pub(crate) session_id: Option<String>,
}

pub(crate) async fn selected_uar_deployment_mode(
) -> Result<crate::uar::UarDeploymentMode, AgentError> {
    if let Some(mode) = crate::execution_profiles::deployment_mode().await? { return Ok(mode); }
    let configured = string_setting(SETTING_UAR_DEPLOYMENT_MODE)
        .await?
        .or_else(|| std::env::var("KNOWME_UAR_DEPLOYMENT_MODE").ok());
    match configured.as_deref() {
        Some("embedded") | Some("embedded_library") => {
            Ok(crate::uar::UarDeploymentMode::EmbeddedLibrary)
        }
        Some("service") | Some("http") | Some("http_service") => {
            Ok(crate::uar::UarDeploymentMode::HttpService)
        }
        Some("cloud") | Some("cloud_service") => Ok(crate::uar::UarDeploymentMode::CloudService),
        Some(other) => Err(AgentError::Config(format!(
            "unknown UAR deployment mode '{other}'"
        ))),
        None if cfg!(any(
            target_os = "ios",
            target_os = "android",
            target_os = "macos"
        )) =>
        {
            // iOS/Android and now macOS host the embedded UAR kernel in-process.
            Ok(crate::uar::UarDeploymentMode::EmbeddedLibrary)
        }
        None => Ok(crate::uar::UarDeploymentMode::HttpService),
    }
}

async fn selected_uar_runtime() -> Result<
    (
        std::sync::Arc<dyn crate::uar::UarRuntimeFacade>,
        UarRunConfig,
        &'static str,
    ),
    AgentError,
> {
    let deployment_mode = selected_uar_deployment_mode().await?;
    match deployment_mode {
        crate::uar::UarDeploymentMode::EmbeddedLibrary => {
            let runtime: std::sync::Arc<dyn crate::uar::UarRuntimeFacade> =
                state::embedded_uar()?.clone();
            Ok((
                runtime,
                UarRunConfig {
                    artifact: load_optional_uar_artifact().await?,
                    session_id: string_setting(SETTING_UAR_SESSION_ID).await?,
                },
                "embedded_library",
            ))
        }
        crate::uar::UarDeploymentMode::HttpService
        | crate::uar::UarDeploymentMode::CloudService => {
            let config = load_uar_service_chat_config().await?;
            crate::execution_profiles::guard_remote(&config.service_url).await?;
            let runtime: std::sync::Arc<dyn crate::uar::UarRuntimeFacade> = std::sync::Arc::new(
                crate::uar::UarServiceRuntime::new(&config.service_url, config.api_key)?,
            );
            let mode = if matches!(deployment_mode, crate::uar::UarDeploymentMode::CloudService) {
                "cloud_service"
            } else {
                "http_service"
            };
            Ok((
                runtime,
                UarRunConfig {
                    artifact: config.artifact,
                    session_id: config.session_id,
                },
                mode,
            ))
        }
    }
}

async fn send_uar(
    user_message: String,
    history: Vec<InferenceMessage>,
    conversation_id: String,
    run_id: String,
) -> Result<String, AgentError> {
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
            phase: RunPhase::ContextAssembly,
            status: RunStatus::Preparing,
            message: "Delegating agent definition, skills, tools, and A2UI selection to UAR".into(),
        });

        let (runtime, config, runtime_mode) = match selected_uar_runtime().await {
            Ok(runtime) => runtime,
            Err(error) => {
                publish_run_error(
                    &run_id_for_task,
                    "uar_runtime_not_configured",
                    RunPhase::ContextAssembly,
                    error.to_string(),
                    true,
                );
                return;
            }
        };
        if active_run.cancelled() {
            active_run.publish_cancelled(RunPhase::ContextAssembly);
            return;
        }

        state::publish(A2uiEvent::RunPhaseChanged {
            run_id: run_id_for_task.clone(),
            phase: RunPhase::Generation,
            status: RunStatus::Preparing,
            message: "Starting Universal Agent Runtime run".into(),
        });

        if let Err(error) = crate::execution_profiles::begin(&run_id_for_task, &conversation_id, runtime_mode).await {
            publish_run_error(&run_id_for_task, "uar_admission_journal_failed", RunPhase::Generation, error.to_string(), true);
            return;
        }
        let selected_profile = match crate::execution_profiles::current().await {
            Ok(profile) => profile,
            Err(error) => {
                let _ = crate::execution_profiles::terminal(&run_id_for_task, false).await;
                publish_run_error(&run_id_for_task, "uar_profile_failed", RunPhase::Generation, error.to_string(), true);
                return;
            }
        };
        // Key the session strictly by the per-conversation id. A global
        // `uar_session_id` setting (a service-era concept) must NOT collapse every
        // conversation into one shared session on the embedded runtime — that
        // bleeds context across conversations. Only honor it on the service path,
        // where the session id is the caller's contract with a remote UAR.
        let session_id = match runtime_mode {
            "embedded_library" => Some(conversation_id.clone()),
            _ if selected_profile.is_some() => Some(conversation_id.clone()),
            _ => config.session_id.clone().or(Some(conversation_id.clone())),
        };
        // Seed the embedded session from the host's durable history so a
        // cold-started conversation still presents the full thread to the model.
        let history = if selected_profile.as_ref().is_some_and(|profile| profile.kind != crate::execution_profiles::ProfileKind::StandaloneEmbedded) { Vec::new() } else { history };
        let seed_history = history
            .into_iter()
            .map(|message| crate::uar::UarSeedMessage {
                role: message.role.as_str().to_owned(),
                content: message.content,
                tool_call_id: message.tool_call_id,
            })
            .collect::<Vec<_>>();
        let projected = match runtime
            .start_run(crate::uar::UarRunRequest {
                artifact: config.artifact,
                input: user_message,
                session_id,
                agent_id: None,
                run_policy: None,
                history: seed_history,
            })
            .await
        {
            Ok(projected) => projected,
            Err(error) => {
                let _ = crate::execution_profiles::terminal(&run_id_for_task, false).await;
                publish_run_error(
                    &run_id_for_task,
                    "uar_run_start_failed",
                    RunPhase::Generation,
                    error.to_string(),
                    true,
                );
                return;
            }
        };

        if let Err(error) = crate::execution_profiles::admitted(&run_id_for_task, &projected.run_id).await {
            publish_run_error(&run_id_for_task, "uar_admission_journal_failed", RunPhase::Generation, error.to_string(), true);
            return;
        }
        let _observation_lease = match crate::execution_profiles::claim_observation(&run_id_for_task) {
            Ok(Some(lease)) => lease,
            _ => {
                publish_run_error(&run_id_for_task, "uar_observer_conflict", RunPhase::Generation, "Execution already has an observation owner".into(), true);
                return;
            }
        };
        state::publish(A2uiEvent::ContextTelemetry {
            run_id: run_id_for_task.clone(),
            payload: serde_json::json!({
                "runtime": "universal-agent-runtime",
                "mode": runtime_mode,
                "uarRunId": projected.run_id,
            }),
        });
        let uar_run_id = projected.run_id.clone();
        state::register_uar_run(&conversation_id, &run_id_for_task, &uar_run_id);
        let mut events = std::pin::pin!(projected.events);
        let mut completed = false;
        while let Some(event) = events.next().await {
            if active_run.cancelled() {
                let _ = runtime.cancel_run(&uar_run_id).await;
                active_run.publish_cancelled(RunPhase::Generation);
                return;
            }
            match event {
                Ok(event) => {
                    completed |= matches!(&event, A2uiEvent::RunFinished { .. });
                    if !crate::execution_profiles::observation_detached(&run_id_for_task).await {
                        state::publish(event_with_run_id(event, &run_id_for_task));
                    }
                    if completed { break; }
                },
                Err(error) => {
                    let _ = crate::execution_profiles::terminal(&run_id_for_task, false).await;
                    publish_run_error(
                        &run_id_for_task,
                        "uar_stream_failed",
                        RunPhase::Generation,
                        error.to_string(),
                        true,
                    );
                    return;
                }
            }
        }

        let _ = crate::execution_profiles::terminal(&run_id_for_task, completed).await;
        if !completed {
            publish_run_error(&run_id_for_task, "uar_stream_interrupted", RunPhase::Generation,
                "Execution stream ended without a completion event; reconnect to the existing run".into(), true);
        }
    });
    Ok(run_id)
}

pub(crate) async fn load_uar_service_chat_config() -> Result<UarServiceChatConfig, AgentError> {
    let service_url = string_setting(SETTING_UAR_SERVICE_URL)
        .await?
        .or_else(|| std::env::var("KNOWME_UAR_SERVICE_URL").ok())
        .ok_or_else(|| {
            AgentError::Config(format!(
                "missing {SETTING_UAR_SERVICE_URL} setting or KNOWME_UAR_SERVICE_URL env var"
            ))
        })?;
    let api_key = string_setting(SETTING_UAR_SERVICE_API_KEY_REF)
        .await?
        .map(|key_ref| secrets::resolve_api_key(&key_ref))
        .transpose()?
        .or_else(|| std::env::var("KNOWME_UAR_API_KEY").ok());
    let session_id = string_setting(SETTING_UAR_SESSION_ID).await?;
    let artifact = load_optional_uar_artifact().await?;
    Ok(UarServiceChatConfig {
        service_url,
        api_key,
        artifact,
        session_id,
    })
}

async fn load_optional_uar_artifact() -> Result<Option<serde_json::Value>, AgentError> {
    Ok(
        match state::config()?
            .get_setting(SETTING_UAR_AGENT_ARTIFACT)
            .await?
        {
            Some(serde_json::Value::Null) | None => load_uar_artifact_from_env().await.ok(),
            Some(value) => Some(artifact_from_setting_value(value).await?),
        },
    )
}

/// Validate and persist an A2UI surface in the UAR-owned component library.
///
/// This is a settings/artifact operation rather than a conversation turn, so
/// it deliberately avoids creating a synthetic assistant message or run.
pub async fn promote_a2ui_component(
    title: String,
    source: String,
    description: Option<String>,
) -> Result<serde_json::Value, AgentError> {
    let (runtime, _, _) = selected_uar_runtime().await?;
    runtime
        .promote_a2ui_component(&title, &source, description.as_deref())
        .await
}

pub(crate) async fn uar_service_client() -> Result<universal_agent_runtime_sdk::Client, AgentError>
{
    let config = load_uar_service_chat_config().await?;
    let client = universal_agent_runtime_sdk::Client::new(&config.service_url)
        .map_err(|error| AgentError::Client(format!("UAR client: {error}")))?;
    Ok(match config.api_key {
        Some(api_key) if !api_key.is_empty() => client.with_api_key(api_key),
        _ => client,
    })
}

async fn string_setting(key: &str) -> Result<Option<String>, AgentError> {
    Ok(state::config()?
        .get_setting(key)
        .await?
        .and_then(|value| value.as_str().map(str::to_owned))
        .filter(|value| !value.trim().is_empty()))
}

async fn artifact_from_setting_value(
    value: serde_json::Value,
) -> Result<serde_json::Value, AgentError> {
    match value {
        serde_json::Value::String(value) => artifact_from_string_or_path(&value).await,
        serde_json::Value::Object(_) => Ok(value),
        other => Err(AgentError::Config(format!(
            "{SETTING_UAR_AGENT_ARTIFACT} must be an agent artifact object, JSON string, or file path; got {other}"
        ))),
    }
}

async fn load_uar_artifact_from_env() -> Result<serde_json::Value, AgentError> {
    if let Ok(json) = std::env::var("KNOWME_UAR_AGENT_ARTIFACT_JSON") {
        return artifact_from_string_or_path(&json).await;
    }
    if let Ok(path) = std::env::var("KNOWME_UAR_AGENT_ARTIFACT_PATH") {
        return artifact_from_string_or_path(&path).await;
    }
    Err(AgentError::Config(format!(
        "missing {SETTING_UAR_AGENT_ARTIFACT} setting, KNOWME_UAR_AGENT_ARTIFACT_JSON env var, or KNOWME_UAR_AGENT_ARTIFACT_PATH env var"
    )))
}

async fn artifact_from_string_or_path(value: &str) -> Result<serde_json::Value, AgentError> {
    let trimmed = value.trim();
    let json = if trimmed.starts_with('{') {
        trimmed.to_owned()
    } else {
        tokio::fs::read_to_string(trimmed)
            .await
            .map_err(|error| AgentError::Config(format!("cannot read UAR artifact: {error}")))?
    };
    let artifact: serde_json::Value = serde_json::from_str(&json)
        .map_err(|error| AgentError::Config(format!("invalid UAR artifact JSON: {error}")))?;
    if !artifact.is_object() {
        return Err(AgentError::Config(
            "UAR artifact JSON must decode to an object".into(),
        ));
    }
    Ok(artifact)
}

fn uar_artifact_response_from_interaction(
    value: &serde_json::Value,
) -> Option<crate::uar::UarArtifactResponse> {
    let value = value.get("payload").unwrap_or(value);
    if let Some(response) = value
        .get("a2uiResponse")
        .and_then(serde_json::Value::as_object)
    {
        let artifact_id = response
            .get("artifactId")
            .or_else(|| response.get("artifact_id"))
            .and_then(serde_json::Value::as_str)?
            .to_owned();
        let response = response
            .get("response")
            .cloned()
            .unwrap_or_else(|| serde_json::Value::Object(response.clone()));
        return Some(crate::uar::UarArtifactResponse {
            artifact_id,
            response,
        });
    }

    let action = value.get("action")?;
    let artifact_id = value
        .get("artifactId")
        .or_else(|| value.get("artifact_id"))
        .and_then(serde_json::Value::as_str)
        .or_else(|| {
            action
                .get("artifactId")
                .or_else(|| action.get("artifact_id"))
                .and_then(serde_json::Value::as_str)
        })
        .or_else(|| {
            action
                .get("context")
                .and_then(|context| {
                    context
                        .get("artifactId")
                        .or_else(|| context.get("artifact_id"))
                })
                .and_then(serde_json::Value::as_str)
        })
        .or_else(|| {
            action
                .get("surfaceId")
                .or_else(|| value.get("surfaceId"))
                .and_then(serde_json::Value::as_str)
        })?
        .to_owned();

    Some(crate::uar::UarArtifactResponse {
        artifact_id,
        response: serde_json::json!({
            "action": action,
            "a2uiClientDataModel": value.get("a2uiClientDataModel").cloned(),
        }),
    })
}

fn uar_a2ui_action_from_interaction(
    value: &serde_json::Value,
) -> Option<crate::uar::UarA2uiAction> {
    let value = value.get("payload").unwrap_or(value);
    let action = value.get("action")?.as_object()?;
    Some(crate::uar::UarA2uiAction {
        surface_id: action.get("surfaceId")?.as_str()?.to_owned(),
        name: action.get("name")?.as_str()?.to_owned(),
        source_component_id: action.get("sourceComponentId")?.as_str()?.to_owned(),
        timestamp: action
            .get("timestamp")
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned),
        context: action
            .get("context")
            .cloned()
            .unwrap_or_else(|| serde_json::json!({})),
        client_data_model: value.get("a2uiClientDataModel").cloned(),
    })
}

fn uar_source_run_id_from_interaction(value: &serde_json::Value) -> Option<String> {
    value
        .get("sourceRunId")
        .or_else(|| value.get("source_run_id"))
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(str::to_owned)
}

fn uar_a2ui_promotion_from_interaction(
    value: &serde_json::Value,
) -> Option<(String, String, Option<String>)> {
    if value.get("kind")?.as_str()? != "a2ui_promote" {
        return None;
    }
    let payload = value.get("payload")?.as_object()?;
    Some((
        payload.get("title")?.as_str()?.to_owned(),
        payload.get("source")?.as_str()?.to_owned(),
        payload
            .get("description")
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned),
    ))
}

pub(crate) fn event_with_run_id(event: A2uiEvent, run_id: &str) -> A2uiEvent {
    match event {
        A2uiEvent::RunStarted { .. } => A2uiEvent::RunStarted {
            run_id: run_id.to_owned(),
        },
        A2uiEvent::RunPhaseChanged {
            phase,
            status,
            message,
            ..
        } => A2uiEvent::RunPhaseChanged {
            run_id: run_id.to_owned(),
            phase,
            status,
            message,
        },
        A2uiEvent::Block { block, .. } => A2uiEvent::Block {
            run_id: run_id.to_owned(),
            block,
        },
        A2uiEvent::RunFinished { .. } => A2uiEvent::RunFinished {
            run_id: run_id.to_owned(),
        },
        A2uiEvent::RunCancelled { phase, message, .. } => A2uiEvent::RunCancelled {
            run_id: run_id.to_owned(),
            phase,
            message,
        },
        A2uiEvent::RunError { error, .. } => A2uiEvent::RunError {
            run_id: run_id.to_owned(),
            error,
        },
        A2uiEvent::ContextTelemetry { payload, .. } => A2uiEvent::ContextTelemetry {
            run_id: run_id.to_owned(),
            payload,
        },
    }
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
    let history_summary = history
        .iter()
        .map(|message| {
            format!(
                "{}:{}",
                message.role.as_str(),
                message.content.chars().count()
            )
        })
        .collect::<Vec<_>>()
        .join(",");
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
        tracing::info!(
            run_id = %run_id_for_task,
            history_count = history.len(),
            history_summary = %history_summary,
            "local chat history received for context assembly"
        );

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
                let skill_name = skill_name_for_local_tool(&call.name);
                publish_skill_activation(&mut adapter, &skill_name, "running");
                let (follow_up, continuation) = tool_gate
                    .execute_or_reject(&run_id_for_task, &mcp_registry, call)
                    .await;
                let skill_status = skill_status_for_tool_follow_up(&follow_up);
                for a2ui in adapter.ingest(&follow_up) {
                    state::publish(a2ui);
                }
                publish_skill_activation(&mut adapter, &skill_name, skill_status);
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
                    let skill_name = skill_name_for_local_tool(&call.name);
                    publish_skill_activation(&mut adapter, &skill_name, "running");
                    let (follow_up, continuation) = tool_gate
                        .execute_or_reject(&run_id_for_task, &mcp_registry, call)
                        .await;
                    let skill_status = skill_status_for_tool_follow_up(&follow_up);
                    for a2ui in adapter.ingest(&follow_up) {
                        state::publish(a2ui);
                    }
                    publish_skill_activation(&mut adapter, &skill_name, skill_status);
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
    } else if model_id == "qwen3.5-4b-mlx-4bit" {
        presets::ios_qwen35_4b_mlx()
    } else if model_id == "qwen3.5-9b-mlx-4bit" {
        presets::ios_qwen35_9b_mlx()
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
        // A settings-store failure is surfaced as a context source failure
        // rather than silently falling back to the default: if the user edited
        // the prompt and we cannot read it, running with different framing than
        // they configured is worse than reporting the failure.
        system_prompt: local_system_prompt()
            .await
            .map_err(|error| ContextError::Source(format!("local system prompt: {error}")))?,
        user_text: user_message.into(),
        thinking_requested: cloud
            || model_id == "qwen3.5-4b-mlx-4bit"
            || model_id == "qwen3.5-9b-mlx-4bit",
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

/// Build the shortlisted tool declarations sent to a local model for one turn.
///
/// Local models run in a tight context budget — the Android LiteRT-LM lane in
/// particular — so declaring every cached tool from every registered server
/// crowds out the conversation and degrades tool selection. The shortlist is
/// therefore **bounded** at [`MAX_LOCAL_TOOL_DECLARATIONS`] and ordered
/// **trusted-first**: tools from trusted-local servers are declared before
/// untrusted ones, so the safest and most likely-useful tools survive the cap.
///
/// Ordering is a stable sort over the registry's deterministic server/tool
/// order, so the same registry always yields the same shortlist. Tools beyond
/// the cap are simply not declared this turn; if the model nonetheless asks for
/// one, [`LocalToolCallGate`] already rejects it as outside the shortlisted set.
fn local_tool_definitions(registry: &gen_ui_mcp::McpRegistry) -> Vec<InferenceTool> {
    let mut candidates: Vec<(bool, InferenceTool)> = Vec::new();
    for server_name in registry.server_names() {
        let Some(server) = registry.get(&server_name) else {
            continue;
        };
        let trusted = server.trust() == gen_ui_mcp::McpServerTrust::TrustedLocal;
        for tool in server.cached_tools() {
            candidates.push((trusted, inference_tool_from_mcp(&server_name, tool)));
        }
    }
    // Stable sort keeps registry order within each trust tier.
    candidates.sort_by_key(|(trusted, _)| !*trusted);
    candidates.truncate(MAX_LOCAL_TOOL_DECLARATIONS);
    candidates
        .into_iter()
        .map(|(_, definition)| definition)
        .collect()
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

fn skill_name_for_local_tool(tool_name: &str) -> String {
    tool_name
        .split_once("__")
        .map(|(namespace, _)| namespace)
        .filter(|namespace| !namespace.is_empty())
        .unwrap_or("local_tool")
        .to_owned()
}

fn skill_status_for_tool_follow_up(event: &StreamEvent) -> &'static str {
    match event {
        StreamEvent::ToolResult { is_error, .. } if *is_error => "failed",
        StreamEvent::Error { .. } => "failed",
        _ => "complete",
    }
}

fn publish_skill_activation(adapter: &mut A2uiAdapter, name: &str, status: &str) {
    for event in adapter.ingest(&StreamEvent::SkillActivation {
        name: name.to_owned(),
        status: status.to_owned(),
    }) {
        state::publish(event);
    }
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

    #[test]
    fn chat_modes_map_onto_execution_lanes() {
        // The picker's vocabulary is user-facing; lanes are execution paths.
        assert_eq!(lane_for_chat_mode("local"), Some(LANE_LOCAL));
        // `agent` and `uar` both run through the embedded UAR — they differ only
        // in which agent artifact the run resolves — so both are LANE_UAR.
        assert_eq!(lane_for_chat_mode("uar"), Some(LANE_UAR));
        assert_eq!(lane_for_chat_mode("agent"), Some(LANE_UAR));
        assert_eq!(lane_for_chat_mode("cloud"), Some(LANE_CLOUD));
        // An unrecognized mode must NOT invent a lane; the caller falls back to
        // the global setting rather than silently routing somewhere unintended.
        assert_eq!(lane_for_chat_mode("nonsense"), None);
        assert_eq!(lane_for_chat_mode(""), None);
    }

    #[test]
    fn conversation_chat_mode_is_read_from_the_bare_run_policy() {
        // The control plane hands back the bare `RunPolicy`, not the storage
        // `ConversationPolicyRecord` wrapper. This was the defect behind "Local
        // does nothing": the accessor leaked `{conversation_id, policy, updated_at}`
        // while every consumer — Dart's `state.policy`, the TypeScript binding's
        // declared `Promise<RunPolicy | null>`, and `save_conversation_policy`'s
        // own input — expects `chat_mode` at the top level. Because every
        // `RunPolicy` field is `#[serde(default)]`, sending the wrapper back to
        // save deserialized to defaults *without erroring*, silently discarding
        // the selection.
        let bare = serde_json::json!({"version": 1, "chat_mode": "local"});
        assert_eq!(chat_mode_lane(&bare), Some(LANE_LOCAL));

        // The wrapper must NOT resolve — if it ever reappears, this fails loudly
        // instead of silently falling back to the global lane.
        let wrapped = serde_json::json!({
            "conversation_id": "c1",
            "policy": {"version": 1, "chat_mode": "local"},
            "updated_at": "2026-07-26T00:00:00Z",
        });
        assert_eq!(chat_mode_lane(&wrapped), None);

        // A conversation with no stored mode falls through to the global lane.
        assert_eq!(chat_mode_lane(&serde_json::json!({"version": 1})), None);
        assert_eq!(chat_mode_lane(&serde_json::Value::Null), None);
    }

    #[test]
    fn every_send_path_resolves_the_lane_per_conversation() {
        // Routing must not depend on which entry point the UI happened to call:
        // `send` was fixed first while `send_with_attachments`, `send_content`
        // and `send_interaction` still read the global setting, so sending an
        // attachment from a Local conversation silently ran on the global lane.
        //
        // Resolving the lane needs a live config and control plane, so this
        // guards the source instead. Only two places may *read* the global
        // setting: the `active_lane()` getter and the resolver's own fallback.
        // Count `get_setting(SETTING_ACTIVE_LANE)` call sites specifically —
        // the bare constant also appears in its own declaration and in
        // `set_active_lane`'s write. The test module is excluded so this
        // assertion does not count its own source.
        let source = include_str!("chat.rs");
        let production_source = source
            .split_once("mod attachment_tests {")
            .map(|(before, _)| before)
            .expect("test module marks the end of production source");
        let global_reads = production_source
            .matches(".get_setting(SETTING_ACTIVE_LANE)")
            .count();
        assert_eq!(
            global_reads, 2,
            "the global active_lane setting may only be read by active_lane() \
             and resolve_lane_for_conversation()'s fallback; a send path \
             reading it directly bypasses per-conversation scope"
        );
    }

    #[test]
    fn a2ui_interaction_keeps_the_completed_source_uar_run() {
        let interaction = serde_json::json!({
            "kind": "a2ui",
            "sourceRunId": "uar-source-run-42",
            "payload": {
                "action": {
                    "surfaceId": "surface-1",
                    "name": "continue",
                    "sourceComponentId": "continue-button",
                    "context": {}
                }
            }
        });

        assert_eq!(
            uar_source_run_id_from_interaction(&interaction).as_deref(),
            Some("uar-source-run-42")
        );
        assert_eq!(
            uar_a2ui_action_from_interaction(&interaction)
                .expect("typed action")
                .name,
            "continue"
        );
    }

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
        // (mime, is_image, is_document, is_video)
        assert_eq!(attachment_kind("png"), ("image/png", true, false, false));
        assert_eq!(
            attachment_kind("pdf"),
            ("application/pdf", false, true, false)
        );
        // Video previously reported false for every flag, so a renderer had to
        // re-derive "is this video?" by string-matching the mime. It now says so.
        assert_eq!(attachment_kind("mp4"), ("video/mp4", false, false, true));
        assert_eq!(attachment_kind("webm"), ("video/webm", false, false, true));
        assert_eq!(
            attachment_kind("mov"),
            ("video/quicktime", false, false, true)
        );
        // An unknown extension must claim nothing rather than guess.
        assert_eq!(
            attachment_kind("xyzzy"),
            ("application/octet-stream", false, false, false)
        );
    }

    #[test]
    fn local_tool_namespace_maps_to_skill_activation_name() {
        assert_eq!(
            skill_name_for_local_tool("knowme_diagnostics__local_runtime_status"),
            "knowme_diagnostics"
        );
        assert_eq!(skill_name_for_local_tool("memory__search"), "memory");
        assert_eq!(skill_name_for_local_tool("unqualified"), "local_tool");
        assert_eq!(
            skill_status_for_tool_follow_up(&StreamEvent::ToolResult {
                tool_use_id: "call-1".into(),
                output_json: "{}".into(),
                is_error: false,
            }),
            "complete"
        );
        assert_eq!(
            skill_status_for_tool_follow_up(&StreamEvent::ToolResult {
                tool_use_id: "call-1".into(),
                output_json: "{}".into(),
                is_error: true,
            }),
            "failed"
        );
    }

    #[cfg(not(any(target_os = "ios", target_os = "android")))]
    #[tokio::test]
    async fn uar_artifact_config_accepts_object_or_json_string() {
        let object = serde_json::json!({
            "kind": "agent",
            "id": "knowme-test-agent",
            "version": "1.0.0"
        });
        assert_eq!(
            artifact_from_setting_value(object.clone())
                .await
                .expect("object artifact accepted"),
            object
        );

        assert_eq!(
            artifact_from_setting_value(serde_json::Value::String(object.to_string()))
                .await
                .expect("JSON string artifact accepted"),
            object
        );

        let error = artifact_from_setting_value(serde_json::json!(["not", "an", "artifact"]))
            .await
            .expect_err("non-object artifacts fail closed");
        assert!(error.to_string().contains(SETTING_UAR_AGENT_ARTIFACT));
    }

    #[test]
    fn uar_projected_events_are_remapped_to_knowme_run_id() {
        let event = event_with_run_id(
            A2uiEvent::Block {
                run_id: "uar-run".into(),
                block: gen_ui_types::content_block::ContentBlock::Skill {
                    name: "calendar".into(),
                    status: "running".into(),
                },
            },
            "knowme-run",
        );

        assert!(matches!(
            event,
            A2uiEvent::Block {
                run_id,
                block: gen_ui_types::content_block::ContentBlock::Skill { name, status },
            } if run_id == "knowme-run" && name == "calendar" && status == "running"
        ));
    }

    #[cfg(not(any(target_os = "ios", target_os = "android")))]
    #[test]
    fn configured_uar_service_reports_service_runtime_status() {
        use crate::uar::UarRuntimeFacade;
        let runtime = crate::uar::UarServiceRuntime::new("http://127.0.0.1:9999", None)
            .expect("valid service URL");
        let status = runtime.status();
        assert_eq!(
            status.target_mode,
            crate::uar::UarDeploymentMode::HttpService
        );
        assert_eq!(
            status.current_state,
            crate::uar::UarIntegrationState::ServiceRuntime
        );
    }

    #[cfg(not(any(target_os = "ios", target_os = "android")))]
    #[test]
    fn uar_a2ui_response_accepts_explicit_envelope() {
        let response = uar_artifact_response_from_interaction(&serde_json::json!({
            "a2uiResponse": {
                "artifactId": "artifact-1",
                "response": {"accepted": true}
            }
        }))
        .expect("explicit UAR A2UI response envelope parses");

        assert_eq!(response.artifact_id, "artifact-1");
        assert_eq!(response.response["accepted"], true);
    }

    #[cfg(not(any(target_os = "ios", target_os = "android")))]
    #[test]
    fn uar_a2ui_response_accepts_flutter_action_envelope() {
        let response = uar_artifact_response_from_interaction(&serde_json::json!({
            "version": "v0.9",
            "action": {
                "surfaceId": "artifact-2",
                "name": "submit",
                "sourceComponentId": "button-1",
                "context": {"form": "settings"}
            },
            "a2uiClientDataModel": {
                "version": "v0.9",
                "surfaces": {"artifact-2": {"choice": "yes"}}
            }
        }))
        .expect("Flutter A2UI action envelope parses");

        assert_eq!(response.artifact_id, "artifact-2");
        assert_eq!(response.response["action"]["name"], "submit");
        assert_eq!(
            response.response["a2uiClientDataModel"]["surfaces"]["artifact-2"]["choice"],
            "yes"
        );
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
    async fn local_tool_shortlist_is_bounded_and_prefers_trusted_servers() {
        // A transport that advertises `count` tools, so a server can exceed the
        // per-turn declaration cap on its own.
        #[derive(Debug)]
        struct BulkTransport {
            prefix: &'static str,
            count: usize,
        }

        #[async_trait::async_trait]
        impl gen_ui_mcp::McpTransport for BulkTransport {
            async fn request(
                &self,
                method: &str,
                _params: Option<serde_json::Value>,
            ) -> gen_ui_types::CoreResult<serde_json::Value> {
                match method {
                    "tools/list" => Ok(serde_json::json!({
                        "tools": (0..self.count)
                            .map(|index| serde_json::json!({
                                "name": format!("{}_{index}", self.prefix),
                                "description": "bulk tool",
                                "inputSchema": {"type": "object"}
                            }))
                            .collect::<Vec<_>>()
                    })),
                    other => Err(gen_ui_types::CoreError::Terminal(format!(
                        "unexpected MCP method {other}"
                    ))),
                }
            }
        }

        let registry = gen_ui_mcp::McpRegistry::new();
        // Register the UNTRUSTED server first so plain registry order would put
        // its tools ahead of the trusted server's — the shortlist must reorder.
        let untrusted = registry.register(gen_ui_mcp::McpServerHandle::new(
            "untrusted",
            Box::new(BulkTransport {
                prefix: "untrusted",
                count: MAX_LOCAL_TOOL_DECLARATIONS,
            }),
        ));
        untrusted.refresh_tools().await.expect("untrusted tools");
        let trusted = registry.register(gen_ui_mcp::McpServerHandle::new_trusted_local(
            "trusted",
            Box::new(BulkTransport {
                prefix: "trusted",
                count: 4,
            }),
        ));
        trusted.refresh_tools().await.expect("trusted tools");

        let shortlist = local_tool_definitions(&registry);

        // Bounded: never more declarations than the per-turn cap, even though
        // the registry advertises more.
        assert_eq!(shortlist.len(), MAX_LOCAL_TOOL_DECLARATIONS);

        // Trusted-first: every tool from the trusted server survives the cap and
        // is declared before any untrusted tool.
        let trusted_count = shortlist
            .iter()
            .filter(|tool| tool.name.starts_with("trusted__"))
            .count();
        assert_eq!(trusted_count, 4, "all trusted tools survive the cap");
        let first_untrusted = shortlist
            .iter()
            .position(|tool| tool.name.starts_with("untrusted__"))
            .expect("untrusted tools fill the remainder");
        assert_eq!(
            first_untrusted, 4,
            "trusted declarations precede untrusted ones"
        );
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
