pub mod db;
pub mod export;
pub mod import;
pub mod types;
pub mod workdir;

// Compatibility aliases
pub use db as storage;
pub use export::*;
pub use import::*;
pub use types as models;
pub use types::*;
pub use workdir as work_dir;
pub use workdir::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::{Path, PathBuf};

    #[test]
    fn test_workdir_resolver_cli_override() {
        let custom = r"D:\custom_work";
        let res = WorkDirResolver::resolve_work_dir(Some(custom)).expect("resolve work dir");
        assert_eq!(clean_path_str(&res), "D:/custom_work");
    }

    #[test]
    fn test_path_relativity_roundtrip() {
        let base = Path::new(r"D:\work");
        let sample = Path::new(r"D:\work\Antigravity-Manager\src-tauri");
        let rel = to_relative_work_path(sample, base).expect("to_relative_work_path");
        assert_eq!(rel, "Antigravity-Manager/src-tauri");

        let reanchored = from_relative_work_path(&rel, base);
        assert_eq!(
            clean_path_str(&reanchored),
            "D:/work/Antigravity-Manager/src-tauri"
        );
    }

    #[test]
    fn test_compact_db_roundtrip() {
        let temp_dir =
            std::env::temp_dir().join(format!("agm_test_compact_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).expect("create temp dir");
        let db_file = temp_dir.join("test-compact.db");

        let data = CompactExportData {
            meta: CompactMetadata {
                schema_version: "2.0".to_string(),
                created_at: "2026-10-11T12:00:00Z".to_string(),
                source_machine: "TEST-HOST".to_string(),
                source_work_dir: "D:/work".to_string(),
                is_portable: true,
                notes: None,
            },
            pinned_projects: vec![CompactPinnedProject {
                project_key: "P001".to_string(),
                repo_name: "Antigravity-Manager".to_string(),
                relative_path: "Antigravity-Manager".to_string(),
                pinned_at: "2026-10-11T12:00:00Z".to_string(),
                is_active: true,
                display_order: 1,
                notes: None,
            }],
            conversations: vec![CompactConversation {
                conversation_id: "c-1001".to_string(),
                project_key: "P001".to_string(),
                seq_id: 1,
                title: "Test Session".to_string(),
                relative_path: "Antigravity-Manager".to_string(),
                last_active_at: "2026-10-11T12:00:00Z".to_string(),
                message_count: 5,
                is_archived: false,
                has_active_prompt: true,
            }],
            running_prompts: vec![CompactRunningPrompt {
                prompt_id: "p-2001".to_string(),
                project_key: "P001".to_string(),
                conversation_id: "c-1001".to_string(),
                prompt_content: "Run test suite".to_string(),
                model: Some("gemini-2.5-pro".to_string()),
                status: "running".to_string(),
                created_at: "2026-10-11T12:00:00Z".to_string(),
                updated_at: "2026-10-11T12:00:00Z".to_string(),
                image_payload: None,
                is_restored: false,
            }],
            prompt_history: vec![CompactPromptHistory {
                history_id: "h-3001".to_string(),
                project_key: "P001".to_string(),
                conversation_id: "c-1001".to_string(),
                prompt_content: "Initial setup".to_string(),
                model: None,
                role: "user".to_string(),
                executed_at: "2026-10-11T11:00:00Z".to_string(),
                exit_code: Some(0),
                has_completed: true,
                duration_ms: 1200,
            }],
        };

        // Write
        db::write_compact_db(&db_file, &data).expect("write compact db");

        // Verify file exists
        assert!(db_file.exists());

        // Read back
        let read_back = db::read_compact_db(&db_file).expect("read compact db");
        assert_eq!(read_back.pinned_projects.len(), 1);
        assert_eq!(read_back.pinned_projects[0].project_key, "P001");
        assert_eq!(read_back.conversations.len(), 1);
        assert_eq!(read_back.running_prompts.len(), 1);
        assert_eq!(read_back.prompt_history.len(), 1);

        // Cleanup
        let _ = fs::remove_dir_all(temp_dir);
    }
}
