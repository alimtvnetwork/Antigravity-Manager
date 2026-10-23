//! Unit tests for Streaming Chunk Boundary Guard (`Imp-682`).

use super::streaming_chunk_boundary_guard::*;

#[test]
fn test_validate_compliant_chunk() {
    let config = ChunkBoundaryConfig::default();
    let chunk = b"data: {\"text\": \"hello\"}\n\n";

    assert!(validate_streaming_chunk(chunk, &config).is_ok());
    assert!(contains_sse_frame_delimiter(chunk));
}

#[test]
fn test_validate_oversized_chunk_rejection() {
    let config = ChunkBoundaryConfig {
        max_chunk_bytes: 100,
        allow_incomplete_tail: true,
    };
    let oversized = vec![b'a'; 150];

    let result = validate_streaming_chunk(&oversized, &config);
    assert!(result.is_err());
    match result.unwrap_err() {
        ChunkBoundaryViolation::OversizedChunk { chunk_len, max_allowed } => {
            assert_eq!(chunk_len, 150);
            assert_eq!(max_allowed, 100);
        }
        other => panic!("Unexpected violation: {other:?}"),
    }
}

#[test]
fn test_validate_empty_chunk_rejection() {
    let config = ChunkBoundaryConfig::default();
    let result = validate_streaming_chunk(&[], &config);
    assert_eq!(result, Err(ChunkBoundaryViolation::EmptyChunk));
}

#[test]
fn test_audit_streaming_stream_tail_closure() {
    let config = ChunkBoundaryConfig {
        max_chunk_bytes: 1024,
        allow_incomplete_tail: false,
    };

    let chunk1: &[u8] = b"data: {\"index\": 1}\n\n";
    let chunk2: &[u8] = b"data: {\"index\": 2}"; // Incomplete tail!

    let report = audit_streaming_stream(&[chunk1, chunk2], &config);
    assert!(!report.is_valid);
    assert_eq!(report.completed_frames_count, 1);
    assert_eq!(report.violations.len(), 1);
    match &report.violations[0] {
        ChunkBoundaryViolation::UnclosedStreamFrame { trailing_bytes } => {
            assert_eq!(*trailing_bytes, chunk2.len());
        }
        other => panic!("Unexpected violation: {other:?}"),
    }
}

#[test]
fn test_contains_sse_frame_delimiter() {
    assert!(contains_sse_frame_delimiter(b"event: message\ndata: 123\n\n"));
    assert!(contains_sse_frame_delimiter(b"event: message\r\ndata: 123\r\n\r\n"));
    assert!(!contains_sse_frame_delimiter(b"data: incomplete chunk"));
}
