#[cfg(test)]
mod tests {
    use super::super::db_size_watchdog::DbSizeWatchdog;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn test_default_constant() {
        assert_eq!(DbSizeWatchdog::DEFAULT_MAX_DB_BYTES, 100 * 1024 * 1024);
    }

    #[test]
    fn test_inspect_db_size_nonexistent() {
        let fake_path = PathBuf::from("non_existent_test_db.db");
        let report = DbSizeWatchdog::inspect_db_size(&fake_path, None);

        assert_eq!(report.db_name, "non_existent_test_db.db");
        assert_eq!(report.main_db_bytes, 0);
        assert_eq!(report.wal_bytes, 0);
        assert_eq!(report.total_bytes, 0);
        assert!(!report.is_over_threshold);
    }

    #[test]
    fn test_inspect_db_size_with_wal_and_threshold() {
        let temp_dir = std::env::temp_dir().join(format!("agm_test_db_{}", fastrand::u64(..)));
        let _ = fs::create_dir_all(&temp_dir);

        let db_file = temp_dir.join("analytics.db");
        let wal_file = temp_dir.join("analytics.db-wal");

        let _ = fs::write(&db_file, vec![0u8; 1000]);
        let _ = fs::write(&wal_file, vec![0u8; 500]);

        // Under 2000 byte threshold
        let report_under = DbSizeWatchdog::inspect_db_size(&db_file, Some(2000));
        assert_eq!(report_under.db_name, "analytics.db");
        assert_eq!(report_under.main_db_bytes, 1000);
        assert_eq!(report_under.wal_bytes, 500);
        assert_eq!(report_under.total_bytes, 1500);
        assert!(!report_under.is_over_threshold);

        // Over 1200 byte threshold
        let report_over = DbSizeWatchdog::inspect_db_size(&db_file, Some(1200));
        assert!(report_over.is_over_threshold);

        // Cleanup
        let _ = fs::remove_file(&db_file);
        let _ = fs::remove_file(&wal_file);
        let _ = fs::remove_dir(&temp_dir);
    }
}
