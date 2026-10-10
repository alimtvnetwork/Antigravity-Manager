//! Repo DB: clone rows

use super::schema::{connect_db, init_tables};
use rusqlite::params;
use rusqlite::Connection;
use std::path::Path;

/// Duplicate one instance's repo rows onto another instance id.
/// Project and prompt ids are suffixed so the source rows stay intact.
pub fn clone_instance_repo_rows(source_id: &str, target_id: &str) -> Result<usize, String> {
    if source_id == target_id || target_id.is_empty() {
        return Ok(0);
    }
    let conn = connect_db()?;
    clone_repo_rows_on(&conn, source_id, target_id)
}

pub(crate) fn clone_repo_rows_on(
    conn: &Connection,
    source_id: &str,
    target_id: &str,
) -> Result<usize, String> {
    let mut project_stmt = conn
        .prepare(
            "SELECT id, repo_name, repo_path, workspace_storage_path, is_running, last_detected_at, updated_at
             FROM running_projects
             WHERE instance_id = ?1 OR (instance_id = '__default__' AND ?1 = 'default')",
        )
        .map_err(|e| format!("Failed to read source repos: {}", e))?;
    let projects: Vec<(String, String, String, Option<String>, i64, i64, i64)> = project_stmt
        .query_map([source_id], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get(6)?,
            ))
        })
        .map_err(|e| format!("Failed to map source repos: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect source repos: {}", e))?;
    drop(project_stmt);

    let mut copied = 0usize;
    for (id, name, path, storage, _running, detected, updated) in projects {
        let Some(ref storage_path) = storage else {
            continue;
        };
        if !Path::new(storage_path).exists() {
            continue;
        }

        let base_id = id.split("__").next().unwrap_or(&id);
        let new_id = format!("{}__{}", base_id, target_id);
        conn.execute(
            "INSERT OR IGNORE INTO running_projects
             (id, instance_id, repo_name, repo_path, workspace_storage_path, is_running, last_detected_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6, ?7)",
            rusqlite::params![new_id, target_id, name, path, storage, detected, updated],
        )
        .map_err(|e| format!("Failed to clone repo {}: {}", id, e))?;

        let mut prompt_stmt = conn
            .prepare(
                "SELECT id, prompt_content, model, session_id, status, created_at, updated_at, image_payload
                 FROM active_prompts
                 WHERE project_id = ?1 AND (instance_id = ?2 OR (instance_id = '__default__' AND ?2 = 'default'))",
            )
            .map_err(|e| format!("Failed to read source prompts: {}", e))?;
        let prompts: Vec<(
            String,
            String,
            Option<String>,
            Option<String>,
            String,
            i64,
            i64,
            Option<String>,
        )> = prompt_stmt
            .query_map(rusqlite::params![id, source_id], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                    row.get(7)?,
                ))
            })
            .map_err(|e| format!("Failed to map source prompts: {}", e))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to collect source prompts: {}", e))?;
        drop(prompt_stmt);

        for (prompt_id, content, model, session, status, created, prompt_updated, image) in prompts
        {
            let new_prompt_id = format!("{}__{}", prompt_id, target_id);
            let sanitized_status = match status.as_str() {
                "running" | "dispatched" => "completed".to_string(),
                other => other.to_string(),
            };
            conn.execute(
                "INSERT OR IGNORE INTO active_prompts
                 (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                rusqlite::params![
                    new_prompt_id,
                    new_id,
                    target_id,
                    path,
                    content,
                    model,
                    session,
                    sanitized_status,
                    created,
                    prompt_updated,
                    image
                ],
            )
            .map_err(|e| format!("Failed to clone prompt {}: {}", prompt_id, e))?;
        }
        copied += 1;
    }
    Ok(copied)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::repo_db::init_tables;
    use rusqlite::Connection;

    #[test]
    fn clone_repo_rows_keeps_source_and_copies_onto_target() {
        let conn = Connection::open_in_memory().unwrap();
        assert!(init_tables(&conn).is_ok());
        conn.execute(
            "INSERT INTO running_projects
             (id, instance_id, repo_name, repo_path, workspace_storage_path, is_running, last_detected_at, updated_at)
             VALUES ('proj-1', 'default', 'Antigravity', 'D:/work/antigravity-manager', NULL, 1, 10, 10)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO active_prompts
             (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload)
             VALUES ('prompt-1', 'proj-1', 'default', 'D:/work/antigravity-manager', 'keep going', NULL, NULL, 'running', 10, 10, NULL)",
            [],
        )
        .unwrap();

        let copied = clone_repo_rows_on(&conn, "default", "copy-1").unwrap();
        assert_eq!(copied, 1);

        let source_projects: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM running_projects WHERE instance_id = 'default'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let target_projects: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM running_projects WHERE instance_id = 'copy-1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let target_prompts: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM active_prompts WHERE instance_id = 'copy-1' AND prompt_content = 'keep going'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(source_projects, 1);
        assert_eq!(target_projects, 1);
        assert_eq!(target_prompts, 1);

        let cloned_running: i64 = conn
            .query_row(
                "SELECT is_running FROM running_projects WHERE instance_id = 'copy-1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(cloned_running, 0);

        let cloned_status: String = conn
            .query_row(
                "SELECT status FROM active_prompts WHERE instance_id = 'copy-1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(cloned_status, "completed");
    }

    #[test]
    fn test_clone_repo_rows_matches_default_and_under_default() {
        let conn = Connection::open_in_memory().unwrap();
        assert!(init_tables(&conn).is_ok());

        // Project with 'default'
        conn.execute(
            "INSERT INTO running_projects
             (id, instance_id, repo_name, repo_path, workspace_storage_path, is_running, last_detected_at, updated_at)
             VALUES ('proj-def', 'default', 'DefRepo', 'D:/work/def-repo', NULL, 1, 10, 10)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO active_prompts
             (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload)
             VALUES ('prompt-def', 'proj-def', 'default', 'D:/work/def-repo', 'prompt def', NULL, NULL, 'running', 10, 10, NULL)",
            [],
        )
        .unwrap();

        // Project with '__default__'
        conn.execute(
            "INSERT INTO running_projects
             (id, instance_id, repo_name, repo_path, workspace_storage_path, is_running, last_detected_at, updated_at)
             VALUES ('proj-under', '__default__', 'UnderRepo', 'D:/work/under-repo', NULL, 1, 10, 10)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO active_prompts
             (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload)
             VALUES ('prompt-under', 'proj-under', '__default__', 'D:/work/under-repo', 'prompt under', NULL, NULL, 'running', 10, 10, NULL)",
            [],
        )
        .unwrap();

        let copied = clone_repo_rows_on(&conn, "default", "target-inst").unwrap();
        assert_eq!(copied, 2);

        let target_projects: Vec<(String, String)> = conn
            .prepare(
                "SELECT id, instance_id FROM running_projects WHERE instance_id = 'target-inst'",
            )
            .unwrap()
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(target_projects.len(), 2);
        for (_, inst_id) in &target_projects {
            assert_eq!(inst_id, "target-inst");
        }

        let target_prompts: Vec<(String, String, String)> = conn
            .prepare("SELECT id, project_id, instance_id FROM active_prompts WHERE instance_id = 'target-inst'")
            .unwrap()
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(target_prompts.len(), 2);
        for (_, proj_id, inst_id) in &target_prompts {
            assert_eq!(inst_id, "target-inst");
            assert!(proj_id.ends_with("__target-inst"));
        }
    }
}
