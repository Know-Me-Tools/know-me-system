// TJ-ARCH-MOB-001 compliant
//! gen_ui_inference (L2) — local-inference engines behind the
//! `gen_ui_types::inference::InferenceProvider` trait. Per-lane engines
//! (see versions.toml [inference]): pinned llama.cpp (`llama-cpp-2`) on desktop
//! and mobile, WebLLM on web, and mistral.rs as an optional experiment.
//!
//! Callers (gen_ui_agent, the platform leaves) depend on the trait only — never
//! on an engine crate — so adding or swapping a lane never ripples past here.

#[cfg(all(
    feature = "local-mistral",
    not(any(target_os = "ios", target_os = "android", target_arch = "wasm32"))
))]
pub mod mistral;

#[cfg(all(
    feature = "local-mistral",
    not(any(target_os = "ios", target_os = "android", target_arch = "wasm32"))
))]
pub use mistral::MistralEngine;

#[cfg(feature = "local-llama")]
pub mod llama;

#[cfg(feature = "local-llama")]
pub use llama::LlamaCppEngine;
