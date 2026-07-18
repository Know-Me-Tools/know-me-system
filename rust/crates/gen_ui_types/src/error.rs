// TJ-ARCH-MOB-001 compliant
//! Shared error taxonomy. Library crates map their errors into CoreError.
use thiserror::Error;

pub type CoreResult<T> = Result<T, CoreError>;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("transient (retryable): {0}")]
    Transient(String),
    #[error("terminal (do not retry): {0}")]
    Terminal(String),
    #[error("serialization: {0}")]
    Serde(String),
    #[error("io: {0}")]
    Io(String),
}
