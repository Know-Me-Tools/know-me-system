// TJ-ARCH-MOB-001 compliant
// Tauri application entry point. Intent commands (chat, entity CRUD, memory,
// startup) are registered by tauri-plugin-gen-ui, NOT here — this file owns
// only app-shell concerns (menu, window chrome, other Tauri plugins).

use tauri::menu::{AboutMetadata, Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::Manager;

fn init_logging() {
    let _ = tracing_log::LogTracer::init();
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    let diagnostic_root = std::env::var_os("GEN_UI_APP_DATA_DIR")
        .map(std::path::PathBuf::from)
        .or_else(|| {
            directories::ProjectDirs::from("ai", "prometheusags", "knowme-poc")
                .map(|project| project.data_local_dir().to_path_buf())
        });
    let file = diagnostic_root.and_then(|root| {
        let directory = root.join("diagnostics");
        std::fs::create_dir_all(&directory).ok()?;
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(directory.join("desktop.log"))
            .ok()
    });

    match file {
        Some(file) => {
            let _ = tracing_subscriber::fmt()
                .with_env_filter(filter)
                .with_writer(std::sync::Mutex::new(file))
                .with_ansi(false)
                .try_init();
            tracing::info!("persistent desktop diagnostics initialized");
        }
        None => {
            let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
            tracing::warn!("persistent diagnostic file unavailable; using process output");
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_logging();
    tracing::info!("desktop boot starting");
    tauri::Builder::default()
        // Must be registered first: a second launch attempt is redirected here
        // instead of racing the first instance for the config-db PGlite lock.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_focus();
                let _ = window.unminimize();
            }
        }))
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_gen_ui::init())
        .setup(|app| {
            tracing::info!("tauri setup starting");
            let exit = MenuItem::with_id(app, "exit", "Exit", true, Some("CmdOrCtrl+Q"))?;
            let file_menu = Submenu::with_items(app, "File", true, &[&exit])?;

            let toggle_devtools = MenuItem::with_id(
                app,
                "toggle_devtools",
                "Toggle Developer Tools",
                true,
                Some("CmdOrCtrl+Alt+I"),
            )?;
            // Toggle Developer Tools is the LAST item in View, per design.
            let view_menu = Submenu::with_items(
                app,
                "View",
                true,
                &[
                    &PredefinedMenuItem::fullscreen(app, None)?,
                    &PredefinedMenuItem::separator(app)?,
                    &toggle_devtools,
                ],
            )?;

            let about = PredefinedMenuItem::about(
                app,
                Some("About KnowMe"),
                Some(AboutMetadata::default()),
            )?;
            let help_menu = Submenu::with_items(app, "Help", true, &[&about])?;

            let menu = Menu::with_items(app, &[&file_menu, &view_menu, &help_menu])?;
            app.set_menu(menu)?;

            app.on_menu_event(move |app_handle, event| match event.id().as_ref() {
                "toggle_devtools" => {
                    if let Some(window) = app_handle.get_webview_window("main") {
                        if window.is_devtools_open() {
                            window.close_devtools();
                        } else {
                            window.open_devtools();
                        }
                    }
                }
                "exit" => app_handle.exit(0),
                _ => {}
            });

            tracing::info!("tauri setup ready");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
