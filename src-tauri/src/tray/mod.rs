use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};
use anyhow::Result;

pub fn setup_tray(app: &tauri::App) -> Result<()> {
    let show = MenuItem::with_id(app, "show", "Open SimpleClipper", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &quit])?;

    TrayIconBuilder::new()
        .icon(app.default_window_icon().cloned().unwrap())
        .menu(&menu)
        .tooltip("SimpleClipper — Recording")
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => show_main_window(app),
            "quit" => {
                // Stop capture before quitting
                std::process::exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main_window(tray.app_handle());
            }
        })
        .build(app)?;

    Ok(())
}

fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        // Position bottom-right, just above the taskbar
        if let Ok(monitor) = window.primary_monitor() {
            if let Some(monitor) = monitor {
                let screen = monitor.size();
                let scale = monitor.scale_factor();
                let win_w = (320.0 * scale) as i32;
                let win_h = (460.0 * scale) as i32;
                let taskbar_gap = (60.0 * scale) as i32;
                let margin = (12.0 * scale) as i32;
                let x = screen.width as i32 - win_w - margin;
                let y = screen.height as i32 - win_h - taskbar_gap;
                let _ = window.set_position(tauri::PhysicalPosition::new(x, y));
            }
        }
        let _ = window.show();
        let _ = window.set_focus();
    }
}
