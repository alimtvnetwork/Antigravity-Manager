//! SSE Stream Preamble Buffer Padder
//!
//! Generates initial SSE comment frames (`: padding to bypass intermediate proxy buffers\n\n`)
//! that exceed intermediate proxy buffering ceilings (typically 1KB to 2KB in Nginx/Cloudflare),
//! ensuring that downstream clients receive the first stream chunk immediately without stalls.

use bytes::Bytes;
use tracing::debug;

/// Standard padding size in bytes (2048 bytes).
pub const DEFAULT_PREAMBLE_PADDING_BYTES: usize = 2048;

/// Generates an RFC-compliant SSE comment preamble frame exceeding the target size.
pub fn generate_sse_preamble_padding(target_bytes: Option<usize>) -> Bytes {
    let size = target_bytes.unwrap_or(DEFAULT_PREAMBLE_PADDING_BYTES).max(64);

    let mut buf = String::with_capacity(size + 16);
    buf.push_str(": open-stream-preamble-padding ");

    while buf.len() < size {
        buf.push_str("--------------------------------");
    }

    buf.push('\n');
    buf.push('\n');

    debug!(
        padding_bytes = buf.len(),
        "Generated SSE stream preamble padding frame"
    );

    Bytes::from(buf)
}
