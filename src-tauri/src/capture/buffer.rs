use bytes::Bytes;
use parking_lot::Mutex;
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct EncodedPacket {
    /// Presentation timestamp in milliseconds
    pub pts_ms: i64,
    /// Is this an IDR (keyframe) packet?
    pub is_keyframe: bool,
    /// Encoded H.264 NAL unit bytes
    pub data: Bytes,
}

pub struct RollingBuffer {
    inner: Mutex<BufferInner>,
    max_bytes: usize,
}

struct BufferInner {
    packets: VecDeque<EncodedPacket>,
    total_bytes: usize,
}

impl RollingBuffer {
    pub fn new(max_bytes: usize) -> Self {
        Self {
            inner: Mutex::new(BufferInner {
                packets: VecDeque::new(),
                total_bytes: 0,
            }),
            max_bytes,
        }
    }

    /// Push a new encoded packet, evicting oldest if over cap.
    pub fn push(&self, packet: EncodedPacket) {
        let mut inner = self.inner.lock();
        let packet_size = packet.data.len();
        inner.packets.push_back(packet);
        inner.total_bytes += packet_size;

        // Evict oldest until under cap
        while inner.total_bytes > self.max_bytes {
            if let Some(evicted) = inner.packets.pop_front() {
                inner.total_bytes = inner.total_bytes.saturating_sub(evicted.data.len());
            } else {
                break;
            }
        }
    }

    /// Drain the last `duration_ms` worth of packets.
    /// Always snaps start to the nearest IDR (keyframe) at or before the requested start.
    pub fn drain_last_ms(&self, duration_ms: i64) -> Vec<EncodedPacket> {
        let inner = self.inner.lock();
        if inner.packets.is_empty() {
            return vec![];
        }

        let latest_pts = inner.packets.back().map(|p| p.pts_ms).unwrap_or(0);
        let target_start_pts = latest_pts - duration_ms;

        // Find index of first packet at or after target_start_pts
        let start_idx = inner
            .packets
            .iter()
            .position(|p| p.pts_ms >= target_start_pts)
            .unwrap_or(0);

        // Walk backwards from start_idx to find the nearest IDR frame
        let idr_idx = inner.packets.iter().enumerate().rev()
            .skip(inner.packets.len().saturating_sub(start_idx + 1))
            .find(|(_, p)| p.is_keyframe)
            .map(|(i, _)| i)
            .unwrap_or(start_idx);

        inner.packets.iter().skip(idr_idx).cloned().collect()
    }

    /// Returns (total_packets, total_bytes, duration_ms_approx)
    pub fn stats(&self) -> (usize, usize, i64) {
        let inner = self.inner.lock();
        let duration = if let (Some(first), Some(last)) = (
            inner.packets.front(),
            inner.packets.back(),
        ) {
            last.pts_ms - first.pts_ms
        } else {
            0
        };
        (inner.packets.len(), inner.total_bytes, duration)
    }

    pub fn clear(&self) {
        let mut inner = self.inner.lock();
        inner.packets.clear();
        inner.total_bytes = 0;
    }
}
