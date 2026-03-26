//! Screen capture source abstraction.
//!
//! Spawns a dedicated OS thread running DXGI Desktop Duplication capture,
//! then forwards frames to the encoder channel via a Tokio task bridge.

use super::dxgi::DxgiCapture;
use super::encoder::RawFrame;
use crossbeam_channel::Sender;
use std::time::Instant;

/// Start capturing the full screen, sending frames to `tx`.
/// Runs until `stop_rx` resolves or the capture thread dies.
pub async fn run_fullscreen_capture(
    tx: Sender<RawFrame>,
    mut stop_rx: tokio::sync::oneshot::Receiver<()>,
) {
    // DXGI must run on a dedicated OS thread (COM apartment model).
    // We use a bounded crossbeam channel to bridge the capture thread to async world.
    let (frame_tx, frame_rx) = crossbeam_channel::bounded::<RawFrame>(4);
    let stop_flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let stop_flag_clone = stop_flag.clone();

    std::thread::spawn(move || {
        // Initialize COM for this thread in multithreaded apartment mode.
        // SAFETY: CoInitializeEx must be called before any COM usage on this thread.
        // CoUninitialize is called at the end to balance.
        #[cfg(target_os = "windows")]
        unsafe {
            let _ = windows::Win32::System::Com::CoInitializeEx(
                None,
                windows::Win32::System::Com::COINIT_MULTITHREADED,
            );
        }

        let mut capture = match DxgiCapture::new() {
            Ok(c) => c,
            Err(e) => {
                log::error!("DXGI capture init failed: {}", e);
                #[cfg(target_os = "windows")]
                unsafe {
                    windows::Win32::System::Com::CoUninitialize();
                }
                return;
            }
        };

        let start = Instant::now();
        // Target ~30fps: one frame every ~33ms
        let frame_interval = std::time::Duration::from_millis(33);

        loop {
            if stop_flag_clone.load(std::sync::atomic::Ordering::Relaxed) {
                break;
            }

            let frame_start = Instant::now();

            match capture.capture_frame() {
                Ok(Some(frame)) => {
                    let raw = RawFrame {
                        data: frame.data,
                        width: frame.width,
                        height: frame.height,
                        pts_ms: start.elapsed().as_millis() as i64,
                    };
                    // Non-blocking: if the encoder channel is full, drop the frame
                    // rather than stalling the capture thread.
                    let _ = frame_tx.try_send(raw);
                }
                Ok(None) => {
                    // AcquireNextFrame timed out — no new frame yet, continue
                }
                Err(e) => {
                    log::warn!("Capture frame error: {}", e);
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            }

            // Rate-limit to ~30fps
            let elapsed = frame_start.elapsed();
            if elapsed < frame_interval {
                std::thread::sleep(frame_interval - elapsed);
            }
        }

        #[cfg(target_os = "windows")]
        unsafe {
            windows::Win32::System::Com::CoUninitialize();
        }
    });

    // Bridge: forward frames from the capture thread channel to the encoder channel.
    // We use spawn_blocking so the crossbeam recv_timeout doesn't block the Tokio runtime.
    loop {
        tokio::select! {
            _ = &mut stop_rx => {
                stop_flag.store(true, std::sync::atomic::Ordering::Relaxed);
                break;
            }
            result = tokio::task::spawn_blocking({
                let rx = frame_rx.clone();
                move || rx.recv_timeout(std::time::Duration::from_millis(50))
            }) => {
                match result {
                    Ok(Ok(frame)) => {
                        // Non-blocking forward to encoder
                        let _ = tx.try_send(frame);
                    }
                    Ok(Err(_)) => {
                        // recv_timeout timed out or channel closed — loop again
                    }
                    Err(e) => {
                        log::warn!("spawn_blocking error in source bridge: {}", e);
                    }
                }
            }
        }
    }
}
