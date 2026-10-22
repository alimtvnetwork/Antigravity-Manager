//! UTF-8 Stream Boundary Guard
//! Buffers incoming streaming byte chunks to prevent splitting multibyte UTF-8 codepoints
//! across frame boundaries, ensuring all downstream consumer parsers receive valid UTF-8 strings.

#[derive(Debug, Default)]
pub struct StreamBoundaryGuard {
    remainder: Vec<u8>,
}

impl StreamBoundaryGuard {
    pub fn new() -> Self {
        Self {
            remainder: Vec::with_capacity(4),
        }
    }

    /// Ingests a new slice of bytes, prepends any lingering trailing bytes,
    /// and extracts the maximal valid UTF-8 substring. Lingering partial bytes
    /// are retained for the subsequent call.
    pub fn process_chunk<'a>(&mut self, chunk: &'a [u8]) -> String {
        let mut combined = if self.remainder.is_empty() {
            chunk.to_vec()
        } else {
            let mut buf = std::mem::take(&mut self.remainder);
            buf.extend_from_slice(chunk);
            buf
        };

        // Determine valid UTF-8 prefix length
        match std::str::from_utf8(&combined) {
            Ok(valid_str) => {
                let res = valid_str.to_string();
                self.remainder.clear();
                res
            }
            Err(e) => {
                let valid_up_to = e.valid_up_to();
                let error_len = e.error_len();

                if error_len.is_none() {
                    // Incomplete multibyte sequence at the very end
                    let valid_slice = &combined[..valid_up_to];
                    let valid_str = String::from_utf8_lossy(valid_slice).into_owned();
                    self.remainder = combined[valid_up_to..].to_vec();
                    valid_str
                } else {
                    // Truly invalid byte sequence within the stream; replace lossily
                    let valid_str = String::from_utf8_lossy(&combined).into_owned();
                    self.remainder.clear();
                    valid_str
                }
            }
        }
    }

    /// Flushes any remaining bytes at the end of the stream.
    pub fn flush_remaining(&mut self) -> Option<String> {
        if self.remainder.is_empty() {
            None
        } else {
            let s = String::from_utf8_lossy(&self.remainder).into_owned();
            self.remainder.clear();
            Some(s)
        }
    }
}
