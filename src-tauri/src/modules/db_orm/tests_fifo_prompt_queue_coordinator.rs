//! Unit tests for FIFO Prompt Queue Coordinator (`Imp-622`).

use super::fifo_prompt_queue_coordinator::{
    FifoPromptItem, FifoPromptQueueCoordinator, FifoQueryValidator, QueueItemStatus,
};

#[test]
fn test_dispatch_query_sql_validation_success() {
    let sql = FifoPromptQueueCoordinator::build_dispatch_query("active_prompts");
    let result = FifoQueryValidator::validate_dispatch_query_sql(&sql);
    assert!(result.is_ok());
    assert!(sql.contains("ORDER BY created_at ASC, id ASC"));
}

#[test]
fn test_restore_query_sql_validation_success() {
    let sql = FifoPromptQueueCoordinator::build_restore_query("prompt_backups");
    let result = FifoQueryValidator::validate_dispatch_query_sql(&sql);
    assert!(result.is_ok());
    assert!(sql.contains("ORDER BY created_at ASC, id ASC"));
}

#[test]
fn test_forbidden_lifo_query_rejected() {
    let bad_sql_1 = "SELECT * FROM active_prompts ORDER BY updated_at DESC LIMIT 10";
    let res1 = FifoQueryValidator::validate_dispatch_query_sql(bad_sql_1);
    assert!(res1.is_err());
    assert!(res1.unwrap_err().to_string().contains("Forbidden LIFO ordering"));

    let bad_sql_2 = "SELECT * FROM active_prompts ORDER BY created_at DESC LIMIT 10";
    let res2 = FifoQueryValidator::validate_dispatch_query_sql(bad_sql_2);
    assert!(res2.is_err());

    let bad_sql_3 = "SELECT * FROM active_prompts ORDER BY rowid DESC LIMIT 10";
    let res3 = FifoQueryValidator::validate_dispatch_query_sql(bad_sql_3);
    assert!(res3.is_err());
}

#[test]
fn test_missing_fifo_ordering_rejected() {
    let sql_no_order = "SELECT * FROM active_prompts WHERE status = 'Enqueued'";
    let res = FifoQueryValidator::validate_dispatch_query_sql(sql_no_order);
    assert!(res.is_err());
    assert!(res.unwrap_err().to_string().contains("Missing mandatory FIFO ordering"));
}

#[test]
fn test_in_memory_fifo_queue_ordering() {
    let mut coordinator = FifoPromptQueueCoordinator::new();

    // Enqueue out of order
    let p3 = FifoPromptItem::new("p3", "c1", "Third prompt", 300);
    let p1 = FifoPromptItem::new("p1", "c1", "First prompt", 100);
    let p2 = FifoPromptItem::new("p2", "c1", "Second prompt", 200);

    coordinator.enqueue(p3).expect("enqueue p3");
    coordinator.enqueue(p1).expect("enqueue p1");
    coordinator.enqueue(p2).expect("enqueue p2");

    assert_eq!(coordinator.len(), 3);

    // Lease batch of 2 at t = 350
    let leased = coordinator.lease_batch("instance_1", 2, 5000, 350);
    assert_eq!(leased.len(), 2);
    assert_eq!(leased[0].id, "p1"); // oldest created_at = 100
    assert_eq!(leased[1].id, "p2"); // second oldest = 200
    assert_eq!(leased[0].status, QueueItemStatus::Leased);
    assert_eq!(leased[0].leased_by_instance.as_deref(), Some("instance_1"));

    // Lease next batch of 2
    let leased_next = coordinator.lease_batch("instance_2", 2, 5000, 350);
    assert_eq!(leased_next.len(), 1);
    assert_eq!(leased_next[0].id, "p3"); // created_at = 300

    // Complete p1
    let completed = coordinator.complete("p1").expect("complete p1");
    assert_eq!(completed.status, QueueItemStatus::Completed);
    assert_eq!(coordinator.len(), 2);
}

#[test]
fn test_lease_expiration_and_reclaim() {
    let mut coordinator = FifoPromptQueueCoordinator::new();
    let p1 = FifoPromptItem::new("p1", "c1", "Prompt", 100);
    coordinator.enqueue(p1).expect("enqueue");

    // Lease with 1000ms duration at t = 1000 -> expires at 2000
    let leased = coordinator.lease_batch("inst_1", 1, 1000, 1000);
    assert_eq!(leased.len(), 1);

    // At t = 1500, not expired
    let reclaimed_early = coordinator.reclaim_expired_leases(1500);
    assert_eq!(reclaimed_early, 0);

    // At t = 2001, expired and reclaimed
    let reclaimed_late = coordinator.reclaim_expired_leases(2001);
    assert_eq!(reclaimed_late, 1);

    // Should now be re-leasable
    let re_leased = coordinator.lease_batch("inst_2", 1, 1000, 2005);
    assert_eq!(re_leased.len(), 1);
    assert_eq!(re_leased[0].leased_by_instance.as_deref(), Some("inst_2"));
    assert_eq!(re_leased[0].attempt_count, 2);
}
