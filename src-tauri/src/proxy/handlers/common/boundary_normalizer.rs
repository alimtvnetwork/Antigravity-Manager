//! SSE Stream Chunk Boundary Normalizer.
//! Reconstructs complete SSE event messages from arbitrarily fragmented network byte chunks.

#[derive(Debug, Default)]
pub struct StreamBoundaryNormalizer {
    buffer: String,
}

impl StreamBoundaryNormalizer {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
        }
    }

    /// Feeds a new chunk of raw string data and extracts all fully formed SSE event frames.
    pub fn feed_chunk(&mut self, chunk: &str) -> Vec<String> {
        self.buffer.push_str(chunk);

        let mut events = Vec::new();

        // Standard SSE event boundary is double newline: \n\n or \r\n\r\n
        while let Some(pos) = self.buffer.find("\n\n") {
            let event_raw = self.buffer[..pos].to_string();
            self.buffer = self.buffer[pos + 2..].to_string();
            let trimmed = event_raw.trim();
            if !trimmed.is_empty() {
                events.push(trimmed.to_string());
            }
        }

        events
    }

    /// Flushes any remaining non-empty data in the buffer.
    pub fn flush_remaining(&mut self) -> Option<String> {
        let trimmed = self.buffer.trim().to_string();
        self.buffer.clear();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    }
}
