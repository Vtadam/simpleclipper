mod capture;
mod clip;
mod commands;
mod config;
mod hotkeys;
mod startup;
mod tray;

use tauri::Manager;

pub use commands::handlers::*;

pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // Focus existing window if second instance launched
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .setup(|app| {
            // Initialize config
            let config = config::load_or_default(app.handle())?;
            app.manage(config::ConfigState::new(config));

            // Initialize capture state
            app.manage(capture::CaptureState::new());

            // Set up system tray
            tray::setup_tray(app)?;

            // Hide main window on start (tray-only)
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.hide();
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::handlers::get_config,
            commands::handlers::save_config,
            commands::handlers::open_folder_dialog,
            commands::handlers::open_file_in_explorer,
            commands::handlers::get_clips_list,
            commands::handlers::start_capture,
            commands::handlers::stop_capture,
            commands::handlers::get_capture_status,
            commands::handlers::register_keybinds,
            commands::handlers::capture_next_keypress,
            commands::handlers::set_start_with_windows,
            commands::handlers::get_windows_list,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
