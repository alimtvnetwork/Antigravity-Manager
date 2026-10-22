// Unit Test Suite for Split-SQLite WAL Page Fragmentation Surveyor (Imp-544)
// Validates page size retrieval, estimated uncheckpointed WAL page calculations,
// and truncate recommendation thresholds.

#[cfg(test)]
mod tests {
    use crate::modules::db_orm::wal_fragmentation_surveyor::{
        WalFragmentationReport, WalFragmentationSurveyor, DEFAULT_MAX_WAL_PAGES_BEFORE_TRUNCATE,
    };
    use rusqlite::Connection;

    #[test]
    fn test_wal_surveyor_constants_and_serde() {
        assert_eq!(DEFAULT_MAX_WAL_PAGES_BEFORE_TRUNCATE, 1000);

        let report = WalFragmentationReport {
            wal_size_bytes: 4096000,
            page_size_bytes: 4096,
            estimated_wal_pages: 1000,
            is_truncate_recommended: true,
        };

        let json_str = serde_json::to_string(&report).expect("Serialize must succeed");
        let deserialized: WalFragmentationReport =
            serde_json::from_str(&json_str).expect("Deserialize must succeed");

        assert_eq!(deserialized.wal_size_bytes, 4096000);
        assert_eq!(deserialized.page_size_bytes, 4096);
        assert_eq!(deserialized.estimated_wal_pages, 1000);
        assert!(deserialized.is_truncate_recommended);
    }

    #[test]
    fn test_survey_wal_under_threshold() {
        let conn = Connection::open_in_memory().expect("In-memory DB must open");
        // 500 pages * 4096 = 2,048,000 bytes (< 1000 threshold)
        let report = WalFragmentationSurveyor::survey_wal(&conn, 2_048_000, None)
            .expect("Survey must succeed");

        assert_eq!(report.estimated_wal_pages, 500);
        assert!(!report.is_truncate_recommended);
    }

    #[test]
    fn test_survey_wal_at_and_above_threshold() {
        let conn = Connection::open_in_memory().expect("In-memory DB must open");

        // Exactly at default 1000 pages (4,096,000 bytes)
        let rep_exact = WalFragmentationSurveyor::survey_wal(&conn, 4_096_000, None).unwrap();
        assert_eq!(rep_exact.estimated_wal_pages, 1000);
        assert!(rep_exact.is_truncate_recommended);

        // Custom threshold Some(250)
        let rep_custom = WalFragmentationSurveyor::survey_wal(&conn, 1_024_000, Some(250)).unwrap();
        assert_eq!(rep_custom.estimated_wal_pages, 250);
        assert!(rep_custom.is_truncate_recommended);
    }

    #[test]
    fn test_survey_wal_empty_zero_size() {
        let conn = Connection::open_in_memory().expect("In-memory DB must open");
        let report = WalFragmentationSurveyor::survey_wal(&conn, 0, None).unwrap();

        assert_eq!(report.wal_size_bytes, 0);
        assert_eq!(report.estimated_wal_pages, 0);
        assert!(!report.is_truncate_recommended);
    }
}
