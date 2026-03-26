//! H.264 video encoder using Windows Media Foundation H.264 MFT.
//!
//! Receives BGRA frames from the capture thread, converts to NV12, encodes
//! with the built-in Windows H.264 encoder, and pushes AVCC packets to the
//! rolling buffer.  Also extracts SPS/PPS from the first IDR frame and stores
//! them in the shared `sps_pps` slot so the clip saver can build the avcC box.

use super::buffer::{EncodedPacket, RollingBuffer};
use bytes::Bytes;
use crossbeam_channel::Receiver;
use parking_lot::Mutex;
use std::sync::Arc;

/// Raw BGRA frame from the capture source.
pub struct RawFrame {
    pub data: Vec<u8>, // BGRA bytes, width * height * 4
    pub width: u32,
    pub height: u32,
    pub pts_ms: i64,
}

/// Runs on a dedicated thread.
/// Receives BGRA frames, encodes to H.264 AVCC, pushes to `buffer`.
/// `bitrate` is in bits per second (e.g. 8_000_000 for 8 Mbps).
pub fn encode_loop(
    rx: Receiver<RawFrame>,
    buffer: Arc<RollingBuffer>,
    bitrate: u64,
    sps_pps: Arc<Mutex<Option<(Vec<u8>, Vec<u8>)>>>,
) {
    #[cfg(target_os = "windows")]
    windows_impl::run(rx, buffer, bitrate, sps_pps);

    #[cfg(not(target_os = "windows"))]
    {
        let _ = (bitrate, sps_pps);
        for _ in rx.into_iter() {}
    }
}

// ── BGRA → NV12 conversion ───────────────────────────────────────────────────

pub fn bgra_to_nv12(bgra: &[u8], width: u32, height: u32) -> Vec<u8> {
    let w = width as usize;
    let h = height as usize;
    let mut nv12 = vec![0u8; w * h * 3 / 2];

    // Y plane
    for y in 0..h {
        for x in 0..w {
            let s = (y * w + x) * 4;
            if s + 2 >= bgra.len() { break; }
            let b = bgra[s] as f32;
            let g = bgra[s + 1] as f32;
            let r = bgra[s + 2] as f32;
            nv12[y * w + x] = (16.0 + 65.481 * r / 255.0
                + 128.553 * g / 255.0
                + 24.966 * b / 255.0)
                .clamp(0.0, 255.0) as u8;
        }
    }

    // Interleaved UV plane (half resolution)
    let uv_base = w * h;
    for y in 0..(h / 2) {
        for x in 0..(w / 2) {
            let s = (y * 2 * w + x * 2) * 4;
            if s + 2 >= bgra.len() { break; }
            let b = bgra[s] as f32;
            let g = bgra[s + 1] as f32;
            let r = bgra[s + 2] as f32;
            let uv = uv_base + y * w + x * 2;
            nv12[uv] = (128.0 - 37.797 * r / 255.0 - 74.203 * g / 255.0
                + 112.0 * b / 255.0)
                .clamp(0.0, 255.0) as u8;
            nv12[uv + 1] = (128.0 + 112.0 * r / 255.0 - 93.786 * g / 255.0
                - 18.214 * b / 255.0)
                .clamp(0.0, 255.0) as u8;
        }
    }

    nv12
}

// ── AVCC parsing helpers ──────────────────────────────────────────────────────

/// Extract SPS (type 7) and PPS (type 8) NAL units from an AVCC byte stream.
pub fn extract_sps_pps(avcc: &[u8]) -> Option<(Vec<u8>, Vec<u8>)> {
    let mut sps: Option<Vec<u8>> = None;
    let mut pps: Option<Vec<u8>> = None;
    let mut pos = 0usize;
    while pos + 4 <= avcc.len() {
        let len =
            u32::from_be_bytes([avcc[pos], avcc[pos + 1], avcc[pos + 2], avcc[pos + 3]]) as usize;
        pos += 4;
        if len == 0 || pos + len > avcc.len() {
            break;
        }
        let nalu = &avcc[pos..pos + len];
        if !nalu.is_empty() {
            match nalu[0] & 0x1F {
                7 => sps = Some(nalu.to_vec()),
                8 => pps = Some(nalu.to_vec()),
                _ => {}
            }
        }
        pos += len;
    }
    match (sps, pps) {
        (Some(s), Some(p)) => Some((s, p)),
        _ => None,
    }
}

// ── Windows implementation ────────────────────────────────────────────────────

#[cfg(target_os = "windows")]
mod windows_impl {
    use super::*;
    use std::mem::ManuallyDrop;
    use windows::{
        Win32::Media::MediaFoundation::*,
        Win32::System::Com::{CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED},
    };

    fn pack_ratio(hi: u32, lo: u32) -> u64 {
        ((hi as u64) << 32) | (lo as u64)
    }

    unsafe fn create_h264_output_type(
        width: u32,
        height: u32,
        bitrate: u64,
    ) -> windows::core::Result<IMFMediaType> {
        let t = MFCreateMediaType()?;
        t.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video)?;
        t.SetGUID(&MF_MT_SUBTYPE, &MFVideoFormat_H264)?;
        t.SetUINT64(&MF_MT_FRAME_SIZE, pack_ratio(width, height))?;
        t.SetUINT64(&MF_MT_FRAME_RATE, pack_ratio(30, 1))?;
        t.SetUINT32(&MF_MT_AVG_BITRATE, bitrate as u32)?;
        t.SetUINT32(
            &MF_MT_INTERLACE_MODE,
            MFVideoInterlace_Progressive.0 as u32,
        )?;
        Ok(t)
    }

    unsafe fn create_nv12_input_type(
        width: u32,
        height: u32,
    ) -> windows::core::Result<IMFMediaType> {
        let t = MFCreateMediaType()?;
        t.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video)?;
        t.SetGUID(&MF_MT_SUBTYPE, &MFVideoFormat_NV12)?;
        t.SetUINT64(&MF_MT_FRAME_SIZE, pack_ratio(width, height))?;
        t.SetUINT64(&MF_MT_FRAME_RATE, pack_ratio(30, 1))?;
        t.SetUINT32(
            &MF_MT_INTERLACE_MODE,
            MFVideoInterlace_Progressive.0 as u32,
        )?;
        Ok(t)
    }

    unsafe fn feed_to_transform(
        transform: &IMFTransform,
        nv12: &[u8],
        pts_ms: i64,
    ) -> windows::core::Result<()> {
        let buf = MFCreateMemoryBuffer(nv12.len() as u32)?;
        {
            let mut ptr: *mut u8 = std::ptr::null_mut();
            buf.Lock(&mut ptr, None, None)?;
            std::ptr::copy_nonoverlapping(nv12.as_ptr(), ptr, nv12.len());
            buf.Unlock()?;
        }
        buf.SetCurrentLength(nv12.len() as u32)?;

        let sample = MFCreateSample()?;
        sample.AddBuffer(&buf)?;
        sample.SetSampleTime(pts_ms * 10_000)?; // ms → 100ns
        sample.SetSampleDuration(333_333)?;     // ~30 fps

        transform.ProcessInput(0, &sample, 0)?;
        Ok(())
    }

    unsafe fn drain_output(
        transform: &IMFTransform,
        buffer: &Arc<RollingBuffer>,
        sps_pps: &Arc<Mutex<Option<(Vec<u8>, Vec<u8>)>>>,
        provides_samples: bool,
        cbsize: u32,
    ) {
        loop {
            let presample: Option<IMFSample> = if provides_samples {
                None
            } else {
                let s = match MFCreateSample() {
                    Ok(s) => s,
                    Err(_) => break,
                };
                let b = match MFCreateMemoryBuffer(cbsize.max(65536)) {
                    Ok(b) => b,
                    Err(_) => break,
                };
                let _ = s.AddBuffer(&b);
                Some(s)
            };

            let mut out_buf = MFT_OUTPUT_DATA_BUFFER {
                dwStreamID: 0,
                pSample: ManuallyDrop::new(presample),
                dwStatus: 0,
                pEvents: ManuallyDrop::new(None),
            };
            let mut status: u32 = 0;
            let hr = transform.ProcessOutput(0, std::slice::from_mut(&mut out_buf), &mut status);

            // Always reclaim ownership before checking error
            let sample_opt = ManuallyDrop::into_inner(out_buf.pSample);
            drop(ManuallyDrop::into_inner(out_buf.pEvents));

            if hr.is_err() {
                break; // MF_E_TRANSFORM_NEED_MORE_INPUT or end of data
            }

            if let Some(sample) = sample_opt {
                let pts_ms = sample.GetSampleTime().unwrap_or(0) / 10_000;
                let is_keyframe =
                    sample.GetUINT32(&MFSampleExtension_CleanPoint).unwrap_or(0) != 0;

                if let Ok(total) = sample.GetTotalLength() {
                    if total > 0 {
                        let mut data = vec![0u8; total as usize];
                        if let Ok(cb) = sample.ConvertToContiguousBuffer() {
                            let mut ptr: *mut u8 = std::ptr::null_mut();
                            let mut cur: u32 = 0;
                            if cb.Lock(&mut ptr, None, Some(&mut cur)).is_ok() {
                                let len = cur as usize;
                                std::ptr::copy_nonoverlapping(ptr, data.as_mut_ptr(), len);
                                data.truncate(len);
                                let _ = cb.Unlock();
                            }
                        }

                        // Store SPS/PPS from first IDR frame
                        if is_keyframe && sps_pps.lock().is_none() {
                            if let Some(pair) = extract_sps_pps(&data) {
                                *sps_pps.lock() = Some(pair);
                            }
                        }

                        if !data.is_empty() {
                            buffer.push(EncodedPacket {
                                pts_ms,
                                is_keyframe,
                                data: Bytes::from(data),
                            });
                        }
                    }
                }
            }
        }
    }

    pub fn run(
        rx: Receiver<RawFrame>,
        buffer: Arc<RollingBuffer>,
        bitrate: u64,
        sps_pps: Arc<Mutex<Option<(Vec<u8>, Vec<u8>)>>>,
    ) {
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            if MFStartup(MF_VERSION, 1 /* MFSTARTUP_NOSOCKET */).is_err() {
                CoUninitialize();
                return;
            }
            run_inner(rx, buffer, bitrate, sps_pps);
            let _ = MFShutdown();
            CoUninitialize();
        }
    }

    unsafe fn run_inner(
        rx: Receiver<RawFrame>,
        buffer: Arc<RollingBuffer>,
        bitrate: u64,
        sps_pps: Arc<Mutex<Option<(Vec<u8>, Vec<u8>)>>>,
    ) {
        let first = match rx.recv() {
            Ok(f) => f,
            Err(_) => return,
        };
        let width = first.width;
        let height = first.height;

        // Create H.264 encoder MFT
        let transform: IMFTransform = match CoCreateInstance(
            &CLSID_MSH264EncoderMFT,
            None,
            CLSCTX_INPROC_SERVER,
        ) {
            Ok(t) => t,
            Err(e) => {
                log::error!("H.264 MFT create failed: {e}");
                return;
            }
        };

        // Set output type first, then input type
        let out_type = match create_h264_output_type(width, height, bitrate) {
            Ok(t) => t,
            Err(e) => { log::error!("H264 output type: {e}"); return; }
        };
        if let Err(e) = transform.SetOutputType(0, &out_type, 0) {
            log::error!("SetOutputType H264: {e}");
            return;
        }

        let in_type = match create_nv12_input_type(width, height) {
            Ok(t) => t,
            Err(e) => { log::error!("NV12 input type: {e}"); return; }
        };
        if let Err(e) = transform.SetInputType(0, &in_type, 0) {
            log::error!("SetInputType NV12: {e}");
            return;
        }

        let _ = transform.ProcessMessage(MFT_MESSAGE_NOTIFY_BEGIN_STREAMING, 0);
        let _ = transform.ProcessMessage(MFT_MESSAGE_NOTIFY_START_OF_STREAM, 0);

        // Check if MFT allocates its own output samples
        let stream_info = match transform.GetOutputStreamInfo(0) {
            Ok(info) => info,
            Err(_) => MFT_OUTPUT_STREAM_INFO::default(),
        };
        let provides =
            (stream_info.dwFlags & MFT_OUTPUT_STREAM_PROVIDES_SAMPLES.0 as u32) != 0;

        // Process first frame already in hand
        let nv12 = bgra_to_nv12(&first.data, first.width, first.height);
        let _ = feed_to_transform(&transform, &nv12, first.pts_ms);
        drain_output(&transform, &buffer, &sps_pps, provides, stream_info.cbSize);

        for frame in rx.into_iter() {
            let nv12 = bgra_to_nv12(&frame.data, frame.width, frame.height);
            let _ = feed_to_transform(&transform, &nv12, frame.pts_ms);
            drain_output(&transform, &buffer, &sps_pps, provides, stream_info.cbSize);
        }

        // Flush
        let _ = transform.ProcessMessage(MFT_MESSAGE_COMMAND_FLUSH, 0);
        drain_output(&transform, &buffer, &sps_pps, provides, stream_info.cbSize);
    }
}
