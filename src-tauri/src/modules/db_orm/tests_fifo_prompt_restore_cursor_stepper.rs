//! Unit tests for FIFO Prompt Restore Cursor Stepper (`Imp-697`).

use super::fifo_prompt_restore_cursor_stepper::*;

#[test]
fn test_initial_query_generation_without_cursor() {
    let config = CursorStepConfig::default();
    let sql = build_cursor_query_sql(None, &config).expect("Query generation should succeed");

    assert_eq!(
        sql,
        "SELECT id, created_at, prompt_text, status FROM prompt_backups ORDER BY created_at ASC, id ASC LIMIT 50"
    );
    assert!(sql.contains("ORDER BY created_at ASC, id ASC LIMIT 50"));

    // Custom batch size and table
    let custom_config = CursorStepConfig::new(25, "active_prompts");
    let custom_sql = build_cursor_query_sql(None, &custom_config).expect("Custom query generation should succeed");
    assert_eq!(
        custom_sql,
        "SELECT id, created_at, prompt_text, status FROM active_prompts ORDER BY created_at ASC, id ASC LIMIT 25"
    );

    // Max drain limit capping batch size
    let capped_config = CursorStepConfig::new(50, "repo_prompts").with_max_drain_limit(10);
    let capped_sql = build_cursor_query_sql(None, &capped_config).expect("Capped query generation should succeed");
    assert_eq!(
        capped_sql,
        "SELECT id, created_at, prompt_text, status FROM repo_prompts ORDER BY created_at ASC, id ASC LIMIT 10"
    );
}

#[test]
fn test_cursor_query_with_prompt_cursor() {
    let cursor = PromptCursor::new("2026-10-23T08:00:00Z", 42);
    let config = CursorStepConfig::default();
    let sql = build_cursor_query_sql(Some(&cursor), &config).expect("Cursor query generation should succeed");

    assert_eq!(
        sql,
        "SELECT id, created_at, prompt_text, status FROM prompt_backups WHERE (created_at > '2026-10-23T08:00:00Z') OR (created_at = '2026-10-23T08:00:00Z' AND id > 42) ORDER BY created_at ASC, id ASC LIMIT 50"
    );
    assert!(sql.contains("WHERE (created_at > '2026-10-23T08:00:00Z') OR (created_at = '2026-10-23T08:00:00Z' AND id > 42)"));
    assert!(sql.contains("ORDER BY created_at ASC, id ASC LIMIT 50"));
}

#[test]
fn test_lifo_or_inverted_ordering_query_attempts_fail_validation() {
    // Queries containing DESC on created_at
    let desc_query = "SELECT id, created_at, prompt_text, status FROM prompt_backups ORDER BY created_at DESC, id ASC LIMIT 50";
    match validate_query_ordering(desc_query) {
        Err(CursorValidationError::LifoOrderingDetected(detected)) => {
            assert!(detected.contains("DESC"));
        }
        other => panic!("Expected LifoOrderingDetected, got {other:?}"),
    }

    // Queries containing updated_at DESC
    let updated_at_desc = "SELECT * FROM active_prompts ORDER BY updated_at DESC";
    match validate_query_ordering(updated_at_desc) {
        Err(CursorValidationError::LifoOrderingDetected(detected)) => {
            assert!(detected.contains("DESC"));
        }
        other => panic!("Expected LifoOrderingDetected, got {other:?}"),
    }

    // Queries missing tie-breaker id ASC
    let missing_tiebreaker = "SELECT * FROM prompt_backups ORDER BY created_at ASC";
    match validate_query_ordering(missing_tiebreaker) {
        Err(CursorValidationError::MissingTieBreakerId(detected)) => {
            assert!(detected.contains("created_at ASC"));
        }
        other => panic!("Expected MissingTieBreakerId, got {other:?}"),
    }

    // Queries missing ORDER BY entirely
    let missing_order_by = "SELECT * FROM prompt_backups WHERE id = 1";
    match validate_query_ordering(missing_order_by) {
        Err(CursorValidationError::MissingTieBreakerId(detected)) => {
            assert!(detected.contains("Missing ORDER BY"));
        }
        other => panic!("Expected MissingTieBreakerId for missing ORDER BY, got {other:?}"),
    }

    // Custom SQL in CursorStepConfig containing DESC
    let bad_config = CursorStepConfig::default().with_custom_sql("SELECT * FROM prompt_backups ORDER BY created_at DESC");
    assert!(matches!(
        build_cursor_query_sql(None, &bad_config),
        Err(CursorValidationError::LifoOrderingDetected(_))
    ));

    // Target table injecting DESC
    let bad_table_config = CursorStepConfig::new(50, "prompt_backups ORDER BY created_at DESC");
    assert!(matches!(
        build_cursor_query_sql(None, &bad_table_config),
        Err(CursorValidationError::LifoOrderingDetected(_))
    ));
}

#[test]
fn test_invalid_cursor_timestamp_validation() {
    let empty_ts_cursor = PromptCursor::new("", 1);
    let config = CursorStepConfig::default();
    assert!(matches!(
        build_cursor_query_sql(Some(&empty_ts_cursor), &config),
        Err(CursorValidationError::InvalidCursorTimestamp(_))
    ));

    let injection_cursor = PromptCursor::new("'; DROP TABLE prompt_backups; --", 1);
    assert!(matches!(
        build_cursor_query_sql(Some(&injection_cursor), &config),
        Err(CursorValidationError::InvalidCursorTimestamp(_))
    ));

    let invalid_chars_cursor = PromptCursor::new("2026-10-23<dangerous>", 1);
    assert!(matches!(
        build_cursor_query_sql(Some(&invalid_chars_cursor), &config),
        Err(CursorValidationError::InvalidCursorTimestamp(_))
    ));
}

#[test]
fn test_monotonic_step_batch_tracking_extracts_next_cursor() {
    let items = vec![
        PromptQueueItem::new(10, "2026-10-23T08:00:00Z", "first prompt", "pending"),
        PromptQueueItem::new(11, "2026-10-23T08:00:00Z", "second prompt", "pending"),
        PromptQueueItem::new(15, "2026-10-23T08:05:00Z", "third prompt", "pending"),
    ];

    let config = CursorStepConfig::new(3, "prompt_backups");
    let result = step_cursor_batch(&items, &config);

    assert_eq!(result.items.len(), 3);
    assert_eq!(result.total_scanned, 3);
    assert!(!result.is_end_of_queue); // Batch size 3 matches scanned 3
    assert_eq!(
        result.next_cursor,
        Some(PromptCursor::new("2026-10-23T08:05:00Z", 15))
    );

    // End of queue scenario (less items than batch_size)
    let small_items = vec![
        PromptQueueItem::new(20, "2026-10-23T08:10:00Z", "last prompt", "pending"),
    ];
    let eof_result = step_cursor_batch(&small_items, &config);
    assert_eq!(eof_result.items.len(), 1);
    assert!(eof_result.is_end_of_queue);
    assert_eq!(
        eof_result.next_cursor,
        Some(PromptCursor::new("2026-10-23T08:10:00Z", 20))
    );

    // Empty batch
    let empty_items: Vec<PromptQueueItem> = Vec::new();
    let empty_result = step_cursor_batch(&empty_items, &config);
    assert!(empty_result.is_empty());
    assert!(empty_result.is_end_of_queue);
    assert_eq!(empty_result.next_cursor, None);
}

#[test]
fn test_verify_cursor_monotonicity() {
    let c1 = PromptCursor::new("2026-10-23T08:00:00Z", 10);
    let c2 = PromptCursor::new("2026-10-23T08:00:00Z", 11);
    let c3 = PromptCursor::new("2026-10-23T08:05:00Z", 1);

    // Advance by tie-breaker ID
    assert!(verify_cursor_monotonicity(&c1, &c2));
    // Advance by timestamp
    assert!(verify_cursor_monotonicity(&c2, &c3));

    // Stagnant cursor (same timestamp and ID)
    assert!(!verify_cursor_monotonicity(&c1, &c1));
    // Regressing timestamp
    assert!(!verify_cursor_monotonicity(&c3, &c1));
    // Regressing tie-breaker ID
    assert!(!verify_cursor_monotonicity(&c2, &c1));
}

#[test]
fn test_out_of_order_item_sequences_are_detected() {
    let config = CursorStepConfig::default();

    // Timestamp regression
    let regressed_items = vec![
        PromptQueueItem::new(1, "2026-10-23T08:05:00Z", "item 1", "pending"),
        PromptQueueItem::new(2, "2026-10-23T08:00:00Z", "item 2", "pending"),
    ];
    assert!(!verify_items_monotonicity(&regressed_items));
    assert!(validate_items_monotonicity(&regressed_items).is_err());
    assert!(try_step_cursor_batch(&regressed_items, &config).is_err());

    // Tie-breaker ID regression
    let tiebreaker_regressed = vec![
        PromptQueueItem::new(5, "2026-10-23T08:00:00Z", "item A", "pending"),
        PromptQueueItem::new(4, "2026-10-23T08:00:00Z", "item B", "pending"),
    ];
    assert!(!verify_items_monotonicity(&tiebreaker_regressed));
    assert!(validate_items_monotonicity(&tiebreaker_regressed).is_err());
    assert!(try_step_cursor_batch(&tiebreaker_regressed, &config).is_err());

    // Duplicate item ID at same timestamp
    let duplicate_items = vec![
        PromptQueueItem::new(10, "2026-10-23T08:00:00Z", "dup A", "pending"),
        PromptQueueItem::new(10, "2026-10-23T08:00:00Z", "dup B", "pending"),
    ];
    assert!(!verify_items_monotonicity(&duplicate_items));
    assert!(validate_items_monotonicity(&duplicate_items).is_err());
    assert!(try_step_cursor_batch(&duplicate_items, &config).is_err());

    // Partial valid prefix in step_cursor_batch
    let mixed_items = vec![
        PromptQueueItem::new(1, "2026-10-23T08:00:00Z", "valid 1", "pending"),
        PromptQueueItem::new(2, "2026-10-23T08:05:00Z", "valid 2", "pending"),
        PromptQueueItem::new(3, "2026-10-23T07:59:00Z", "invalid 3", "pending"),
    ];
    let result = step_cursor_batch(&mixed_items, &config);
    // Should stop at valid 2
    assert_eq!(result.items.len(), 2);
    assert_eq!(result.total_scanned, 2);
    assert_eq!(
        result.next_cursor,
        Some(PromptCursor::new("2026-10-23T08:05:00Z", 2))
    );
}
