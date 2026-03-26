use anyhow::Result;
use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;

pub fn set_start_with_windows(app: &AppHandle, enabled: bool) -> Result<()> {
    let autostart = app.autolaunch();
    if enabled {
        autostart.enable()?;
    } else {
        autostart.disable()?;
    }
    Ok(())
}
