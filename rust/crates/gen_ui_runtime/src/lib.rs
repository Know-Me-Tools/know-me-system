// TJ-ARCH-MOB-001 compliant
//! gen_ui_runtime (L1) — one runtime abstraction, two backends.
//! Native: global multi-thread Tokio (one per process). wasm: spawn_local.

#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(not(target_arch = "wasm32"))]
pub use native::{handle, init, spawn, spawn_blocking};

#[cfg(target_arch = "wasm32")]
mod web;
#[cfg(target_arch = "wasm32")]
pub use web::spawn;
