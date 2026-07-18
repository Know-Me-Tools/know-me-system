// TJ-ARCH-MOB-001 compliant
//! gen_ui_audio (L2, LEAF) — the Scribe feature's shared Rust implementation:
//! record → transcribe on-device with whisper-rs (whisper-tiny) → hand back
//! plain text for the caller to fold into `memory_ingest`. No cloud call.
//!
//! Native-only by design (see Cargo.toml): whisper-rs binds whisper.cpp (C++
//! via cc/cmake) and cpal binds platform audio APIs — neither targets wasm32.
//! Scribe therefore ships on mobile (Flutter/iOS+Android via gen_ui_ffi) and
//! desktop (Tauri via tauri-plugin-gen-ui) from this one crate, matching the
//! "no duplicated business logic between mobile/desktop" rule. Web is a
//! documented gap — see the crate-level FOLLOW-UP note below.
//!
//! FOLLOW-UP (flagged, not silently decided): a web Scribe would need a
//! completely different stack (MediaRecorder API + a wasm/JS STT model, e.g.
//! transformers.js' whisper-tiny ONNX build or WebGPU whisper), mirroring the
//! C-105 precedent where WebLLM is a separate TS adapter alongside the native
//! mistral.rs lane rather than a wasm build of the same crate. Out of scope
//! for this change; the intent-level seam (`Scribe::record_and_transcribe`)
//! is the shape a future web adapter should also expose to the UI layer.
#![forbid(unsafe_code)]

mod error;
mod model;
mod recorder;
mod transcribe;

pub use error::{ScribeError, ScribeResult};
pub use model::ModelHandle;
pub use recorder::{Recorder, WHISPER_SAMPLE_RATE};

/// The Scribe intent facade: the one type FFI/Tauri callers construct. Owns no
/// long-lived OS resources itself (the mic stream lives only for the duration
/// of a recording) so it is cheap to construct per call.
#[derive(Default)]
pub struct Scribe;

impl Scribe {
    pub fn new() -> Self {
        Self
    }

    /// Ensure the whisper-tiny model is present (downloading on first run —
    /// see `model.rs` doc comment for the bundle-vs-download tradeoff), then
    /// report the resolved path. Callers that want to show "preparing Scribe…"
    /// before the mic UI appears can await this ahead of `start_recording`.
    pub async fn ensure_ready(&self) -> ScribeResult<ModelHandle> {
        model::ensure_model().await
    }

    /// Start a microphone recording. Returns a `Recorder` handle; call
    /// `.stop()` on it (from a `spawn_blocking` context) to finalise capture
    /// and get back resampled PCM.
    pub fn start_recording(&self) -> ScribeResult<Recorder> {
        Recorder::start()
    }

    /// Stop `recorder` and transcribe the captured audio on-device. This is
    /// the one call sites should use end-to-end: it does the CPU-bound
    /// resample + whisper-rs inference off the async runtime via
    /// `gen_ui_runtime::spawn_blocking`, matching the discipline
    /// `gen_ui_db_graph::embed_blocking` uses for fastembed.
    pub async fn stop_and_transcribe(&self, recorder: Recorder) -> ScribeResult<String> {
        let model = self.ensure_ready().await?;
        let pcm = recorder.stop()?;
        gen_ui_runtime::spawn_blocking(move || transcribe::transcribe_pcm(&model.path, &pcm))
            .await
            .map_err(|e| ScribeError::Whisper(format!("transcribe task join: {e}")))?
    }
}
