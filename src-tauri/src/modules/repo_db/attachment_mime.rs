//! Attachment MIME Type Detector & Sanitizer.
//! Inspects raw binary header magic bytes to verify genuine image formats.

/// Returns the detected image MIME type from binary magic bytes.
pub fn detect_image_mime(bytes: &[u8]) -> Option<&'static str> {
    if bytes.len() < 8 {
        return None;
    }

    // 1. PNG: 89 50 4E 47 0D 0A 1A 0A
    if bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
        return Some("image/png");
    }

    // 2. JPEG: FF D8 FF
    if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return Some("image/jpeg");
    }

    // 3. GIF: 47 49 46 38 (GIF87a / GIF89a)
    if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        return Some("image/gif");
    }

    // 4. WebP: RIFF .... WEBP
    if bytes.len() >= 12 && bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WEBP" {
        return Some("image/webp");
    }

    None
}

/// Checks if a MIME string is on the whitelisted supported image MIME list.
pub fn is_supported_image_mime(mime: &str) -> bool {
    let lower = mime.trim().to_ascii_lowercase();
    matches!(
        lower.as_str(),
        "image/png" | "image/jpeg" | "image/jpg" | "image/webp" | "image/gif"
    )
}
