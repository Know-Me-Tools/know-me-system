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

/// A bounded image supplied to a local multimodal model. The bytes stay in the
/// Rust inference boundary; Flutter only forwards the selected file path.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImageInput {
    pub file_name: String,
    pub mime_type: String,
    pub bytes: Vec<u8>,
}

/// Backward-compatible local inference request. Text-only callers continue to
/// use `InferenceProvider::generate`; multimodal callers use this request so
/// the mtmd path can attach validated image tensors without changing the old
/// prompt contract.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InferenceRequest {
    pub prompt: String,
    #[serde(default)]
    pub images: Vec<ImageInput>,
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

    /// Generate from a typed request. Engines that do not support images fail
    /// explicitly; text-only requests retain the original implementation path.
    async fn generate_request(
        &self,
        request: &InferenceRequest,
        params: &SampleParams,
    ) -> CoreResult<BoxStream<'static, StreamEvent>> {
        if !request.images.is_empty() {
            return Err(crate::error::CoreError::Terminal(
                "this inference backend does not support image inputs".into(),
            ));
        }
        self.generate(&request.prompt, params).await
    }

    /// Release model memory. Safe to call when nothing is loaded.
    async fn unload(&self) -> CoreResult<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_request_round_trips_without_images() {
        let request = InferenceRequest {
            prompt: "Describe the image".into(),
            images: Vec::new(),
        };
        let encoded = serde_json::to_string(&request).expect("request serializes");
        let decoded: InferenceRequest = serde_json::from_str(&encoded).expect("request parses");
        assert_eq!(decoded, request);
    }

    #[test]
    fn image_input_preserves_bytes_and_metadata() {
        let request = InferenceRequest {
            prompt: "What is shown?".into(),
            images: vec![ImageInput {
                file_name: "sample.png".into(),
                mime_type: "image/png".into(),
                bytes: vec![137, 80, 78, 71],
            }],
        };
        let decoded: InferenceRequest =
            serde_json::from_value(serde_json::to_value(&request).expect("request serializes"))
                .expect("request parses");
        assert_eq!(decoded.images[0].mime_type, "image/png");
        assert_eq!(decoded.images[0].bytes, vec![137, 80, 78, 71]);
    }
}
