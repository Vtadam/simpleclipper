//! Audio encoder bridge.
//!
//! Receives raw PCM f32 chunks from WASAPI, converts to i16, and stores them
//! in the AudioRollingBuffer as raw PCM packets.  The actual AAC encoding
//! happens in the clip saver via IMFSinkWriter, which encodes PCM → AAC during
//! the mux step — eliminating the need for a separate AAC encoder here.

use super::audio::{AudioChunk, AudioPacket, AudioRollingBuffer};
use bytes::Bytes;
use crossbeam_channel::Receiver;
use std::sync::Arc;

pub fn audio_encode_loop(rx: Receiver<AudioChunk>, buffer: Arc<AudioRollingBuffer>) {
    for chunk in rx.into_iter() {
        // Convert float32 → int16 PCM
        let i16_bytes: Vec<u8> = chunk
            .data
            .iter()
            .flat_map(|&s| {
                let v = (s.clamp(-1.0, 1.0) * 32767.0) as i16;
                v.to_le_bytes()
            })
            .collect();

        if !i16_bytes.is_empty() {
            buffer.push(AudioPacket {
                pts_ms: chunk.pts_ms,
                data: Bytes::from(i16_bytes),
                sample_rate: chunk.sample_rate,
                channels: chunk.channels,
            });
        }
    }
}
