pub mod audio;
pub mod audio_encoder;
pub mod buffer;
pub mod dxgi;
pub mod encoder;
pub mod source;

use parking_lot::RwLock;
use std::sync::Arc;

#[derive(Debug, Clone, serde::Serialize)]
pub struct CaptureStatus {
    pub running: bool,
    pub source: String,
    pub fps: u32,
    pub buffer_secs: f64,
}

pub struct CaptureState {
    pub status: Arc<RwLock<CaptureStatus>>,
    pub buffer: Arc<buffer::RollingBuffer>,
    pub audio_buffer: Arc<audio::AudioRollingBuffer>,
    pub stop_tx: Arc<parking_lot::Mutex<Option<tokio::sync::oneshot::Sender<()>>>>,
    pub audio_stop: Arc<std::sync::atomic::AtomicBool>,
    /// Current video frame dimensions — set when the first frame is captured.
    pub video_dims: Arc<RwLock<(u32, u32)>>,
    /// SPS + PPS NAL units extracted from the first IDR frame.
    pub sps_pps: Arc<parking_lot::Mutex<Option<(Vec<u8>, Vec<u8>)>>>,
}

impl CaptureState {
    pub fn new() -> Self {
        Self {
            status: Arc::new(RwLock::new(CaptureStatus {
                running: false,
                source: "FullScreen".to_string(),
                fps: 0,
                buffer_secs: 0.0,
            })),
            // 2 GB video buffer
            buffer: Arc::new(buffer::RollingBuffer::new(2 * 1024 * 1024 * 1024)),
            // 256 MB audio buffer (stores i16 PCM; covers ~11 min stereo 48 kHz)
            audio_buffer: Arc::new(audio::AudioRollingBuffer::new(256 * 1024 * 1024)),
            stop_tx: Arc::new(parking_lot::Mutex::new(None)),
            audio_stop: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            video_dims: Arc::new(RwLock::new((0, 0))),
            sps_pps: Arc::new(parking_lot::Mutex::new(None)),
        }
    }
}
