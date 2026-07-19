// TJ-ARCH-MOB-001 compliant
//! Universal Agent Runtime boundary for KnowMe.
//!
//! UAR owns real agent execution, skills, A2UI registry management, tool
//! governance, and general provider routing. KnowMe owns the product shell,
//! local-device provider adapters, and the local conversation projection. This
//! module captures that boundary without linking the full UAR runtime yet.

use gen_ui_protocol::{normalize_uar_agui_event, A2uiAdapter};
use gen_ui_types::events::A2uiEvent;
use serde::{Deserialize, Serialize};

/// How this platform should access UAR.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UarDeploymentMode {
    /// Flutter mobile target: linked library/kernel behind Rust FFI.
    EmbeddedLibrary,
    /// Desktop/web target: HTTP/SSE service, possibly supervised as a sidecar.
    HttpService,
    /// Remote authoritative runtime when local capabilities are unavailable.
    CloudService,
}

/// What is actually wired in this build today.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UarIntegrationState {
    /// UAR is pinned and planned, but chat execution is not routed through it.
    NotWired,
    /// UAR event payloads can be projected into KnowMe content blocks.
    ProjectionOnly,
    /// Future state: in-process UAR kernel owns mobile agent execution.
    EmbeddedKernel,
    /// Future state: HTTP/service UAR owns web/desktop agent execution.
    ServiceRuntime,
}

/// Runtime-mode facts the UI/certification tools can surface without implying
/// UAR execution is certified before it is.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UarRuntimeStatus {
    pub authority: String,
    pub target_mode: UarDeploymentMode,
    pub current_state: UarIntegrationState,
    pub local_provider_role: String,
    pub mobile_sidecar_allowed: bool,
    pub submodule_path: String,
    pub submodule_commit: Option<String>,
}

impl UarRuntimeStatus {
    pub fn current() -> Self {
        Self {
            authority: "universal-agent-runtime".into(),
            target_mode: default_target_mode(),
            current_state: UarIntegrationState::ProjectionOnly,
            local_provider_role: "knowme_local_provider_backend".into(),
            mobile_sidecar_allowed: false,
            submodule_path: "rust/vendor/universal-agent-runtime".into(),
            submodule_commit: option_env!("KNOWME_UAR_SUBMODULE_COMMIT").map(str::to_owned),
        }
    }
}

#[cfg(any(target_os = "ios", target_os = "android"))]
const fn default_target_mode() -> UarDeploymentMode {
    UarDeploymentMode::EmbeddedLibrary
}

#[cfg(not(any(target_os = "ios", target_os = "android")))]
const fn default_target_mode() -> UarDeploymentMode {
    UarDeploymentMode::HttpService
}

/// Stateful projector from UAR AG-UI payloads into KnowMe A2UI events.
///
/// Keep one projector per run so streamed tool-call argument chunks share the
/// same `A2uiAdapter` buffer.
pub struct UarEventProjector {
    adapter: A2uiAdapter,
}

impl UarEventProjector {
    pub fn new(run_id: impl Into<String>) -> Self {
        Self {
            adapter: A2uiAdapter::new(run_id),
        }
    }

    pub fn ingest_agui_payload(&mut self, payload: &serde_json::Value) -> Vec<A2uiEvent> {
        let event = normalize_uar_agui_event(payload);
        self.adapter.ingest(&event)
    }
}

/// Current user-visible UAR status. This intentionally reports projection-only
/// until chat execution actually flows through a UAR facade.
pub fn runtime_status() -> UarRuntimeStatus {
    UarRuntimeStatus::current()
}

#[cfg(test)]
mod tests {
    use super::*;
    use gen_ui_types::content_block::ContentBlock;

    #[test]
    fn mobile_sidecar_is_not_allowed_in_status_contract() {
        let status = runtime_status();
        assert_eq!(status.authority, "universal-agent-runtime");
        assert_eq!(status.current_state, UarIntegrationState::ProjectionOnly);
        assert!(!status.mobile_sidecar_allowed);
    }

    #[test]
    fn projector_maps_uar_skill_activation_to_visible_block() {
        let mut projector = UarEventProjector::new("run-1");
        let output = projector.ingest_agui_payload(&serde_json::json!({
            "type": "CUSTOM",
            "profile": "uar.agui/1",
            "name": "uar.skill.activated",
            "value": {
                "skill": {"id": "meeting_summary"},
                "selectionMethod": "runtime"
            },
            "runId": "run-1"
        }));

        assert!(matches!(
            &output[0],
            A2uiEvent::Block {
                block: ContentBlock::Skill { name, status },
                ..
            } if name == "meeting_summary" && status == "activated"
        ));
    }

    #[test]
    fn projector_keeps_tool_stream_state_across_uar_events() {
        let mut projector = UarEventProjector::new("run-1");
        assert!(projector
            .ingest_agui_payload(&serde_json::json!({
                "type": "TOOL_CALL_START",
                "toolCallId": "tool-1",
                "name": "docs.search"
            }))
            .is_empty());
        assert!(projector
            .ingest_agui_payload(&serde_json::json!({
                "type": "TOOL_CALL_ARGS",
                "toolCallId": "tool-1",
                "delta": "{\"q\":\"rag\"}"
            }))
            .is_empty());
        let output = projector.ingest_agui_payload(&serde_json::json!({
            "type": "TOOL_CALL_END",
            "toolCallId": "tool-1"
        }));

        assert!(matches!(
            &output[0],
            A2uiEvent::Block {
                block: ContentBlock::ToolUse { id, name, input_json },
                ..
            } if id == "tool-1" && name == "docs.search" && input_json == "{\"q\":\"rag\"}"
        ));
    }
}
