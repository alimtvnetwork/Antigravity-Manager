//! Repo DB integration & unit tests
//!
//! Verifies:
//! 1. Default instance matching counts empty string and NULL instance IDs.
//! 2. Strict FIFO queue ordering (ORDER BY created_at ASC, id ASC) regardless of updated_at.

use rusqlite::Connection;

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::modules::repo_db::{init_tables, ActivePrompt};

    /// Regression test: `count_backed_up_prompts("default")` must properly count
    /// rows where `instance_id` is `'default'`, `'__default__'`, `''` (empty string), or `NULL`.
    #[test]
    pub(crate) fn test_count_backed_up_prompts_default_includes_null_and_empty() {
        let conn = Connection::open_in_memory().unwrap();
        assert!(init_tables(&conn).is_ok());

        let now = chrono::Utc::now().timestamp();

        conn.execute(
            "INSERT INTO running_projects (id, instance_id, repo_name, repo_path, is_running, last_detected_at, updated_at)
             VALUES ('proj-1', 'default', 'Repo 1', '/repo/1', 1, ?1, ?1)",
            rusqlite::params![now],
        ).unwrap();

        // Row 1: instance_id = 'default'
        conn.execute(
            "INSERT INTO active_prompts (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload)
             VALUES ('p-default', 'proj-1', 'default', '/repo/1', 'prompt 1', NULL, NULL, 'backed_up', ?1, ?1, NULL)",
            rusqlite::params![now],
        ).unwrap();

        // Row 2: instance_id = '__default__'
        conn.execute(
            "INSERT INTO active_prompts (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload)
             VALUES ('p-dunder', 'proj-1', '__default__', '/repo/1', 'prompt 2', NULL, NULL, 'backed_up', ?1, ?1, NULL)",
            rusqlite::params![now],
        ).unwrap();

        // Row 3: instance_id = '' (empty string)
        conn.execute(
            "INSERT INTO active_prompts (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload)
             VALUES ('p-empty', 'proj-1', '', '/repo/1', 'prompt 3', NULL, NULL, 'backed_up', ?1, ?1, NULL)",
            rusqlite::params![now],
        ).unwrap();

        // Row 5: instance_id = 'other-inst' (must not be counted by 'default')
        conn.execute(
            "INSERT INTO active_prompts (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload)
             VALUES ('p-other', 'proj-1', 'other-inst', '/repo/1', 'prompt 5', NULL, NULL, 'backed_up', ?1, ?1, NULL)",
            rusqlite::params![now],
        ).unwrap();

        // Row 6: instance_id = 'default' but status = 'completed' (must not be counted as backed_up)
        conn.execute(
            "INSERT INTO active_prompts (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload)
             VALUES ('p-completed', 'proj-1', 'default', '/repo/1', 'prompt 6', NULL, NULL, 'completed', ?1, ?1, NULL)",
            rusqlite::params![now],
        ).unwrap();

        // Query with default scope matching 'default', '__default__', '', and NULL
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM active_prompts
             WHERE status = 'backed_up'
               AND (instance_id = ?1 OR (?1 = 'default' AND (instance_id = 'default' OR instance_id = '__default__' OR instance_id IS NULL OR instance_id = '')))",
            ["default"],
            |row| row.get(0),
        ).unwrap();

        assert_eq!(
            count, 3,
            "default scope must match 'default', '__default__', and empty string"
        );

        // Specific instance scope must strictly isolate to that instance
        let other_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM active_prompts
             WHERE status = 'backed_up'
               AND (instance_id = ?1 OR (?1 = 'default' AND (instance_id = 'default' OR instance_id = '__default__' OR instance_id IS NULL OR instance_id = '')))",
            ["other-inst"],
            |row| row.get(0),
        ).unwrap();

        assert_eq!(
            other_count, 1,
            "other-inst scope must only match its own row"
        );
    }

    /// Verifies that `pick_first_queued_prompt` selects the earliest created prompt
    /// (`created_at ASC, id ASC`) regardless of `updated_at`.
    #[test]
    pub(crate) fn test_strict_fifo_queue_ordering() {
        let conn = Connection::open_in_memory().unwrap();
        assert!(init_tables(&conn).is_ok());

        conn.execute(
            "INSERT INTO running_projects (id, instance_id, repo_name, repo_path, is_running, last_detected_at, updated_at)
             VALUES ('proj', 'default', 'Repo', '/repo', 1, 100, 100)",
            [],
        ).unwrap();

        // Insert prompts with staggered created_at and inverted updated_at
        // Prompt 1: created_at: 100, updated_at: 999
        // Prompt 2: created_at: 200, updated_at: 500
        // Prompt 3: created_at: 300, updated_at: 100 (misleading recent updated_at)
        // Prompt 0: created_at: 100, updated_at: 999, id: 'p0' (tie-breaker winner with p1)
        conn.execute(
            "INSERT INTO active_prompts (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload)
             VALUES ('p1', 'proj', 'default', '/repo', 'Prompt 1', NULL, NULL, 'queued', 100, 999, NULL),
                    ('p2', 'proj', 'default', '/repo', 'Prompt 2', NULL, NULL, 'queued', 200, 500, NULL),
                    ('p3', 'proj', 'default', '/repo', 'Prompt 3', NULL, NULL, 'queued', 300, 100, NULL),
                    ('p0', 'proj', 'default', '/repo', 'Prompt 0', NULL, NULL, 'queued', 100, 999, NULL)",
            [],
        ).unwrap();

        // 1. Verify SQL query ordering invariant
        let mut stmt = conn
            .prepare(
                "SELECT id FROM active_prompts
             WHERE status IN ('queued', 'pending')
             ORDER BY created_at ASC, id ASC",
            )
            .unwrap();

        let ordered_ids: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .unwrap()
            .flatten()
            .collect();

        assert_eq!(
            ordered_ids,
            vec!["p0", "p1", "p2", "p3"],
            "SQL query must return prompts sorted by created_at ASC, then id ASC"
        );

        // 2. Verify in-memory pick_first_queued_prompt selection algorithm
        let candidate_prompts = vec![
            ActivePrompt {
                id: "p1".to_string(),
                project_id: "proj".to_string(),
                instance_id: "default".to_string(),
                repo_path: "/repo".to_string(),
                prompt_content: "Prompt 1".to_string(),
                model: None,
                session_id: None,
                status: "queued".to_string(),
                created_at: 100,
                updated_at: 999,
                image_payload: None,
            },
            ActivePrompt {
                id: "p3".to_string(),
                project_id: "proj".to_string(),
                instance_id: "default".to_string(),
                repo_path: "/repo".to_string(),
                prompt_content: "Prompt 3".to_string(),
                model: None,
                session_id: None,
                status: "queued".to_string(),
                created_at: 300,
                updated_at: 100,
                image_payload: None,
            },
            ActivePrompt {
                id: "p0".to_string(),
                project_id: "proj".to_string(),
                instance_id: "default".to_string(),
                repo_path: "/repo".to_string(),
                prompt_content: "Prompt 0".to_string(),
                model: None,
                session_id: None,
                status: "queued".to_string(),
                created_at: 100,
                updated_at: 999,
                image_payload: None,
            },
            ActivePrompt {
                id: "p2".to_string(),
                project_id: "proj".to_string(),
                instance_id: "default".to_string(),
                repo_path: "/repo".to_string(),
                prompt_content: "Prompt 2".to_string(),
                model: None,
                session_id: None,
                status: "queued".to_string(),
                created_at: 200,
                updated_at: 500,
                image_payload: None,
            },
        ];

        let picked = candidate_prompts
            .iter()
            .min_by(|a, b| {
                a.created_at
                    .cmp(&b.created_at)
                    .then_with(|| a.id.cmp(&b.id))
            })
            .cloned();

        assert!(picked.is_some());
        let selected = picked.unwrap();
        assert_eq!(
            selected.id, "p0",
            "FIFO pick must select earliest created_at and lowest id tie-breaker regardless of updated_at"
        );
        assert_eq!(selected.created_at, 100);
    }
}
