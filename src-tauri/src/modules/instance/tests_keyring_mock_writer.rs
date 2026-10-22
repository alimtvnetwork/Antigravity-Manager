// Unit Test Suite for Sandboxed Keyring Mock Writer (Imp-533)
// Validates mock credentials generation, bypass marker creation,
// idempotence of marker checks, and parent directory auto-creation.

#[cfg(test)]
mod tests {
    use crate::modules::instance::keyring_mock_writer::{
        KeyringMockWriter, KEYRING_MARKER_FILENAME,
    };
    use std::fs;

    #[test]
    fn test_keyring_mock_writer_constants() {
        assert_eq!(KEYRING_MARKER_FILENAME, "antigravity-keyring-unavailable");
    }

    #[test]
    fn test_keyring_mock_writer_creates_marker_in_new_dir() {
        let temp_dir = std::env::temp_dir().join(format!(
            "agm_mock_keyring_test_{}",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));

        assert!(!temp_dir.exists());

        let res = KeyringMockWriter::ensure_keyring_marker(&temp_dir);
        assert!(res.is_ok(), "Writing keyring marker in missing dir must succeed");

        let status = res.unwrap();
        assert!(status.is_present);
        assert!(status.created_new);

        let marker_file = temp_dir.join(KEYRING_MARKER_FILENAME);
        assert!(marker_file.is_file());

        let content = fs::read_to_string(&marker_file).expect("Must read marker file");
        assert!(content.contains("mock-keyring-enabled=true"));
        assert!(content.contains("created_epoch="));

        // Second invocation must be idempotent and report created_new = false
        let res2 = KeyringMockWriter::ensure_keyring_marker(&temp_dir);
        assert!(res2.is_ok());
        let status2 = res2.unwrap();
        assert!(status2.is_present);
        assert!(!status2.created_new);

        // Justification: Clean up test fixture directory
        crate::error::record_ignored(fs::remove_dir_all(&temp_dir), "clean mock keyring test dir");
    }

    #[test]
    fn test_keyring_mock_writer_preserves_existing_content() {
        let temp_dir = std::env::temp_dir().join(format!(
            "agm_mock_keyring_exist_{}",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        // Justification: Create test fixture directory
        crate::error::record_ignored(fs::create_dir_all(&temp_dir), "create test dir");

        let marker_file = temp_dir.join(KEYRING_MARKER_FILENAME);
        let custom_content = "custom-keyring-override=1\n";
        // Justification: Write custom test marker
        crate::error::record_ignored(fs::write(&marker_file, custom_content), "write custom marker");

        let res = KeyringMockWriter::ensure_keyring_marker(&temp_dir);
        assert!(res.is_ok());
        let status = res.unwrap();
        assert!(status.is_present);
        assert!(!status.created_new);

        let read_back = fs::read_to_string(&marker_file).unwrap();
        assert_eq!(read_back, custom_content, "Existing marker content must not be overwritten");

        // Justification: Clean up test fixture directory
        crate::error::record_ignored(fs::remove_dir_all(&temp_dir), "clean test dir");
    }
}
