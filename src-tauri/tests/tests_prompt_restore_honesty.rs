//! Integration Test for Prompt Restore Honesty & Quota Drop Recovery (`Imp-731`).
//!
//! Validates:
//! 1. `test_sweep_and_reattribute_orphaned_backups`: orphaned backed-up records with a different
//!    target_instance_id are reattributed and restored in strict FIFO order (`created_at ASC, id ASC`).

use antigravity_tools_lib::modules::backup_prompts_db::{
    connect_backup_db, restore_running_prompts_for_instance_with_outcome,
    sweep_and_reattribute_orphaned_backups,
};
use chrono::Utc;
use rusqlite::params;
use std::fs;
use uuid::Uuid;

#[test]
fn test_sweep_and_reattribute_orphaned_backups() {
    let temp_dir = std::env::temp_dir().join(format!(
        "agm_int_test_sweep_orphaned_{}",
        Uuid::new_v4().simple()
    ));
    let db_path = temp_dir.join("test-sweep-orphaned.db");
    let custom_file = db_path.to_str().expect("valid path");

    let conn = connect_backup_db(Some(custom_file)).expect("Must connect to backup DB");
    let now = Utc::now().timestamp();

    conn.execute(
        "INSERT INTO backup_batches (id, created_at, prompts_count, file_path, retention_days, is_fully_restored)
         VALUES ('b-orph', ?1, 5, ?2, 1, 0)",
        params![now, custom_file],
    )
    .expect("Must insert batch");

    // Insert 3 orphaned records with DIFFERENT created_at timestamps and DIFFERENT target_instance_ids:
    // r-orph-1: created at now + 10 (target: inst-crashed-1, status: backed_up)
    // r-orph-2: created at now + 5  (target: inst-crashed-2, status: backed_up)
    // r-orph-3: created at now + 20 (target: inst-crashed-3, status: backed_up)
    conn.execute(
        "INSERT INTO prompt_backups (id, backup_batch_id, prompt_id, project_name, project_path, project_id, conversation_id, sequence_id, prompt_text, status, created_at, is_restored, instance_id)
         VALUES ('r-orph-1', 'b-orph', 'p-orph-1', 'proj-1', '/work/p1', 'proj-1', 'conv-1', 1, 'text 1', 'backed_up', ?1, 0, 'inst-crashed-1')",
        params![now + 10],
    )
    .expect("Must insert r-orph-1");

    conn.execute(
        "INSERT INTO prompt_backups (id, backup_batch_id, prompt_id, project_name, project_path, project_id, conversation_id, sequence_id, prompt_text, status, created_at, is_restored, instance_id)
         VALUES ('r-orph-2', 'b-orph', 'p-orph-2', 'proj-2', '/work/p2', 'proj-2', 'conv-2', 2, 'text 2', 'backed_up', ?1, 0, 'inst-crashed-2')",
        params![now + 5],
    )
    .expect("Must insert r-orph-2");

    conn.execute(
        "INSERT INTO prompt_backups (id, backup_batch_id, prompt_id, project_name, project_path, project_id, conversation_id, sequence_id, prompt_text, status, created_at, is_restored, instance_id)
         VALUES ('r-orph-3', 'b-orph', 'p-orph-3', 'proj-3', '/work/p3', 'proj-3', 'conv-3', 3, 'text 3', 'backed_up', ?1, 0, 'inst-crashed-3')",
        params![now + 20],
    )
    .expect("Must insert r-orph-3");

    // Also insert a record that is already restored (must NOT be reattributed)
    conn.execute(
        "INSERT INTO prompt_backups (id, backup_batch_id, prompt_id, project_name, project_path, project_id, conversation_id, sequence_id, prompt_text, status, created_at, is_restored, instance_id)
         VALUES ('r-done', 'b-orph', 'p-done', 'proj-done', '/work/pdone', 'proj-done', 'conv-done', 4, 'done text', 'completed', ?1, 1, 'inst-crashed-1')",
        params![now + 1],
    )
    .expect("Must insert r-done");

    // Also insert a record already on target instance (must NOT be counted as an orphaned reattribution)
    conn.execute(
        "INSERT INTO prompt_backups (id, backup_batch_id, prompt_id, project_name, project_path, project_id, conversation_id, sequence_id, prompt_text, status, created_at, is_restored, instance_id)
         VALUES ('r-target-already', 'b-orph', 'p-target', 'proj-target', '/work/ptarget', 'proj-target', 'conv-target', 5, 'target text', 'backed_up', ?1, 0, 'inst-active')",
        params![now + 2],
    )
    .expect("Must insert r-target-already");

    // Invoke sweep_and_reattribute_orphaned_backups to reattribute orphaned records to 'inst-active'
    let swept = sweep_and_reattribute_orphaned_backups(&conn, "inst-active")
        .expect("sweep_and_reattribute_orphaned_backups should succeed");

    assert_eq!(
        swept, 3,
        "Exactly 3 orphaned records should be reattributed"
    );

    // Verify all 3 are reattributed to 'inst-active' and retain their created_at timestamps
    let mut stmt = conn
        .prepare(
            "SELECT id, prompt_id, instance_id, created_at FROM prompt_backups
             WHERE instance_id = 'inst-active' AND is_restored = 0
             ORDER BY created_at ASC, id ASC",
        )
        .expect("prepare select");

    let active_records: Vec<(String, String, String, i64)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))
        .expect("query map")
        .collect::<Result<Vec<_>, _>>()
        .expect("collect");

    // 4 records total for 'inst-active': r-target-already (now+2), r-orph-2 (now+5), r-orph-1 (now+10), r-orph-3 (now+20)
    assert_eq!(active_records.len(), 4);
    assert_eq!(active_records[0].0, "r-target-already");
    assert_eq!(active_records[0].3, now + 2);

    // Strict FIFO order of reattributed records:
    assert_eq!(active_records[1].0, "r-orph-2");
    assert_eq!(active_records[1].1, "p-orph-2");
    assert_eq!(active_records[1].2, "inst-active");
    assert_eq!(active_records[1].3, now + 5);

    assert_eq!(active_records[2].0, "r-orph-1");
    assert_eq!(active_records[2].1, "p-orph-1");
    assert_eq!(active_records[2].2, "inst-active");
    assert_eq!(active_records[2].3, now + 10);

    assert_eq!(active_records[3].0, "r-orph-3");
    assert_eq!(active_records[3].1, "p-orph-3");
    assert_eq!(active_records[3].2, "inst-active");
    assert_eq!(active_records[3].3, now + 20);

    // Verify already restored record r-done was NOT touched
    let r_done_inst: String = conn
        .query_row(
            "SELECT instance_id FROM prompt_backups WHERE id = 'r-done'",
            [],
            |r| r.get(0),
        )
        .expect("query r-done");
    assert_eq!(r_done_inst, "inst-crashed-1");

    // Now verify restore_running_prompts_for_instance restores them in strict FIFO order
    let (restored, outcome) = restore_running_prompts_for_instance_with_outcome(
        Some("inst-active"),
        true,
        Some(custom_file),
    )
    .expect("restore should succeed");

    assert_eq!(restored.len(), 4);
    assert_eq!(restored[0].prompt_id, "p-target");
    assert_eq!(restored[1].prompt_id, "p-orph-2");
    assert_eq!(restored[2].prompt_id, "p-orph-1");
    assert_eq!(restored[3].prompt_id, "p-orph-3");
    assert_eq!(outcome.restored_count(), 4);

    // Test cleanup of temporary directory
    let _ = fs::remove_dir_all(temp_dir);
}
