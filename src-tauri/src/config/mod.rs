pub mod schema;
pub use schema::*;

use std::sync::Arc;
use parking_lot::RwLock;
use tauri::Manager;
use anyhow::Result;

pub struct ConfigState(pub Arc<RwLock<AppConfig>>);

impl ConfigState {
    pub fn new(config: AppConfig) -> Self {
        Self(Arc::new(RwLock::new(config)))
    }
}

pub fn load_or_default(app: &tauri::AppHandle) -> Result<AppConfig> {
    let config_path = get_config_path(app)?;
    if config_path.exists() {
        let contents = std::fs::read_to_string(&config_path)?;
        let config: AppConfig = serde_json::from_str(&contents)?;
        return Ok(config);
    }
    Ok(AppConfig::default())
}

pub fn save(app: &tauri::AppHandle, config: &AppConfig) -> Result<()> {
    let config_path = get_config_path(app)?;
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let contents = serde_json::to_string_pretty(config)?;
    std::fs::write(&config_path, contents)?;
    Ok(())
}

fn get_config_path(app: &tauri::AppHandle) -> Result<std::path::PathBuf> {
    let config_dir = app.path().app_config_dir()?;
    Ok(config_dir.join("config.json"))
}
