// TJ-ARCH-MOB-001 compliant
//! UAR mode + app configuration (pure, wasm-safe).
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum UarMode {
    #[default]
    Embedded,
    External {
        url: String,
        api_key: Option<String>,
        timeout_secs: u64,
    },
}

// (Default derived on the enum via #[default] on the Embedded variant.)
