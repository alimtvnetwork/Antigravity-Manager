//! Multimodal Image Attachment Hash Resolver.
//! Computes SHA-256 fingerprints to deduplicate identical prompt attachments.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Result};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AttachmentFingerprint {
    pub sha256_hex: String,
    pub byte_size: usize,
    pub mime_type: Option<String>,
}

/// Computes a SHA-256 fingerprint for in-memory image bytes.
pub fn compute_bytes_fingerprint(bytes: &[u8], mime_type: Option<&str>) -> AttachmentFingerprint {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let hash_result = hasher.finalize();
    let sha256_hex = hex::encode(hash_result);

    AttachmentFingerprint {
        sha256_hex,
        byte_size: bytes.len(),
        mime_type: mime_type.map(|s| s.to_string()),
    }
}

/// Computes a SHA-256 fingerprint directly from a filesystem file.
pub fn compute_file_fingerprint(path: &Path) -> Result<AttachmentFingerprint> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    let mut total_bytes = 0usize;

    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
        total_bytes += count;
    }

    let hash_result = hasher.finalize();
    let sha256_hex = hex::encode(hash_result);

    let mime_type = match path.extension().and_then(|ext| ext.to_str()) {
        Some(e) if e.eq_ignore_ascii_case("png") => Some("image/png".to_string()),
        Some(e) if e.eq_ignore_ascii_case("jpg") || e.eq_ignore_ascii_case("jpeg") => {
            Some("image/jpeg".to_string())
        }
        Some(e) if e.eq_ignore_ascii_case("webp") => Some("image/webp".to_string()),
        Some(e) if e.eq_ignore_ascii_case("gif") => Some("image/gif".to_string()),
        _ => None,
    };

    Ok(AttachmentFingerprint {
        sha256_hex,
        byte_size: total_bytes,
        mime_type,
    })
}
