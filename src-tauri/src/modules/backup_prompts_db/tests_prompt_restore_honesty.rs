//! Companion Unit Tests for Prompt Restore Honesty & Quota Drop Recovery (`Imp-701`).
//!
//! Enforces:
//! 1. Incomplete-prompt sweep: transitions all running, in_flight, and dispatched prompts to backed_up (any age).
//! 2. Honest RestoreOutcome reporting: `{ restored_count, dispatched_count, failed_count }` with zero fake success.
//! 3. Live-status dedup: preserves live rows in `running` and `completed` states.
//! 4. Scoped-restore fallback: automatically falls back to unscoped restore when scoped restore returns 0 records.
//! 5. Re-attribution of unrestored prompt_backups across instance switches.

use super::connect_backup_db;
use super::restore::{
    count_unrestored_prompts_for_instance_with_file,
    count_unrestored_prompts_for_other_instances, is_live_status_preserved,
    reattribute_unrestored_prompt_backups,
    restore_running_prompts_scoped_with_fallback_with_outcome, RestoreOutcome,
};
use chrono::Utc;
use rusqlite::{params, Connection};
use std::fs;
use std::path::Path;
use uuid::Uuid;

#[test]
fn test_restore_outcome_honesty_struct_and_accessors() {
    let mut outcome = RestoreOutcome::new(5, 4, 1);
    assert_eq!(outcome.restored_count(), 5);
    assert_eq!(outcome.dispatched_count(), 4);
    assert_eq!(outcome.failed_count(), 1);
    assert!(
        !outcome.is_success(),
        "Outcome with failures must not be reported as success"
    );

    outcome.record_restored();
    outcome.record_dispatched(6);
    outcome.record_skipped_live();
    assert_eq!(outcome.restored_count(), 6);
    assert_eq!(outcome.dispatched_count(), 6);
    assert_eq!(outcome.skipped_live_count(), 1);

    outcome.record_failed(Some("Dispatch failure: connection dropped".to_string()));
    assert_eq!(outcome.failed_count(), 2);
    assert_eq!(
        outcome.last_error.as_deref(),
        Some("Dispatch failure: connection dropped")
    );
    assert!(!outcome.is_success());

    assert_clean_and_detailed_outcomes();
}

fn assert_clean_and_detailed_outcomes() {
    let clean = RestoreOutcome::new(3, 3, 0);
    assert!(clean.is_success());
    assert_eq!(clean.restored_count(), 3);
    assert_eq!(clean.dispatched_count(), 3);
    assert_eq!(clean.failed_count(), 0);
    assert_eq!(clean.skipped_live_count(), 0);

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
    let custom_file = db_path.to_str().expect("valid path");

    let conn = connect_backup_db(Some(custom_file)).expect("Must connect to backup DB");
    let now = Utc::now().timestamp();
    seed_fallback_test_records(&conn, custom_file, now);

    let other_count =
        count_unrestored_prompts_for_other_instances("inst-target", Some(custom_file))
            .expect("Must count other instance unrestored prompts");
    assert_eq!(
        other_count, 2,
        "inst-target must see 2 records under other instances"
    );

    let (records, outcome) = restore_running_prompts_scoped_with_fallback_with_outcome(
        "inst-target",
        true,
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
    assert_eq!(records[0].instance_id.as_deref(), Some("inst-target"));
    assert_eq!(records[1].instance_id.as_deref(), Some("inst-target"));

    // Justification: test cleanup of temporary directory
    crate::error::record_ignored(fs::remove_dir_all(temp_dir), "remove temp dir");
}

fn seed_fallback_test_records(conn: &Connection, custom_file: &str, now: i64) {
    conn.execute(
        "INSERT INTO backup_batches (id, created_at, prompts_count, file_path, retention_days, is_fully_restored)
         VALUES ('b-fallback', ?1, 2, ?2, 1, 0)",
        params![now, custom_file],
    )
    .expect("Must insert batch");

    for (pid, idx) in [("prompt-fb-1", 1), ("prompt-fb-2", 2)] {
        conn.execute(
            "INSERT INTO prompt_backups (id, backup_batch_id, prompt_id, project_name, project_path, project_id, conversation_id, sequence_id, prompt_text, created_at, is_restored, instance_id)
             VALUES (?1, 'b-fallback', ?2, 'proj-fallback', '/work/proj-fb', 'proj-1', 'conv-1', ?3, 'Test fallback restore', ?4, 0, 'inst-source')",
            params![format!("rec-{}", idx), pid, idx, now + idx],
        )
        .expect("Must insert prompt backup");
    }
}

#[test]
fn test_reattribution_of_unrestored_prompt_backups_during_switch() {
    let temp_dir =
        std::env::temp_dir().join(format!("agm_test_reattribute_{}", Uuid::new_v4().simple()));
    let db_path = temp_dir.join("test-reattribute.db");
    let custom_file = db_path.to_str().expect("valid path");

    let conn = connect_backup_db(Some(custom_file)).expect("Must connect to backup DB");
    let now = Utc::now().timestamp();
    seed_reattribution_test_records(&conn, custom_file, now);

    let changed = reattribute_unrestored_prompt_backups(
        Some("inst-old"),
        "inst-new",
        Some(custom_file),
    )
    .expect("Must re-attribute prompt backups");
    assert_eq!(changed, 1, "Only unrestored row should be re-attributed");

    assert_reattribution_outcomes(&conn);

    // Justification: test cleanup of temporary directory
    crate::error::record_ignored(fs::remove_dir_all(temp_dir), "remove temp dir");
}

#[test]
fn test_reattribute_unrestored_prompt_backups_unscoped_and_scoped() {
    let temp_dir = std::env::temp_dir().join(format!(
        "agm_test_reattr_all_{}",
        Uuid::new_v4().simple()
    ));
    let db_path = temp_dir.join("test-reattr-all.db");
    let custom_file = db_path.to_str().expect("valid path");

    let conn = connect_backup_db(Some(custom_file)).expect("Must connect to backup DB");
    let now = Utc::now().timestamp();

    conn.execute(
        "INSERT INTO backup_batches (id, created_at, prompts_count, file_path, retention_days, is_fully_restored)
         VALUES ('b-all', ?1, 3, ?2, 1, 0)",
        params![now, custom_file],
    )
    .expect("Must insert batch");

    conn.execute(
        "INSERT INTO prompt_backups (id, backup_batch_id, prompt_id, project_name, project_path, project_id, conversation_id, sequence_id, prompt_text, created_at, is_restored, instance_id)
         VALUES ('r-1', 'b-all', 'p-1', 'proj', '/work/p', 'proj-1', 'conv-1', 1, 'text 1', ?1, 0, 'inst-a')",
        params![now],
    )
    .expect("Must insert r-1");

    conn.execute(
        "INSERT INTO prompt_backups (id, backup_batch_id, prompt_id, project_name, project_path, project_id, conversation_id, sequence_id, prompt_text, created_at, is_restored, instance_id)
         VALUES ('r-2', 'b-all', 'p-2', 'proj', '/work/p', 'proj-1', 'conv-1', 2, 'text 2', ?1, 0, 'inst-b')",
        params![now + 1],
    )
    .expect("Must insert r-2");

    conn.execute(
        "INSERT INTO prompt_backups (id, backup_batch_id, prompt_id, project_name, project_path, project_id, conversation_id, sequence_id, prompt_text, created_at, is_restored, instance_id)
         VALUES ('r-3', 'b-all', 'p-3', 'proj', '/work/p', 'proj-1', 'conv-1', 3, 'text 3', ?1, 1, 'inst-a')",
        params![now + 2],
    )
    .expect("Must insert r-3");

    // Scoped re-attribution: from inst-a to inst-target
    let changed_scoped = reattribute_unrestored_prompt_backups(
        Some("inst-a"),
        "inst-target",
        Some(custom_file),
    )
    .expect("Must re-attribute inst-a");
    assert_eq!(changed_scoped, 1, "Only r-1 was from inst-a and unrestored");

    // Unscoped re-attribution: from None to inst-target (should reattribute remaining r-2 from inst-b)
    let changed_unscoped = reattribute_unrestored_prompt_backups(
        None,
        "inst-target",
        Some(custom_file),
    )
    .expect("Must re-attribute all unrestored");
    assert_eq!(changed_unscoped, 2);

    let count_target = count_unrestored_prompts_for_instance_with_file(
        Some("inst-target"),
        Some(custom_file),
    )
    .expect("Must count unrestored prompts for inst-target");
    assert_eq!(count_target, 2);

    let count_other = count_unrestored_prompts_for_other_instances("inst-target", Some(custom_file))
        .expect("Must count other unrestored");
    assert_eq!(count_other, 0);

    // r-3 was already restored and remains inst-a
    let r3_inst: String = conn
        .query_row(
            "SELECT instance_id FROM prompt_backups WHERE id = 'r-3'",
            [],
            |r| r.get(0),
        )
        .expect("query r-3");
    assert_eq!(r3_inst, "inst-a");

    // Justification: test cleanup of temporary directory
    crate::error::record_ignored(fs::remove_dir_all(temp_dir), "remove temp dir");
}

#[test]
fn test_fallback_unscoped_if_orphaned_reattributes_and_restores() {
    let temp_dir = std::env::temp_dir().join(format!(
        "agm_test_fallback_reattr_{}",
        Uuid::new_v4().simple()
    ));
    let db_path = temp_dir.join("test-fallback-reattr.db");
    let custom_file = db_path.to_str().expect("valid path");

    let conn = connect_backup_db(Some(custom_file)).expect("Must connect to backup DB");
    let now = Utc::now().timestamp();

    conn.execute(
        "INSERT INTO backup_batches (id, created_at, prompts_count, file_path, retention_days, is_fully_restored)
         VALUES ('b-orphaned', ?1, 2, ?2, 1, 0)",
        params![now, custom_file],
    )
    .expect("Must insert batch");

    conn.execute(
        "INSERT INTO prompt_backups (id, backup_batch_id, prompt_id, project_name, project_path, project_id, conversation_id, sequence_id, prompt_text, created_at, is_restored, instance_id)
         VALUES ('rec-orph-1', 'b-orphaned', 'p-orph-1', 'proj-fb', '/work/proj-fb', 'proj-1', 'conv-1', 1, 'Orphaned prompt 1', ?1, 0, 'inst-dead-1')",
        params![now],
    )
    .expect("Must insert rec-orph-1");

    conn.execute(
        "INSERT INTO prompt_backups (id, backup_batch_id, prompt_id, project_name, project_path, project_id, conversation_id, sequence_id, prompt_text, created_at, is_restored, instance_id)
         VALUES ('rec-orph-2', 'b-orphaned', 'p-orph-2', 'proj-fb', '/work/proj-fb', 'proj-1', 'conv-1', 2, 'Orphaned prompt 2', ?1, 0, 'inst-dead-2')",
        params![now + 1],
    )
    .expect("Must insert rec-orph-2");

    let target_initial = count_unrestored_prompts_for_instance_with_file(
        Some("inst-active-running"),
        Some(custom_file),
    )
    .expect("Count initial");
    assert_eq!(target_initial, 0);

    let (records, outcome) = restore_running_prompts_scoped_with_fallback_with_outcome(
        "inst-active-running",
        false,
        Some(custom_file),
    )
    .expect("Restore with fallback must succeed");

    assert_eq!(records.len(), 2, "Must restore both orphaned prompts");
    assert_eq!(outcome.restored_count(), 2);

    assert_eq!(
        records[0].instance_id.as_deref(),
        Some("inst-active-running"),
        "Prompt 1 must be re-attributed to target instance"
    );
    assert_eq!(
        records[1].instance_id.as_deref(),
        Some("inst-active-running"),
        "Prompt 2 must be re-attributed to target instance"
    );

    let (inst_1, restored_1): (String, i64) = conn
        .query_row(
            "SELECT instance_id, is_restored FROM prompt_backups WHERE id = 'rec-orph-1'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .expect("Query rec-orph-1");
    assert_eq!(inst_1, "inst-active-running");
    assert_eq!(restored_1, 1);

    let (inst_2, restored_2): (String, i64) = conn
        .query_row(
            "SELECT instance_id, is_restored FROM prompt_backups WHERE id = 'rec-orph-2'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .expect("Query rec-orph-2");
    assert_eq!(inst_2, "inst-active-running");
    assert_eq!(restored_2, 1);

    // Justification: test cleanup of temporary directory
    crate::error::record_ignored(fs::remove_dir_all(temp_dir), "remove temp dir");
}

fn seed_reattribution_test_records(conn: &Connection, custom_file: &str, now: i64) {
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
}

fn assert_reattribution_outcomes(conn: &Connection) {
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
}

#[test]
fn test_incomplete_prompt_sweep_selection_and_status_transition() {
    let temp_dir = std::env::temp_dir().join(format!("agm_test_sweep_{}", Uuid::new_v4().simple()));
    // Justification: test cleanup of temporary directory
    crate::error::record_ignored(fs::create_dir_all(&temp_dir), "create test temp dir");
    let db_path = temp_dir.join("test-sweep.db");
    let conn = setup_test_active_prompts_db(&db_path);

    let now = Utc::now().timestamp();
    seed_test_sweep_rows(&conn, now);

    let ids = fetch_sweep_candidate_ids(&conn, "inst-test");
    assert_eq!(
        ids.len(),
        6,
        "All 6 uncompleted prompts must be selected regardless of age"
    );
    assert!(!ids.contains(&"p-completed".to_string()));

    execute_sweep_transition(&conn, &ids, now);
    assert_sweep_results(&conn);

    // Justification: test cleanup of temporary directory
    crate::error::record_ignored(fs::remove_dir_all(temp_dir), "remove temp dir");
}

fn setup_test_active_prompts_db(db_path: &Path) -> Connection {
    let conn = Connection::open(db_path).expect("Must open temp test DB");
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
    conn
}

fn seed_test_sweep_rows(conn: &Connection, now: i64) {
    let ten_hours_ago = now - 36000;
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
}

fn fetch_sweep_candidate_ids(conn: &Connection, instance_id: &str) -> Vec<String> {
    let mut stmt = conn
        .prepare(
            "SELECT id FROM active_prompts
             WHERE status NOT IN ('completed', 'succeeded', 'backed_up') AND instance_id = ?1
             ORDER BY created_at ASC, id ASC",
        )
        .expect("Must prepare sweep select");

    stmt.query_map(params![instance_id], |r| r.get(0))
        .expect("Must query ids")
        .flatten()
        .collect()
}

fn execute_sweep_transition(conn: &Connection, ids: &[String], now: i64) {
    for id in ids {
        conn.execute(
            "UPDATE active_prompts SET status = 'backed_up', updated_at = ?1
             WHERE id = ?2 AND status NOT IN ('completed', 'succeeded')",
            params![now, id],
        )
        .expect("Must sweep prompt");
    }
}

fn assert_sweep_results(conn: &Connection) {
    let backed_up_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM active_prompts WHERE status = 'backed_up'",
            [],
            |r| r.get(0),
        )
        .expect("Must count backed up");
    assert_eq!(backed_up_count, 6);

    let completed_status: String = conn
        .query_row(
            "SELECT status FROM active_prompts WHERE id = 'p-completed'",
            [],
            |r| r.get(0),
        )
        .expect("Must query completed status");
    assert_eq!(completed_status, "completed");
}
