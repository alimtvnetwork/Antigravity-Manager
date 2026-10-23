//! Comprehensive unit tests for `FifoQueueBatchDrainGuard` (`Imp-664`).

use super::fifo_queue_batch_drain_guard::{
    build_batch_chunk_query, drain_chunks_safely, validate_chunk_query_ordering, BatchDrainReport,
    BatchDrainViolation, DrainBatchSpec, DrainedItem, MANDATORY_FIFO_ORDER_CLAUSE,
};

#[test]
fn test_build_batch_chunk_query_structure() {
    let query = build_batch_chunk_query("active_prompts", 50, 100);
    let expected = format!(
        "SELECT id, created_at, payload FROM active_prompts {} LIMIT 50 OFFSET 100;",
        MANDATORY_FIFO_ORDER_CLAUSE
    );

    assert_eq!(query, expected);
    assert!(query.contains("ORDER BY created_at ASC, id ASC"));
    assert!(validate_chunk_query_ordering(&query).is_ok());
}

#[test]
fn test_validate_chunk_query_ordering_valid_queries() {
    let sql1 = "SELECT id, created_at, payload FROM active_prompts ORDER BY created_at ASC, id ASC LIMIT 50 OFFSET 0;";
    assert!(validate_chunk_query_ordering(sql1).is_ok());

    let sql2 = "select id, created_at, payload from repo_prompts order by created_at asc, id asc limit 10 offset 20;";
    assert!(validate_chunk_query_ordering(sql2).is_ok());

    let sql3 = "SELECT * FROM prompt_backups ORDER    BY   created_at   ASC ,   id   ASC LIMIT 10;";
    assert!(validate_chunk_query_ordering(sql3).is_ok());

    let sql4 = "SELECT t.id, t.created_at, t.payload FROM active_prompts t ORDER BY t.created_at ASC, t.id ASC LIMIT 10;";
    assert!(validate_chunk_query_ordering(sql4).is_ok());
}

#[test]
fn test_validate_chunk_query_ordering_rejects_lifo_desc() {
    let sql_lifo1 = "SELECT * FROM active_prompts ORDER BY updated_at DESC LIMIT 10;";
    match validate_chunk_query_ordering(sql_lifo1) {
        Err(BatchDrainViolation::LifoOrderingForbidden { query }) => {
            assert_eq!(query, sql_lifo1);
        }
        other => panic!("Expected LifoOrderingForbidden, got {:?}", other),
    }

    let sql_lifo2 = "SELECT * FROM prompt_backups ORDER BY created_at DESC, id ASC;";
    match validate_chunk_query_ordering(sql_lifo2) {
        Err(BatchDrainViolation::LifoOrderingForbidden { query }) => {
            assert_eq!(query, sql_lifo2);
        }
        other => panic!("Expected LifoOrderingForbidden, got {:?}", other),
    }

    let sql_lifo3 = "SELECT * FROM repo_prompts ORDER BY created_at ASC, id DESC;";
    match validate_chunk_query_ordering(sql_lifo3) {
        Err(BatchDrainViolation::LifoOrderingForbidden { query }) => {
            assert_eq!(query, sql_lifo3);
        }
        other => panic!("Expected LifoOrderingForbidden, got {:?}", other),
    }
}

#[test]
fn test_validate_chunk_query_ordering_rejects_missing_tiebreaker() {
    let sql_missing_tie = "SELECT * FROM active_prompts ORDER BY created_at ASC LIMIT 10;";
    match validate_chunk_query_ordering(sql_missing_tie) {
        Err(BatchDrainViolation::MissingOrderTieBreaker { query }) => {
            assert_eq!(query, sql_missing_tie);
        }
        other => panic!("Expected MissingOrderTieBreaker, got {:?}", other),
    }

    let sql_no_order = "SELECT * FROM active_prompts LIMIT 50;";
    match validate_chunk_query_ordering(sql_no_order) {
        Err(BatchDrainViolation::MissingOrderTieBreaker { query }) => {
            assert_eq!(query, sql_no_order);
        }
        other => panic!("Expected MissingOrderTieBreaker, got {:?}", other),
    }

    let sql_only_id = "SELECT * FROM active_prompts ORDER BY id ASC LIMIT 50;";
    match validate_chunk_query_ordering(sql_only_id) {
        Err(BatchDrainViolation::MissingOrderTieBreaker { query }) => {
            assert_eq!(query, sql_only_id);
        }
        other => panic!("Expected MissingOrderTieBreaker, got {:?}", other),
    }
}

#[test]
fn test_validate_chunk_query_ordering_does_not_false_positive_on_description_column() {
    let sql = "SELECT id, prompt_description, created_at, payload FROM active_prompts ORDER BY created_at ASC, id ASC LIMIT 10;";
    assert!(validate_chunk_query_ordering(sql).is_ok());
}

#[test]
fn test_drain_chunks_safely_valid_multi_chunk() {
    let chunk0 = vec![
        DrainedItem::new(1, 1000, "item_alpha"),
        DrainedItem::new(2, 1000, "item_beta"),
        DrainedItem::new(3, 1050, "item_gamma"),
    ];
    let chunk1 = vec![
        DrainedItem::new(4, 1050, "item_delta"),
        DrainedItem::new(5, 1100, "item_epsilon"),
    ];
    let chunk2 = vec![DrainedItem::new(6, 1200, "item_zeta")];

    let chunks = vec![chunk0, chunk1, chunk2];
    let report: BatchDrainReport = drain_chunks_safely(&chunks);

    assert!(report.is_fifo_maintained());
    assert_eq!(report.total_chunks_processed(), 3);
    assert_eq!(report.total_records_drained(), 6);
    assert_eq!(report.drained_items().len(), 6);
    assert!(report.violation().is_none());
    assert!(!report.has_violation());
    assert_eq!(report.drained_items()[0].id(), 1);
    assert_eq!(report.drained_items()[5].id(), 6);
}

#[test]
fn test_drain_chunks_safely_detects_timestamp_inversion() {
    let chunk0 = vec![
        DrainedItem::new(1, 2000, "item_1"),
        DrainedItem::new(2, 2050, "item_2"),
    ];
    let chunk1 = vec![DrainedItem::new(3, 2020, "item_3_inverted")];

    let chunks = vec![chunk0, chunk1];
    let report = drain_chunks_safely(&chunks);

    assert!(!report.is_fifo_maintained());
    assert_eq!(report.total_chunks_processed(), 2);
    assert_eq!(report.total_records_drained(), 2);
    assert!(report.has_violation());

    match report.violation() {
        Some(BatchDrainViolation::ChunkSequenceInversion {
            previous_created_at,
            next_created_at,
        }) => {
            assert_eq!(*previous_created_at, 2050);
            assert_eq!(*next_created_at, 2020);
        }
        other => panic!("Expected ChunkSequenceInversion, got {:?}", other),
    }
}

#[test]
fn test_drain_chunks_safely_detects_id_inversion_at_same_timestamp() {
    let chunk0 = vec![
        DrainedItem::new(10, 5000, "item_high_id"),
        DrainedItem::new(5, 5000, "item_low_id_inverted"),
    ];

    let chunks = vec![chunk0];
    let report = drain_chunks_safely(&chunks);

    assert!(!report.is_fifo_maintained());
    assert_eq!(report.total_chunks_processed(), 1);
    assert_eq!(report.total_records_drained(), 1);
    assert!(report.has_violation());

    match report.violation() {
        Some(BatchDrainViolation::ChunkSequenceInversion {
            previous_created_at,
            next_created_at,
        }) => {
            assert_eq!(*previous_created_at, 5000);
            assert_eq!(*next_created_at, 5000);
        }
        other => panic!("Expected ChunkSequenceInversion, got {:?}", other),
    }
}

#[test]
fn test_drain_chunks_safely_empty_and_single_element_chunks() {
    let empty_report = drain_chunks_safely(&[]);
    assert!(empty_report.is_fifo_maintained());
    assert_eq!(empty_report.total_chunks_processed(), 0);
    assert_eq!(empty_report.total_records_drained(), 0);
    assert!(empty_report.drained_items().is_empty());

    let empty_vec_chunks: Vec<Vec<DrainedItem>> = vec![vec![], vec![]];
    let empty_chunks_report = drain_chunks_safely(&empty_vec_chunks);
    assert!(empty_chunks_report.is_fifo_maintained());
    assert_eq!(empty_chunks_report.total_chunks_processed(), 2);
    assert_eq!(empty_chunks_report.total_records_drained(), 0);

    let single_chunk = vec![vec![DrainedItem::new(1, 100, "item_solo")]];
    let single_report = drain_chunks_safely(&single_chunk);
    assert!(single_report.is_fifo_maintained());
    assert_eq!(single_report.total_chunks_processed(), 1);
    assert_eq!(single_report.total_records_drained(), 1);
}

#[test]
fn test_batch_drain_violation_display_formatting() {
    let v_lifo = BatchDrainViolation::LifoOrderingForbidden {
        query: "SELECT * FROM active_prompts ORDER BY updated_at DESC;".to_string(),
    };
    let s_lifo = format!("{}", v_lifo);
    assert!(s_lifo.contains("LIFO ordering is forbidden"));
    assert!(s_lifo.contains("updated_at DESC"));

    let v_tie = BatchDrainViolation::MissingOrderTieBreaker {
        query: "SELECT * FROM prompt_backups;".to_string(),
    };
    let s_tie = format!("{}", v_tie);
    assert!(s_tie.contains("Missing mandatory secondary tie-breaker 'id ASC'"));

    let v_inv = BatchDrainViolation::ChunkSequenceInversion {
        previous_created_at: 200,
        next_created_at: 100,
    };
    let s_inv = format!("{}", v_inv);
    assert!(s_inv.contains("FIFO chunk sequence inversion detected"));
    assert!(s_inv.contains("200"));
    assert!(s_inv.contains("100"));

    use std::error::Error;
    assert!(v_lifo.source().is_none());
}

#[test]
fn test_drain_batch_spec_and_drained_item_api() {
    let spec = DrainBatchSpec::new("active_prompts", 100, 500);
    assert_eq!(spec.table_name(), "active_prompts");
    assert_eq!(spec.chunk_size(), 100);
    assert_eq!(spec.max_total_records(), 500);

    let chunk_query = spec.build_chunk_query(200);
    assert!(chunk_query.contains("LIMIT 100 OFFSET 200"));
    assert!(chunk_query.contains("ORDER BY created_at ASC, id ASC"));

    let item = DrainedItem::new(77, 999999, "task_payload");
    assert_eq!(item.id(), 77);
    assert_eq!(item.created_at(), 999999);
    assert_eq!(item.payload(), "task_payload");
}
