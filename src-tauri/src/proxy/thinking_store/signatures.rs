const MIN_SIGNATURE_LENGTH: usize = 32;

use super::*;

pub fn is_real_signature(sig: &str) -> bool {
    let s = sig.trim();
    if s == SENTINEL_SIGNATURE {
        return false;
    }
    if s.as_bytes().first() == Some(&0x12) && s.len() >= 16 {
        return true;
    }
    s.len() >= MIN_SIGNATURE_LENGTH
}

/// Determine whether the signature conforms to the Google Gemini native Protobuf signature characteristics:
/// 1. Official skip signature verification sentinel (skip_thought_signature_validator);
/// 2. Or meet the valid length (>= MIN_SIGNATURE_LENGTH), and the first byte after Base64 decoding is Protobuf Tag 2 (0x12)
///    (Single-layer Base64 usually starts with 'E', double-layer Base64 wrapper usually starts with 'R')
pub fn is_likely_gemini_signature(sig: &str) -> bool {
    let s = sig.trim();
    if s == SENTINEL_SIGNATURE {
        return true;
    }
    // A Claude signature must never be mistaken for a Gemini signature
    if is_claude_signature(s) {
        return false;
    }
    // Compatible with signatures misdecoded as raw binary Protobuf (first byte 0x12) in historical dirty data
    if s.as_bytes().first() == Some(&0x12) && s.len() >= 16 {
        return true;
    }
    if s.len() < MIN_SIGNATURE_LENGTH {
        return false;
    }
    if !s.starts_with('E') && !s.starts_with('R') {
        return false;
    }
    use base64::Engine;
    if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(s) {
        if decoded.first() == Some(&0x12) {
            return true;
        }
        // Dual-layer Base64 wrapping support (Google Vertex AI format)
        if let Ok(inner_str) = std::str::from_utf8(&decoded) {
            if inner_str.starts_with('E') {
                if let Ok(inner) = base64::engine::general_purpose::STANDARD.decode(inner_str) {
                    if inner.first() == Some(&0x12) {
                        return true;
                    }
                }
            }
        }
    }
    false
}

/// Determine whether the signature belongs to the Claude family
pub fn is_claude_signature(sig: &str) -> bool {
    let s = sig.trim();
    if s.is_empty() || s == SENTINEL_SIGNATURE {
        return false;
    }
    use base64::Engine;
    if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(s) {
        if decoded
            .windows(6)
            .any(|w| w.eq_ignore_ascii_case(b"claude"))
        {
            return true;
        }
        if let Ok(inner) = base64::engine::general_purpose::STANDARD.decode(&decoded) {
            if inner.windows(6).any(|w| w.eq_ignore_ascii_case(b"claude")) {
                return true;
            }
        }
    }
    false
}

/// Normalize Claude signatures into the format required for sending to the Google Vertex AI interface
/// Google's REST API automatically performs base64_decode on bytes fields.
/// Therefore the thoughtSignature sent to Google must be the Base64 encoding of the ASCII signature bytes (i.e. "RXU4..." format)
pub fn ensure_google_claude_thought_signature(sig: &str) -> String {
    let s = sig.trim();
    if s.is_empty() || s == SENTINEL_SIGNATURE {
        return s.to_string();
    }
    use base64::Engine;
    // If it has been packaged by Base64 (that is, b"claude" can be decoded after base64 decoding), there is no need to package it again.
    if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(s) {
        if let Ok(inner) = base64::engine::general_purpose::STANDARD.decode(&decoded) {
            if inner.windows(6).any(|w| w.eq_ignore_ascii_case(b"claude")) {
                return s.to_string();
            }
        }
    }
    // Only perform a Base64 wrap if the current signature is indeed the original Claude client signature (containing b"claude" after one layer of decoding)!
    // Uncontrolled repackaging of non-Claude signatures or unknown strings is strictly prohibited, completely blocking the infinite loop of geometric expansion.
    if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(s) {
        if decoded
            .windows(6)
            .any(|w| w.eq_ignore_ascii_case(b"claude"))
        {
            return base64::engine::general_purpose::STANDARD.encode(s.as_bytes());
        }
    }
    s.to_string()
}

/// Restore Claude signatures to the native format (Eu8...) expected by the client (Claude Code / Anthropic SDK)
pub fn ensure_raw_claude_thought_signature(sig: &str) -> String {
    let s = sig.trim();
    if s.is_empty() || s == SENTINEL_SIGNATURE {
        return s.to_string();
    }
    use base64::Engine;
    if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(s) {
        if let Ok(inner) = base64::engine::general_purpose::STANDARD.decode(&decoded) {
            if inner.windows(6).any(|w| w.eq_ignore_ascii_case(b"claude")) {
                if let Ok(raw_s) = String::from_utf8(decoded) {
                    return raw_s;
                }
            }
        }
    }
    s.to_string()
}

/// Used for internal comparison and hashing of ThinkingStore / SignatureCache:
/// Unified and normalized to the original client signature form (Eu8...), so that "RXU4..." and "Eu8..." are determined to be the same signature
pub fn normalize_signature_for_comparison(sig: &str) -> std::borrow::Cow<'_, str> {
    use base64::Engine;
    if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(sig) {
        if let Ok(inner) = base64::engine::general_purpose::STANDARD.decode(&decoded) {
            if inner.windows(6).any(|w| w == b"claude") {
                if let Ok(s) = String::from_utf8(decoded) {
                    return std::borrow::Cow::Owned(s);
                }
            }
        }
    }
    std::borrow::Cow::Borrowed(sig)
}

pub fn signatures_match(a: &str, b: &str) -> bool {
    if a == b {
        return true;
    }
    normalize_signature_for_comparison(a) == normalize_signature_for_comparison(b)
}
