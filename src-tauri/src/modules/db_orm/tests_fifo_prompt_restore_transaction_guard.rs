//! Unit tests for Strict FIFO Prompt Restore Transaction Guard (`Imp-659`).

#[cfg(test)]
mod tests {
    use super::super::fifo_prompt_restore_transaction_guard::*;
    use rusqlite::Connection;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().expect("In-memory database creation failed");
        conn.execute_batch(
            r#"
            CREATE TABLE active_prompts (
                id TEXT PRIMARY KEY,
                prompt_text TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                status TEXT NOT NULL
            );
            CREATE TABLE prompt_backups (
                id TEXT PRIMARY KEY,
                prompt_text TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                status TEXT NOT NULL
            );
            CREATE TABLE users (
                id TEXT PRIMARY KEY,
                username TEXT NOT NULL
            );
            "#,
        )
        .expect("Schema initialization failed");
        conn
    }

    #[test]
    fn test_canonical_fifo_query_executes_and_commits() {
        let mut conn = setup_test_db();
        conn.execute(
            "INSERT INTO active_prompts VALUES ('p1', 'Prompt 1', 100, 'Pending'), ('p2', 'Prompt 2', 200, 'Pending')",
            [],
        )
        .expect("Seed insert failed");

        let mut guard = FifoPromptRestoreTransactionGuard::begin(&mut conn)
            .expect("Begin transaction guard failed");

        let sql = "SELECT id, prompt_text FROM active_prompts ORDER BY created_at ASC, id ASC";
        let rows = guard
            .query_dispatch_prompts(sql, &[], |r| {
                let id: String = r.get(0)?;
                let text: String = r.get(1)?;
                Ok((id, text))
            })
            .expect("Query should succeed");

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].0, "p1");
        assert_eq!(rows[1].0, "p2");
        assert!(!guard.is_aborted());
        assert_eq!(guard.audit_log().len(), 1);
        assert!(guard.audit_log()[0].is_compliant);

        guard.commit().expect("Commit should succeed");
    }

    #[test]
    fn test_inverted_lifo_order_aborts_transaction_and_rolls_back() {
        let mut conn = setup_test_db();
        conn.execute(
            "INSERT INTO active_prompts VALUES ('p1', 'Prompt 1', 100, 'Pending')",
            [],
        )
        .expect("Seed insert failed");

        let mut guard = FifoPromptRestoreTransactionGuard::begin(&mut conn)
            .expect("Begin transaction guard failed");

        // Insert within transaction
        guard
            .execute(
                "INSERT INTO active_prompts VALUES ('p2', 'Prompt 2', 200, 'Pending')",
                &[],
            )
            .expect("Insert should pass verification");

        // Now issue forbidden LIFO query
        let forbidden_sql = "SELECT * FROM active_prompts ORDER BY updated_at DESC";
        let err = guard
            .execute(forbidden_sql, &[])
            .expect_err("LIFO query must be intercepted and rejected");

        assert!(matches!(
            err,
            FifoTransactionViolation::InvertedLIFOOrderDetected { .. }
        ));
        assert!(guard.is_aborted());
        assert!(guard.abort_reason().is_some());

        // Subsequent query attempts should fail-fast
        let subsequent_err = guard
            .execute("SELECT * FROM active_prompts", &[])
            .expect_err("Poisoned transaction must reject queries");
        assert!(matches!(
            subsequent_err,
            FifoTransactionViolation::TransactionAlreadyAborted { .. }
        ));

        // Commit attempt should fail
        let commit_err = guard.commit().expect_err("Poisoned commit must fail");
        assert!(matches!(
            commit_err,
            FifoTransactionViolation::TransactionAlreadyAborted { .. }
        ));

        // Verify that 'p2' was rolled back in the underlying connection
        let count: i64 = conn
            .query_row("SELECT count(*) FROM active_prompts", [], |r| r.get(0))
            .expect("Count query failed");
        assert_eq!(count, 1, "Uncommitted data must be rolled back on abort");
    }

    #[test]
    fn test_inverted_created_at_desc_aborts_transaction() {
        let mut conn = setup_test_db();
        let mut guard = FifoPromptRestoreTransactionGuard::begin(&mut conn)
            .expect("Begin transaction failed");

        let sql = "SELECT * FROM active_prompts ORDER BY created_at DESC";
        let err = guard
            .execute(sql, &[])
            .expect_err("created_at DESC must be rejected");

        assert!(matches!(
            err,
            FifoTransactionViolation::InvertedLIFOOrderDetected { .. }
        ));
        assert!(guard.is_aborted());
    }

    #[test]
    fn test_missing_tie_breaker_id_asc_aborts_transaction() {
        let mut conn = setup_test_db();
        let mut guard = FifoPromptRestoreTransactionGuard::begin(&mut conn)
            .expect("Begin transaction failed");

        let sql = "SELECT * FROM active_prompts ORDER BY created_at ASC";
        let err = guard
            .execute(sql, &[])
            .expect_err("Missing tie-breaker id ASC must be rejected");

        assert!(matches!(
            err,
            FifoTransactionViolation::MissingTieBreakerIdAsc { .. }
        ));
        assert!(guard.is_aborted());
    }

    #[test]
    fn test_missing_order_by_on_prompt_queue_table_aborts() {
        let mut conn = setup_test_db();
        let mut guard = FifoPromptRestoreTransactionGuard::begin(&mut conn)
            .expect("Begin transaction failed");

        let sql = "SELECT id, prompt_text FROM active_prompts WHERE status = 'Pending'";
        let err = guard
            .execute(sql, &[])
            .expect_err("Missing ORDER BY on prompt queue table must be rejected");

        assert!(matches!(
            err,
            FifoTransactionViolation::MissingOrderByClause { .. }
        ));
        assert!(guard.is_aborted());
    }

    #[test]
    fn test_disallowed_sort_column_aborts() {
        let mut conn = setup_test_db();
        let mut guard = FifoPromptRestoreTransactionGuard::begin(&mut conn)
            .expect("Begin transaction failed");

        let sql = "SELECT * FROM active_prompts ORDER BY status ASC, id ASC";
        let err = guard
            .execute(sql, &[])
            .expect_err("Sorting by status instead of created_at must be rejected");

        assert!(matches!(
            err,
            FifoTransactionViolation::DisallowedSortColumn { .. }
        ));
        assert!(guard.is_aborted());
    }

    #[test]
    fn test_non_queue_table_passes_without_enforcement() {
        let mut conn = setup_test_db();
        let mut guard = FifoPromptRestoreTransactionGuard::begin(&mut conn)
            .expect("Begin transaction failed");

        let sql = "SELECT * FROM users ORDER BY username ASC";
        guard
            .execute(sql, &[])
            .expect("Non-queue query should pass without FIFO enforcement");

        assert!(!guard.is_aborted());
        guard.commit().expect("Commit should succeed");
    }

    #[test]
    fn test_point_mutation_by_id_passes_clean() {
        let mut conn = setup_test_db();
        conn.execute(
            "INSERT INTO active_prompts VALUES ('p1', 'Prompt 1', 100, 'Pending')",
            [],
        )
        .expect("Seed insert failed");

        let mut guard = FifoPromptRestoreTransactionGuard::begin(&mut conn)
            .expect("Begin transaction failed");

        let update_sql = "UPDATE active_prompts SET status = 'Completed' WHERE id = 'p1'";
        let affected = guard
            .execute(update_sql, &[])
            .expect("Point update should succeed");

        assert_eq!(affected, 1);
        assert!(!guard.is_aborted());
        guard.commit().expect("Commit should succeed");
    }

    #[test]
    fn test_batch_restore_with_fifo_order_succeeds() {
        let mut conn = setup_test_db();
        conn.execute(
            "INSERT INTO prompt_backups VALUES ('b1', 'Backup 1', 100, 'Archived'), ('b2', 'Backup 2', 200, 'Archived')",
            [],
        )
        .expect("Seed backups failed");

        let mut guard = FifoPromptRestoreTransactionGuard::begin(&mut conn)
            .expect("Begin transaction failed");

        let restore_sql = "INSERT INTO active_prompts SELECT id, prompt_text, created_at, 'Restored' FROM prompt_backups ORDER BY created_at ASC, id ASC";
        let affected = guard
            .execute_restore_batch(restore_sql, &[])
            .expect("FIFO batch restore should succeed");

        assert_eq!(affected, 2);
        assert!(!guard.is_aborted());
        guard.commit().expect("Commit should succeed");

        let restored_count: i64 = conn
            .query_row("SELECT count(*) FROM active_prompts WHERE status = 'Restored'", [], |r| r.get(0))
            .expect("Query count failed");
        assert_eq!(restored_count, 2);
    }

    #[test]
    fn test_batch_restore_with_lifo_aborts() {
        let mut conn = setup_test_db();
        conn.execute(
            "INSERT INTO prompt_backups VALUES ('b1', 'Backup 1', 100, 'Archived')",
            [],
        )
        .expect("Seed backups failed");

        let mut guard = FifoPromptRestoreTransactionGuard::begin(&mut conn)
            .expect("Begin transaction failed");

        let restore_lifo = "INSERT INTO active_prompts SELECT * FROM prompt_backups ORDER BY created_at DESC";
        let err = guard
            .execute_restore_batch(restore_lifo, &[])
            .expect_err("LIFO batch restore must be rejected");

        assert!(matches!(
            err,
            FifoTransactionViolation::InvertedLIFOOrderDetected { .. }
        ));
        assert!(guard.is_aborted());
    }

    #[test]
    fn test_raii_drop_uncommitted_rolls_back() {
        let mut conn = setup_test_db();
        {
            let mut guard = FifoPromptRestoreTransactionGuard::begin(&mut conn)
                .expect("Begin transaction failed");
            guard
                .execute(
                    "INSERT INTO active_prompts VALUES ('p_tmp', 'Temporary', 500, 'Pending')",
                    &[],
                )
                .expect("Insert should succeed");
            // Drop guard without calling commit
        }

        let count: i64 = conn
            .query_row("SELECT count(*) FROM active_prompts WHERE id = 'p_tmp'", [], |r| r.get(0))
            .expect("Count query failed");
        assert_eq!(count, 0, "RAII drop must roll back uncommitted changes");
    }
}
