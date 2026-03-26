use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub first_run_complete: bool,
    pub save_folder: PathBuf,
    pub capture_source: CaptureSource,
    pub target_window_title: Option<String>,
    pub quality: QualityPreset,
    pub theme: Theme,
    pub start_with_windows: bool,
    pub keybinds: Vec<KeybindEntry>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            first_run_complete: false,
            save_folder: dirs_next::video_dir()
                .unwrap_or_else(|| PathBuf::from("C:\\Users\\Public\\Videos"))
                .join("SimpleClipper"),
            capture_source: CaptureSource::FullScreen,
            target_window_title: None,
            quality: QualityPreset::Medium,
            theme: Theme::Dark,
            start_with_windows: false,
            keybinds: vec![
                KeybindEntry { duration_secs: 30, shortcut: None },
                KeybindEntry { duration_secs: 60, shortcut: None },
                KeybindEntry { duration_secs: 300, shortcut: None },
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CaptureSource {
    FullScreen,
    Window,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum QualityPreset {
    Low,
    Medium,
    High,
}

impl QualityPreset {
    pub fn bitrate_bps(&self) -> u64 {
        match self {
            QualityPreset::Low => 4_000_000,
            QualityPreset::Medium => 8_000_000,
            QualityPreset::High => 16_000_000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Theme {
    Light,
    Dark,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeybindEntry {
    pub duration_secs: u32,
    pub shortcut: Option<String>,
}
