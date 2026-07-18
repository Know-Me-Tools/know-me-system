// TJ-ARCH-MOB-001 compliant
//! Plugin command error. Serializes to a JS string so `invoke` rejects cleanly.
//! Maps gen_ui_types::CoreError; Terminal vs Transient is preserved in the message
//! so the store's retry policy can branch on it.
use serde::{Serialize, Serializer};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Core(#[from] gen_ui_types::CoreError),
    #[error("tauri: {0}")]
    Tauri(#[from] tauri::Error),
}

impl Serialize for Error {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}
