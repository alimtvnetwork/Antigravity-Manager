//! SSE Chunk Coalescer & Incomplete Frame Buffer
//!
//! Buffers streaming byte fragments from upstream SSE transports, ensures chunks are
//! sliced strictly along complete newline and event boundaries (`\n\n`), and preserves
//! incomplete trailing slices across stream chunk arrivals.

use bytes::{BufMut, Bytes, BytesMut};
use tracing::trace;

/// Manages incoming byte fragments and coalesces them into discrete, RFC-compliant SSE frames.
#[derive(Debug, Default)]
pub struct SseChunkCoalescer {
    /// Internal accumulator buffer for fragmented incoming bytes.
    buffer: BytesMut,
    /// Total bytes processed across the lifetime of the stream.
    total_bytes_processed: usize,
    /// Total complete frames emitted.
    total_frames_emitted: usize,
}

impl SseChunkCoalescer {
    /// Creates a new empty coalescer with default capacity (4KB).
    pub fn new() -> Self {
        Self {
            buffer: BytesMut::with_capacity(4096),
            total_bytes_processed: 0,
            total_frames_emitted: 0,
        }
    }

    /// Pushes a new incoming raw byte chunk into the accumulator and extracts all complete
    /// SSE frames demarcated by `\n\n` (or `\r\n\r\n`).
    pub fn push_chunk(&mut self, chunk: &[u8]) -> Vec<Bytes> {
        self.total_bytes_processed += chunk.len();
        self.buffer.put_slice(chunk);

        let mut frames = Vec::new();

        loop {
            // Find next event boundary delimiter: "\n\n" or "\r\n\r\n"
            if let Some((end_idx, delimiter_len)) = find_delimiter(&self.buffer) {
                let frame_len = end_idx + delimiter_len;
                let frame_bytes = self.buffer.split_to(frame_len).freeze();
                frames.push(frame_bytes);
                self.total_frames_emitted += 1;
            } else {
                break;
            }
        }

        trace!(
            new_frames = frames.len(),
            remaining_bytes = self.buffer.len(),
            "Coalesced SSE incoming chunk"
        );

        frames
    }

    /// Flushes any remaining bytes left in the internal buffer upon stream termination.
    pub fn finish(mut self) -> Option<Bytes> {
        if self.buffer.is_empty() {
            None
        } else {
            Some(self.buffer.freeze())
        }
    }

    /// Returns the number of currently buffered incomplete bytes.
    pub fn buffered_len(&self) -> usize {
        self.buffer.len()
    }

    /// Returns lifetime statistics: (bytes_processed, frames_emitted).
    pub fn stats(&self) -> (usize, usize) {
        (self.total_bytes_processed, self.total_frames_emitted)
    }
}

/// Helper to find either "\n\n" or "\r\n\r\n" in a byte slice.
fn find_delimiter(buf: &[u8]) -> Option<(usize, usize)> {
    let len = buf.len();
    for i in 0..len {
        if i + 1 < len && buf[i] == b'\n' && buf[i + 1] == b'\n' {
            return Some((i, 2));
        }
        if i + 3 < len && buf[i] == b'\r' && buf[i + 1] == b'\n' && buf[i + 2] == b'\r' && buf[i + 3] == b'\n' {
            return Some((i, 4));
        }
    }
    None
}
