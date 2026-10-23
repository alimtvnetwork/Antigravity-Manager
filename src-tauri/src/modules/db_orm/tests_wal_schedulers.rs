// Unit Test Suite for Split-SQLite WAL Truncate & Passive Checkpoint Schedulers (Imp-493 & Imp-523)
// Validates threshold evaluation, truncate checkpoint execution,
// passive checkpoint stats, and remaining uncheckpointed frames.

#[cfg(test)]
mod tests {
    use super::super::wal_passive_checkpoint_scheduler::{
        WalPassiveCheckpointScheduler, DEFAULT_IDLE_PASSIVE_INTERVAL_SECS,
    };
    use super::super::wal_truncate_scheduler::{WalTruncateScheduler, DEFAULT_MAX_WAL_PAGES};
    use rusqlite::Connection;

    #[test]
    fn test_constants_and_checkpoint_advisories() {
        assert_eq!(DEFAULT_MAX_WAL_PAGES, 500);

        assert!(!WalTruncateScheduler::is_checkpoint_advised(0, None));
        assert!(!WalTruncateScheduler::is_checkpoint_advised(499, None));
        assert!(WalTruncateScheduler::is_checkpoint_advised(500, None));
        assert!(WalTruncateScheduler::is_checkpoint_advised(501, None));

        assert!(!WalTruncateScheduler::is_checkpoint_advised(99, Some(100)));
        assert!(WalTruncateScheduler::is_checkpoint_advised(100, Some(100)));
        assert!(WalTruncateScheduler::is_checkpoint_advised(250, Some(100)));
    }

    #[test]
    fn test_execute_truncate_checkpoint_in_memory() {
        let conn = Connection::open_in_memory().expect("In-memory DB must open");
        let result = WalTruncateScheduler::execute_truncate_checkpoint(&conn)
            .expect("Checkpoint pragma execution must succeed");

        assert!(!result.busy);
    }

    #[test]
    fn test_execute_truncate_checkpoint_file_wal_database() {
        let temp_dir = tempfile::tempdir().expect("Must create tempdir");
        let db_path = temp_dir.path().join("test_truncate.db");
        let conn = Connection::open(&db_path).expect("Must open temp database");

        conn.execute_batch(
            "
            PRAGMA journal_mode = WAL;
            CREATE TABLE test_records (id INTEGER PRIMARY KEY, note TEXT);
            INSERT INTO test_records (note) VALUES ('entry-1'), ('entry-2');
        ",
        )
        .expect("Must populate initial records");

        let result = WalTruncateScheduler::execute_truncate_checkpoint(&conn)
            .expect("Truncate checkpoint must succeed");

        assert!(!result.busy);
        assert_eq!(result.log_pages, 0);
        assert!(result.checkpointed_pages >= 0);
        assert!(result.truncated);
    }

    #[test]
    fn test_default_constants_passive() {
        assert_eq!(DEFAULT_IDLE_PASSIVE_INTERVAL_SECS, 60);
    }

    #[test]
    fn test_execute_passive_checkpoint_in_memory() {
        let conn = Connection::open_in_memory().expect("In-memory DB must open");
        let stats = WalPassiveCheckpointScheduler::execute_passive_checkpoint(&conn)
            .expect("Passive checkpoint pragma must succeed");

        assert!(!stats.busy);
        assert_eq!(
            stats.remaining_uncheckpointed,
            stats.log_frames.saturating_sub(stats.checkpointed_frames)
        );
    }

    #[test]
    fn test_execute_passive_checkpoint_file_wal_database() {
        let temp_dir = tempfile::tempdir().expect("Must create tempdir");
        let db_path = temp_dir.path().join("test_passive.db");
        let conn = Connection::open(&db_path).expect("Must open temp database");

        conn.execute_batch(
            "
            PRAGMA journal_mode = WAL;
            CREATE TABLE telemetry_kv (k TEXT PRIMARY KEY, v TEXT);
            INSERT INTO telemetry_kv (k, v) VALUES ('node_1', 'healthy'), ('node_2', 'active');
        ",
        )
        .expect("Must create table and insert rows");

        let stats = WalPassiveCheckpointScheduler::execute_passive_checkpoint(&conn)
            .expect("Passive checkpoint must succeed");

        assert!(!stats.busy);
        assert!(stats.log_frames >= 0);
        assert!(stats.checkpointed_frames >= 0);
        assert_eq!(
            stats.remaining_uncheckpointed,
            stats.log_frames.saturating_sub(stats.checkpointed_frames)
        );
    }
}
