#[cfg(test)]
mod tests {
    use super::super::vacuum_stepper::{
        step_incremental_vacuum, IncrementalVacuumReport, DEFAULT_MAX_PAGES_PER_PASS,
        DEFAULT_VACUUM_CHUNK_SIZE,
    };
    use rusqlite::Connection;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().expect("Open in-memory db");
        // Incremental vacuum mode must be configured before creating tables
        conn.execute_batch("PRAGMA auto_vacuum = INCREMENTAL;")
            .expect("Set auto_vacuum");
        conn.execute_batch(
            "CREATE TABLE test_data (id INTEGER PRIMARY KEY, payload TEXT);
             CREATE INDEX idx_payload ON test_data(payload);",
        )
        .expect("Create tables");
        conn
    }

    fn populate_and_delete_for_freelist(conn: &Connection, count: usize, keep: usize) {
        {
            let mut stmt = conn
                .prepare("INSERT INTO test_data (payload) VALUES (?)")
                .unwrap();
            for i in 0..count {
                stmt.execute([format!("filler_payload_row_data_block_{}", i)])
                    .unwrap();
            }
        }
        conn.execute_batch(&format!("DELETE FROM test_data WHERE id > {};", keep))
            .unwrap();
    }

    #[test]
    fn test_vacuum_stepper_constants() {
        assert_eq!(DEFAULT_VACUUM_CHUNK_SIZE, 100);
        assert_eq!(DEFAULT_MAX_PAGES_PER_PASS, 1000);
    }

    #[test]
    fn test_report_serde_roundtrip() {
        let report = IncrementalVacuumReport {
            freelist_before: 500,
            freelist_after: 300,
            pages_reclaimed: 200,
            steps_executed: 2,
        };

        let json = serde_json::to_string(&report).expect("Serialize report");
        let parsed: IncrementalVacuumReport =
            serde_json::from_str(&json).expect("Deserialize report");

        assert_eq!(parsed.freelist_before, 500);
        assert_eq!(parsed.freelist_after, 300);
        assert_eq!(parsed.pages_reclaimed, 200);
        assert_eq!(parsed.steps_executed, 2);
    }

    #[test]
    fn test_zero_freelist_fast_path() {
        let conn = setup_test_db();
        let report = step_incremental_vacuum(&conn, None, None).expect("Vacuum execution");

        assert_eq!(report.freelist_before, 0);
        assert_eq!(report.freelist_after, 0);
        assert_eq!(report.pages_reclaimed, 0);
        assert_eq!(report.steps_executed, 0);
    }

    #[test]
    fn test_step_incremental_vacuum_bounded_chunks() {
        let conn = setup_test_db();
        populate_and_delete_for_freelist(&conn, 600, 50);

        let freelist: i64 = conn
            .query_row("PRAGMA freelist_count", [], |r| r.get(0))
            .unwrap();
        if freelist > 0 {
            let max_to_reclaim = 50.min(freelist);
            let chunk_size = 20;
            let report =
                step_incremental_vacuum(&conn, Some(max_to_reclaim), Some(chunk_size)).unwrap();

            assert_eq!(report.freelist_before, freelist);
            assert!(report.steps_executed >= 1);
            let expected_steps = ((max_to_reclaim + chunk_size - 1) / chunk_size) as usize;
            assert_eq!(report.steps_executed, expected_steps);
            assert_eq!(
                report.pages_reclaimed,
                report.freelist_before - report.freelist_after
            );
            assert!(report.freelist_after <= freelist);
        }
    }

    #[test]
    fn test_step_vacuum_with_default_parameters() {
        let conn = setup_test_db();
        populate_and_delete_for_freelist(&conn, 800, 20);

        let freelist_before: i64 = conn
            .query_row("PRAGMA freelist_count", [], |r| r.get(0))
            .unwrap();
        if freelist_before > 0 {
            let report = step_incremental_vacuum(&conn, None, None).unwrap();
            assert_eq!(report.freelist_before, freelist_before);
            assert!(report.steps_executed >= 1);
            assert_eq!(
                report.pages_reclaimed,
                report.freelist_before - report.freelist_after
            );
        }
    }
}
