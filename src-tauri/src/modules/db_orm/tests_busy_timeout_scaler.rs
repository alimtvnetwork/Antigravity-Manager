// Unit Test Suite for Dynamic Busy Timeout Scaler (Imp-524)
// Validates concurrency load scaling matrix, bounded clamp ranges,
// monotonic timeout growth, and live connection PRAGMA application.

#[cfg(test)]
mod tests {
    use crate::modules::db_orm::connection_busy_timeout_scaler::{
        ConnectionBusyTimeoutScaler, BASE_BUSY_TIMEOUT_MS, MAX_BUSY_TIMEOUT_MS, MIN_BUSY_TIMEOUT_MS,
    };
    use rusqlite::Connection;

    #[test]
    fn test_busy_timeout_scaler_constants() {
        assert_eq!(MIN_BUSY_TIMEOUT_MS, 3000);
        assert_eq!(BASE_BUSY_TIMEOUT_MS, 5000);
        assert_eq!(MAX_BUSY_TIMEOUT_MS, 15000);
        assert!(MIN_BUSY_TIMEOUT_MS < BASE_BUSY_TIMEOUT_MS);
        assert!(BASE_BUSY_TIMEOUT_MS < MAX_BUSY_TIMEOUT_MS);
    }

    #[test]
    fn test_compute_timeout_ms_scaling_matrix() {
        assert_eq!(ConnectionBusyTimeoutScaler::compute_timeout_ms(0), 5000);
        assert_eq!(ConnectionBusyTimeoutScaler::compute_timeout_ms(1), 5000);

        assert_eq!(ConnectionBusyTimeoutScaler::compute_timeout_ms(2), 7000);
        assert_eq!(ConnectionBusyTimeoutScaler::compute_timeout_ms(3), 8000);
        assert_eq!(ConnectionBusyTimeoutScaler::compute_timeout_ms(4), 9000);
        assert_eq!(ConnectionBusyTimeoutScaler::compute_timeout_ms(5), 10000);
        assert_eq!(ConnectionBusyTimeoutScaler::compute_timeout_ms(6), 11000);
        assert_eq!(ConnectionBusyTimeoutScaler::compute_timeout_ms(7), 12000);
        assert_eq!(ConnectionBusyTimeoutScaler::compute_timeout_ms(8), 13000);
        assert_eq!(ConnectionBusyTimeoutScaler::compute_timeout_ms(9), 14000);

        assert_eq!(ConnectionBusyTimeoutScaler::compute_timeout_ms(10), 15000);

        assert_eq!(ConnectionBusyTimeoutScaler::compute_timeout_ms(11), 15000);
        assert_eq!(ConnectionBusyTimeoutScaler::compute_timeout_ms(15), 15000);
        assert_eq!(ConnectionBusyTimeoutScaler::compute_timeout_ms(50), 15000);
        assert_eq!(ConnectionBusyTimeoutScaler::compute_timeout_ms(100), 15000);
        assert_eq!(ConnectionBusyTimeoutScaler::compute_timeout_ms(1000), 15000);
    }

    #[test]
    fn test_compute_timeout_ms_monotonic_progression() {
        let mut prev_timeout = ConnectionBusyTimeoutScaler::compute_timeout_ms(0);
        for concurrency in 1..=25 {
            let curr_timeout = ConnectionBusyTimeoutScaler::compute_timeout_ms(concurrency);
            assert!(
                curr_timeout >= prev_timeout,
                "Timeout must be monotonic non-decreasing: curr ({}) >= prev ({}) at concurrency {}",
                curr_timeout, prev_timeout, concurrency
            );
            assert!(
                curr_timeout >= MIN_BUSY_TIMEOUT_MS && curr_timeout <= MAX_BUSY_TIMEOUT_MS,
                "Timeout must remain bounded between MIN ({}) and MAX ({}): got {}",
                MIN_BUSY_TIMEOUT_MS,
                MAX_BUSY_TIMEOUT_MS,
                curr_timeout
            );
            prev_timeout = curr_timeout;
        }
    }

    #[test]
    fn test_apply_scaled_timeout_in_memory_connection() {
        let conn = Connection::open_in_memory().expect("In-memory connection must open");

        let t1 = ConnectionBusyTimeoutScaler::apply_scaled_timeout(&conn, 1)
            .expect("Must apply timeout for concurrency 1");
        assert_eq!(t1, 5000);

        let pragma_val1: u32 = conn
            .query_row("PRAGMA busy_timeout;", [], |row| row.get(0))
            .expect("Query busy_timeout pragma must succeed");
        assert_eq!(pragma_val1, 5000);

        let t5 = ConnectionBusyTimeoutScaler::apply_scaled_timeout(&conn, 5)
            .expect("Must apply timeout for concurrency 5");
        assert_eq!(t5, 10000);

        let pragma_val5: u32 = conn
            .query_row("PRAGMA busy_timeout;", [], |row| row.get(0))
            .expect("Query busy_timeout pragma must succeed");
        assert_eq!(pragma_val5, 10000);

        let t20 = ConnectionBusyTimeoutScaler::apply_scaled_timeout(&conn, 20)
            .expect("Must apply timeout for concurrency 20");
        assert_eq!(t20, 15000);

        let pragma_val20: u32 = conn
            .query_row("PRAGMA busy_timeout;", [], |row| row.get(0))
            .expect("Query busy_timeout pragma must succeed");
        assert_eq!(pragma_val20, 15000);

        let t0 = ConnectionBusyTimeoutScaler::apply_scaled_timeout(&conn, 0)
            .expect("Must apply timeout for concurrency 0");
        assert_eq!(t0, 5000);

        let pragma_val0: u32 = conn
            .query_row("PRAGMA busy_timeout;", [], |row| row.get(0))
            .expect("Query busy_timeout pragma must succeed");
        assert_eq!(pragma_val0, 5000);
    }

    #[test]
    fn test_apply_scaled_timeout_file_database() {
        let temp_dir = tempfile::tempdir().expect("Must create tempdir");
        let db_path = temp_dir.path().join("test_busy_scaler.db");
        let conn = Connection::open(&db_path).expect("Must open temp file database");

        conn.execute_batch(
            "
            PRAGMA journal_mode = WAL;
            CREATE TABLE dummy (id INTEGER PRIMARY KEY, note TEXT);
        ",
        )
        .expect("Init file db must succeed");

        let applied = ConnectionBusyTimeoutScaler::apply_scaled_timeout(&conn, 3)
            .expect("Must apply scaled timeout");
        assert_eq!(applied, 8000);

        let pragma_timeout: u32 = conn
            .query_row("PRAGMA busy_timeout;", [], |row| row.get(0))
            .expect("Query pragma must succeed");
        assert_eq!(pragma_timeout, 8000);
    }
}
