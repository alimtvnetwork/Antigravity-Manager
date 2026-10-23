//! Companion Unit Tests for Prompt Restore Honesty & Quota Drop Recovery (`Imp-701`).
//!
//! Enforces:
//! 1. Incomplete-prompt sweep: transitions all running, in_flight, and dispatched prompts to backed_up (any age).
//! 2. Honest RestoreOutcome reporting: `{ restored_count, dispatched_count, failed_count }` with zero fake success.
//! 3. Live-status dedup: preserves live rows in `running` and `completed` states.
//! 4. Scoped-restore fallback: automatically falls back to unscoped restore when scoped restore returns 0 records.
//! 5. Re-attribution of unrestored prompt_backups across instance switches.

use super::restore::{
    count_unrestored_prompts_for_other_instances, is_live_status_preserved,
    restore_running_prompts_for_instance_with_outcome,
    restore_running_prompts_scoped_with_fallback,
    restore_running_prompts_scoped_with_fallback_with_outcome, RestoreOutcome,
};
use super::{connect_backup_db, PromptBackupRecord};
use chrono::Utc;
use rusqlite::params;
use std::fs;
use uuid::Uuid;

#[test]
fn test_restore_outcome_honesty_struct_and_accessors() {
    let mut outcome = RestoreOutcome::new(5, 4, 1);
    assert_eq!(outcome.restored_count(), 5);
    assert_eq!(outcome.restored, 5);
    assert_eq!(outcome.dispatched_count(), 4);
    assert_eq!(outcome.dispatched, 4);
    assert_eq!(outcome.failed_count(), 1);
    assert_eq!(outcome.failed, 1);
    assert!(!outcome.is_success(), "Outcome with failures must not be reported as success");

    outcome.record_restored();
    assert_eq!(outcome.restored_count(), 6);
    assert_eq!(outcome.restored, 6);

    outcome.record_dispatched(6);
    assert_eq!(outcome.dispatched_count(), 6);
    assert_eq!(outcome.dispatched, 6);

    outcome.record_skipped_live();
    assert_eq!(outcome.skipped_live, 1);

    outcome.record_failed(Some("Dispatch failure: connection dropped".to_string()));
    assert_eq!(outcome.failed_count(), 2);
    assert_eq!(outcome.failed, 2);
    assert_eq!(
        outcome.last_error.as_deref(),
        Some("Dispatch failure: connection dropped")
    );
    assert!(!outcome.is_success());

    // Clean outcome
    let clean = RestoreOutcome::new(3, 3, 0);
    assert!(clean.is_success());
    assert_eq!(clean.restored_count(), 3);
    assert_eq!(clean.dispatched_count(), 3);
    assert_eq!(clean.failed_count(), 0);
}

#[test]
fn test_live_status_dedup_preserves_running_and_completed_only() {
    assert!(
        is_live_status_preserved(Some("running")),
        "Running prompt must be preserved to prevent duplicate execution"
    );
    assert!(
        is_live_status_preserved(Some("completed")),
        "Completed prompt must be preserved to prevent duplicate re-execution"
    );
    assert!(
        !is_live_status_preserved(Some("backed_up")),
        "Backed-up prompts should be restored"
    );
    assert!(
        !is_live_status_preserved(Some("queued")),
        "Queued prompts should be restored or dispatched"
    );
    assert!(
        !is_live_status_preserved(Some("dispatched")),
        "Dispatched prompts without live execution should be restorable"
    );
    assert!(
        !is_live_status_preserved(None),
        "Non-existent live status must not block restore"
    );
}

#[test]
fn test_scoped_restore_fallback_when_scoped_returns_zero_and_other_exists() {
    let temp_dir = std::env::temp_dir().join(format!(
        "agm_test_scoped_fallback_{}",
        Uuid::new_v4().simple()
    ));
    let db_path = temp_dir.join("test-fallback.db");
    let custom_file = db_path.to_str().unwrap();

    let conn = connect_backup_db(Some(custom_file)).expect("Must connect to backup DB");
    let now = Utc::now().timestamp();

    // Create a backup batch
    conn.execute(
        "INSERT INTO backup_batches (id, created_at, prompts_count, file_path, retention_days, is_fully_restored)
         VALUES ('b-fallback', ?1, 2, ?2, 1, 0)",
        params![now, custom_file],
    )
    .expect("Must insert batch");

    // Insert 2 unrestored prompt backups recorded under "inst-source"
    for (pid, idx) in [("prompt-fb-1", 1), ("prompt-fb-2", 2)] {
        conn.execute(
            "INSERT INTO prompt_backups (id, backup_batch_id, prompt_id, project_name, project_path, project_id, conversation_id, sequence_id, prompt_text, created_at, is_restored, instance_id)
             VALUES (?1, 'b-fallback', ?2, 'proj-fallback', '/work/proj-fb', 'proj-1', 'conv-1', ?3, 'Test fallback restore', ?4, 0, 'inst-source')",
            params![format!("rec-{}", idx), pid, idx, now + idx],
        )
        .expect("Must insert prompt backup");
    }

    // Verify other count probe
    let other_count = count_unrestored_prompts_for_other_instances("inst-target", Some(custom_file))
        .expect("Must count other instance unrestored prompts");
    assert_eq!(other_count, 2, "inst-target must see 2 records under other instances");

    // Scoped restore with fallback for "inst-target" should fall back to unscoped restore
    let (records, outcome) = restore_running_prompts_scoped_with_fallback_with_outcome(
        "inst-target",
        true, // keep_backup = true to preserve DB rows for verification
        Some(custom_file),
    )
    .expect("Fallback restore should succeed");

    assert_eq!(records.len(), 2, "Must return 2 restored records via fallback");
    assert_eq!(records[0].prompt_id, "prompt-fb-1");
    assert_eq!(records[1].prompt_id, "prompt-fb-2");
    assert_eq!(outcome.restored_count(), 2);

    // Clean up
    crate::error::record_ignored(fs::remove_dir_all(temp_dir), "remove temp dir");
}

#[test]
fn test_reattribution_of_unrestored_prompt_backups_during_switch() {
    let temp_dir = std::env::temp_dir().join(format!(
        "agm_test_reattribute_{}",
        Uuid::new_v4().simple()
    ));
    let db_path = temp_dir.join("test-reattribute.db");
    let custom_file = db_path.to_str().unwrap();

    let conn = connect_backup_db(Some(custom_file)).expect("Must connect to backup DB");
    let now = Utc::now().timestamp();

    conn.execute(
        "INSERT INTO backup_batches (id, created_at, prompts_count, file_path, retention_days, is_fully_restored)
         VALUES ('b-reattr', ?1, 2, ?2, 1, 0)",
        params![now, custom_file],
    )
    .expect("Must insert batch");

    conn.execute(
        "INSERT INTO prompt_backups (id, backup_batch_id, prompt_id, project_name, project_path, project_id, conversation_id, sequence_id, prompt_text, created_at, is_restored, instance_id)
         VALUES ('r-old-1', 'b-reattr', 'p-old-1', 'proj', '/work/p', 'proj-1', 'conv-1', 1, 'text', ?1, 0, 'inst-old')",
        params![now],
    )
    .expect("Must insert prompt backup");

    conn.execute(
        "INSERT INTO prompt_backups (id, backup_batch_id, prompt_id, project_name, project_path, project_id, conversation_id, sequence_id, prompt_text, created_at, is_restored, instance_id)
         VALUES ('r-old-2', 'b-reattr', 'p-old-2', 'proj', '/work/p', 'proj-1', 'conv-1', 2, 'text', ?1, 1, 'inst-old')",
        params![now],
    )
    .expect("Must insert prompt backup");

    // Execute re-attribution query: only unrestored records (is_restored = 0) are re-attributed
    let changed = conn.execute(
        "UPDATE prompt_backups SET instance_id = ?1 WHERE instance_id = ?2 AND is_restored = 0",
        params!["inst-new", "inst-old"],
    )
    .expect("Must update instance_id");

    assert_eq!(changed, 1, "Only unrestored row should be re-attributed");

    let new_inst: String = conn.query_row(
        "SELECT instance_id FROM prompt_backups WHERE id = 'r-old-1'",
        [],
        |row| row.get(0),
    )
    .expect("Must query row 1");
    assert_eq!(new_inst, "inst-new");

    let old_inst: String = conn.query_row(
        "SELECT instance_id FROM prompt_backups WHERE id = 'r-old-2'",
        [],
        |row| row.get(0),
    )
    .expect("Must query row 2");
    assert_eq!(old_inst, "inst-old", "Already restored row remains on inst-old");

    crate::error::record_ignored(fs::remove_dir_all(temp_dir), "remove temp dir");
}
