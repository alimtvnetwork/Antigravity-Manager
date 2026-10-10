use chrono::Utc;
use rusqlite::{params, Connection};
use uuid::Uuid;

use super::*;

// ---------------------------------------------------------------------------
// Green Projects Store for FPUG & SUG Watchers
// ---------------------------------------------------------------------------

pub fn add_green_project(
    identifier: &str,
    project_path: &str,
    custom_file: Option<&str>,
) -> Result<(), String> {
    let conn = connect_backup_db(custom_file)?;
    let now = Utc::now().timestamp();
    conn.execute(
        "INSERT INTO green_projects (id, project_identifier, project_path, added_at, status, last_checked_at)
         VALUES (?, ?, ?, ?, 'pending', ?)
         ON CONFLICT(project_identifier) DO UPDATE SET project_path = excluded.project_path, last_checked_at = excluded.last_checked_at",
        params![
            format!("green_{}", Uuid::new_v4().simple()),
            identifier,
            project_path,
            now,
            now
        ],
    )
    .map_err(|e| format!("Failed to add green project: {}", e))?;
    Ok(())
}

pub fn remove_green_project(identifier: &str, custom_file: Option<&str>) -> Result<bool, String> {
    let conn = connect_backup_db(custom_file)?;
    let deleted = conn
        .execute(
            "DELETE FROM green_projects WHERE project_identifier = ? OR project_path = ?",
            params![identifier, identifier],
        )
        .map_err(|e| format!("Failed to remove green project: {}", e))?;
    Ok(deleted > 0)
}

pub fn list_green_projects(custom_file: Option<&str>) -> Result<Vec<GreenProjectRecord>, String> {
    let conn = connect_backup_db(custom_file)?;
    let mut stmt = conn
        .prepare(
            "SELECT id, project_identifier, project_path, added_at, status, last_checked_at
             FROM green_projects ORDER BY added_at DESC",
        )
        .map_err(|e| format!("Failed to prepare list green projects query: {}", e))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(GreenProjectRecord {
                id: row.get(0)?,
                project_identifier: row.get(1)?,
                project_path: row.get(2)?,
                added_at: row.get(3)?,
                status: row.get(4)?,
                last_checked_at: row.get(5)?,
            })
        })
        .map_err(|e| format!("Failed to query green projects: {}", e))?
        .flatten()
        .collect();

    Ok(rows)
}
