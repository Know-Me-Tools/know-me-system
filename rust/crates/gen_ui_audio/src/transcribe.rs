// TJ-ARCH-MOB-001 compliant
//! whisper-rs transcription engine. Loading the ggml model + running inference
//! are both CPU-bound and synchronous by design in whisper.cpp — every call
//! into this module MUST go through `gen_ui_runtime::spawn_blocking` (the same
//! discipline `gen_ui_db_graph::embed_blocking` uses for fastembed), never
//! called directly on the async runtime.
use crate::error::{ScribeError, ScribeResult};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

/// Transcribe mono 16kHz f32 PCM audio to text using the whisper-tiny model at
/// `model_path`. Synchronous + CPU-bound — call via `spawn_blocking`.
pub fn transcribe_pcm(model_path: &std::path::Path, pcm: &[f32]) -> ScribeResult<String> {
    if pcm.is_empty() {
        return Err(ScribeError::EmptyRecording);
    }

    let ctx = WhisperContext::new_with_params(
        &*model_path.to_string_lossy(),
        WhisperContextParameters::default(),
    )
    .map_err(|e| ScribeError::Whisper(format!("failed to load model: {e}")))?;

    let mut state = ctx
        .create_state()
        .map_err(|e| ScribeError::Whisper(format!("failed to create state: {e}")))?;

    // Greedy decoding: whisper-tiny is small enough that beam search buys little
    // accuracy for a real latency cost on-device. Revisit if transcript quality
    // becomes the bottleneck rather than a good-enough on-device draft.
    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    params.set_print_special(false);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);
    // Scribe is a dictation tool, not a subtitler — force auto language
    // detection off only if a fixed locale is ever configured; for the PoC we
    // let whisper.cpp auto-detect.
    params.set_language(Some("auto"));

    state
        .full(params, pcm)
        .map_err(|e| ScribeError::Whisper(format!("inference failed: {e}")))?;

    let num_segments = state.full_n_segments();

    let mut text = String::new();
    for i in 0..num_segments {
        let segment = state
            .get_segment(i)
            .ok_or_else(|| ScribeError::Whisper(format!("segment {i} out of range")))?;
        let segment_text = segment
            .to_str()
            .map_err(|e| ScribeError::Whisper(format!("failed to read segment {i}: {e}")))?;
        text.push_str(segment_text);
    }

    Ok(text.trim().to_string())
}
