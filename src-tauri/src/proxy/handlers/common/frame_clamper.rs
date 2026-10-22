//! Stream Frame Size Clamping Sentinel.
//! Clamps outgoing SSE chunks and message payloads within safe buffer bounds (default 64KB per frame).

/// Maximum safe payload size per SSE event frame: 64 KB.
pub const DEFAULT_MAX_FRAME_SIZE_BYTES: usize = 65536;

/// Splits a large payload chunk into bounded frame slices if exceeding maximum size limit.
pub fn clamp_stream_frame(data: &[u8], max_size: Option<usize>) -> Vec<Vec<u8>> {
    let limit = max_size.unwrap_or(DEFAULT_MAX_FRAME_SIZE_BYTES).max(1024);

    if data.len() <= limit {
        return vec![data.to_vec()];
    }

    let mut frames = Vec::new();
    let mut offset = 0;

    while offset < data.len() {
        let end = (offset + limit).min(data.len());
        frames.push(data[offset..end].to_vec());
        offset = end;
    }

    frames
}

/// Evaluates whether a payload frame exceeds the safe buffer threshold.
pub fn is_frame_oversized(data: &[u8], max_size: Option<usize>) -> bool {
    let limit = max_size.unwrap_or(DEFAULT_MAX_FRAME_SIZE_BYTES);
    data.len() > limit
}
