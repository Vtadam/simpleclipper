//! WASAPI loopback audio capture and rolling audio buffer.
//!
//! Captures system audio (rendered output, i.e. "what you hear") via the
//! WASAPI loopback mode. On non-Windows builds this is a no-op stub.

use bytes::Bytes;
use crossbeam_channel::Sender;
use parking_lot::Mutex;
use std::collections::VecDeque;
use std::sync::Arc;

/// Raw audio chunk from WASAPI.
pub struct AudioChunk {
    pub data: Vec<f32>, // interleaved PCM float32 samples
    pub sample_rate: u32,
    pub channels: u16,
    pub pts_ms: i64,
}

/// Raw PCM i16 audio packet (stored in the rolling buffer; encoded to AAC during save).
#[derive(Clone)]
pub struct AudioPacket {
    pub pts_ms: i64,
    pub data: Bytes,     // i16 LE PCM bytes
    pub sample_rate: u32,
    pub channels: u16,
}

// ── Rolling audio buffer ────────────────────────────────────────────────────

pub struct AudioRollingBuffer {
    inner: Mutex<VecDeque<AudioPacket>>,
    max_bytes: usize,
    total_bytes: Mutex<usize>,
}

impl AudioRollingBuffer {
    pub fn new(max_bytes: usize) -> Self {
        Self {
            inner: Mutex::new(VecDeque::new()),
            max_bytes,
            total_bytes: Mutex::new(0),
        }
    }

    pub fn push(&self, packet: AudioPacket) {
        let size = packet.data.len();
        let mut inner = self.inner.lock();
        let mut total = self.total_bytes.lock();
        inner.push_back(packet);
        *total += size;
        while *total > self.max_bytes {
            if let Some(p) = inner.pop_front() {
                *total = total.saturating_sub(p.data.len());
            } else {
                break;
            }
        }
    }

    pub fn drain_last_ms(&self, duration_ms: i64) -> Vec<AudioPacket> {
        let inner = self.inner.lock();
        if inner.is_empty() {
            return vec![];
        }
        let latest = inner.back().map(|p| p.pts_ms).unwrap_or(0);
        let start = latest - duration_ms;
        inner.iter().filter(|p| p.pts_ms >= start).cloned().collect()
    }
}

// ── WASAPI loopback capture ─────────────────────────────────────────────────

#[cfg(target_os = "windows")]
pub fn run_audio_capture(
    tx: Sender<AudioChunk>,
    stop: Arc<std::sync::atomic::AtomicBool>,
) {
    use windows::{
        Win32::Media::Audio::{
            eConsole, eRender, IAudioCaptureClient, IAudioClient, IMMDeviceEnumerator,
            MMDeviceEnumerator, AUDCLNT_BUFFERFLAGS_SILENT, AUDCLNT_SHAREMODE_SHARED,
            AUDCLNT_STREAMFLAGS_LOOPBACK,
        },
        Win32::System::Com::{
            CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize, CLSCTX_ALL,
            COINIT_MULTITHREADED,
        },
    };

    // SAFETY: CoInitializeEx is required before any COM usage on this thread.
    // CoUninitialize is called at the end to balance this call.
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);

        let enumerator: IMMDeviceEnumerator = match CoCreateInstance(
            &MMDeviceEnumerator,
            None,
            CLSCTX_ALL,
        ) {
            Ok(e) => e,
            Err(e) => {
                log::error!("CoCreateInstance IMMDeviceEnumerator failed: {}", e);
                CoUninitialize();
                return;
            }
        };

        let device = match enumerator.GetDefaultAudioEndpoint(eRender, eConsole) {
            Ok(d) => d,
            Err(e) => {
                log::error!("GetDefaultAudioEndpoint failed: {}", e);
                CoUninitialize();
                return;
            }
        };

        let audio_client: IAudioClient = match device.Activate(CLSCTX_ALL, None) {
            Ok(c) => c,
            Err(e) => {
                log::error!("IAudioClient activate failed: {}", e);
                CoUninitialize();
                return;
            }
        };

        let mix_format = match audio_client.GetMixFormat() {
            Ok(f) => f,
            Err(e) => {
                log::error!("GetMixFormat failed: {}", e);
                CoUninitialize();
                return;
            }
        };

        let sample_rate = (*mix_format).nSamplesPerSec;
        let channels = (*mix_format).nChannels;

        // 1-second buffer in 100-ns units
        let hns_buffer: i64 = 10_000_000;

        if let Err(e) = audio_client.Initialize(
            AUDCLNT_SHAREMODE_SHARED,
            AUDCLNT_STREAMFLAGS_LOOPBACK,
            hns_buffer,
            0,
            mix_format,
            None,
        ) {
            log::error!("IAudioClient Initialize failed: {}", e);
            CoTaskMemFree(Some(mix_format as *const _ as *const _));
            CoUninitialize();
            return;
        }

        let capture_client: IAudioCaptureClient = match audio_client.GetService() {
            Ok(c) => c,
            Err(e) => {
                log::error!("GetService IAudioCaptureClient failed: {}", e);
                CoTaskMemFree(Some(mix_format as *const _ as *const _));
                CoUninitialize();
                return;
            }
        };

        let _ = audio_client.Start();
        let start = std::time::Instant::now();

        'outer: loop {
            if stop.load(std::sync::atomic::Ordering::Relaxed) {
                break;
            }

            std::thread::sleep(std::time::Duration::from_millis(10));

            // Drain all available audio packets from WASAPI
            loop {
                let mut data_ptr: *mut u8 = std::ptr::null_mut();
                let mut frames_available: u32 = 0;
                let mut flags: u32 = 0;
                let mut device_position: u64 = 0;
                let mut qpc_position: u64 = 0;

                match capture_client.GetBuffer(
                    &mut data_ptr,
                    &mut frames_available,
                    &mut flags,
                    Some(&mut device_position),
                    Some(&mut qpc_position),
                ) {
                    Ok(_) => {}
                    Err(_) => break,
                }

                if frames_available == 0 {
                    let _ = capture_client.ReleaseBuffer(0);
                    break;
                }

                let pts_ms = start.elapsed().as_millis() as i64;

                // SAFETY: `data_ptr` is valid while the buffer is held by GetBuffer.
                // The frame count and channel count determine the total sample count.
                // We release the buffer immediately after copying.
                let samples: Vec<f32> =
                    if flags & (AUDCLNT_BUFFERFLAGS_SILENT.0 as u32) != 0 {
                        vec![0.0f32; frames_available as usize * channels as usize]
                    } else {
                        let sample_count = frames_available as usize * channels as usize;
                        let slice = std::slice::from_raw_parts(
                            data_ptr as *const f32,
                            sample_count,
                        );
                        slice.to_vec()
                    };

                let _ = capture_client.ReleaseBuffer(frames_available);

                let chunk = AudioChunk {
                    data: samples,
                    sample_rate,
                    channels,
                    pts_ms,
                };

                if tx.try_send(chunk).is_err() {
                    // Encoder channel is full or disconnected
                    break 'outer;
                }
            }
        }

        let _ = audio_client.Stop();
        // SAFETY: mix_format was allocated by the WASAPI runtime and must be freed
        // with CoTaskMemFree, which is the correct paired deallocation.
        CoTaskMemFree(Some(mix_format as *const _ as *const _));
        CoUninitialize();
    }
}

#[cfg(not(target_os = "windows"))]
pub fn run_audio_capture(
    _tx: Sender<AudioChunk>,
    stop: Arc<std::sync::atomic::AtomicBool>,
) {
    // No-op on non-Windows
    while !stop.load(std::sync::atomic::Ordering::Relaxed) {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}
