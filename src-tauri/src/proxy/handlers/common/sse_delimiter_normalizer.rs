//! Server-Sent Events (SSE) Delimiter Normalizer.
//! Ensures strict protocol conformance for SSE event frames by normalizing
//! mixed delimiters (`\r\n\r\n`, `\r\r`, `\n\n`) into canonical `\n\n`.

/// Normalizes SSE framing delimiters in raw string or byte slices.
#[derive(Debug, Default)]
pub struct SseDelimiterNormalizer {
    remainder: Vec<u8>,
}

impl SseDelimiterNormalizer {
    /// Creates a new delimiter normalizer instance with empty carryover buffer.
    pub fn new() -> Self {
        Self {
            remainder: Vec::new(),
        }
    }

    /// Feeds a chunk of incoming raw SSE bytes, returning a vector of clean,
    /// complete SSE event frames delimited by double newlines.
    pub fn feed(&mut self, chunk: &[u8]) -> Vec<String> {
        let mut buffer = Vec::with_capacity(self.remainder.len() + chunk.len());
        buffer.extend_from_slice(&self.remainder);
        buffer.extend_from_slice(chunk);
        self.remainder.clear();

        // Convert CRLF into LF for normalized scanning
        let text = String::from_utf8_lossy(&buffer);
        let normalized = text.replace("\r\n", "\n").replace('\r', "\n");

        let mut events = Vec::new();
        let parts: Vec<&str> = normalized.split("\n\n").collect();

        if parts.is_empty() {
            return events;
        }

        // All parts except the last one are guaranteed complete events
        for part in &parts[..parts.len() - 1] {
            let trimmed = part.trim();
            if !trimmed.is_empty() {
                events.push(format!("{}\n\n", trimmed));
            }
        }

        // If the chunk ended exactly with a delimiter, the last part is empty
        if normalized.ends_with("\n\n") {
            let last = parts[parts.len() - 1].trim();
            if !last.is_empty() {
                events.push(format!("{}\n\n", last));
            }
        } else {
            // Otherwise, save uncompleted trailing fragment for next feed
            self.remainder = parts[parts.len() - 1].as_bytes().to_vec();
        }

        events
    }

    /// Flushes any pending un-delimited bytes as a final event frame if non-empty.
    pub fn flush(&mut self) -> Option<String> {
        if self.remainder.is_empty() {
            return None;
        }

        let text = String::from_utf8_lossy(&self.remainder).to_string();
        self.remainder.clear();
        let trimmed = text.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(format!("{}\n\n", trimmed))
        }
    }

    /// Checks if there is un-flushed trailing data in the normalizer.
    pub fn has_pending_bytes(&self) -> bool {
        !self.remainder.is_empty()
    }
}

/// Helper function to normalize an entire static SSE payload in one pass.
pub fn normalize_sse_payload(payload: &str) -> String {
    let normalized = payload.replace("\r\n", "\n").replace('\r', "\n");
    let mut result = String::with_capacity(normalized.len());

    for event in normalized.split("\n\n") {
        let trimmed = event.trim();
        if !trimmed.is_empty() {
            result.push_str(trimmed);
            result.push_str("\n\n");
        }
    }

    result
}
