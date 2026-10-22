//! Downstream SSE Framing Validator.
//! Validates and normalizes SSE blocks to ensure compliance with the W3C EventSource specification (`\n\n` delimiter).

/// Validates that an SSE chunk ends with the standard double newline delimiter.
pub fn is_valid_sse_frame(data: &[u8]) -> bool {
    data.ends_with(b"\n\n") || data.ends_with(b"\r\n\r\n")
}

/// Normalizes outgoing SSE frame bytes, guaranteeing trailing double newlines to prevent downstream parser stalls.
pub fn ensure_sse_framing(data: &[u8]) -> Vec<u8> {
    if data.is_empty() {
        return Vec::new();
    }

    let mut result = data.to_vec();

    if result.ends_with(b"\n\n") || result.ends_with(b"\r\n\r\n") {
        return result;
    }

    if result.ends_with(b"\n") {
        result.push(b'\n');
    } else {
        result.extend_from_slice(b"\n\n");
    }

    result
}
