// Unit Test Suite for Database Page Freelist Reclaim Analyzer (Imp-528)
// Validates freelist page inspection, threshold recommendation matrix,
// reclaimable byte calculations, and incremental vacuum execution.

#[cfg(test)]
mod tests {
    use crate::modules::db_orm::page_freelist_analyzer::{
        FreelistReport, PageFreelistAnalyzer, DEFAULT_VACUUM_FREELIST_THRESHOLD,
    };
    use rusqlite::Connection;

    #[test]
    fn test_freelist_constants_and_serde_roundtrip() {
        assert_eq!(DEFAULT_VACUUM_FREELIST_THRESHOLD, 100);

        let report = FreelistReport {
            freelist_pages: 150,
            page_size_bytes: 4096,
            reclaimable_bytes: 150 * 4096,
            is_vacuum_recommended: true,
        };

        let serialized = serde_json::to_string(&report).expect("Serialization must succeed");
        let deserialized: FreelistReport =
            serde_json::from_str(&serialized).expect("Deserialization must succeed");

        assert_eq!(deserialized.freelist_pages, 150);
        assert_eq!(deserialized.page_size_bytes, 4096);
        assert_eq!(deserialized.reclaimable_bytes, 614400);
        assert!(deserialized.is_vacuum_recommended);
    }

    #[test]
    fn test_inspect_freelist_clean_in_memory_db() {
        let conn = Connection::open_in_memory().expect("In-memory DB must open");
        let report = PageFreelistAnalyzer::inspect_freelist(&conn, None)
            .expect("Inspect freelist on clean db must succeed");

        assert_eq!(report.freelist_pages, 0);
        assert!(report.page_size_bytes > 0);
        assert_eq!(report.reclaimable_bytes, 0);
        assert!(!report.is_vacuum_recommended);
    }

    #[test]
    fn test_inspect_freelist_threshold_matrix() {
        let conn = Connection::open_in_memory().expect("In-memory DB must open");

        let rep_default = PageFreelistAnalyzer::inspect_freelist(&conn, None).unwrap();
        assert!(!rep_default.is_vacuum_recommended);

        let rep_one = PageFreelistAnalyzer::inspect_freelist(&conn, Some(1)).unwrap();
        assert!(!rep_one.is_vacuum_recommended);

        let rep_zero = PageFreelistAnalyzer::inspect_freelist(&conn, Some(0)).unwrap();
        assert!(rep_zero.is_vacuum_recommended);

        let rep_large = PageFreelistAnalyzer::inspect_freelist(&conn, Some(500)).unwrap();
        assert!(!rep_large.is_vacuum_recommended);
    }

    #[test]
    fn test_freelist_pages_with_deleted_records_and_threshold_recommendations() {
        let temp_dir = tempfile::tempdir().expect("Must create tempdir");
        let db_path = temp_dir.path().join("test_freelist.db");
        let conn = Connection::open(&db_path).expect("Must open temp database");

        conn.execute_batch(
            "
            PRAGMA auto_vacuum = INCREMENTAL;
            PRAGMA page_size = 4096;
            CREATE TABLE large_data (id INTEGER PRIMARY KEY, payload TEXT);
        ",
        )
        .expect("Table initialization must succeed");

        let chunk = "X".repeat(4000);
        for _ in 0..300 {
            conn.execute("INSERT INTO large_data (payload) VALUES (?1);", [&chunk])
                .expect("Insert chunk must succeed");
        }

        let report_before_delete = PageFreelistAnalyzer::inspect_freelist(&conn, None)
            .expect("Inspect before delete must succeed");
        assert_eq!(report_before_delete.freelist_pages, 0);
        assert_eq!(report_before_delete.reclaimable_bytes, 0);
        assert!(!report_before_delete.is_vacuum_recommended);

        conn.execute("DELETE FROM large_data WHERE id > 50;", [])
            .expect("Delete rows must succeed");

        let report_after_delete = PageFreelistAnalyzer::inspect_freelist(&conn, None)
            .expect("Inspect after delete must succeed");

        assert!(
            report_after_delete.freelist_pages > 0,
            "Freelist pages should be > 0 after deleting rows in incremental_vacuum mode"
        );
        assert_eq!(
            report_after_delete.reclaimable_bytes,
            report_after_delete.freelist_pages * report_after_delete.page_size_bytes
        );

        let free_pages = report_after_delete.freelist_pages;

        let rep_boundary_exact =
            PageFreelistAnalyzer::inspect_freelist(&conn, Some(free_pages)).unwrap();
        assert!(
            rep_boundary_exact.is_vacuum_recommended,
            "freelist == threshold must recommend vacuum"
        );

        let rep_boundary_under =
            PageFreelistAnalyzer::inspect_freelist(&conn, Some(free_pages + 1)).unwrap();
        assert!(
            !rep_boundary_under.is_vacuum_recommended,
            "freelist < threshold must not recommend vacuum"
        );

        if free_pages > 1 {
            let rep_boundary_over =
                PageFreelistAnalyzer::inspect_freelist(&conn, Some(free_pages - 1)).unwrap();
            assert!(
                rep_boundary_over.is_vacuum_recommended,
                "freelist > threshold must recommend vacuum"
            );
        }
    }

    #[test]
    fn test_execute_incremental_vacuum_reduces_freelist() {
        let temp_dir = tempfile::tempdir().expect("Must create tempdir");
        let db_path = temp_dir.path().join("test_vacuum_exec.db");
        let conn = Connection::open(&db_path).expect("Must open temp database");

        conn.execute_batch(
            "
            PRAGMA auto_vacuum = INCREMENTAL;
            CREATE TABLE temp_blobs (id INTEGER PRIMARY KEY, content BLOB);
        ",
        )
        .expect("Create table must succeed");

        let large_blob = vec![0u8; 8192];
        for _ in 0..100 {
            conn.execute("INSERT INTO temp_blobs (content) VALUES (?1);", [&large_blob])
                .expect("Insert blob must succeed");
        }

        conn.execute("DELETE FROM temp_blobs;", [])
            .expect("Delete all rows must succeed");

        let initial_report = PageFreelistAnalyzer::inspect_freelist(&conn, Some(1))
            .expect("Inspect must succeed");
        let initial_free = initial_report.freelist_pages;
        assert!(
            initial_free >= 10,
            "Must have multiple freelist pages before vacuum execution"
        );

        PageFreelistAnalyzer::execute_incremental_vacuum(&conn, 2)
            .expect("Incremental vacuum execution must succeed");

        let report_step1 = PageFreelistAnalyzer::inspect_freelist(&conn, Some(1))
            .expect("Inspect after step 1 must succeed");
        assert_eq!(report_step1.freelist_pages, initial_free - 2);

        PageFreelistAnalyzer::execute_incremental_vacuum(&conn, 0)
            .expect("Vacuum all pages must succeed");

        let final_report = PageFreelistAnalyzer::inspect_freelist(&conn, Some(1))
            .expect("Inspect after full vacuum must succeed");
        assert_eq!(final_report.freelist_pages, 0);
        assert_eq!(final_report.reclaimable_bytes, 0);
        assert!(!final_report.is_vacuum_recommended);
    }

    #[test]
    fn test_execute_incremental_vacuum_in_memory_safe_noop() {
        let conn = Connection::open_in_memory().expect("In-memory DB must open");
        let res = PageFreelistAnalyzer::execute_incremental_vacuum(&conn, 5);
        assert!(res.is_ok(), "Incremental vacuum on in-memory db must succeed safely");
    }
}
