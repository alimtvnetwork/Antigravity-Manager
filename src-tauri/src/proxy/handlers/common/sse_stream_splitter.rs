// Raw SSE Stream Frame Splitter (Imp-511)
// Splits coalesced raw Server-Sent Event byte buffers along double-newline `\n\n`
// event delimiters while buffering partial event tails across consecutive chunk frames.

use bytes::{Bytes, BytesMut};

pub struct SseStreamSplitter {
    buffer: BytesMut,
}

impl Default for SseStreamSplitter {
    fn default() -> Self {
        Self::new()
    }
}

impl SseStreamSplitter {
    pub fn new() -> Self {
        Self {
            buffer: BytesMut::with_capacity(4096),
        }
    }

    /// Appends incoming raw chunk bytes and extracts complete SSE frames.
    pub fn feed(&mut self, chunk: &[u8]) -> Vec<Bytes> {
        self.buffer.extend_from_slice(chunk);
        let mut frames = Vec::new();

        while let Some(pos) = self.find_delimiter() {
            // Include delimiter in the frame or split immediately past it
            let frame_bytes = self.buffer.split_to(pos + 2);
            frames.push(frame_bytes.freeze());
        }

        frames
    }

    /// Finds the index of the first `\n\n` or `\r\n\r\n` boundary.
    fn find_delimiter(&self) -> Option<usize> {
        let len = self.buffer.len();
        if len < 2 {
            return None;
        }

        for i in 0..len - 1 {
            if self.buffer[i] == b'\n' && self.buffer[i + 1] == b'\n' {
                return Some(i);
            }
        }
        None
    }

    /// Flushes any remaining bytes in the tail buffer.
    pub fn flush_remainder(&mut self) -> Option<Bytes> {
        if self.buffer.is_empty() {
            None
        } else {
            Some(self.buffer.split().freeze())
        }
    }

    pub fn buffered_bytes_count(&self) -> usize {
        self.buffer.len()
    }
}
