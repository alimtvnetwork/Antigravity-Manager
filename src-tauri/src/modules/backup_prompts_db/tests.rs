use chrono::Utc;
use rusqlite::{params, Connection};
use std::fs;
use uuid::Uuid;

use super::*;

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    pub(crate) fn test_backup_db_lifecycle() {
        let temp_dir =
            std::env::temp_dir().join(format!("agm_test_backup_{}", Uuid::new_v4().simple()));
        let db_path = temp_dir.join("test-backup.db");
        let custom_file = db_path.to_str().unwrap();

        let conn = connect_backup_db(Some(custom_file)).unwrap();
        assert!(db_path.exists());

        // Test insert and list batch
        let now = Utc::now().timestamp();
        conn.execute(
            "INSERT INTO backup_batches (id, created_at, prompts_count, file_path, retention_days, is_fully_restored)
             VALUES ('b1', ?, 1, ?, 1, 0)",
            params![now, custom_file],
        ).unwrap();

        let batches = list_backup_batches(Some(custom_file)).unwrap();
        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].id, "b1");

        // Test insert prompt backup
        conn.execute(
            "INSERT INTO prompt_backups (id, backup_batch_id, prompt_id, project_name, project_path, project_id, conversation_id, sequence_id, prompt_text, created_at, is_restored)
             VALUES ('p1', 'b1', 'prompt_1', 'my-project', '/path/to/repo', 'proj-1', 'conv-1', 1, 'Fix tests', ?, 0)",
            params![now],
        ).unwrap();

        let prompts = list_prompt_backups(None, Some(custom_file)).unwrap();
        assert_eq!(prompts.len(), 1);
        assert_eq!(prompts[0].prompt_text, "Fix tests");

        // Test restore
        let restored = restore_running_prompts(None, false, Some(custom_file)).unwrap();
        assert_eq!(restored.len(), 1);

        // Verify second restore does not re-restore already restored prompts
        let restored_second = restore_running_prompts(None, false, Some(custom_file)).unwrap();
        assert_eq!(
            restored_second.len(),
            0,
            "Already restored prompts must not be restored again"
        );

        // Test auto cleanup (with future cutoff)
        let removed = auto_cleanup_expired(Some(custom_file), -10).unwrap();
        assert_eq!(removed, 1);

        // Justification: best-effort cleanup; a leftover directory is harmless
        crate::error::record_ignored(fs::remove_dir_all(temp_dir), "remove_dir_all");
    }

    /// Regression test: backups recorded under one instance must be restorable
    /// after an instance switch. `restore_running_prompts(None, ..)` restores
    /// across every instance; `Some(id)` stays strictly scoped to that instance.
    #[test]
    pub(crate) fn test_restore_scoping_across_instances() {
        let temp_dir = std::env::temp_dir().join(format!(
            "agm_test_restore_scope_{}",
            Uuid::new_v4().simple()
        ));
        let db_path = temp_dir.join("test-restore-scope.db");
        let custom_file = db_path.to_str().unwrap();

        let conn = connect_backup_db(Some(custom_file)).unwrap();
        let now = Utc::now().timestamp();
        conn.execute(
            "INSERT INTO backup_batches (id, created_at, prompts_count, file_path, retention_days, is_fully_restored)
             VALUES ('b-scope', ?, 3, ?, 1, 0)",
            params![now, custom_file],
        )
        .unwrap();

        // One backup per instance: inst-a, inst-b, plus a legacy row with NULL instance_id.
        for (rid, pid, inst) in [
            ("r-a", "prompt_inst_a", Some("inst-a")),
            ("r-b", "prompt_inst_b", Some("inst-b")),
            ("r-null", "prompt_inst_null", None),
        ] {
            conn.execute(
                "INSERT INTO prompt_backups (id, backup_batch_id, prompt_id, project_name, project_path, project_id, conversation_id, sequence_id, prompt_text, created_at, is_restored, instance_id)
                 VALUES (?1, 'b-scope', ?2, 'proj', '/nonexistent/repo', 'proj-1', 'conv-1', 1, 'scope test', ?3, 0, ?4)",
                params![rid, pid, now, inst],
            )
            .unwrap();
        }

        // Strict scoping: only the requested instance's backups come back.
        let scoped_a = restore_running_prompts(Some("inst-a"), true, Some(custom_file)).unwrap();
        assert_eq!(scoped_a.len(), 1);
        assert_eq!(scoped_a[0].prompt_id, "prompt_inst_a");

        let scoped_b = restore_running_prompts(Some("inst-b"), true, Some(custom_file)).unwrap();
        assert_eq!(scoped_b.len(), 1);
        assert_eq!(scoped_b[0].prompt_id, "prompt_inst_b");

        // Default scope still picks up legacy NULL-instance rows.
        let scoped_default =
            restore_running_prompts(Some("default"), true, Some(custom_file)).unwrap();
        assert_eq!(scoped_default.len(), 1);
        assert_eq!(scoped_default[0].prompt_id, "prompt_inst_null");

        // Unscoped (None): restores everything across all instances — the
        // historical behavior the CLI depends on after an instance switch.
        let unscoped = restore_running_prompts(None, true, Some(custom_file)).unwrap();
        assert_eq!(unscoped.len(), 3);
        let mut ids: Vec<&str> = unscoped.iter().map(|r| r.prompt_id.as_str()).collect();
        ids.sort_unstable();
        assert_eq!(
            ids,
            vec!["prompt_inst_a", "prompt_inst_b", "prompt_inst_null"]
        );

        // Justification: best-effort cleanup; a leftover directory is harmless
        crate::error::record_ignored(fs::remove_dir_all(temp_dir), "remove_dir_all");
    }

    #[test]
    pub(crate) fn test_multi_image_extraction_collects_all_uris() {
        use crate::modules::repo_db::detection::{
            extract_image_uris_or_paths, parse_image_uris, serialize_image_uris,
        };

        let content = "look at this data:image/png;base64,AAA first and data:image/jpeg;base64,BBB second, also data:image/png;base64,AAA again plus /tmp/shot.png";
        let (uris, paths) = extract_image_uris_or_paths(content);
        // All distinct URIs collected verbatim in document order; duplicate dropped.
        assert_eq!(
            uris,
            vec![
                "data:image/png;base64,AAA".to_string(),
                "data:image/jpeg;base64,BBB".to_string(),
            ]
        );
        assert_eq!(paths, vec!["/tmp/shot.png".to_string()]);

        // Serialization: 3 URIs -> JSON array, none silently dropped (AC2).
        let three = vec!["u1".to_string(), "u2".to_string(), "u3".to_string()];
        let ser = serialize_image_uris(&three).expect("must serialize");
        assert!(ser.starts_with('['));
        assert_eq!(parse_image_uris(&ser), three);

        // Single URI stays verbatim (legacy format, byte-identical).
        let one = vec!["data:image/png;base64,AAA".to_string()];
        assert_eq!(
            serialize_image_uris(&one).as_deref(),
            Some("data:image/png;base64,AAA")
        );
        assert_eq!(parse_image_uris("data:image/png;base64,AAA"), one);

        // Zero URIs -> None.
        assert!(serialize_image_uris(&[]).is_none());
        assert!(parse_image_uris("").is_empty());
    }

    #[test]
    pub(crate) fn test_restore_refuses_to_run_on_resend_path() {
        // Simulate the resend path being active on this thread.
        crate::modules::repo_db::backup::RESEND_IN_PROGRESS.with(|f| f.set(true));
        let result = restore_running_prompts(None, true, Some("/tmp/agm-test-guard.db"));
        // Must fail with the double-dispatch guard error before touching any DB.
        let err = result.expect_err("restore must refuse while resend is active");
        assert!(err.contains("double-dispatch"), "unexpected error: {err}");
        crate::modules::repo_db::backup::RESEND_IN_PROGRESS.with(|f| f.set(false));

        // Guard cleared: restore proceeds past the guard (empty DB -> Ok with
        // zero rows), proving the guard was the only blocker above.
        let db2 = "/tmp/agm-test-guard2.db";
        let result2 = restore_running_prompts(None, true, Some(db2));
        let restored = result2.expect("restore should proceed once guard is clear");
        assert!(restored.is_empty());
        // Justification: best-effort cleanup of the scratch DB file.
        crate::error::record_ignored(fs::remove_file(db2), "remove scratch db");
        crate::error::record_ignored(
            fs::remove_file("/tmp/agm-test-guard.db"),
            "remove scratch db",
        );
    }

    #[test]
    pub(crate) fn test_has_unrestored_backups_for_target_instance() {
        let temp_dir = std::env::temp_dir().join(format!(
            "agm_test_unrestored_backups_{}",
            Uuid::new_v4().simple()
        ));
        let db_path = temp_dir.join("test-unrestored.db");
        let custom_file = db_path.to_str().unwrap();

        let conn = connect_backup_db(Some(custom_file)).unwrap();
        let now = Utc::now().timestamp();

        conn.execute(
            "INSERT INTO backup_batches (id, created_at, prompts_count, file_path, retention_days, is_fully_restored)
             VALUES ('b-unrestored', ?1, 2, ?2, 1, 0)",
            params![now, custom_file],
        )
        .unwrap();

        // Target instance backup
        conn.execute(
            "INSERT INTO prompt_backups (id, backup_batch_id, prompt_id, project_name, project_path, project_id, conversation_id, sequence_id, prompt_text, created_at, is_restored, instance_id)
             VALUES ('p-target', 'b-unrestored', 'prompt_target', 'target-proj', '/path/target', 'proj-target', 'conv-1', 1, 'Target task', ?1, 0, 'target_inst')",
            params![now],
        )
        .unwrap();

        // Other instance backup
        conn.execute(
            "INSERT INTO prompt_backups (id, backup_batch_id, prompt_id, project_name, project_path, project_id, conversation_id, sequence_id, prompt_text, created_at, is_restored, instance_id)
             VALUES ('p-other', 'b-unrestored', 'prompt_other', 'other-proj', '/path/other', 'proj-other', 'conv-2', 1, 'Other task', ?1, 0, 'other_inst')",
            params![now],
        )
        .unwrap();

        // 1. Target instance has 1 unrestored prompt
        let target_count =
            count_unrestored_prompts_for_instance_with_file(Some("target_inst"), Some(custom_file))
                .unwrap();
        assert_eq!(
            target_count, 1,
            "target instance should have exactly 1 unrestored prompt"
        );

        // 2. Unmatched instance has 0 unrestored prompts
        let unmatched_count = count_unrestored_prompts_for_instance_with_file(
            Some("nonexistent_inst"),
            Some(custom_file),
        )
        .unwrap();
        assert_eq!(
            unmatched_count, 0,
            "nonexistent instance should have 0 unrestored prompts"
        );

        // 3. Unscoped (None) sees all 2 unrestored prompts
        let total_count =
            count_unrestored_prompts_for_instance_with_file(None, Some(custom_file)).unwrap();
        assert_eq!(
            total_count, 2,
            "unscoped probe should see all unrestored prompts"
        );

        // 4. Mark target prompt as restored
        conn.execute(
            "UPDATE prompt_backups SET is_restored = 1, restored_at = ?1 WHERE id = 'p-target'",
            params![now],
        )
        .unwrap();

        let target_count_after =
            count_unrestored_prompts_for_instance_with_file(Some("target_inst"), Some(custom_file))
                .unwrap();
        assert_eq!(
            target_count_after, 0,
            "target instance should have 0 unrestored prompts after restore"
        );

        // Justification: best-effort cleanup; a leftover directory is harmless
        crate::error::record_ignored(fs::remove_dir_all(temp_dir), "remove_dir_all");
    }

    #[test]
    pub(crate) fn test_prompt_status_after_restore_discoverability() {
        let status_matrix = [
            ("running", "in_progress"),
            ("in_progress", "in_progress"),
            ("queued", "queued"),
            ("dispatched", "pending"),
            ("backed_up", "pending"),
            ("interrupted", "pending"),
            ("failed", "pending"),
            ("pending", "pending"),
            ("", "pending"),
            ("unknown_custom_state", "pending"),
        ];

        let valid_discoverable_statuses = ["in_progress", "queued", "pending"];

        for (input_status, expected_restored_status) in status_matrix {
            let output_status = prompt_status_after_restore(input_status);

            // Assert exact expected status
            assert_eq!(
                output_status, expected_restored_status,
                "input status '{}' must map to '{}'",
                input_status, expected_restored_status
            );

            // Assert output is non-empty
            assert!(
                !output_status.is_empty(),
                "restored status for input '{}' must not be empty",
                input_status
            );

            // Assert output is always in the discoverable set recognized by resend/scheduler
            assert!(
                valid_discoverable_statuses.contains(&output_status),
                "restored status '{}' for input '{}' must be one of {:?}",
                output_status,
                input_status,
                valid_discoverable_statuses
            );
        }
    }
}
