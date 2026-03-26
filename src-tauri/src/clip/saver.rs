//! Clip saver: drains the rolling buffers and muxes an MP4 file via
//! Windows Media Foundation IMFSinkWriter.
//!
//! Video packets (H.264 AVCC) are written as passthrough.
//! Audio packets (i16 PCM) are encoded to AAC by the sink writer's built-in
//! AAC MFT, so no external encoder is required.

use crate::capture::audio::{AudioPacket, AudioRollingBuffer};
use crate::capture::buffer::{EncodedPacket, RollingBuffer};
use anyhow::Result;
use chrono::Local;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub struct SavedClip {
    pub path: PathBuf,
    pub duration_secs: u32,
    pub size_bytes: u64,
}

pub async fn save_clip(
    video_buffer: Arc<RollingBuffer>,
    audio_buffer: Arc<AudioRollingBuffer>,
    save_folder: &Path,
    duration_secs: u32,
    video_width: u32,
    video_height: u32,
    sps_pps: Option<(Vec<u8>, Vec<u8>)>,
) -> Result<SavedClip> {
    let duration_ms = duration_secs as i64 * 1000;
    let video_packets = video_buffer.drain_last_ms(duration_ms);
    let audio_packets = audio_buffer.drain_last_ms(duration_ms);

    if video_packets.is_empty() {
        anyhow::bail!("No video data in buffer yet. Wait a moment for the buffer to fill.");
    }

    std::fs::create_dir_all(save_folder)?;

    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let filename = format!("clip_{}_{}s.mp4", timestamp, duration_secs);
    let output_path = save_folder.join(&filename);

    let output_path_clone = output_path.clone();
    tokio::task::spawn_blocking(move || {
        mux_to_mp4(
            &video_packets,
            &audio_packets,
            &output_path_clone,
            video_width,
            video_height,
            sps_pps,
        )
    })
    .await??;

    let size_bytes = std::fs::metadata(&output_path)?.len();

    Ok(SavedClip {
        path: output_path,
        duration_secs,
        size_bytes,
    })
}

// ── Platform implementations ─────────────────────────────────────────────────

fn mux_to_mp4(
    video_packets: &[EncodedPacket],
    audio_packets: &[AudioPacket],
    output_path: &Path,
    video_width: u32,
    video_height: u32,
    sps_pps: Option<(Vec<u8>, Vec<u8>)>,
) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        windows_impl::mux(
            video_packets,
            audio_packets,
            output_path,
            video_width,
            video_height,
            sps_pps,
        )
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = (video_packets, audio_packets, output_path, video_width, video_height, sps_pps);
        Err(anyhow::anyhow!("MP4 muxing only supported on Windows"))
    }
}

// ── Windows: IMFSinkWriter ────────────────────────────────────────────────────

#[cfg(target_os = "windows")]
mod windows_impl {
    use super::*;
    use windows::{
        Win32::Media::MediaFoundation::*,
        Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_MULTITHREADED},
        core::PCWSTR,
    };

    fn pack_ratio(hi: u32, lo: u32) -> u64 {
        ((hi as u64) << 32) | (lo as u64)
    }

    /// Build an AVCDecoderConfigurationRecord blob for MF_MT_MPEG_SEQUENCE_HEADER.
    fn make_avcc_config(sps: &[u8], pps: &[u8]) -> Vec<u8> {
        if sps.len() < 4 { return vec![]; }
        let mut v = Vec::new();
        v.push(0x01);        // configurationVersion
        v.push(sps[1]);      // AVCProfileIndication
        v.push(sps[2]);      // profile_compatibility
        v.push(sps[3]);      // AVCLevelIndication
        v.push(0xFF);        // 0b11111100 | (4-1)  — 4-byte NAL length fields
        v.push(0xE1);        // 0b11100000 | 1 SPS
        v.extend_from_slice(&(sps.len() as u16).to_be_bytes());
        v.extend_from_slice(sps);
        v.push(0x01);        // 1 PPS
        v.extend_from_slice(&(pps.len() as u16).to_be_bytes());
        v.extend_from_slice(pps);
        v
    }

    /// Strip SPS (type 7) and PPS (type 8) NAL units from AVCC data.
    /// The avcC box in the MP4 already carries them; duplicating them in
    /// samples confuses some players.
    fn strip_param_sets(avcc: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        let mut pos = 0usize;
        while pos + 4 <= avcc.len() {
            let len = u32::from_be_bytes([avcc[pos], avcc[pos+1], avcc[pos+2], avcc[pos+3]]) as usize;
            pos += 4;
            if len == 0 || pos + len > avcc.len() { break; }
            let nalu = &avcc[pos..pos+len];
            if !nalu.is_empty() {
                let t = nalu[0] & 0x1F;
                if t != 7 && t != 8 {
                    out.extend_from_slice(&(len as u32).to_be_bytes());
                    out.extend_from_slice(nalu);
                }
            }
            pos += len;
        }
        out
    }

    unsafe fn create_video_media_type(
        width: u32,
        height: u32,
        avcc_config: &[u8],
    ) -> windows::core::Result<IMFMediaType> {
        let t = MFCreateMediaType()?;
        t.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video)?;
        t.SetGUID(&MF_MT_SUBTYPE, &MFVideoFormat_H264)?;
        t.SetUINT64(&MF_MT_FRAME_SIZE, pack_ratio(width, height))?;
        t.SetUINT64(&MF_MT_FRAME_RATE, pack_ratio(30, 1))?;
        t.SetUINT32(&MF_MT_AVG_BITRATE, 8_000_000)?;
        t.SetUINT32(&MF_MT_INTERLACE_MODE, MFVideoInterlace_Progressive.0 as u32)?;
        if !avcc_config.is_empty() {
            t.SetBlob(&MF_MT_MPEG_SEQUENCE_HEADER, avcc_config)?;
        }
        Ok(t)
    }

    unsafe fn create_audio_out_type(
        sample_rate: u32,
        channels: u16,
    ) -> windows::core::Result<IMFMediaType> {
        let t = MFCreateMediaType()?;
        t.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Audio)?;
        t.SetGUID(&MF_MT_SUBTYPE, &MFAudioFormat_AAC)?;
        t.SetUINT32(&MF_MT_AUDIO_SAMPLES_PER_SECOND, sample_rate)?;
        t.SetUINT32(&MF_MT_AUDIO_NUM_CHANNELS, channels as u32)?;
        t.SetUINT32(&MF_MT_AUDIO_AVG_BYTES_PER_SECOND, 16_000)?; // 128 kbps
        t.SetUINT32(&MF_MT_AUDIO_BITS_PER_SAMPLE, 16)?;
        Ok(t)
    }

    unsafe fn create_audio_in_type(
        sample_rate: u32,
        channels: u16,
    ) -> windows::core::Result<IMFMediaType> {
        let t = MFCreateMediaType()?;
        t.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Audio)?;
        t.SetGUID(&MF_MT_SUBTYPE, &MFAudioFormat_PCM)?;
        t.SetUINT32(&MF_MT_AUDIO_SAMPLES_PER_SECOND, sample_rate)?;
        t.SetUINT32(&MF_MT_AUDIO_NUM_CHANNELS, channels as u32)?;
        t.SetUINT32(&MF_MT_AUDIO_BITS_PER_SAMPLE, 16)?;
        t.SetUINT32(&MF_MT_AUDIO_BLOCK_ALIGNMENT, channels as u32 * 2)?;
        t.SetUINT32(&MF_MT_AUDIO_AVG_BYTES_PER_SECOND, sample_rate * channels as u32 * 2)?;
        Ok(t)
    }

    unsafe fn make_video_sample(
        ep: &EncodedPacket,
        pts_offset: i64,
    ) -> windows::core::Result<IMFSample> {
        let data = strip_param_sets(&ep.data);
        if data.is_empty() { return Err(windows::core::Error::from_hresult(windows::Win32::Foundation::E_FAIL)); }

        let buf = MFCreateMemoryBuffer(data.len() as u32)?;
        {
            let mut ptr: *mut u8 = std::ptr::null_mut();
            buf.Lock(&mut ptr, None, None)?;
            std::ptr::copy_nonoverlapping(data.as_ptr(), ptr, data.len());
            buf.Unlock()?;
        }
        buf.SetCurrentLength(data.len() as u32)?;

        let sample = MFCreateSample()?;
        sample.AddBuffer(&buf)?;
        let pts_100ns = (ep.pts_ms - pts_offset) * 10_000;
        sample.SetSampleTime(pts_100ns)?;
        sample.SetSampleDuration(333_333)?; // ~30 fps

        if ep.is_keyframe {
            sample.SetUINT32(&MFSampleExtension_CleanPoint, 1)?;
        }
        Ok(sample)
    }

    unsafe fn make_audio_sample(
        ap: &AudioPacket,
        pts_offset: i64,
    ) -> windows::core::Result<IMFSample> {
        let buf = MFCreateMemoryBuffer(ap.data.len() as u32)?;
        {
            let mut ptr: *mut u8 = std::ptr::null_mut();
            buf.Lock(&mut ptr, None, None)?;
            std::ptr::copy_nonoverlapping(ap.data.as_ptr(), ptr, ap.data.len());
            buf.Unlock()?;
        }
        buf.SetCurrentLength(ap.data.len() as u32)?;

        let sample = MFCreateSample()?;
        sample.AddBuffer(&buf)?;
        let pts_100ns = (ap.pts_ms - pts_offset) * 10_000;
        sample.SetSampleTime(pts_100ns.max(0))?;

        // Duration in 100ns from sample count (bytes / (channels*2) / sample_rate)
        let samples_per_channel = ap.data.len() as u64 / (ap.channels as u64 * 2);
        let dur_100ns = samples_per_channel * 10_000_000 / ap.sample_rate as u64;
        sample.SetSampleDuration(dur_100ns as i64)?;
        Ok(sample)
    }

    pub fn mux(
        video_packets: &[EncodedPacket],
        audio_packets: &[AudioPacket],
        output_path: &Path,
        video_width: u32,
        video_height: u32,
        sps_pps: Option<(Vec<u8>, Vec<u8>)>,
    ) -> Result<()> {
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            let result = mux_inner(
                video_packets,
                audio_packets,
                output_path,
                video_width,
                video_height,
                sps_pps,
            );
            CoUninitialize();
            result
        }
    }

    unsafe fn mux_inner(
        video_packets: &[EncodedPacket],
        audio_packets: &[AudioPacket],
        output_path: &Path,
        video_width: u32,
        video_height: u32,
        sps_pps: Option<(Vec<u8>, Vec<u8>)>,
    ) -> Result<()> {
        MFStartup(MF_VERSION, 1)?;

        let path_str = output_path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Non-UTF-8 path"))?;
        let wide: Vec<u16> = path_str
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        let sink_writer = MFCreateSinkWriterFromURL(PCWSTR(wide.as_ptr()), None, None)
            .map_err(|e| anyhow::anyhow!("MFCreateSinkWriterFromURL: {e}"))?;

        // ── Video stream ────────────────────────────────────────────────────
        let avcc_config = sps_pps
            .as_ref()
            .map(|(sps, pps)| make_avcc_config(sps, pps))
            .unwrap_or_default();

        let video_type = create_video_media_type(video_width, video_height, &avcc_config)
            .map_err(|e| anyhow::anyhow!("video media type: {e}"))?;

        let video_idx = sink_writer
            .AddStream(&video_type)
            .map_err(|e| anyhow::anyhow!("AddStream video: {e}"))?;

        // Passthrough: input type = output type (same H.264)
        sink_writer
            .SetInputMediaType(video_idx, &video_type, None)
            .map_err(|e| anyhow::anyhow!("SetInputMediaType video: {e}"))?;

        // ── Audio stream (optional) ──────────────────────────────────────────
        let audio_stream_idx: Option<u32> = if let Some(ap) = audio_packets.first() {
            let audio_out = create_audio_out_type(ap.sample_rate, ap.channels)
                .map_err(|e| anyhow::anyhow!("audio out type: {e}"))?;
            let idx = sink_writer
                .AddStream(&audio_out)
                .map_err(|e| anyhow::anyhow!("AddStream audio: {e}"))?;

            let audio_in = create_audio_in_type(ap.sample_rate, ap.channels)
                .map_err(|e| anyhow::anyhow!("audio in type: {e}"))?;
            sink_writer
                .SetInputMediaType(idx, &audio_in, None)
                .map_err(|e| anyhow::anyhow!("SetInputMediaType audio: {e}"))?;

            Some(idx)
        } else {
            None
        };

        sink_writer
            .BeginWriting()
            .map_err(|e| anyhow::anyhow!("BeginWriting: {e}"))?;

        // ── Interleaved write ────────────────────────────────────────────────
        let v_offset = video_packets.first().map(|p| p.pts_ms).unwrap_or(0);
        let a_offset = audio_packets.first().map(|p| p.pts_ms).unwrap_or(0);

        let mut vi = 0usize;
        let mut ai = 0usize;

        loop {
            let vp = video_packets.get(vi);
            let ap = audio_stream_idx.and_then(|_| audio_packets.get(ai));

            match (vp, ap) {
                (None, None) => break,
                (Some(vp), None) => {
                    if let Ok(s) = make_video_sample(vp, v_offset) {
                        let _ = sink_writer.WriteSample(video_idx, &s);
                    }
                    vi += 1;
                }
                (None, Some(ap)) => {
                    if let Some(idx) = audio_stream_idx {
                        if let Ok(s) = make_audio_sample(ap, a_offset) {
                            let _ = sink_writer.WriteSample(idx, &s);
                        }
                    }
                    ai += 1;
                }
                (Some(vp), Some(ap)) => {
                    let vt = vp.pts_ms - v_offset;
                    let at = ap.pts_ms - a_offset;
                    if vt <= at {
                        if let Ok(s) = make_video_sample(vp, v_offset) {
                            let _ = sink_writer.WriteSample(video_idx, &s);
                        }
                        vi += 1;
                    } else {
                        if let Some(idx) = audio_stream_idx {
                            if let Ok(s) = make_audio_sample(ap, a_offset) {
                                let _ = sink_writer.WriteSample(idx, &s);
                            }
                        }
                        ai += 1;
                    }
                }
            }
        }

        sink_writer
            .Finalize()
            .map_err(|e| anyhow::anyhow!("Finalize: {e}"))?;

        let _ = MFShutdown();
        Ok(())
    }
}
