//! Unit tests for FIFO Queue Drain Monotonicity Verifier (`Imp-681`).

use super::fifo_queue_drain_monotonicity_verifier::*;

#[test]
fn test_verify_queue_items_strictly_monotonic() {
    let items = vec![
        QueueItemRecord::new(101, 1_000, "active_prompts"),
        QueueItemRecord::new(102, 1_000, "active_prompts"), // Identical timestamp, higher ID (compliant)
        QueueItemRecord::new(103, 1_500, "active_prompts"),
        QueueItemRecord::new(104, 2_000, "active_prompts"),
    ];

    let result = verify_queue_items_monotonicity(&items);
    assert!(result.is_ok());
    let report = result.unwrap();
    assert!(report.is_strictly_monotonic);
    assert_eq!(report.total_items_verified, 4);
    assert_eq!(report.min_created_at, Some(1_000));
    assert_eq!(report.max_created_at, Some(2_000));
}

#[test]
fn test_detect_timestamp_regression() {
    let items = vec![
        QueueItemRecord::new(101, 2_000, "prompt_backups"),
        QueueItemRecord::new(102, 1_500, "prompt_backups"), // Regressed from 2000 to 1500!
    ];

    let result = verify_queue_items_monotonicity(&items);
    assert!(result.is_err());
    match result.unwrap_err() {
        MonotonicityViolation::TimestampRegression { prev_id, curr_id, .. } => {
            assert_eq!(prev_id, 101);
            assert_eq!(curr_id, 102);
        }
        other => panic!("Unexpected violation: {other:?}"),
    }
}

#[test]
fn test_detect_id_tie_breaker_inversion() {
    let items = vec![
        QueueItemRecord::new(205, 1_000, "active_prompts"),
        QueueItemRecord::new(204, 1_000, "active_prompts"), // Same timestamp, lower ID (inversion!)
    ];

    let result = verify_queue_items_monotonicity(&items);
    assert!(result.is_err());
    match result.unwrap_err() {
        MonotonicityViolation::IdTieBreakerInversion { prev_id, curr_id, created_at, .. } => {
            assert_eq!(prev_id, 205);
            assert_eq!(curr_id, 204);
            assert_eq!(created_at, 1_000);
        }
        other => panic!("Unexpected violation: {other:?}"),
    }
}

#[test]
fn test_detect_duplicate_item_id() {
    let items = vec![
        QueueItemRecord::new(300, 1_000, "repo_prompts"),
        QueueItemRecord::new(300, 1_000, "repo_prompts"),
    ];

    let result = verify_queue_items_monotonicity(&items);
    assert!(result.is_err());
    match result.unwrap_err() {
        MonotonicityViolation::DuplicateItemId { id, .. } => {
            assert_eq!(id, 300);
        }
        other => panic!("Unexpected violation: {other:?}"),
    }
}

#[test]
fn test_audit_chunked_batch_stream_end_to_end() {
    let batch1 = vec![
        QueueItemRecord::new(1, 100, "active_prompts"),
        QueueItemRecord::new(2, 200, "active_prompts"),
    ];
    let batch2 = vec![
        QueueItemRecord::new(3, 300, "active_prompts"),
        QueueItemRecord::new(4, 400, "active_prompts"),
    ];

    let report = audit_chunked_batch_stream(&[batch1, batch2]);
    assert!(report.is_strictly_monotonic);
    assert_eq!(report.total_items_verified, 4);
    assert_eq!(report.min_created_at, Some(100));
    assert_eq!(report.max_created_at, Some(400));
}
