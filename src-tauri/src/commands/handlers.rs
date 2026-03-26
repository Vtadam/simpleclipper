use crate::{
    capture::{CaptureState, CaptureStatus},
    clip::{saver, thumbnailer},
    config::{AppConfig, ConfigState, KeybindEntry},
    hotkeys, startup,
};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_shell::ShellExt;

// ── Config ───────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_config(state: State<'_, ConfigState>) -> Result<AppConfig, String> {
    Ok(state.0.read().clone())
}

#[tauri::command]
pub async fn save_config(
    app: AppHandle,
    state: State<'_, ConfigState>,
    config: AppConfig,
) -> Result<(), String> {
    *state.0.write() = config.clone();
    crate::config::save(&app, &config).map_err(|e| e.to_string())
}

// ── File System ──────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn open_folder_dialog(app: AppHandle) -> Result<Option<String>, String> {
    let folder = app.dialog().file().blocking_pick_folder();
    Ok(folder.map(|p| p.to_string()))
}

#[tauri::command]
pub async fn open_file_in_explorer(app: AppHandle, path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        app.shell()
            .command("explorer")
            .args(["/select,", &path])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(not(target_os = "windows"))]
    let _ = (app, path);
    Ok(())
}

// ── Clips ────────────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct ClipMeta {
    pub path: String,
    pub thumbnail_b64: Option<String>,
    pub duration_secs: u32,
    pub size_bytes: u64,
    pub timestamp: String,
    pub name: String,
}

#[tauri::command]
pub async fn get_clips_list(state: State<'_, ConfigState>) -> Result<Vec<ClipMeta>, String> {
    let save_folder = state.0.read().save_folder.clone();
    if !save_folder.exists() {
        return Ok(vec![]);
    }

    let mut clips = vec![];
    let entries = std::fs::read_dir(&save_folder).map_err(|e| e.to_string())?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("mp4") {
            continue;
        }

        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("clip")
            .to_string();

        let meta = entry.metadata().map_err(|e| e.to_string())?;
        let size_bytes = meta.len();

        let modified = meta
            .modified()
            .ok()
            .and_then(|t| {
                t.duration_since(std::time::UNIX_EPOCH)
                    .ok()
                    .map(|d| d.as_secs())
            })
            .unwrap_or(0);

        let timestamp = chrono::DateTime::from_timestamp(modified as i64, 0)
            .unwrap_or_default()
            .to_rfc3339();

        // Parse duration from filename: clip_YYYYMMDD_HHMMSS_Ns.mp4
        let duration_secs: u32 = name
            .split('_')
            .last()
            .and_then(|s| s.trim_end_matches('s').parse().ok())
            .unwrap_or(0);

        let thumbnail_b64 = thumbnailer::get_or_create_thumbnail(&path)
            .ok()
            .filter(|s| !s.is_empty());

        clips.push(ClipMeta {
            path: path.to_string_lossy().to_string(),
            thumbnail_b64,
            duration_secs,
            size_bytes,
            timestamp,
            name,
        });
    }

    clips.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    Ok(clips)
}

// ── Capture ──────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn start_capture(
    app: AppHandle,
    config_state: State<'_, ConfigState>,
    capture_state: State<'_, CaptureState>,
) -> Result<(), String> {
    let config = config_state.0.read().clone();
    let buffer = capture_state.buffer.clone();
    let audio_buffer = capture_state.audio_buffer.clone();
    let status = capture_state.status.clone();
    let video_dims = capture_state.video_dims.clone();

    // Stop any existing capture before starting a new one
    if let Some(tx) = capture_state.stop_tx.lock().take() {
        let _ = tx.send(());
    }
    capture_state
        .audio_stop
        .store(true, std::sync::atomic::Ordering::Relaxed);

    let (stop_tx, stop_rx) = tokio::sync::oneshot::channel::<()>();
    *capture_state.stop_tx.lock() = Some(stop_tx);

    // Reset the audio stop flag for the new session
    capture_state
        .audio_stop
        .store(false, std::sync::atomic::Ordering::Relaxed);

    // Determine bitrate from the quality preset
    let bitrate: u64 = config.quality.bitrate_bps();

    // ── Video encoder thread ────────────────────────────────────────────────
    let sps_pps = capture_state.sps_pps.clone();
    let (frame_tx, frame_rx) = crossbeam_channel::bounded::<crate::capture::encoder::RawFrame>(8);
    let buffer_clone = buffer.clone();
    std::thread::spawn(move || {
        crate::capture::encoder::encode_loop(frame_rx, buffer_clone, bitrate, sps_pps);
    });

    // ── Video capture task (async, spawns its own OS thread internally) ─────
    // We intercept frames to update video_dims on the first one.
    let (wrapped_tx, wrapped_rx) =
        crossbeam_channel::bounded::<crate::capture::encoder::RawFrame>(8);
    let video_dims_clone = video_dims.clone();
    let frame_tx_clone = frame_tx.clone();

    // Dimension-sniffing forwarder thread
    std::thread::spawn(move || {
        let mut dims_set = false;
        for frame in wrapped_rx.into_iter() {
            if !dims_set {
                *video_dims_clone.write() = (frame.width, frame.height);
                dims_set = true;
            }
            let _ = frame_tx_clone.try_send(frame);
        }
    });

    let app_clone = app.clone();
    let status_clone = status.clone();
    tokio::spawn(async move {
        crate::capture::source::run_fullscreen_capture(wrapped_tx, stop_rx).await;
        status_clone.write().running = false;
        let _ = app_clone.emit(
            "capture-error",
            serde_json::json!({ "message": "Capture stopped" }),
        );
    });

    // ── Audio pipeline ──────────────────────────────────────────────────────
    let (audio_chunk_tx, audio_chunk_rx) =
        crossbeam_channel::bounded::<crate::capture::audio::AudioChunk>(32);
    let audio_buffer_clone = audio_buffer.clone();
    let audio_stop = capture_state.audio_stop.clone();

    // Audio encoder thread
    std::thread::spawn(move || {
        crate::capture::audio_encoder::audio_encode_loop(audio_chunk_rx, audio_buffer_clone);
    });

    // Audio capture thread
    std::thread::spawn(move || {
        crate::capture::audio::run_audio_capture(audio_chunk_tx, audio_stop);
    });

    // ── Status ──────────────────────────────────────────────────────────────
    status.write().running = true;

    // Buffer stats emitter
    let buffer_ref = buffer.clone();
    let app_ref = app.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            let (_, bytes, duration_ms) = buffer_ref.stats();
            let _ = app_ref.emit(
                "buffer-updated",
                serde_json::json!({
                    "buffer_secs": duration_ms as f64 / 1000.0,
                    "buffer_bytes": bytes
                }),
            );
        }
    });

    Ok(())
}

#[tauri::command]
pub async fn stop_capture(capture_state: State<'_, CaptureState>) -> Result<(), String> {
    if let Some(tx) = capture_state.stop_tx.lock().take() {
        let _ = tx.send(());
    }
    capture_state
        .audio_stop
        .store(true, std::sync::atomic::Ordering::Relaxed);
    capture_state.status.write().running = false;
    Ok(())
}

#[tauri::command]
pub async fn get_capture_status(
    capture_state: State<'_, CaptureState>,
) -> Result<CaptureStatus, String> {
    Ok(capture_state.status.read().clone())
}

/// Called by the hotkey manager to save a clip of the last `duration_secs` seconds.
pub async fn trigger_clip_save(app: &AppHandle, duration_secs: u32) {
    let config_state = app.state::<ConfigState>();
    let capture_state = app.state::<CaptureState>();

    let save_folder = config_state.0.read().save_folder.clone();
    let video_buffer = capture_state.buffer.clone();
    let audio_buffer = capture_state.audio_buffer.clone();
    let (video_width, video_height) = *capture_state.video_dims.read();
    let sps_pps = capture_state.sps_pps.lock().clone();

    let _ = app.emit(
        "hotkey-triggered",
        serde_json::json!({ "duration_secs": duration_secs }),
    );

    match saver::save_clip(
        video_buffer,
        audio_buffer,
        &save_folder,
        duration_secs,
        video_width,
        video_height,
        sps_pps,
    )
    .await
    {
        Ok(clip) => {
            let _ = app.emit(
                "clip-saved",
                serde_json::json!({
                    "path": clip.path.to_string_lossy(),
                    "duration_secs": clip.duration_secs,
                    "size_bytes": clip.size_bytes,
                }),
            );
        }
        Err(e) => {
            log::error!("Failed to save clip: {}", e);
            let _ = app.emit(
                "capture-error",
                serde_json::json!({ "message": e.to_string() }),
            );
        }
    }
}

// ── Hotkeys ──────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn register_keybinds(
    app: AppHandle,
    entries: Vec<KeybindEntry>,
) -> Result<(), String> {
    hotkeys::register_all(&app, &entries).map_err(|e| e.to_string())
}

/// Keybind capture is now handled entirely in the frontend via DOM keydown events.
/// This command is retained for backwards compatibility but always returns an error
/// directing callers to use the frontend hook instead.
#[tauri::command]
pub async fn capture_next_keypress(_app: AppHandle) -> Result<String, String> {
    Err("Use frontend keydown listener directly".to_string())
}

// ── System ────────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn set_start_with_windows(app: AppHandle, enabled: bool) -> Result<(), String> {
    startup::set_start_with_windows(&app, enabled).map_err(|e| e.to_string())
}

#[derive(Debug, Clone, Serialize)]
pub struct WindowInfo {
    pub hwnd: usize,
    pub title: String,
    pub exe: String,
}

#[tauri::command]
pub async fn get_windows_list() -> Result<Vec<WindowInfo>, String> {
    #[cfg(target_os = "windows")]
    {
        use std::ffi::OsString;
        use std::os::windows::ffi::OsStringExt;
        use windows::Win32::Foundation::{BOOL, HWND, LPARAM};
        use windows::Win32::UI::WindowsAndMessaging::{
            EnumWindows, GetWindowTextW, GetWindowThreadProcessId, IsWindowVisible,
        };

        // Shared mutable list passed via LPARAM to the EnumWindows callback.
        // We use a raw pointer to Arc<Mutex<Vec<WindowInfo>>> for FFI compatibility.
        let windows_list: std::sync::Arc<parking_lot::Mutex<Vec<WindowInfo>>> =
            std::sync::Arc::new(parking_lot::Mutex::new(Vec::new()));
        let list_clone = windows_list.clone();

        // SAFETY: enum_proc is an `unsafe extern "system"` callback as required by EnumWindows.
        // lparam contains a raw pointer to a parking_lot::Mutex<Vec<WindowInfo>> whose
        // lifetime is guaranteed by the Arc kept alive on the stack above.
        unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
            // SAFETY: lparam is a raw pointer to parking_lot::Mutex<Vec<WindowInfo>>.
            // It was created from a valid Arc and is alive for the duration of EnumWindows.
            let list = &*(lparam.0 as *const parking_lot::Mutex<Vec<WindowInfo>>);

            // Skip invisible windows
            if !IsWindowVisible(hwnd).as_bool() {
                return BOOL(1);
            }

            // Get window title
            let mut title_buf = [0u16; 512];
            let len = GetWindowTextW(hwnd, &mut title_buf);
            if len == 0 {
                return BOOL(1);
            }
            let title = OsString::from_wide(&title_buf[..len as usize])
                .to_string_lossy()
                .to_string();

            if title.trim().is_empty() {
                return BOOL(1);
            }

            // Get the process name for this window
            let mut pid = 0u32;
            GetWindowThreadProcessId(hwnd, Some(&mut pid));

            let exe = get_process_name(pid);

            list.lock().push(WindowInfo {
                hwnd: hwnd.0 as usize,
                title,
                exe,
            });

            BOOL(1)
        }

        // SAFETY: EnumWindows is a safe Windows API. We pass a raw pointer to our
        // Mutex which is kept alive by the Arc on the stack for the entire call duration.
        unsafe {
            EnumWindows(
                Some(enum_proc),
                LPARAM(list_clone.as_ref() as *const _ as isize),
            )
            .map_err(|e| e.to_string())?;
        }

        let result = windows_list.lock().clone();
        return Ok(result);
    }

    #[cfg(not(target_os = "windows"))]
    Ok(vec![])
}

/// Resolve a Windows PID to its executable file name (e.g. "chrome.exe").
#[cfg(target_os = "windows")]
fn get_process_name(pid: u32) -> String {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
        PROCESS_QUERY_LIMITED_INFORMATION,
    };
    use windows::core::PWSTR;

    // SAFETY: OpenProcess returns a handle that must be closed with CloseHandle.
    // We close it in all code paths below.
    unsafe {
        let handle = match OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
            Ok(h) => h,
            Err(_) => return String::new(),
        };

        let mut buf = [0u16; 512];
        let mut size = buf.len() as u32;
        let result = QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_WIN32,
            PWSTR(buf.as_mut_ptr()),
            &mut size,
        );
        let _ = CloseHandle(handle);

        if result.is_err() || size == 0 {
            return String::new();
        }

        let full_path = OsString::from_wide(&buf[..size as usize])
            .to_string_lossy()
            .to_string();

        std::path::Path::new(&full_path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default()
    }
}
