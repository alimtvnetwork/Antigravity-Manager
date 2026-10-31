//! Comprehensive unit tests for `FifoPromptQueuePartitionAllocator` (`Imp-712`).
//!
//! Enforces AGENTS.md FIFO Restore & Queue Invariant:
//! "All prompt restore, dispatch, and queue processing queries across SQLite databases
//! (`active_prompts`, `prompt_backups`, `repo_prompts.db`) MUST enforce strict First-In-First-Out
//! ordering using `ORDER BY created_at ASC, id ASC`. Inverted LIFO queries (`ORDER BY updated_at DESC`,
//! `ORDER BY created_at DESC`) are strictly forbidden in dispatch logic."
//!
//! Specification Reference: `02-spec/21-app/217-implementing-next-five-tasks/spec.md`.

use super::fifo_prompt_queue_partition_allocator::{
    allocate_items_to_partitions, build_partition_query_sql, build_partition_query_sql_for_table,
    clean_column_name, compare_timestamps, is_prompt_queue_table, validate_partition_query_sql,
    verify_allocated_partition_monotonicity, AllocatedPromptItem, PartitionAllocationError,
    PartitionAllocationReport, QueuePartitionBucket, QueuePriorityBand, CANONICAL_FIFO_ORDER_BY,
    PROMPT_QUEUE_TABLES,
};

#[test]
fn test_priority_band_display_rank_and_loose_parsing() {
    assert_eq!(QueuePriorityBand::Urgent.as_str(), "urgent");
    assert_eq!(QueuePriorityBand::Normal.as_str(), "normal");
    assert_eq!(QueuePriorityBand::Background.as_str(), "background");

    assert_eq!(format!("{}", QueuePriorityBand::Urgent), "urgent");
    assert_eq!(format!("{}", QueuePriorityBand::Normal), "normal");
    assert_eq!(format!("{}", QueuePriorityBand::Background), "background");

    assert_eq!(QueuePriorityBand::Urgent.rank(), 0);
    assert_eq!(QueuePriorityBand::Normal.rank(), 1);
    assert_eq!(QueuePriorityBand::Background.rank(), 2);

    assert_eq!(
        QueuePriorityBand::from_str_loose("URGENT"),
        Some(QueuePriorityBand::Urgent)
    );
    assert_eq!(
        QueuePriorityBand::from_str_loose("  normal "),
        Some(QueuePriorityBand::Normal)
    );
    assert_eq!(
        QueuePriorityBand::from_str_loose("BackGround"),
        Some(QueuePriorityBand::Background)
    );
    assert_eq!(QueuePriorityBand::from_str_loose("invalid"), None);
}

#[test]
fn test_queue_partition_bucket_construction_and_getters() {
    let bucket = QueuePartitionBucket::new(
        "part_urgent_01",
        "worker_node_a",
        QueuePriorityBand::Urgent,
        25,
    );
    assert_eq!(bucket.partition_id(), "part_urgent_01");
    assert_eq!(bucket.instance_id(), "worker_node_a");
    assert_eq!(bucket.priority(), QueuePriorityBand::Urgent);
    assert_eq!(bucket.limit(), 25);
}

#[test]
fn test_allocated_prompt_item_construction_and_getters() {
    let item = AllocatedPromptItem::new(
        101,
        "2026-10-23T09:15:00Z",
        "Refactor database partition queries",
        QueuePriorityBand::Urgent,
        "part_urgent_01",
    );
    assert_eq!(item.id(), 101);
    assert_eq!(item.created_at(), "2026-10-23T09:15:00Z");
    assert_eq!(item.prompt_text(), "Refactor database partition queries");
    assert_eq!(item.priority(), QueuePriorityBand::Urgent);
    assert_eq!(item.partition_id(), "part_urgent_01");
}

#[test]
fn test_build_partition_query_sql_urgent_and_normal() {
    let urgent_bucket =
        QueuePartitionBucket::new("urgent_part", "inst_01", QueuePriorityBand::Urgent, 10);
    let urgent_sql = build_partition_query_sql(&urgent_bucket).expect("Failed to build urgent SQL");

    assert!(urgent_sql.contains("WHERE priority = 'urgent'"));
    assert!(urgent_sql.contains("AND instance_id = 'inst_01'"));
    assert!(urgent_sql.contains(CANONICAL_FIFO_ORDER_BY));
    assert!(urgent_sql.contains("LIMIT 10;"));

    let normal_bucket = QueuePartitionBucket::new("normal_part", "", QueuePriorityBand::Normal, 50);
    let normal_sql = build_partition_query_sql(&normal_bucket).expect("Failed to build normal SQL");

    assert!(normal_sql.contains("WHERE priority = 'normal'"));
    assert!(!normal_sql.contains("instance_id"));
    assert!(normal_sql.contains(CANONICAL_FIFO_ORDER_BY));
    assert!(normal_sql.contains("LIMIT 50;"));

    let custom_sql = build_partition_query_sql_for_table("prompt_backups", &normal_bucket)
        .expect("Failed to build custom table SQL");
    assert!(custom_sql.starts_with(
        "SELECT id, created_at, prompt_text, priority, partition_id FROM prompt_backups"
    ));
    assert!(custom_sql.contains(CANONICAL_FIFO_ORDER_BY));
}

#[test]
fn test_build_partition_query_sql_rejects_zero_limit() {
    let zero_bucket =
        QueuePartitionBucket::new("invalid_part", "inst_01", QueuePriorityBand::Urgent, 0);
    let result = build_partition_query_sql(&zero_bucket);

    match result {
        Err(PartitionAllocationError::InvalidPartitionLimit) => {}
        other => panic!("Expected InvalidPartitionLimit error, got: {other:?}"),
    }
}

#[test]
fn test_validate_partition_query_sql_accepts_valid_fifo_queries() {
    let valid_queries = [
        "SELECT * FROM active_prompts ORDER BY created_at ASC, id ASC LIMIT 50;",
        "SELECT id, prompt_text FROM prompt_backups ORDER BY created_at ASC, id ASC LIMIT 10",
        "SELECT * FROM repo_prompts ORDER BY `created_at` ASC, `id` ASC",
        "SELECT * FROM active_prompts ORDER BY \"created_at\" ASC, \"id\" ASC",
        "SELECT * FROM active_prompts ORDER BY t.created_at ASC, t.id ASC",
        "SELECT * FROM active_prompts ORDER BY priority ASC, created_at ASC, id ASC LIMIT 20;",
    ];

    for query in valid_queries {
        let res = validate_partition_query_sql(query);
        assert!(
            res.is_ok(),
            "Expected valid query '{query}' to pass, got: {res:?}"
        );
    }
}

#[test]
fn test_validate_partition_query_sql_rejects_inverted_lifo_orderings() {
    let lifo_queries = [
        "SELECT * FROM active_prompts ORDER BY updated_at DESC",
        "SELECT * FROM active_prompts ORDER BY created_at DESC, id ASC",
        "SELECT * FROM active_prompts ORDER BY created_at ASC, id DESC",
        "SELECT * FROM prompt_backups ORDER BY id DESC",
        "SELECT * FROM active_prompts ORDER BY rowid DESC",
        "SELECT * FROM repo_prompts ORDER BY id ASC, created_at ASC", // Inverted primary ordering
    ];

    for query in lifo_queries {
        let res = validate_partition_query_sql(query);
        match res {
            Err(PartitionAllocationError::InvertedLifoOrderingDetected) => {}
            other => {
                panic!("Expected InvertedLifoOrderingDetected for query '{query}', got: {other:?}")
            }
        }
    }
}

#[test]
fn test_validate_partition_query_sql_rejects_missing_tiebreaker() {
    let missing_tiebreaker_queries = [
        "SELECT * FROM active_prompts ORDER BY created_at ASC",
        "SELECT * FROM active_prompts ORDER BY created_at ASC, id", // Missing explicit ASC
        "SELECT * FROM prompt_backups ORDER BY created_at ASC, status ASC",
        "SELECT * FROM active_prompts WHERE id = 10", // Missing ORDER BY
    ];

    for query in missing_tiebreaker_queries {
        let res = validate_partition_query_sql(query);
        match res {
            Err(PartitionAllocationError::MissingTieBreakerId) => {}
            other => panic!("Expected MissingTieBreakerId for query '{query}', got: {other:?}"),
        }
    }
}

#[test]
fn test_verify_allocated_partition_monotonicity_iso8601() {
    // 1. Empty items
    assert!(verify_allocated_partition_monotonicity(&[]));

    // 2. Single item with valid timestamp
    let single = vec![AllocatedPromptItem::new(
        1,
        "2026-10-23T09:00:00Z",
        "Test 1",
        QueuePriorityBand::Urgent,
        "p1",
    )];
    assert!(verify_allocated_partition_monotonicity(&single));

    // 3. Single item with blank timestamp fails
    let blank_single = vec![AllocatedPromptItem::new(
        1,
        "",
        "Test blank",
        QueuePriorityBand::Urgent,
        "p1",
    )];
    assert!(!verify_allocated_partition_monotonicity(&blank_single));

    // 4. Strictly increasing timestamps pass
    let valid_seq = vec![
        AllocatedPromptItem::new(
            1,
            "2026-10-23T09:00:00Z",
            "A",
            QueuePriorityBand::Urgent,
            "p1",
        ),
        AllocatedPromptItem::new(
            2,
            "2026-10-23T09:05:00Z",
            "B",
            QueuePriorityBand::Urgent,
            "p1",
        ),
        AllocatedPromptItem::new(
            3,
            "2026-10-23T09:10:00Z",
            "C",
            QueuePriorityBand::Urgent,
            "p1",
        ),
    ];
    assert!(verify_allocated_partition_monotonicity(&valid_seq));

    // 5. Equal timestamps with strictly increasing id pass
    let equal_ts_valid = vec![
        AllocatedPromptItem::new(
            10,
            "2026-10-23T09:00:00Z",
            "A",
            QueuePriorityBand::Normal,
            "p1",
        ),
        AllocatedPromptItem::new(
            11,
            "2026-10-23T09:00:00Z",
            "B",
            QueuePriorityBand::Normal,
            "p1",
        ),
    ];
    assert!(verify_allocated_partition_monotonicity(&equal_ts_valid));

    // 6. Inverted timestamp fails
    let inverted_ts = vec![
        AllocatedPromptItem::new(
            1,
            "2026-10-23T09:10:00Z",
            "A",
            QueuePriorityBand::Urgent,
            "p1",
        ),
        AllocatedPromptItem::new(
            2,
            "2026-10-23T09:00:00Z",
            "B",
            QueuePriorityBand::Urgent,
            "p1",
        ),
    ];
    assert!(!verify_allocated_partition_monotonicity(&inverted_ts));

    // 7. Equal timestamp with inverted id fails
    let inverted_id = vec![
        AllocatedPromptItem::new(
            20,
            "2026-10-23T09:00:00Z",
            "A",
            QueuePriorityBand::Urgent,
            "p1",
        ),
        AllocatedPromptItem::new(
            19,
            "2026-10-23T09:00:00Z",
            "B",
            QueuePriorityBand::Urgent,
            "p1",
        ),
    ];
    assert!(!verify_allocated_partition_monotonicity(&inverted_id));
}

#[test]
fn test_verify_allocated_partition_monotonicity_numeric_epoch() {
    let epoch_valid = vec![
        AllocatedPromptItem::new(
            1,
            "1761234000",
            "Epoch A",
            QueuePriorityBand::Background,
            "p2",
        ),
        AllocatedPromptItem::new(
            2,
            "1761234500",
            "Epoch B",
            QueuePriorityBand::Background,
            "p2",
        ),
        AllocatedPromptItem::new(
            3,
            "1761234500",
            "Epoch C",
            QueuePriorityBand::Background,
            "p2",
        ),
    ];
    assert!(verify_allocated_partition_monotonicity(&epoch_valid));

    let epoch_inverted = vec![
        AllocatedPromptItem::new(
            1,
            "1761234900",
            "Epoch A",
            QueuePriorityBand::Background,
            "p2",
        ),
        AllocatedPromptItem::new(
            2,
            "1761234100",
            "Epoch B",
            QueuePriorityBand::Background,
            "p2",
        ),
    ];
    assert!(!verify_allocated_partition_monotonicity(&epoch_inverted));
}

#[test]
fn test_allocate_items_to_partitions_flow() {
    let raw_items = vec![
        AllocatedPromptItem::new(
            1,
            "2026-10-23T09:00:00Z",
            "P1",
            QueuePriorityBand::Urgent,
            "",
        ),
        AllocatedPromptItem::new(
            2,
            "2026-10-23T09:05:00Z",
            "P2",
            QueuePriorityBand::Normal,
            "",
        ),
        AllocatedPromptItem::new(
            3,
            "2026-10-23T09:10:00Z",
            "P3",
            QueuePriorityBand::Urgent,
            "",
        ),
        AllocatedPromptItem::new(
            4,
            "2026-10-23T09:15:00Z",
            "P4",
            QueuePriorityBand::Normal,
            "",
        ),
        AllocatedPromptItem::new(
            5,
            "2026-10-23T09:20:00Z",
            "P5",
            QueuePriorityBand::Urgent,
            "",
        ),
    ];

    let buckets = vec![
        QueuePartitionBucket::new("part_urgent", "inst_01", QueuePriorityBand::Urgent, 2),
        QueuePartitionBucket::new("part_normal", "inst_01", QueuePriorityBand::Normal, 10),
    ];

    let report = allocate_items_to_partitions(&raw_items, &buckets);

    assert_eq!(report.total_allocated(), 4);
    assert!(report.is_fifo_compliant());

    let urgent_allocated = report.items_for_partition("part_urgent");
    assert_eq!(urgent_allocated.len(), 2);
    assert_eq!(urgent_allocated[0].id(), 1);
    assert_eq!(urgent_allocated[1].id(), 3);

    let normal_allocated = report.items_for_partition("part_normal");
    assert_eq!(normal_allocated.len(), 2);
    assert_eq!(normal_allocated[0].id(), 2);
    assert_eq!(normal_allocated[1].id(), 4);

    assert!(report.query_sql().contains("WHERE priority = 'urgent'"));
    assert!(report.query_sql().contains("WHERE priority = 'normal'"));
}

#[test]
fn test_allocate_items_to_partitions_detects_inverted_raw_items() {
    let inverted_raw = vec![
        AllocatedPromptItem::new(
            10,
            "2026-10-23T09:20:00Z",
            "Later",
            QueuePriorityBand::Urgent,
            "",
        ),
        AllocatedPromptItem::new(
            11,
            "2026-10-23T09:10:00Z",
            "Earlier",
            QueuePriorityBand::Urgent,
            "",
        ),
    ];

    let buckets = vec![QueuePartitionBucket::new(
        "part_test",
        "inst_01",
        QueuePriorityBand::Urgent,
        5,
    )];

    let report = allocate_items_to_partitions(&inverted_raw, &buckets);
    assert_eq!(report.total_allocated(), 2);
    assert!(!report.is_fifo_compliant());
}

#[test]
fn test_empty_partition_allocation_report() {
    let report = allocate_items_to_partitions(&[], &[]);
    assert_eq!(report.total_allocated(), 0);
    assert!(report.items().is_empty());
    assert!(report.is_fifo_compliant());
    assert!(report.query_sql().contains(CANONICAL_FIFO_ORDER_BY));
}

#[test]
fn test_clean_column_name_and_is_prompt_queue_table() {
    assert_eq!(clean_column_name("`created_at`"), "created_at");
    assert_eq!(clean_column_name("\"id\""), "id");
    assert_eq!(clean_column_name("t.created_at"), "created_at");
    assert_eq!(clean_column_name("`repo_prompts`.`id`"), "id");
    assert_eq!(clean_column_name("created_at,"), "created_at");

    for table in PROMPT_QUEUE_TABLES {
        assert!(is_prompt_queue_table(table));
        assert!(is_prompt_queue_table(&format!("`{table}`")));
        assert!(is_prompt_queue_table(&format!("\"{table}\"")));
    }
    assert!(!is_prompt_queue_table("users"));
    assert!(!is_prompt_queue_table("audit_log"));
}

#[test]
fn test_partition_allocation_error_display_and_conversion() {
    let err1 = PartitionAllocationError::InvertedLifoOrderingDetected;
    assert!(err1.to_string().contains("Inverted LIFO ordering detected"));

    let err2 = PartitionAllocationError::MissingTieBreakerId;
    assert!(err2
        .to_string()
        .contains("Missing mandatory tie-breaker 'id ASC'"));

    let err3 = PartitionAllocationError::InvalidPartitionLimit;
    assert!(err3.to_string().contains("Invalid partition limit"));

    let app_err: crate::error::AppError = err1.into();
    match app_err {
        crate::error::AppError::Config(msg) => {
            assert!(msg.contains("Inverted LIFO ordering detected"));
        }
        other => panic!("Expected AppError::Config, got: {other:?}"),
    }
}
