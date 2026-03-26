use crate::config::KeybindEntry;
use tauri::AppHandle;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
use anyhow::Result;

pub fn register_all(app: &AppHandle, keybinds: &[KeybindEntry]) -> Result<()> {
    // Unregister all existing shortcuts first
    let _ = app.global_shortcut().unregister_all();

    for entry in keybinds {
        let shortcut_str = match &entry.shortcut {
            Some(s) if !s.is_empty() => s.clone(),
            _ => continue,
        };

        let shortcut: Shortcut = match shortcut_str.parse() {
            Ok(s) => s,
            Err(e) => {
                log::warn!("Invalid shortcut '{}': {}", shortcut_str, e);
                continue;
            }
        };

        let duration_secs = entry.duration_secs;
        let app_handle = app.clone();
        let label = shortcut_str.clone();

        match app.global_shortcut().on_shortcut(shortcut, move |_app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                let handle = app_handle.clone();
                tokio::spawn(async move {
                    crate::commands::handlers::trigger_clip_save(&handle, duration_secs).await;
                });
            }
        }) {
            Ok(_) => log::info!("Registered shortcut: {} for {}s", label, duration_secs),
            Err(e) => log::warn!("Failed to register shortcut {}: {}", label, e),
        }
    }

    Ok(())
}
