//! Comprehensive unit tests for `FifoRestoreTransactionCoordinator` (`Task-208`).

use super::fifo_restore_transaction_coordinator::{
    build_coordinated_restore_sql, coordinate_restore_transaction,
    validate_coordinated_query_sql, validate_restore_stream_monotonicity,
    CoordinatedRestoreReport, RestoreItemRecord, RestoreTableKind,
    TransactionCoordinatorViolation, CANONICAL_FIFO_ORDER,
};

#[test]
fn test_table_name_mappings_for_all_variants() {
    assert_eq!(
        RestoreTableKind::ActivePrompts.as_table_name(),
        "active_prompts"
    );
    assert_eq!(
        RestoreTableKind::PromptBackups.as_table_name(),
        "prompt_backups"
    );
    assert_eq!(
        RestoreTableKind::RepoPrompts.as_table_name(),
        "repo_prompts"
    );

    assert_eq!(RestoreTableKind::ActivePrompts.as_str(), "active_prompts");
    assert_eq!(RestoreTableKind::PromptBackups.as_str(), "prompt_backups");
    assert_eq!(RestoreTableKind::RepoPrompts.as_str(), "repo_prompts");

    assert_eq!(
        format!("{}", RestoreTableKind::ActivePrompts),
        "active_prompts"
    );
    assert_eq!(
        format!("{}", RestoreTableKind::PromptBackups),
        "prompt_backups"
    );
    assert_eq!(
        format!("{}", RestoreTableKind::RepoPrompts),
        "repo_prompts"
    );

    let all = RestoreTableKind::all();
    assert_eq!(all.len(), 3);
    assert_eq!(all[0], RestoreTableKind::ActivePrompts);
    assert_eq!(all[1], RestoreTableKind::PromptBackups);
    assert_eq!(all[2], RestoreTableKind::RepoPrompts);
}

#[test]
fn test_query_construction_enforces_canonical_fifo_order() {
    let sql_active = build_coordinated_restore_sql(RestoreTableKind::ActivePrompts, 50);
    assert_eq!(
        sql_active,
        "SELECT id, created_at, prompt_text FROM active_prompts ORDER BY created_at ASC, id ASC LIMIT 50;"
    );
    assert!(sql_active.contains(CANONICAL_FIFO_ORDER));

    let sql_backups = build_coordinated_restore_sql(RestoreTableKind::PromptBackups, 100);
    assert_eq!(
        sql_backups,
        "SELECT id, created_at, prompt_text FROM prompt_backups ORDER BY created_at ASC, id ASC LIMIT 100;"
    );
    assert!(sql_backups.contains(CANONICAL_FIFO_ORDER));

    let sql_repo = build_coordinated_restore_sql(RestoreTableKind::RepoPrompts, 10);
    assert_eq!(
        sql_repo,
        "SELECT id, created_at, prompt_text FROM repo_prompts ORDER BY created_at ASC, id ASC LIMIT 10;"
    );
    assert!(sql_repo.contains(CANONICAL_FIFO_ORDER));
}

#[test]
fn test_query_validation_for_canonical_and_violations() {
    let valid_sql = build_coordinated_restore_sql(RestoreTableKind::ActivePrompts, 25);
    assert!(validate_coordinated_query_sql(&valid_sql).is_ok());

    let lifo_sql = "SELECT id FROM active_prompts ORDER BY updated_at DESC LIMIT 10;";
    let lifo_err = validate_coordinated_query_sql(lifo_sql).unwrap_err();
    assert!(matches!(
        lifo_err,
        TransactionCoordinatorViolation::LifoClauseForbidden { .. }
    ));

    let missing_tiebreaker_sql = "SELECT id FROM active_prompts ORDER BY created_at ASC LIMIT 10;";
    let tiebreaker_err = validate_coordinated_query_sql(missing_tiebreaker_sql).unwrap_err();
    assert!(matches!(
        tiebreaker_err,
        TransactionCoordinatorViolation::MissingOrderTieBreaker { .. }
    ));
}

#[test]
fn test_coordinated_restore_across_all_three_tables_succeeds() {
    let active_records = vec![
        RestoreItemRecord::new(1, RestoreTableKind::ActivePrompts, 1000, "prompt 1"),
        RestoreItemRecord::new(2, RestoreTableKind::ActivePrompts, 1001, "prompt 2"),
        RestoreItemRecord::new(3, RestoreTableKind::ActivePrompts, 1001, "prompt 3"),
    ];
    let backup_records = vec![
        RestoreItemRecord::new(10, RestoreTableKind::PromptBackups, 2000, "backup 1"),
        RestoreItemRecord::new(11, RestoreTableKind::PromptBackups, 2050, "backup 2"),
    ];
    let repo_records = vec![
        RestoreItemRecord::new(100, RestoreTableKind::RepoPrompts, 3000, "repo 1"),
        RestoreItemRecord::new(101, RestoreTableKind::RepoPrompts, 3001, "repo 2"),
        RestoreItemRecord::new(102, RestoreTableKind::RepoPrompts, 3002, "repo 3"),
    ];

    let streams = vec![
        (RestoreTableKind::ActivePrompts, active_records),
        (RestoreTableKind::PromptBackups, backup_records),
        (RestoreTableKind::RepoPrompts, repo_records),
    ];

    let report: CoordinatedRestoreReport = coordinate_restore_transaction(&streams);

    assert!(report.is_committed());
    assert_eq!(report.total_restored(), 8);
    assert_eq!(
        report.processed_tables,
        vec![
            RestoreTableKind::ActivePrompts,
            RestoreTableKind::PromptBackups,
            RestoreTableKind::RepoPrompts,
        ]
    );
    assert!(report.violation().is_none());
}

#[test]
fn test_inverted_timestamp_triggers_rollback_with_violation() {
    let invalid_records = vec![
        RestoreItemRecord::new(1, RestoreTableKind::ActivePrompts, 1000, "prompt 1"),
        RestoreItemRecord::new(2, RestoreTableKind::ActivePrompts, 999, "prompt 2"),
    ];

    let err = validate_restore_stream_monotonicity(&invalid_records).unwrap_err();
    assert_eq!(
        err,
        TransactionCoordinatorViolation::InvertedTimestampDetected {
            previous_created_at: 1000,
            current_created_at: 999,
        }
    );

    let streams = vec![(RestoreTableKind::ActivePrompts, invalid_records)];
    let report = coordinate_restore_transaction(&streams);

    assert!(!report.is_committed());
    assert_eq!(report.total_restored(), 0);
    assert_eq!(
        report.violation(),
        Some(&TransactionCoordinatorViolation::InvertedTimestampDetected {
            previous_created_at: 1000,
            current_created_at: 999,
        })
    );
}

#[test]
fn test_out_of_order_sequence_tiebreaker_triggers_rollback() {
    let tiebreaker_violation_records = vec![
        RestoreItemRecord::new(5, RestoreTableKind::PromptBackups, 1500, "backup 5"),
        RestoreItemRecord::new(4, RestoreTableKind::PromptBackups, 1500, "backup 4"),
    ];

    let err = validate_restore_stream_monotonicity(&tiebreaker_violation_records).unwrap_err();
    assert_eq!(
        err,
        TransactionCoordinatorViolation::OutOrOrderSequenceDetected {
            previous_id: 5,
            current_id: 4,
        }
    );

    let streams = vec![(RestoreTableKind::PromptBackups, tiebreaker_violation_records)];
    let report = coordinate_restore_transaction(&streams);

    assert!(!report.is_committed());
    assert_eq!(report.total_restored(), 0);
    assert_eq!(
        report.violation(),
        Some(&TransactionCoordinatorViolation::OutOrOrderSequenceDetected {
            previous_id: 5,
            current_id: 4,
        })
    );
}

#[test]
fn test_violation_display_formatting() {
    let out_of_order = TransactionCoordinatorViolation::OutOrOrderSequenceDetected {
        previous_id: 10,
        current_id: 8,
    };
    let formatted_seq = format!("{out_of_order}");
    assert!(formatted_seq.contains("Out of order sequence detected"));
    assert!(formatted_seq.contains("previous_id=10"));
    assert!(formatted_seq.contains("current_id=8"));

    let inverted_ts = TransactionCoordinatorViolation::InvertedTimestampDetected {
        previous_created_at: 500,
        current_created_at: 400,
    };
    let formatted_ts = format!("{inverted_ts}");
    assert!(formatted_ts.contains("Inverted timestamp detected"));
    assert!(formatted_ts.contains("previous_created_at=500"));
    assert!(formatted_ts.contains("current_created_at=400"));

    let lifo = TransactionCoordinatorViolation::LifoClauseForbidden {
        query: "SELECT * FROM active_prompts ORDER BY updated_at DESC".into(),
    };
    let formatted_lifo = format!("{lifo}");
    assert!(formatted_lifo.contains("LIFO clause is forbidden"));

    let missing_tb = TransactionCoordinatorViolation::MissingOrderTieBreaker {
        query: "SELECT * FROM active_prompts".into(),
    };
    let formatted_tb = format!("{missing_tb}");
    assert!(formatted_tb.contains("Missing secondary tie-breaker"));
}

#[test]
fn test_empty_and_single_item_streams_pass() {
    let empty_items: Vec<RestoreItemRecord> = Vec::new();
    assert!(validate_restore_stream_monotonicity(&empty_items).is_ok());

    let single_item = vec![RestoreItemRecord::new(
        1,
        RestoreTableKind::ActivePrompts,
        100,
        "single",
    )];
    assert!(validate_restore_stream_monotonicity(&single_item).is_ok());

    let streams = vec![
        (RestoreTableKind::ActivePrompts, empty_items),
        (RestoreTableKind::PromptBackups, single_item),
    ];
    let report = coordinate_restore_transaction(&streams);
    assert!(report.is_committed());
    assert_eq!(report.total_restored(), 1);
}

#[test]
fn test_multi_table_transaction_rolls_back_on_later_stream() {
    let valid_active = vec![
        RestoreItemRecord::new(1, RestoreTableKind::ActivePrompts, 100, "p1"),
        RestoreItemRecord::new(2, RestoreTableKind::ActivePrompts, 200, "p2"),
    ];
    let invalid_repo = vec![
        RestoreItemRecord::new(10, RestoreTableKind::RepoPrompts, 500, "r1"),
        RestoreItemRecord::new(11, RestoreTableKind::RepoPrompts, 400, "r2"),
    ];

    let streams = vec![
        (RestoreTableKind::ActivePrompts, valid_active),
        (RestoreTableKind::RepoPrompts, invalid_repo),
    ];

    let report = coordinate_restore_transaction(&streams);
    assert!(!report.is_committed());
    assert_eq!(report.total_restored(), 0);
    assert_eq!(
        report.processed_tables,
        vec![RestoreTableKind::ActivePrompts, RestoreTableKind::RepoPrompts]
    );
    assert!(matches!(
        report.violation(),
        Some(TransactionCoordinatorViolation::InvertedTimestampDetected { .. })
    ));
}
