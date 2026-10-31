//! Unit tests for ResponseChunkAssembler (Imp-573)

use super::response_chunk_assembler::ResponseChunkAssembler;

#[test]
fn test_default_assembler_initialization() {
    let assembler = ResponseChunkAssembler::default();
    assert_eq!(assembler.current_size(), 0);
    assert!(assembler.finish().is_empty());
}

#[test]
fn test_append_chunk_and_current_size() {
    let mut assembler = ResponseChunkAssembler::new(1024);

    let res1 = assembler.append_chunk(b"hello ");
    assert_eq!(res1, Ok(6));
    assert_eq!(assembler.current_size(), 6);

    let res2 = assembler.append_chunk(b"world!");
    assert_eq!(res2, Ok(12));
    assert_eq!(assembler.current_size(), 12);

    let full_bytes = assembler.finish();
    assert_eq!(&full_bytes, b"hello world!");
}

#[test]
fn test_buffer_limit_exceeded_returns_err() {
    let mut assembler = ResponseChunkAssembler::new(10);

    // 8 bytes fits within 10
    let res1 = assembler.append_chunk(b"12345678");
    assert_eq!(res1, Ok(8));
    assert_eq!(assembler.current_size(), 8);

    // Adding 5 more bytes would exceed 10 (8 + 5 = 13 > 10)
    let res2 = assembler.append_chunk(b"abcde");
    assert!(res2.is_err());
    assert_eq!(
        res2.unwrap_err(),
        "Response payload exceeds maximum allowed size buffer limit"
    );

    // Existing buffer remains intact
    assert_eq!(assembler.current_size(), 8);
    assert_eq!(&assembler.finish(), b"12345678");
}

#[test]
fn test_exact_capacity_boundary() {
    let mut assembler = ResponseChunkAssembler::new(5);
    let res = assembler.append_chunk(b"12345");
    assert_eq!(res, Ok(5));
    assert_eq!(assembler.current_size(), 5);

    // Any further byte fails
    let res_overflow = assembler.append_chunk(b"6");
    assert!(res_overflow.is_err());
}
