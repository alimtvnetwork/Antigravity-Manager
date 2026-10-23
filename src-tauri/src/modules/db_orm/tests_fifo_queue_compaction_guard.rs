//! Comprehensive Unit Tests for FIFO Queue Compaction Guard (`Imp-692`).
//!
//! Validates:
//! - Strict FIFO query validation enforcing `ORDER BY created_at ASC, id ASC`.
//! - Interception of inverted LIFO ordering, missing tie-breakers, and timestamp regressions.
//! - Clean tombstone and purgeable row elimination without sequence gaps.
//! - Canonical query builders and SQL order-by re-enforcement.

use super::fifo_queue_compaction_guard::*;

#[test]
fn test_compaction_table_mappings() {
    assert_eq!(CompactionTable::ActivePrompts.table_name(), "active_prompts");
    assert_eq!(CompactionTable::PromptBackups.table_name(), "prompt_backups");
    assert_eq!(CompactionTable::RepoPrompts.table_name(), "repo_prompts");

    assert_eq!(
        CompactionTable::from_table_name("active_prompts"),
        Some(CompactionTable::ActivePrompts)
    );
    assert_eq!(
        CompactionTable::from_table_name("`prompt_backups`"),
        Some(CompactionTable::PromptBackups)
    );
    assert_eq!(
        CompactionTable::from_table_name("\"repo_prompts\""),
        Some(CompactionTable::RepoPrompts)
    );
    assert_eq!(
        CompactionTable::from_table_name("[repo_prompts.db]"),
        Some(CompactionTable::RepoPrompts)
    );
    assert_eq!(CompactionTable::from_table_name("unknown_table"), None);

    assert_eq!(CompactionTable::all().len(), 3);
}

#[test]
fn test_validate_compaction_query_sql_success() {
    // Exact canonical query
    let sql1 = "SELECT id, created_at, status, is_tombstone FROM active_prompts ORDER BY created_at ASC, id ASC";
    assert!(validate_compaction_query_sql(sql1).is_ok());

    // Qualified table aliases
    let sql2 = "SELECT a.id FROM prompt_backups a ORDER BY a.created_at ASC, a.id ASC";
    assert!(validate_compaction_query_sql(sql2).is_ok());

    // Lowercase with trailing semicolon and LIMIT
    let sql3 = "select * from repo_prompts order by created_at asc, id asc limit 100;";
    assert!(validate_compaction_query_sql(sql3).is_ok());

    // Omitting optional ASC keyword (defaults to ASC in SQLite)
    let sql4 = "SELECT * FROM active_prompts ORDER BY created_at, id";
    assert!(validate_compaction_query_sql(sql4).is_ok());
}

#[test]
fn test_validate_compaction_query_sql_empty() {
    assert_eq!(
        validate_compaction_query_sql("   "),
        Err(CompactionViolation::EmptyQuery)
    );
}

#[test]
fn test_validate_compaction_query_sql_missing_order_by() {
    let sql = "SELECT * FROM active_prompts WHERE status = 'pending'";
    let res = validate_compaction_query_sql(sql);
    assert!(matches!(res, Err(CompactionViolation::MissingOrderByClause { .. })));
}

#[test]
fn test_validate_compaction_query_sql_inverted_lifo_desc() {
    // created_at DESC
    let sql1 = "SELECT * FROM active_prompts ORDER BY created_at DESC, id ASC";
    let res1 = validate_compaction_query_sql(sql1);
    assert!(matches!(res1, Err(CompactionViolation::InvertedLifoOrder { .. })));

    // updated_at DESC (typical inverted LIFO query)
    let sql2 = "SELECT * FROM prompt_backups ORDER BY updated_at DESC";
    let res2 = validate_compaction_query_sql(sql2);
    assert!(matches!(res2, Err(CompactionViolation::InvertedLifoOrder { .. })));

    // id DESC
    let sql3 = "SELECT * FROM repo_prompts ORDER BY created_at ASC, id DESC";
    let res3 = validate_compaction_query_sql(sql3);
    assert!(matches!(res3, Err(CompactionViolation::InvertedLifoOrder { .. })));
}

#[test]
fn test_validate_compaction_query_sql_missing_id_tie_breaker() {
    let sql = "SELECT * FROM active_prompts ORDER BY created_at ASC";
    let res = validate_compaction_query_sql(sql);
    assert!(matches!(res, Err(CompactionViolation::MissingIdTieBreaker { .. })));
}

#[test]
fn test_validate_compaction_query_sql_invalid_primary_column() {
    let sql = "SELECT * FROM active_prompts ORDER BY id ASC, created_at ASC";
    let res = validate_compaction_query_sql(sql);
    match res {
        Err(CompactionViolation::InvalidOrderByClause { found, expected }) => {
            assert!(found.contains("id"));
            assert_eq!(expected, "created_at ASC");
        }
        other => panic!("Unexpected result: {other:?}"),
    }
}

#[test]
fn test_validate_compaction_query_sql_invalid_secondary_column() {
    let sql = "SELECT * FROM active_prompts ORDER BY created_at ASC, updated_at ASC";
    let res = validate_compaction_query_sql(sql);
    match res {
        Err(CompactionViolation::InvalidOrderByClause { found, expected }) => {
            assert!(found.contains("updated_at"));
            assert_eq!(expected, "id ASC");
        }
        other => panic!("Unexpected result: {other:?}"),
    }
}

#[test]
fn test_execute_fifo_compaction_clean() {
    let rows = vec![
        QueueRow::active(1, 1_000, "pending"),
        QueueRow::active(2, 1_000, "pending"), // tie-broken by higher ID
        QueueRow::active(3, 1_500, "pending"),
        QueueRow::active(4, 2_000, "pending"),
    ];

    let report = execute_fifo_compaction(&rows);
    assert!(report.is_clean());
    assert!(report.is_order_preserved());
    assert_eq!(report.total_rows_before(), 4);
    assert_eq!(report.tombstones_purged(), 0);
    assert_eq!(report.retained_rows_count(), 4);
    assert!(report.violations().is_empty());
    assert_eq!(report.retained_rows().len(), 4);
}

#[test]
fn test_execute_fifo_compaction_with_tombstone_purging() {
    let rows = vec![
        QueueRow::active(1, 1_000, "pending"),
        QueueRow::tombstone(2, 1_100),
        QueueRow::active(3, 1_200, "pending"),
        QueueRow::new(4, 1_300, "purged", false), // Purgeable via status
        QueueRow::active(5, 1_400, "pending"),
    ];

    let report = execute_fifo_compaction(&rows);
    assert!(report.is_clean());
    assert!(report.is_order_preserved());
    assert_eq!(report.total_rows_before(), 5);
    assert_eq!(report.tombstones_purged(), 2);
    assert_eq!(report.retained_rows_count(), 3);

    let retained_ids: Vec<i64> = report.retained_rows().iter().map(|r| r.id).collect();
    assert_eq!(retained_ids, vec![1, 3, 5]);
}

#[test]
fn test_execute_fifo_compaction_timestamp_regression() {
    let rows = vec![
        QueueRow::active(1, 2_000, "pending"),
        QueueRow::active(2, 1_500, "pending"), // Regressed: 2000 -> 1500!
    ];

    let report = execute_fifo_compaction(&rows);
    assert!(!report.is_clean());
    assert!(!report.is_order_preserved());
    assert_eq!(report.violations().len(), 1);
    assert!(report.violations()[0].contains("Timestamp regression detected"));
}

#[test]
fn test_execute_fifo_compaction_id_tie_breaker_inversion() {
    let rows = vec![
        QueueRow::active(20, 1_000, "pending"),
        QueueRow::active(19, 1_000, "pending"), // Same timestamp, lower ID (inversion!)
    ];

    let report = execute_fifo_compaction(&rows);
    assert!(!report.is_clean());
    assert!(!report.is_order_preserved());
    assert_eq!(report.violations().len(), 1);
    assert!(report.violations()[0].contains("ID tie-breaker inversion"));
}

#[test]
fn test_execute_fifo_compaction_duplicate_id() {
    let rows = vec![
        QueueRow::active(10, 1_000, "pending"),
        QueueRow::active(10, 1_200, "pending"), // Duplicate ID 10!
    ];

    let report = execute_fifo_compaction(&rows);
    assert!(!report.is_clean());
    assert!(!report.is_order_preserved());
    assert_eq!(report.violations().len(), 1);
    assert!(report.violations()[0].contains("Duplicate row ID #10"));
}

#[test]
fn test_execute_fifo_compaction_empty_and_all_tombstones() {
    // Empty rows
    let empty_report = execute_fifo_compaction(&[]);
    assert!(empty_report.is_clean());
    assert_eq!(empty_report.total_rows_before(), 0);
    assert_eq!(empty_report.tombstones_purged(), 0);
    assert_eq!(empty_report.retained_rows_count(), 0);

    // All tombstones
    let tombstones = vec![
        QueueRow::tombstone(1, 100),
        QueueRow::tombstone(2, 200),
        QueueRow::tombstone(3, 300),
    ];
    let tombstone_report = execute_fifo_compaction(&tombstones);
    assert!(tombstone_report.is_clean());
    assert_eq!(tombstone_report.total_rows_before(), 3);
    assert_eq!(tombstone_report.tombstones_purged(), 3);
    assert_eq!(tombstone_report.retained_rows_count(), 0);
}

#[test]
fn test_build_canonical_queries() {
    let select_sql = build_canonical_compaction_query(CompactionTable::ActivePrompts);
    assert_eq!(
        select_sql,
        "SELECT id, created_at, status, is_tombstone FROM active_prompts ORDER BY created_at ASC, id ASC"
    );

    let delete_sql = build_canonical_tombstone_purge_query(CompactionTable::PromptBackups);
    assert_eq!(
        delete_sql,
        "DELETE FROM prompt_backups WHERE is_tombstone = 1 OR status = 'tombstone'"
    );
}

#[test]
fn test_enforce_compaction_order_by() {
    // Appending when ORDER BY is missing
    let sql1 = "SELECT * FROM active_prompts WHERE status = 'pending'";
    let enforced1 = enforce_compaction_order_by(sql1);
    assert_eq!(
        enforced1,
        "SELECT * FROM active_prompts WHERE status = 'pending' ORDER BY created_at ASC, id ASC"
    );

    // Replacing inverted LIFO ORDER BY
    let sql2 = "SELECT * FROM prompt_backups ORDER BY updated_at DESC";
    let enforced2 = enforce_compaction_order_by(sql2);
    assert_eq!(
        enforced2,
        "SELECT * FROM prompt_backups ORDER BY created_at ASC, id ASC"
    );

    // Preserving trailing LIMIT clause
    let sql3 = "SELECT * FROM repo_prompts ORDER BY created_at DESC LIMIT 50";
    let enforced3 = enforce_compaction_order_by(sql3);
    assert_eq!(
        enforced3,
        "SELECT * FROM repo_prompts ORDER BY created_at ASC, id ASC LIMIT 50"
    );
}
