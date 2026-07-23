//! The crate's canonical paths to the `knowme:plugin@1.0.0` WIT data types.
//!
//! The types themselves are bindgen-generated (see `sandbox/bindings.rs`):
//! the `hook` world owns the shared host interfaces, `contributor` owns
//! `contribution-types`, and `app-shell` owns `host-a2ui`; interface
//! remapping guarantees these are the only definitions in the crate. This
//! facade exists so consumers (`PluginHost`, `HostDispatch`, later changes)
//! depend on one stable module path instead of generated module layouts.

pub use crate::sandbox::bindings::app_shell::knowme::plugin::host_a2ui::{
    A2uiAction, A2uiMessage, Component, ComponentKind, CreateSurface, UpdateComponents,
    UpdateDataModel,
};
pub use crate::sandbox::bindings::contributor::knowme::plugin::contribution_types::{
    CitationItem, ContextSnippet, Contribution, ContributionInput, ContributionItem, CustomItem,
    MemoryOpItem, PipelineStage, PromptSection, ToolCallRequest, ToolResultItem,
};
pub use crate::sandbox::bindings::hook::knowme::plugin::host_events::{
    AgUiEvent, CustomEvent, RunCancelled, RunError, RunStarted, ToolCallArgs, ToolCallResult,
    ToolCallStart,
};
pub use crate::sandbox::bindings::hook::knowme::plugin::host_infer::{
    InferChunk, InferRequest, InferResponse, Modality,
};
pub use crate::sandbox::bindings::hook::knowme::plugin::host_log::LogLevel;
pub use crate::sandbox::bindings::hook::knowme::plugin::host_memory::{
    MemoryHit, Provenance, ProvenanceKind, RecallQuery,
};
pub use crate::sandbox::bindings::hook::knowme::plugin::host_net::{HttpRequest, HttpResponse};
