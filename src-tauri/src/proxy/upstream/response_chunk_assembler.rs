//! Upstream Non-Streaming Response Chunk Assembler
//! Assembles streaming byte chunks into complete response bodies with
//! strict maximum size guards to prevent memory exhaustion on oversized payloads.

#[derive(Debug, Clone)]
pub struct ResponseChunkAssembler {
    max_bytes: usize,
    buffer: Vec<u8>,
}

impl Default for ResponseChunkAssembler {
    fn default() -> Self {
        Self::new(32 * 1024 * 1024) // 32 MiB max
    }
}

impl ResponseChunkAssembler {
    pub fn new(max_bytes: usize) -> Self {
        Self {
            max_bytes,
            buffer: Vec::new(),
        }
    }

    /// Appends a new chunk of bytes to the internal buffer.
    /// Returns Err if the total accumulated size exceeds max_bytes.
    pub fn append_chunk(&mut self, chunk: &[u8]) -> Result<usize, &'static str> {
        if self.buffer.len() + chunk.len() > self.max_bytes {
            return Err("Response payload exceeds maximum allowed size buffer limit");
        }
        self.buffer.extend_from_slice(chunk);
        Ok(self.buffer.len())
    }

    /// Consumes the assembler and returns the collected bytes.
    pub fn finish(self) -> Vec<u8> {
        self.buffer
    }

    /// Returns current buffer size in bytes.
    pub fn current_size(&self) -> usize {
        self.buffer.len()
    }
}
