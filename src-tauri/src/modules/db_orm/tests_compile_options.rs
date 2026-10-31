#[cfg(test)]
mod tests {
    use super::super::compile_options::{inspect_sqlite_capabilities, SqliteEngineCapabilities};
    use rusqlite::Connection;

    #[test]
    fn test_inspect_sqlite_capabilities_in_memory() {
        let conn = Connection::open_in_memory().expect("open in-memory SQLite database");
        let capabilities = inspect_sqlite_capabilities(&conn).expect("inspect capabilities");

        // rusqlite bundled sqlite returns multiple compile flags
        assert!(
            !capabilities.compile_options.is_empty(),
            "Expected SQLite to have compile options"
        );

        // Verify that is_threadsafe corresponds to THREADSAFE compile option presence
        let threadsafe_opt = capabilities
            .compile_options
            .iter()
            .find(|o| o.starts_with("THREADSAFE="));
        if let Some(opt) = threadsafe_opt {
            if opt.ends_with("=0") {
                assert!(!capabilities.is_threadsafe);
            } else {
                assert!(capabilities.is_threadsafe);
            }
        }

        // Verify JSON1 flag consistency
        let json_present = capabilities
            .compile_options
            .iter()
            .any(|o| o.contains("ENABLE_JSON1"));
        assert_eq!(capabilities.has_json1, json_present);

        // Verify FTS5 flag consistency
        let fts5_present = capabilities
            .compile_options
            .iter()
            .any(|o| o.contains("ENABLE_FTS5"));
        assert_eq!(capabilities.has_fts5, fts5_present);
    }

    #[test]
    fn test_sqlite_engine_capabilities_serialization_roundtrip() {
        let caps = SqliteEngineCapabilities {
            compile_options: vec![
                "ENABLE_FTS5".to_string(),
                "ENABLE_JSON1".to_string(),
                "THREADSAFE=1".to_string(),
                "DEFAULT_WAL_SYNCHRONOUS=1".to_string(),
            ],
            has_json1: true,
            has_fts5: true,
            is_threadsafe: true,
        };

        let serialized = serde_json::to_string(&caps).expect("serialize capabilities");
        let deserialized: SqliteEngineCapabilities =
            serde_json::from_str(&serialized).expect("deserialize capabilities");

        assert_eq!(deserialized.compile_options.len(), 4);
        assert!(deserialized.has_json1);
        assert!(deserialized.has_fts5);
        assert!(deserialized.is_threadsafe);
    }

    #[test]
    fn test_sqlite_capabilities_threadsafe_logic_matrix() {
        // Evaluate THREADSAFE logic across variants
        let check_threadsafe = |options: &[&str]| -> bool {
            options
                .iter()
                .any(|o| o.starts_with("THREADSAFE=") && !o.ends_with("=0"))
        };

        // Serialized mode (THREADSAFE=1) -> true
        assert!(check_threadsafe(&["THREADSAFE=1"]));
        // Multi-threaded mode (THREADSAFE=2) -> true
        assert!(check_threadsafe(&["THREADSAFE=2"]));
        // Single-threaded mode (THREADSAFE=0) -> false
        assert!(!check_threadsafe(&["THREADSAFE=0"]));
        // Empty options -> false
        assert!(!check_threadsafe(&[]));
        // Unrelated options -> false
        assert!(!check_threadsafe(&["ENABLE_FTS5", "ENABLE_JSON1"]));
    }

    #[test]
    fn test_sqlite_capabilities_extension_detection_matrix() {
        let check_json1 =
            |options: &[&str]| -> bool { options.iter().any(|o| o.contains("ENABLE_JSON1")) };
        let check_fts5 =
            |options: &[&str]| -> bool { options.iter().any(|o| o.contains("ENABLE_FTS5")) };

        // Standard flags
        assert!(check_json1(&["ENABLE_JSON1"]));
        assert!(check_fts5(&["ENABLE_FTS5"]));

        // Prefixed flags (e.g. SQLITE_ENABLE_JSON1)
        assert!(check_json1(&["SQLITE_ENABLE_JSON1"]));
        assert!(check_fts5(&["SQLITE_ENABLE_FTS5"]));

        // Missing flags
        assert!(!check_json1(&["ENABLE_COLUMN_METADATA"]));
        assert!(!check_fts5(&["ENABLE_RTREE"]));
    }

    #[test]
    fn test_sqlite_engine_capabilities_clone_and_debug() {
        let caps = SqliteEngineCapabilities {
            compile_options: vec!["THREADSAFE=1".to_string()],
            has_json1: false,
            has_fts5: false,
            is_threadsafe: true,
        };

        let cloned = caps.clone();
        assert_eq!(cloned.compile_options, caps.compile_options);
        assert_eq!(cloned.is_threadsafe, caps.is_threadsafe);

        let debug_str = format!("{caps:?}");
        assert!(debug_str.contains("SqliteEngineCapabilities"));
        assert!(debug_str.contains("THREADSAFE=1"));
    }
}
