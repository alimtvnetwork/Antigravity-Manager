//! Stream Frame Length Bounding Clamper.
//! Detects and bounds excessively large individual SSE frame payloads,
//! splitting them across sequential data frames to prevent client TCP buffer overflows.

/// Maximum allowable bytes in a single SSE event frame.
pub const MAX_SSE_FRAME_BYTES: usize = 1024 * 1024; // 1 MB per frame

/// Clamps an individual payload string if it exceeds `max_frame_bytes`.
pub fn clamp_sse_frame(payload: &str, max_bytes: Option<usize>) -> Vec<String> {
    let limit = max_bytes.unwrap_or(MAX_SSE_FRAME_BYTES);
    if payload.len() <= limit {
        return vec![payload.to_string()];
    }

    let mut chunks = Vec::new();
    let bytes = payload.as_bytes();
    let mut offset = 0;

    while offset < bytes.len() {
        let mut end = (offset + limit).min(bytes.len());

        // Avoid splitting in the middle of a UTF-8 character sequence
        while end > offset && !is_char_boundary(bytes, end) {
            end -= 1;
        }

        if end == offset {
            // Fallback in case of degenerate byte sequence
            end = (offset + limit).min(bytes.len());
        }

        let slice = &bytes[offset..end];
        chunks.push(String::from_utf8_lossy(slice).to_string());
        offset = end;
    }

    chunks
}

#[inline]
fn is_char_boundary(bytes: &[u8], index: usize) -> bool {
    if index >= bytes.len() {
        return true;
    }
    // In UTF-8, continuation bytes have top bits 10xxxxxx
    (bytes[index] & 0xC0) != 0x80
}
