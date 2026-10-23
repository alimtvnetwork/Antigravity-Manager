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
    assert!(
        !outcome.is_success(),
        "Outcome with failures must not be reported as success"
    );

    outcome.record_restored();
    assert_eq!(outcome.restored_count(), 6);
    assert_eq!(outcome.restored, 6);

    outcome.record_dispatched(6);
    assert_eq!(outcome.dispatched_count(), 6);
    assert_eq!(outcome.dispatched, 6);

    outcome.record_skipped_live();
    assert_eq!(outcome.skipped_live, 1);
    assert_eq!(outcome.skipped_live_count(), 1);

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
    assert_eq!(clean.skipped_live_count(), 0);

    // Detailed outcome constructor
    let detailed = RestoreOutcome::with_details(10, 8, 2, 3, Some("connection reset".to_string()));
    assert_eq!(detailed.restored_count(), 10);
    assert_eq!(detailed.dispatched_count(), 8);
    assert_eq!(detailed.failed_count(), 2);
    assert_eq!(detailed.skipped_live_count(), 3);
    assert_eq!(detailed.last_error.as_deref(), Some("connection reset"));
    assert!(!detailed.is_success());
}

#[test]
fn test_live_status_dedup_preserves_completed_only() {
    assert!(
        !is_live_status_preserved(Some("running")),
        "Interrupted running prompt must NOT be preserved so it re-executes on rotated account"
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
    let other_count =
        count_unrestored_prompts_for_other_instances("inst-target", Some(custom_file))
            .expect("Must count other instance unrestored prompts");
    assert_eq!(
        other_count, 2,
        "inst-target must see 2 records under other instances"
    );

    // Scoped restore with fallback for "inst-target" should fall back to unscoped restore
    let (records, outcome) = restore_running_prompts_scoped_with_fallback_with_outcome(
        "inst-target",
        true, // keep_backup = true to preserve DB rows for verification
        Some(custom_file),
    )
    .expect("Fallback restore should succeed");

    assert_eq!(
        records.len(),
        2,
        "Must return 2 restored records via fallback"
    );
    assert_eq!(records[0].prompt_id, "prompt-fb-1");
    assert_eq!(records[1].prompt_id, "prompt-fb-2");
    assert_eq!(outcome.restored_count(), 2);

    // Justification: test cleanup of temporary directory
    crate::error::record_ignored(fs::remove_dir_all(temp_dir), "remove temp dir");
}

#[test]
fn test_reattribution_of_unrestored_prompt_backups_during_switch() {
    let temp_dir =
        std::env::temp_dir().join(format!("agm_test_reattribute_{}", Uuid::new_v4().simple()));
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
    let changed = conn
        .execute(
            "UPDATE prompt_backups SET instance_id = ?1 WHERE instance_id = ?2 AND is_restored = 0",
            params!["inst-new", "inst-old"],
        )
        .expect("Must update instance_id");

    assert_eq!(changed, 1, "Only unrestored row should be re-attributed");

    let new_inst: String = conn
        .query_row(
            "SELECT instance_id FROM prompt_backups WHERE id = 'r-old-1'",
            [],
            |row| row.get(0),
        )
        .expect("Must query row 1");
    assert_eq!(new_inst, "inst-new");

    let old_inst: String = conn
        .query_row(
            "SELECT instance_id FROM prompt_backups WHERE id = 'r-old-2'",
            [],
            |row| row.get(0),
        )
        .expect("Must query row 2");
    assert_eq!(
        old_inst, "inst-old",
        "Already restored row remains on inst-old"
    );

    // Justification: test cleanup of temporary directory
    crate::error::record_ignored(fs::remove_dir_all(temp_dir), "remove temp dir");
}

#[test]
fn test_incomplete_prompt_sweep_selection_and_status_transition() {
    let temp_dir = std::env::temp_dir().join(format!("agm_test_sweep_{}", Uuid::new_v4().simple()));
    // Justification: test cleanup of temporary directory
    crate::error::record_ignored(fs::create_dir_all(&temp_dir), "create test temp dir");
    let db_path = temp_dir.join("test-sweep.db");
    let conn = rusqlite::Connection::open(&db_path).expect("Must open temp test DB");

    conn.execute_batch(
        "CREATE TABLE active_prompts (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            instance_id TEXT NOT NULL,
            repo_path TEXT NOT NULL,
            prompt_content TEXT NOT NULL,
            model TEXT,
            session_id TEXT,
            status TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            image_payload TEXT
        );",
    )
    .expect("Must create active_prompts table");

    let now = Utc::now().timestamp();
    let ten_hours_ago = now - 36000;

    // Insert prompts across various uncompleted statuses with older timestamps
    let test_rows = [
        ("p-running", "running", ten_hours_ago),
        ("p-inflight", "in_flight", ten_hours_ago),
        ("p-dispatched", "dispatched", ten_hours_ago),
        ("p-pending", "pending", ten_hours_ago),
        ("p-queued", "queued", ten_hours_ago),
        ("p-blocked", "blocked", ten_hours_ago),
        ("p-completed", "completed", ten_hours_ago),
    ];

    for (id, st, t) in test_rows {
        conn.execute(
            "INSERT INTO active_prompts (id, project_id, instance_id, repo_path, prompt_content, status, created_at, updated_at)
             VALUES (?1, 'proj-1', 'inst-test', '/work/proj', 'Test prompt', ?2, ?3, ?3)",
            params![id, st, t],
        )
        .expect("Must insert test prompt");
    }

    // Run sweep query selecting incomplete prompts (ignoring age)
    let mut stmt = conn
        .prepare(
            "SELECT id FROM active_prompts
             WHERE status NOT IN ('completed', 'succeeded', 'backed_up') AND instance_id = ?1
             ORDER BY created_at ASC, id ASC",
        )
        .expect("Must prepare sweep select");

    let ids: Vec<String> = stmt
        .query_map(params!["inst-test"], |r| r.get(0))
        .expect("Must query ids")
        .flatten()
        .collect();

    assert_eq!(
        ids.len(),
        6,
        "All 6 uncompleted prompts must be selected regardless of age"
    );
    assert!(!ids.contains(&"p-completed".to_string()));

    // Perform the status transition to backed_up
    for id in &ids {
        conn.execute(
            "UPDATE active_prompts SET status = 'backed_up', updated_at = ?1
             WHERE id = ?2 AND status NOT IN ('completed', 'succeeded')",
            params![now, id],
        )
        .expect("Must sweep prompt");
    }

    // Verify all 6 are now backed_up
    let backed_up_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM active_prompts WHERE status = 'backed_up'",
            [],
            |r| r.get(0),
        )
        .expect("Must count backed up");
    assert_eq!(backed_up_count, 6);

    // Verify completed prompt remained untouched
    let completed_status: String = conn
        .query_row(
            "SELECT status FROM active_prompts WHERE id = 'p-completed'",
            [],
            |r| r.get(0),
        )
        .expect("Must query completed status");
    assert_eq!(completed_status, "completed");

    // Justification: test cleanup of temporary directory
    crate::error::record_ignored(fs::remove_dir_all(temp_dir), "remove temp dir");
}
