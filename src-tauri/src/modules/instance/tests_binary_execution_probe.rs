// Unit Test Suite for IDE Binary Architecture and Execution Probe (Imp-532)
// Validates file existence checks, directory vs file detection, zero-byte file rejection,
// executable binary verification, and serde serialization contracts.

#[cfg(test)]
mod tests {
    use crate::modules::instance::binary_execution_probe::{BinaryExecutionProbe, BinaryProbeResult};
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn test_binary_execution_probe_rejects_nonexistent_binary() {
        let nonexistent_path = PathBuf::from("nonexistent_path_to_binary_target.exe");

        let result = BinaryExecutionProbe::probe_binary(&nonexistent_path);

        assert_eq!(result.path, nonexistent_path.to_string_lossy());
        assert!(!result.exists, "Nonexistent path must report exists = false");
        assert!(!result.is_file, "Nonexistent path must report is_file = false");
        assert_eq!(result.size_bytes, 0, "Nonexistent path size must be 0");
        assert!(!result.is_executable, "Nonexistent path cannot be executable");
        assert_eq!(
            result.error,
            Some("Executable binary path does not exist".to_string()),
            "Must provide descriptive missing binary error message"
        );
    }

    #[test]
    fn test_binary_execution_probe_rejects_directory_target() {
        let temp_dir = std::env::temp_dir().join(format!(
            "agm_bin_probe_dir_{}",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        // Justification: Test fixture directory creation in temporary folder
        crate::error::record_ignored(fs::create_dir_all(&temp_dir), "create test dir");

        let result = BinaryExecutionProbe::probe_binary(&temp_dir);

        assert!(result.exists, "Directory path exists");
        assert!(!result.is_file, "Directory must report is_file = false");
        assert!(!result.is_executable, "Directory cannot be considered an executable binary");
        assert_eq!(
            result.error,
            Some("Binary path exists but is not a valid executable file".to_string()),
            "Directory target must return invalid executable file error"
        );

        // Justification: Best-effort test fixture cleanup
        crate::error::record_ignored(fs::remove_dir_all(&temp_dir), "clean test dir");
    }

    #[test]
    fn test_binary_execution_probe_rejects_zero_byte_empty_file() {
        let temp_file = std::env::temp_dir().join(format!(
            "agm_bin_probe_zero_{}.exe",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        // Justification: Test fixture file creation in temporary folder
        crate::error::record_ignored(fs::write(&temp_file, b""), "create empty binary test fixture");

        let result = BinaryExecutionProbe::probe_binary(&temp_file);

        assert!(result.exists);
        assert!(result.is_file);
        assert_eq!(result.size_bytes, 0, "Empty file must report size 0 bytes");
        assert!(!result.is_executable, "Zero-byte file must NOT be considered executable");
        assert_eq!(
            result.error,
            Some("Binary path exists but is not a valid executable file".to_string())
        );

        // Justification: Best-effort test fixture cleanup
        crate::error::record_ignored(fs::remove_file(&temp_file), "clean test file");
    }

    #[test]
    fn test_binary_execution_probe_accepts_valid_nonempty_binary_payload() {
        let temp_bin = std::env::temp_dir().join(format!(
            "agm_bin_probe_valid_{}.exe",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));

        let mock_binary_payload = b"MZ\x90\x00\x03\x00\x00\x00\x04\x00\x00\x00\xff\xff\x00\x00";
        // Justification: Test fixture file creation in temporary folder
        crate::error::record_ignored(fs::write(&temp_bin, mock_binary_payload), "write valid mock binary");

        let result = BinaryExecutionProbe::probe_binary(&temp_bin);

        assert!(result.exists, "Target binary must exist");
        assert!(result.is_file, "Target binary must be recognized as a file");
        assert_eq!(result.size_bytes, mock_binary_payload.len() as u64);
        assert!(result.is_executable, "Non-empty binary must be marked executable");
        assert!(result.error.is_none(), "Valid binary probe must produce error: None");

        // Justification: Best-effort test fixture cleanup
        crate::error::record_ignored(fs::remove_file(&temp_bin), "clean test file");
    }

    #[test]
    fn test_binary_execution_probe_validates_host_current_process_binary() {
        if let Ok(current_exe) = std::env::current_exe() {
            let result = BinaryExecutionProbe::probe_binary(&current_exe);

            assert!(result.exists, "Current running executable must exist on disk");
            assert!(result.is_file, "Current running executable must be a file");
            assert!(result.size_bytes > 0, "Current running executable must have positive file size");
            assert!(result.is_executable, "Current running executable must be valid and executable");
            assert!(result.error.is_none(), "No errors expected for host binary probe");
        }
    }

    #[test]
    fn test_binary_execution_probe_serde_roundtrip_and_architecture_envelope() {
        let probe_result = BinaryProbeResult {
            path: "/opt/antigravity/bin/antigravity".to_string(),
            exists: true,
            is_file: true,
            size_bytes: 124508928,
            is_executable: true,
            error: None,
        };

        let json_str = serde_json::to_string(&probe_result).expect("BinaryProbeResult serialize");
        assert!(json_str.contains("\"path\":\"/opt/antigravity/bin/antigravity\""));
        assert!(json_str.contains("\"exists\":true"));
        assert!(json_str.contains("\"is_file\":true"));
        assert!(json_str.contains("\"size_bytes\":124508928"));
        assert!(json_str.contains("\"is_executable\":true"));
        assert!(json_str.contains("\"error\":null"));

        let roundtrip: BinaryProbeResult = serde_json::from_str(&json_str).expect("BinaryProbeResult deserialize");
        assert_eq!(roundtrip.path, probe_result.path);
        assert_eq!(roundtrip.exists, probe_result.exists);
        assert_eq!(roundtrip.is_file, probe_result.is_file);
        assert_eq!(roundtrip.size_bytes, probe_result.size_bytes);
        assert_eq!(roundtrip.is_executable, probe_result.is_executable);
        assert_eq!(roundtrip.error, probe_result.error);
    }
}
