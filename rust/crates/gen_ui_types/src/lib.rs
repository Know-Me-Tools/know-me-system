// TJ-ARCH-MOB-001 compliant
//! gen_ui_types (L0) — pure types + ALL cross-crate trait seams.
//! NO tokio, NO IO. Compiles on every target including wasm32.
//!
//! FROZEN after C-001 review: changing a trait here requires cross-lane sign-off
//! because every downstream crate depends on these signatures.
#![forbid(unsafe_code)]

pub mod config;
pub mod content_block;
pub mod error;
pub mod events;
pub mod inference;
pub mod sync;
pub mod transport;
pub mod view;

pub use content_block::ContentBlock;
pub use error::{CoreError, CoreResult};
