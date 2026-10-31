// Unit Test Suite for Request Gzip Compression Sniffer (Imp-513)
// Validates detection of Gzip magic bytes, Deflate/Zlib headers, and non-compressed payloads.

#[cfg(test)]
mod tests {
    use super::super::request_gzip_sniffer::{CompressionFormat, RequestGzipSniffer};

    #[test]
    fn test_sniff_gzip_magic_bytes() {
        let exact_gzip = [0x1f, 0x8b];
        let stream_gzip = [0x1f, 0x8b, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00];

        assert_eq!(
            RequestGzipSniffer::sniff_compression(&exact_gzip),
            CompressionFormat::Gzip
        );
        assert_eq!(
            RequestGzipSniffer::sniff_compression(&stream_gzip),
            CompressionFormat::Gzip
        );
        assert!(RequestGzipSniffer::requires_decompression(&exact_gzip));
        assert!(RequestGzipSniffer::requires_decompression(&stream_gzip));
    }

    #[test]
    fn test_sniff_deflate_zlib_magic_bytes() {
        let zlib_headers = [
            [0x78, 0x01, 0xaa], // No/low compression
            [0x78, 0x5e, 0xbb], // Default compression
            [0x78, 0x9c, 0xcc], // Default compression alternative
            [0x78, 0xda, 0xdd], // Best compression
        ];

        for header in zlib_headers {
            assert_eq!(
                RequestGzipSniffer::sniff_compression(&header),
                CompressionFormat::Deflate,
                "Expected Deflate for header {:?}",
                header
            );
            assert!(RequestGzipSniffer::requires_decompression(&header));
        }
    }

    #[test]
    fn test_sniff_zlib_invalid_second_byte() {
        let invalid_zlib = [0x78, 0x00];
        let unknown_zlib = [0x78, 0x20];

        assert_eq!(
            RequestGzipSniffer::sniff_compression(&invalid_zlib),
            CompressionFormat::None
        );
        assert_eq!(
            RequestGzipSniffer::sniff_compression(&unknown_zlib),
            CompressionFormat::None
        );
        assert!(!RequestGzipSniffer::requires_decompression(&invalid_zlib));
    }

    #[test]
    fn test_sniff_insufficient_length() {
        assert_eq!(
            RequestGzipSniffer::sniff_compression(&[]),
            CompressionFormat::None
        );
        assert_eq!(
            RequestGzipSniffer::sniff_compression(&[0x1f]),
            CompressionFormat::None
        );
        assert_eq!(
            RequestGzipSniffer::sniff_compression(&[0x78]),
            CompressionFormat::None
        );
        assert!(!RequestGzipSniffer::requires_decompression(&[]));
        assert!(!RequestGzipSniffer::requires_decompression(&[0x1f]));
    }

    #[test]
    fn test_sniff_plaintext_json_payloads() {
        let json_body = br#"{"model":"claude-3-opus","messages":[{"role":"user","content":"hi"}]}"#;
        let http_text = b"POST /v1/messages HTTP/1.1";
        let null_bytes = [0x00, 0x00, 0x00, 0x00];

        assert_eq!(
            RequestGzipSniffer::sniff_compression(json_body),
            CompressionFormat::None
        );
        assert_eq!(
            RequestGzipSniffer::sniff_compression(http_text),
            CompressionFormat::None
        );
        assert_eq!(
            RequestGzipSniffer::sniff_compression(&null_bytes),
            CompressionFormat::None
        );
        assert!(!RequestGzipSniffer::requires_decompression(json_body));
    }
}
