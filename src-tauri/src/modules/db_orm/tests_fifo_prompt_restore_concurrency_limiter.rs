//! Comprehensive unit tests for `fifo_prompt_restore_concurrency_limiter` (`Task-210`).
//!
//! Validates FIFO prompt restore concurrency regulation across `active_prompts`,
//! `prompt_backups`, and `repo_prompts`, ensuring single-permit exclusivity, multi-permit
//! bounding, automatic expiration purging, and multi-table isolation.

use super::fifo_prompt_restore_concurrency_limiter::{
    acquire_restore_lease, count_active_leases, find_active_lease, has_active_lease,
    purge_expired_leases, release_restore_lease, ConcurrencyLimiterViolation, LimiterState,
    RestoreLeaseRecord, RestoreQueueTable,
};
use std::error::Error;

#[test]
fn test_restore_queue_table_variants_and_names() {
    assert_eq!(
        RestoreQueueTable::ActivePrompts.as_table_name(),
        "active_prompts"
    );
    assert_eq!(
        RestoreQueueTable::PromptBackups.as_table_name(),
        "prompt_backups"
    );
    assert_eq!(
        RestoreQueueTable::RepoPrompts.as_table_name(),
        "repo_prompts"
    );

    assert_eq!(RestoreQueueTable::ActivePrompts.as_str(), "active_prompts");
    assert_eq!(RestoreQueueTable::PromptBackups.as_str(), "prompt_backups");
    assert_eq!(RestoreQueueTable::RepoPrompts.as_str(), "repo_prompts");

    assert_eq!(
        format!("{}", RestoreQueueTable::ActivePrompts),
        "active_prompts"
    );
    assert_eq!(
        format!("{}", RestoreQueueTable::PromptBackups),
        "prompt_backups"
    );
    assert_eq!(
        format!("{}", RestoreQueueTable::RepoPrompts),
        "repo_prompts"
    );

    let all_tables = RestoreQueueTable::all();
    assert_eq!(all_tables.len(), 3);
    assert_eq!(all_tables[0], RestoreQueueTable::ActivePrompts);
    assert_eq!(all_tables[1], RestoreQueueTable::PromptBackups);
    assert_eq!(all_tables[2], RestoreQueueTable::RepoPrompts);
}

#[test]
fn test_successful_lease_acquisition_and_release() {
    let mut state = LimiterState::default();
    assert!(!has_active_lease(
        &state,
        RestoreQueueTable::ActivePrompts,
        1_000
    ));

    let lease_res = acquire_restore_lease(
        &mut state,
        RestoreQueueTable::ActivePrompts,
        "task-1",
        1_000,
    );
    assert!(lease_res.is_ok());

    let lease = lease_res.unwrap();
    assert_eq!(lease.table, RestoreQueueTable::ActivePrompts);
    assert_eq!(lease.holder_task_id, "task-1");
    assert_eq!(lease.acquired_epoch_ms, 1_000);
    assert_eq!(lease.expires_epoch_ms, 31_000);

    assert!(has_active_lease(
        &state,
        RestoreQueueTable::ActivePrompts,
        1_000
    ));
    assert_eq!(
        count_active_leases(&state, RestoreQueueTable::ActivePrompts, 1_000),
        1
    );

    let released = release_restore_lease(&mut state, &lease.lease_id);
    assert!(released);
    assert!(!has_active_lease(
        &state,
        RestoreQueueTable::ActivePrompts,
        1_000
    ));

    let released_again = release_restore_lease(&mut state, &lease.lease_id);
    assert!(!released_again);
}

#[test]
fn test_rejection_of_concurrent_restore_conflict_single_permit() {
    let mut state = LimiterState::default();
    let lease1 = acquire_restore_lease(
        &mut state,
        RestoreQueueTable::ActivePrompts,
        "worker-alpha",
        5_000,
    );
    assert!(lease1.is_ok());

    let conflict_res = acquire_restore_lease(
        &mut state,
        RestoreQueueTable::ActivePrompts,
        "worker-beta",
        6_000,
    );
    assert!(conflict_res.is_err());

    let err = conflict_res.unwrap_err();
    match err {
        ConcurrencyLimiterViolation::ConcurrentRestoreConflict {
            table,
            current_holder,
            expires_in_ms,
        } => {
            assert_eq!(table, RestoreQueueTable::ActivePrompts);
            assert_eq!(current_holder, "worker-alpha");
            assert_eq!(expires_in_ms, 29_000);
        }
        other => panic!("expected ConcurrentRestoreConflict, got: {other:?}"),
    }
}

#[test]
fn test_rejection_when_max_permits_exceeded_multi_permit() {
    let mut state = LimiterState::new(2, 10_000);

    let lease1 = acquire_restore_lease(&mut state, RestoreQueueTable::RepoPrompts, "task-1", 100);
    assert!(lease1.is_ok());

    let lease2 = acquire_restore_lease(&mut state, RestoreQueueTable::RepoPrompts, "task-2", 200);
    assert!(lease2.is_ok());
    assert_eq!(
        count_active_leases(&state, RestoreQueueTable::RepoPrompts, 200),
        2
    );

    let exceed_res =
        acquire_restore_lease(&mut state, RestoreQueueTable::RepoPrompts, "task-3", 300);
    assert!(exceed_res.is_err());

    match exceed_res.unwrap_err() {
        ConcurrencyLimiterViolation::MaxPermitsExceeded { table, max_permits } => {
            assert_eq!(table, RestoreQueueTable::RepoPrompts);
            assert_eq!(max_permits, 2);
        }
        other => panic!("expected MaxPermitsExceeded, got: {other:?}"),
    }

    let mut zero_state = LimiterState::new(0, 10_000);
    let zero_res = acquire_restore_lease(
        &mut zero_state,
        RestoreQueueTable::PromptBackups,
        "task-x",
        100,
    );
    assert!(matches!(
        zero_res,
        Err(ConcurrencyLimiterViolation::MaxPermitsExceeded { max_permits: 0, .. })
    ));
}

#[test]
fn test_automatic_expiration_of_old_leases() {
    let mut state = LimiterState::new(1, 5_000);

    let lease1 = acquire_restore_lease(
        &mut state,
        RestoreQueueTable::PromptBackups,
        "task-old",
        1_000,
    );
    assert!(lease1.is_ok());

    assert!(has_active_lease(
        &state,
        RestoreQueueTable::PromptBackups,
        3_000
    ));
    assert_eq!(
        count_active_leases(&state, RestoreQueueTable::PromptBackups, 3_000),
        1
    );

    let blocked_res = acquire_restore_lease(
        &mut state,
        RestoreQueueTable::PromptBackups,
        "task-new",
        3_000,
    );
    assert!(blocked_res.is_err());

    assert!(!has_active_lease(
        &state,
        RestoreQueueTable::PromptBackups,
        6_000
    ));

    let success_res = acquire_restore_lease(
        &mut state,
        RestoreQueueTable::PromptBackups,
        "task-new",
        6_001,
    );
    assert!(success_res.is_ok());

    let new_lease = success_res.unwrap();
    assert_eq!(new_lease.holder_task_id, "task-new");
    assert_eq!(new_lease.acquired_epoch_ms, 6_001);
    assert_eq!(new_lease.expires_epoch_ms, 11_001);
}

#[test]
fn test_multi_table_isolation() {
    let mut state = LimiterState::default();

    let lease_active = acquire_restore_lease(
        &mut state,
        RestoreQueueTable::ActivePrompts,
        "worker-1",
        500,
    );
    let lease_backups = acquire_restore_lease(
        &mut state,
        RestoreQueueTable::PromptBackups,
        "worker-2",
        500,
    );
    let lease_repo =
        acquire_restore_lease(&mut state, RestoreQueueTable::RepoPrompts, "worker-3", 500);

    assert!(lease_active.is_ok());
    assert!(lease_backups.is_ok());
    assert!(lease_repo.is_ok());

    assert!(has_active_lease(
        &state,
        RestoreQueueTable::ActivePrompts,
        500
    ));
    assert!(has_active_lease(
        &state,
        RestoreQueueTable::PromptBackups,
        500
    ));
    assert!(has_active_lease(
        &state,
        RestoreQueueTable::RepoPrompts,
        500
    ));

    assert!(release_restore_lease(
        &mut state,
        &lease_active.unwrap().lease_id
    ));
    assert!(!has_active_lease(
        &state,
        RestoreQueueTable::ActivePrompts,
        500
    ));
    assert!(has_active_lease(
        &state,
        RestoreQueueTable::PromptBackups,
        500
    ));
    assert!(has_active_lease(
        &state,
        RestoreQueueTable::RepoPrompts,
        500
    ));
}

#[test]
fn test_display_and_error_formatting() {
    let conflict = ConcurrencyLimiterViolation::ConcurrentRestoreConflict {
        table: RestoreQueueTable::ActivePrompts,
        current_holder: "worker-omega".to_string(),
        expires_in_ms: 12_345,
    };
    let formatted_conflict = format!("{conflict}");
    assert!(formatted_conflict.contains("Concurrent restore conflict on table 'active_prompts'"));
    assert!(formatted_conflict.contains("worker-omega"));
    assert!(formatted_conflict.contains("12345 ms"));

    let exceeded = ConcurrencyLimiterViolation::MaxPermitsExceeded {
        table: RestoreQueueTable::PromptBackups,
        max_permits: 4,
    };
    let formatted_exceeded = format!("{exceeded}");
    assert!(formatted_exceeded.contains("Max concurrent restore permits (4) exceeded"));
    assert!(formatted_exceeded.contains("prompt_backups"));

    let dyn_err: &dyn Error = &conflict;
    assert!(dyn_err.to_string().contains("Concurrent restore conflict"));
}

#[test]
fn test_lease_record_helpers_and_expiration() {
    let record = RestoreLeaseRecord::new(
        "lease-test-1",
        RestoreQueueTable::RepoPrompts,
        "task-inspect",
        10_000,
        20_000,
    );

    assert_eq!(record.lease_id, "lease-test-1");
    assert_eq!(record.table, RestoreQueueTable::RepoPrompts);
    assert_eq!(record.holder_task_id, "task-inspect");
    assert_eq!(record.acquired_epoch_ms, 10_000);
    assert_eq!(record.expires_epoch_ms, 20_000);

    assert!(record.is_active(15_000));
    assert!(!record.is_expired(15_000));
    assert_eq!(record.remaining_ms(15_000), 5_000);

    assert!(!record.is_active(20_000));
    assert!(record.is_expired(20_000));
    assert_eq!(record.remaining_ms(20_000), 0);

    assert!(!record.is_active(25_000));
    assert!(record.is_expired(25_000));
    assert_eq!(record.remaining_ms(25_000), 0);
}

#[test]
fn test_find_active_lease_and_purge_count() {
    let mut state = LimiterState::new(3, 10_000);

    let l1 =
        acquire_restore_lease(&mut state, RestoreQueueTable::ActivePrompts, "t1", 1_000).unwrap();
    let _l2 =
        acquire_restore_lease(&mut state, RestoreQueueTable::ActivePrompts, "t2", 2_000).unwrap();

    let found = find_active_lease(&state, RestoreQueueTable::ActivePrompts, 5_000);
    assert!(found.is_some());
    assert_eq!(found.unwrap().lease_id, l1.lease_id);

    let purged = purge_expired_leases(&mut state, 15_000);
    assert_eq!(purged, 2);
    assert_eq!(state.active_leases.len(), 0);
    assert!(find_active_lease(&state, RestoreQueueTable::ActivePrompts, 15_000).is_none());
}
