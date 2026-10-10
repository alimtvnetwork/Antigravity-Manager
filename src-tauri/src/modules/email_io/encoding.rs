use base64::engine::general_purpose::STANDARD;

use super::*;

pub fn base64_encode_multi(data: &str, passes: usize) -> String {
    let mut current = data.as_bytes().to_vec();
    let num_passes = passes.max(1);
    for _ in 0..num_passes {
        let encoded = STANDARD.encode(&current);
        current = encoded.into_bytes();
    }
    String::from_utf8(current).unwrap_or_default()
}

/// Multi-pass Base64 decoder for restoring multi-encoded sensitive payloads
pub fn base64_decode_multi(encoded_str: &str, passes: usize) -> Result<String, String> {
    let mut current_bytes = encoded_str.as_bytes().to_vec();
    let num_passes = passes.max(1);
    for pass in 0..num_passes {
        let text = String::from_utf8(current_bytes.clone())
            .map_err(|e| format!("Pass {} utf8 conversion error: {}", pass, e))?;
        current_bytes = STANDARD
            .decode(text.trim())
            .map_err(|e| format!("Pass {} base64 decode error: {}", pass, e))?;
    }
    String::from_utf8(current_bytes).map_err(|e| format!("Final utf8 decode error: {}", e))
}
