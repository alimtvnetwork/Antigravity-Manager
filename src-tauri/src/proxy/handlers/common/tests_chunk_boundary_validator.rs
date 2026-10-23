#[cfg(test)]
mod tests {
    use super::super::chunk_boundary_validator::{
        split_at_safe_utf8_boundary, validate_chunk_boundary, BoundaryValidationResult,
    };

    #[test]
    fn test_chunk_boundary_empty() {
        assert_eq!(
            validate_chunk_boundary(&[]),
            BoundaryValidationResult::Empty
        );
    }

    #[test]
    fn test_chunk_boundary_complete_lines() {
        let chunk = b"data: test message\n";
        assert_eq!(
            validate_chunk_boundary(chunk),
            BoundaryValidationResult::CompleteLines
        );
    }

    #[test]
    fn test_chunk_boundary_incomplete_line() {
        let chunk = b"data: partial message without newline";
        assert_eq!(
            validate_chunk_boundary(chunk),
            BoundaryValidationResult::IncompleteLine
        );
    }

    #[test]
    fn test_chunk_boundary_incomplete_utf8_multibyte() {
        // "你好" in UTF-8 is [0xE4, 0xBD, 0xA0, 0xE5, 0xA5, 0xBD] (6 bytes)
        // Truncate at 4 bytes -> split mid-character at second character
        let bytes = [0xE4, 0xBD, 0xA0, 0xE5];
        match validate_chunk_boundary(&bytes) {
            BoundaryValidationResult::IncompleteUtf8 { valid_up_to } => {
                assert_eq!(valid_up_to, 3);
            }
            other => panic!("Expected IncompleteUtf8, got {:?}", other),
        }
    }

    #[test]
    fn test_split_at_safe_utf8_boundary() {
        let valid = b"hello world\n";
        let (head, tail) = split_at_safe_utf8_boundary(valid);
        assert_eq!(head, valid);
        assert!(tail.is_empty());

        let split_bytes = [0xE4, 0xBD, 0xA0, 0xE5];
        let (head_split, tail_split) = split_at_safe_utf8_boundary(&split_bytes);
        assert_eq!(head_split, &[0xE4, 0xBD, 0xA0]);
        assert_eq!(tail_split, &[0xE5]);
    }
}
