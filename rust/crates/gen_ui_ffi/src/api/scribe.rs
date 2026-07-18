// TJ-ARCH-MOB-001 compliant
//! Scribe (voice-to-memory) intent surface: record -> on-device transcribe ->
//! hand text back for the caller to fold into `chat::memory_ingest`. Delegates
//! entirely to `gen_ui_audio` (shared with tauri-plugin-gen-ui desktop) — no
//! duplicated business logic between mobile/desktop.
//!
//! This module is behind `mobile-scribe`. The default iOS FFI build uses
//! llama.cpp for local chat; whisper.cpp must move to a separate Apple framework
//! before both GGML implementations can coexist safely in one application.
//!
//! One recording in flight at a time per process, mirroring gen_ui_agent's
//! single-chat-turn precedent: `scribe_start` errors if a recording is already
//! live rather than silently discarding it.
// frb's Result<T,E> detection only matches a literal `Result<...>` return
// type — it does NOT resolve through a generic type alias like `CoreResult<T>`
// (verified against flutter_rust_bridge_codegen 2.12.0's alias-parsing filter,
// which drops any `type Foo<T> = ...` with generics before resolution ever
// runs). Every frb-exposed fn in this crate MUST spell out
// `Result<T, gen_ui_types::CoreError>` literally or Dart gets an opaque
// blob with zero field/error access instead of a normal Future<T> that
// throws on Err.
use gen_ui_types::CoreError;
use once_cell::sync::OnceCell;
use std::sync::Mutex;

static RECORDING: OnceCell<Mutex<Option<gen_ui_audio::Recorder>>> = OnceCell::new();

fn slot() -> &'static Mutex<Option<gen_ui_audio::Recorder>> {
    RECORDING.get_or_init(|| Mutex::new(None))
}

/// Start a microphone recording. Errors if one is already in progress.
pub fn scribe_start() -> Result<(), CoreError> {
    let mut guard = slot().lock().expect("scribe recording mutex poisoned");
    if guard.is_some() {
        return Err(gen_ui_types::CoreError::Terminal(
            "a recording is already in progress".to_string(),
        ));
    }
    let recorder = gen_ui_audio::Scribe::new()
        .start_recording()
        .map_err(Into::<gen_ui_types::CoreError>::into)?;
    *guard = Some(recorder);
    Ok(())
}

/// Stop the in-flight recording and transcribe it on-device. Returns the
/// transcript; the caller (Dart UI) decides whether/how to save it to memory.
pub async fn scribe_stop() -> Result<String, CoreError> {
    let recorder = slot()
        .lock()
        .expect("scribe recording mutex poisoned")
        .take()
        .ok_or_else(|| gen_ui_types::CoreError::Terminal("no recording in progress".to_string()))?;
    gen_ui_audio::Scribe::new()
        .stop_and_transcribe(recorder)
        .await
        .map_err(Into::<gen_ui_types::CoreError>::into)
}
