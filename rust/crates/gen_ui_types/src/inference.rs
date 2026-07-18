// TJ-ARCH-MOB-001 compliant
//! InferenceProvider — the local-inference engine seam. One trait, per-lane
//! implementations selected at build time in gen_ui_inference (see
//! `versions.toml` `[inference]`): pinned llama.cpp (`llama-cpp-2`) on desktop
//! and mobile, WebLLM on web, and optional mistral.rs. UI layers
//! and gen_ui_agent depend on this trait only — never on an engine crate —
//! so swapping or adding an engine never ripples past gen_ui_inference.
//!
//! The default is deliberately shared across native surfaces: the reference app
//! must have one reproducible, checksummed local model path that works first run.
use crate::error::CoreResult;
use crate::events::StreamEvent;
use async_trait::async_trait;
use futures::stream::BoxStream;
use serde::{Deserialize, Serialize};

/// Identifies a locally loadable model (catalog entry or user-supplied path).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LocalModelSpec {
    /// Catalog id (e.g. "qwen2.5-3b-instruct-q4") or an absolute file path.
    pub model: String,
    /// Engine-specific context length; None = engine default.
    pub context_len: Option<u32>,
}

/// Sampling parameters shared by every engine lane.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SampleParams {
    pub temperature: f32,
    pub top_p: f32,
    pub max_tokens: u32,
}

#[async_trait]
pub trait InferenceProvider: Send + Sync {
    /// Load (or mmap/attach) the model. Idempotent per spec; heavy work must
    /// run off the async runtime inside the implementation (spawn_blocking).
    async fn load(&self, spec: &LocalModelSpec) -> CoreResult<()>;

    /// Stream a completion for the given chat-formatted prompt. Events use the
    /// same `StreamEvent` shape as the remote-gateway path, so gen_ui_protocol
    /// ingests local and remote tokens identically.
    async fn generate(
        &self,
        prompt: &str,
        params: &SampleParams,
    ) -> CoreResult<BoxStream<'static, StreamEvent>>;

    /// Release model memory. Safe to call when nothing is loaded.
    async fn unload(&self) -> CoreResult<()>;
}
