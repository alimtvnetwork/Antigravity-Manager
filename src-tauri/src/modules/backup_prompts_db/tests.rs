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
}
