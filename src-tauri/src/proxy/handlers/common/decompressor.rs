//! Dynamic Content Decompressor for Upstream Gateway Responses.
//! Decompresses gzip and deflate payloads during upstream error inspection and diagnostics.

use flate2::read::{GzDecoder, ZlibDecoder};
use std::io::{Read, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentEncoding {
    Gzip,
    Deflate,
    Identity,
}

impl ContentEncoding {
    pub fn from_header(header: Option<&str>) -> Self {
        match header.map(|h| h.trim().to_ascii_lowercase()) {
            Some(h) if h.contains("gzip") => ContentEncoding::Gzip,
            Some(h) if h.contains("deflate") => ContentEncoding::Deflate,
            _ => ContentEncoding::Identity,
        }
    }
}

/// Decompresses binary payload bytes into standard raw bytes.
pub fn decompress_payload(bytes: &[u8], content_encoding: Option<&str>) -> Result<Vec<u8>> {
    // 1. Check explicit header
    let encoding = ContentEncoding::from_header(content_encoding);

    match encoding {
        ContentEncoding::Gzip => {
            let mut decoder = GzDecoder::new(bytes);
            let mut out = Vec::new();
            decoder.read_to_end(&mut out)?;
            Ok(out)
        }
        ContentEncoding::Deflate => {
            let mut decoder = ZlibDecoder::new(bytes);
            let mut out = Vec::new();
            decoder.read_to_end(&mut out)?;
            Ok(out)
        }
        ContentEncoding::Identity => {
            // Auto-detect gzip magic bytes (1F 8B) even if unadvertised
            if bytes.len() >= 2 && bytes[0] == 0x1F && bytes[1] == 0x8B {
                let mut decoder = GzDecoder::new(bytes);
                let mut out = Vec::new();
                if decoder.read_to_end(&mut out).is_ok() {
                    return Ok(out);
                }
            }
            Ok(bytes.to_vec())
        }
    }
}
