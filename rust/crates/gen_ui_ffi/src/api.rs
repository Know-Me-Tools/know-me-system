// TJ-ARCH-MOB-001 compliant
//! frb codegen root. Intent-level functions only (no raw SurrealQL / SQL across
//! the bridge). init + submodules for streams and CRUD.
//!
//! `streams`'s `StreamSink<T>` signatures reference the per-crate `StreamSink`
//! type that `flutter_rust_bridge_codegen generate` emits into
//! `frb_generated.rs` — this crate is unbuildable before the first codegen run
//! (see references/rust/new-block-type.md's codegen-first-run note). Codegen
//! itself must run with `--rust-features frb-streams` gone (removed entirely,
//! 2026-07: the module is no longer feature-gated) so it actually sees these
//! functions and generates their SseEncode impls + Dart stream bindings —
//! there is no build of this crate, gated or not, that doesn't need them.
pub mod chat;
pub mod entity;
pub mod streams;
// gen_ui_audio (whisper-rs) is an explicit mobile feature because whisper.cpp
// and llama.cpp cannot safely share one force-loaded iOS static archive.
pub mod boot;
#[cfg(all(feature = "mobile-scribe", not(target_os = "android")))]
pub mod scribe;

use flutter_rust_bridge::frb;

/// Initialise the shared core (global Tokio runtime + platform loggers).
/// Called once from Dart at app start. `#[frb(init)]` also wires frb's own
/// default utilities.
#[frb(init)]
pub fn init_core(worker_threads: Option<usize>) {
    #[cfg(target_os = "android")]
    let _ = android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Info),
    );
    #[cfg(not(target_arch = "wasm32"))]
    gen_ui_runtime::init(worker_threads);
    let _ = worker_threads;
}
