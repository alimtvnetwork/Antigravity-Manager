// Request Gzip Compression Sniffer (Imp-513)
// Inspects magic bytes in incoming client request bodies to accurately detect
// GZIP or DEFLATE compression even if Content-Encoding headers are absent or stripped.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionFormat {
    None,
    Gzip,
    Deflate,
}

pub struct RequestGzipSniffer;

impl RequestGzipSniffer {
    /// Detects whether the byte buffer starts with gzip or zlib magic headers.
    pub fn sniff_compression(bytes: &[u8]) -> CompressionFormat {
        if bytes.len() < 2 {
            return CompressionFormat::None;
        }

        // GZIP magic bytes: 0x1F, 0x8B
        if bytes[0] == 0x1f && bytes[1] == 0x8b {
            return CompressionFormat::Gzip;
        }

        // ZLIB magic bytes: 0x78 and (0x01, 0x5E, 0x9C, 0xDA)
        if bytes[0] == 0x78 && (bytes[1] == 0x01 || bytes[1] == 0x5e || bytes[1] == 0x9c || bytes[1] == 0xda) {
            return CompressionFormat::Deflate;
        }

        CompressionFormat::None
    }

    /// Checks if decompression is required based on detected format.
    pub fn requires_decompression(bytes: &[u8]) -> bool {
        Self::sniff_compression(bytes) != CompressionFormat::None
    }
}
