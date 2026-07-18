// TJ-ARCH-MOB-001 compliant
//! tauri-plugin-gen-ui (LEAF) — Tauri 2 plugin exposing the same intent surface as
//! gen_ui_ffi, but as `#[tauri::command]` handlers. Desktop shares gen_ui_core in
//! the SAME process (no FFI): commands call the intent functions directly.
//!
//! LAYER CONTRACT: the JS side invokes these ONLY from Zustand stores — never from
//! a React component or hook. The npm guest-js package (guest-js/) provides typed
//! wrappers + `listen` helpers for the event channels.
use tauri::{
    plugin::{Builder, TauriPlugin},
    Emitter, Manager, Runtime,
};

mod commands;
mod dev_ollama;
mod error;

pub use error::{Error, Result};

/// Emit-channel names for the three UI-facing event feeds (mirror gen_ui_ffi
/// streams). Stores subscribe with `listen(GEN_UI_CHAT_EVENT, ...)`.
pub const GEN_UI_CHAT_EVENT: &str = "gen-ui://chat-event";
pub const GEN_UI_ENTITY_CHANGE: &str = "gen-ui://entity-change";
pub const GEN_UI_SYNC_STATUS: &str = "gen-ui://sync-status";

/// Build the plugin. Register in the Tauri app with `.plugin(tauri_plugin_gen_ui::init())`.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("gen-ui")
        .invoke_handler(tauri::generate_handler![
            commands::stream_agent_a2ui,
            commands::get_active_lane,
            commands::set_active_lane,
            commands::has_local_engine,
            commands::provider_catalog,
            commands::provider_list,
            commands::provider_save,
            commands::provider_delete,
            commands::cloud_model_get,
            commands::cloud_model_save,
            commands::run_migrations,
            commands::load_seeds,
            commands::attach_sync_shapes,
            commands::memory_ingest,
            commands::entity_runtime_start,
            commands::entity_runtime_stop,
            commands::entity_list,
            commands::entity_get,
            commands::entity_create,
            commands::entity_update,
            commands::entity_delete,
            commands::memory_search,
            commands::graph_expand,
            commands::scribe_start,
            commands::scribe_stop,
        ])
        .setup(|app, _api| {
            // One global Tokio runtime per process (never a second). Desktop
            // shares gen_ui_core in-process, so init here rather than via FFI.
            gen_ui_runtime::init(None);
            // Opt-in dev shortcut (GEN_UI_DEV_OLLAMA_MODEL): if it installs,
            // run_migrations's later state::init call becomes a harmless no-op
            // (see state.rs's doc comment on double-init).
            dev_ollama::install_if_requested();
            spawn_chat_event_forwarder(app.app_handle().clone());
            Ok(())
        })
        .build()
}

/// Forward gen_ui_agent's A2uiEvent broadcast onto the Tauri event channel the
/// frontend already listens on. Runs for the app's lifetime; a subscribe
/// failure (agent state not yet initialised — run_migrations hasn't executed)
/// just means no events exist yet to forward, not an error worth logging loudly
/// on every launch.
fn spawn_chat_event_forwarder<R: Runtime>(app: tauri::AppHandle<R>) {
    gen_ui_runtime::spawn(async move {
        // gen_ui_agent::state::init() runs inside the run_migrations command,
        // which fires after this setup hook — poll briefly for it rather than
        // failing the whole forwarder on a benign startup race.
        let mut rx = loop {
            match gen_ui_agent::state::subscribe() {
                Ok(rx) => break rx,
                Err(_) => tokio::time::sleep(std::time::Duration::from_millis(50)).await,
            }
        };
        while let Ok(event) = rx.recv().await {
            match &event {
                gen_ui_types::events::A2uiEvent::Block {
                    block: gen_ui_types::ContentBlock::Text { text },
                } => tracing::info!(characters = text.chars().count(), "desktop chat text block"),
                gen_ui_types::events::A2uiEvent::RunFinished { run_id } => {
                    tracing::info!(run_id = %run_id, "desktop chat run finished")
                }
                gen_ui_types::events::A2uiEvent::RunError { message } => {
                    tracing::error!(error = %message, "desktop chat run failed")
                }
                _ => {}
            }
            let _ = app.emit(GEN_UI_CHAT_EVENT, &event);
        }
    });
}
