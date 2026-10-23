#[cfg(test)]
mod tests {
    use super::super::decompressor::{decompress_payload, ContentEncoding};
    use flate2::write::{GzEncoder, ZlibEncoder};
    use flate2::Compression;
    use std::io::Write;

    #[test]
    fn test_content_encoding_from_header() {
        assert_eq!(ContentEncoding::from_header(Some("gzip")), ContentEncoding::Gzip);
        assert_eq!(ContentEncoding::from_header(Some("GZIP, Deflate")), ContentEncoding::Gzip);
        assert_eq!(ContentEncoding::from_header(Some("deflate")), ContentEncoding::Deflate);
        assert_eq!(ContentEncoding::from_header(Some("identity")), ContentEncoding::Identity);
        assert_eq!(ContentEncoding::from_header(None), ContentEncoding::Identity);
        assert_eq!(ContentEncoding::from_header(Some("")), ContentEncoding::Identity);
    }

    #[test]
    fn test_decompress_payload_identity() {
        let raw = b"hello plain text payload";
        let out = decompress_payload(raw, None).unwrap();
        assert_eq!(out, raw);

        let out_identity = decompress_payload(raw, Some("identity")).unwrap();
        assert_eq!(out_identity, raw);
    }

    #[test]
    fn test_decompress_payload_gzip() {
        let original = b"payload to be compressed with gzip for proxy transit";
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(original).unwrap();
        let compressed = encoder.finish().unwrap();

        let decompressed = decompress_payload(&compressed, Some("gzip")).unwrap();
        assert_eq!(decompressed, original);
    }

    #[test]
    fn test_decompress_payload_deflate() {
        let original = b"payload to be compressed with zlib deflate";
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(original).unwrap();
        let compressed = encoder.finish().unwrap();

        let decompressed = decompress_payload(&compressed, Some("deflate")).unwrap();
        assert_eq!(decompressed, original);
    }

    #[test]
    fn test_decompress_payload_auto_detect_gzip_magic() {
        let original = b"unadvertised gzip payload with magic bytes";
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(original).unwrap();
        let compressed = encoder.finish().unwrap();

        // Pass None as header - auto-detects 0x1F, 0x8B
        let decompressed = decompress_payload(&compressed, None).unwrap();
        assert_eq!(decompressed, original);
    }

    #[test]
    fn test_decompress_payload_corrupted_gzip_returns_err() {
        let corrupted = vec![0x1F, 0x8B, 0x00, 0xFF, 0xAA];
        assert!(decompress_payload(&corrupted, Some("gzip")).is_err());
    }
}
