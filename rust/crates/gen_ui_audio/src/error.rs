// TJ-ARCH-MOB-001 compliant
//! Scribe error taxonomy. Maps into `gen_ui_types::CoreError` at the FFI/Tauri
//! boundary so callers see one error shape (Terminal vs Transient) regardless
//! of platform.
use thiserror::Error;

pub type ScribeResult<T> = Result<T, ScribeError>;

#[derive(Debug, Error)]
pub enum ScribeError {
    #[error("microphone: {0}")]
    Microphone(String),
    #[error("model not found at {path}: {reason}")]
    ModelMissing { path: String, reason: String },
    #[error("whisper: {0}")]
    Whisper(String),
    #[error("no audio captured")]
    EmptyRecording,
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

impl From<ScribeError> for gen_ui_types::CoreError {
    fn from(err: ScribeError) -> Self {
        match err {
            // A missing on-device model or an unavailable mic are conditions the
            // caller can react to (prompt a download / grant permission) —
            // Transient signals "retry after the user acts", not "retry as-is".
            ScribeError::ModelMissing { .. } | ScribeError::Microphone(_) => {
                gen_ui_types::CoreError::Transient(err.to_string())
            }
            ScribeError::EmptyRecording => gen_ui_types::CoreError::Terminal(err.to_string()),
            ScribeError::Whisper(_) => gen_ui_types::CoreError::Terminal(err.to_string()),
            ScribeError::Io(_) => gen_ui_types::CoreError::Io(err.to_string()),
        }
    }
}
