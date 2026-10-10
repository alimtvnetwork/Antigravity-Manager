//! Repo DB: schema

use super::state::STARTUP_PURGE_ONCE;
use rusqlite::Connection;
use std::fs;
use std::path::PathBuf;

/// Get path to the dedicated split repo prompts SQLite database
pub fn get_repo_db_path() -> Result<PathBuf, String> {
    let mut path = crate::modules::account::get_data_dir()?;
    path.push("repo_prompts.db");
    Ok(path)
}

/// Connect to the repo SQLite database with WAL mode and 5000ms busy timeout
pub fn connect_db() -> Result<Connection, String> {
    let path = get_repo_db_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create repo database directory: {}", e))?;
    }
    let conn =
        Connection::open(&path).map_err(|e| format!("Failed to open repo database: {}", e))?;
    conn.pragma_update(None, "journal_mode", "WAL")
        .map_err(|e| format!("Failed to set WAL mode on repo database: {}", e))?;
    conn.pragma_update(None, "busy_timeout", 5000)
        .map_err(|e| format!("Failed to set busy timeout on repo database: {}", e))?;
    init_tables(&conn)?;
    Ok(conn)
}

/// Initialize SQLite schema for running projects and active prompts
pub(crate) fn init_tables(conn: &Connection) -> Result<(), String> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS running_projects (
            id TEXT PRIMARY KEY,
            instance_id TEXT NOT NULL,
            repo_name TEXT NOT NULL,
            repo_path TEXT NOT NULL,
            workspace_storage_path TEXT,
            is_running INTEGER NOT NULL DEFAULT 1,
            last_detected_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        )",
        [],
    )
    .map_err(|e| format!("Failed to create running_projects table: {}", e))?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS active_prompts (
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
            image_payload TEXT,
            FOREIGN KEY(project_id) REFERENCES running_projects(id)
        )",
        [],
    )
    .map_err(|e| format!("Failed to create active_prompts table: {}", e))?;

    // Migration: add image_payload column if it doesn't exist yet
    // Justification: idempotent schema migration; failure is expected when the column already exists
    crate::error::record_ignored(
        conn.execute(
            "ALTER TABLE active_prompts ADD COLUMN image_payload TEXT",
            [],
        ),
        "add image_payload column migration",
    );

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_active_prompts_instance_status 
         ON active_prompts(instance_id, status)",
        [],
    )
    .map_err(|e| format!("Failed to create index on active_prompts: {}", e))?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS agm_project_sequences (
            project_key TEXT PRIMARY KEY,
            seq_id INTEGER NOT NULL UNIQUE,
            project_id TEXT NOT NULL,
            repo_name TEXT NOT NULL,
            repo_path TEXT NOT NULL,
            instance_id TEXT NOT NULL DEFAULT 'default',
            updated_at INTEGER NOT NULL
        )",
        [],
    )
    .map_err(|e| format!("Failed to create agm_project_sequences table: {}", e))?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS agm_conversation_sequences (
            conversation_id TEXT PRIMARY KEY,
            seq_id INTEGER NOT NULL UNIQUE,
            project_key TEXT NOT NULL,
            title TEXT NOT NULL,
            instance_id TEXT NOT NULL DEFAULT 'default',
            updated_at INTEGER NOT NULL
        )",
        [],
    )
    .map_err(|e| format!("Failed to create agm_conversation_sequences table: {}", e))?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS failed_commands (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            command TEXT NOT NULL DEFAULT '',
            full_args TEXT NOT NULL DEFAULT '',
            domain TEXT NOT NULL DEFAULT 'root',
            error_code TEXT NOT NULL DEFAULT 'E1001',
            message TEXT NOT NULL DEFAULT '',
            suggestions TEXT NOT NULL DEFAULT '',
            hit_count INTEGER NOT NULL DEFAULT 1,
            working_dir TEXT NOT NULL DEFAULT '',
            agm_version TEXT NOT NULL DEFAULT '',
            is_resolved INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            last_seen_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )
    .map_err(|e| format!("Failed to create failed_commands table: {}", e))?;

    // Justification: lookup index is a performance accelerator; queries work without it
    crate::error::record_ignored(
        conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_failed_commands_cmd ON failed_commands(command, domain)",
        [],
    ),
        "create idx_failed_commands_cmd index",
    );
    // Justification: lookup index is a performance accelerator; queries work without it
    crate::error::record_ignored(
        conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_failed_commands_hits ON failed_commands(hit_count DESC)",
        [],
    ),
        "create idx_failed_commands_hits index",
    );
    // Justification: legacy convenience view; nothing queries it and creation is idempotent
    crate::error::record_ignored(
        conn.execute(
            "CREATE VIEW IF NOT EXISTS failed_to_detect_commands AS SELECT * FROM failed_commands",
            [],
        ),
        "create failed_to_detect_commands view",
    );

    conn.execute(
        "CREATE TABLE IF NOT EXISTS prompt_tree_cache (
            cache_key TEXT PRIMARY KEY,
            instance_id TEXT NOT NULL,
            tree_json TEXT NOT NULL,
            project_count INTEGER NOT NULL,
            conversation_count INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            ttl_seconds INTEGER NOT NULL DEFAULT 60
        )",
        [],
    )
    .map_err(|e| format!("Failed to create prompt_tree_cache table: {}", e))?;

    // Justification: lookup index is a performance accelerator; queries work without it
    crate::error::record_ignored(
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_running_projects_inst_running
         ON running_projects(instance_id, is_running)",
            [],
        ),
        "create idx_running_projects_inst_running index",
    );

    STARTUP_PURGE_ONCE.call_once(|| {
        match purge_corrupted_running_projects(conn) {
            Ok((del_proj, del_cache)) => {
                crate::modules::logger::log_info(&format!(
                    "[RepoDB] Startup purge executed: {} polluted running_projects rows deleted, {} prompt_tree_cache rows wiped",
                    del_proj, del_cache
                ));
            }
            Err(err) => {
                crate::modules::logger::log_error(&format!(
                    "[RepoDB] Startup purge error: {}", err
                ));
            }
        }
    });

    Ok(())
}

/// Purge corrupted and un-namespaced running_projects rows and clear stale prompt tree cache
pub fn purge_corrupted_running_projects(conn: &Connection) -> Result<(usize, usize), String> {
    // 1. Delete any active_prompts referencing corrupted/un-namespaced running_projects to prevent FK constraint failures
    let _ = conn.execute(
        "DELETE FROM active_prompts 
         WHERE project_id IN (
             SELECT id FROM running_projects 
             WHERE workspace_storage_path IS NULL 
                OR trim(workspace_storage_path) = '' 
                OR instr(id, '__') = 0 
                OR trim(instance_id) = ''
         )",
        [],
    );

    let deleted_projects = conn
        .execute(
            "DELETE FROM running_projects 
             WHERE workspace_storage_path IS NULL 
                OR trim(workspace_storage_path) = '' 
                OR instr(id, '__') = 0 
                OR trim(instance_id) = ''",
            [],
        )
        .map_err(|e| format!("Failed to purge corrupted running_projects: {}", e))?;

    conn.execute("UPDATE running_projects SET is_running = 0", [])
        .map_err(|e| format!("Failed to reset running flags during purge: {}", e))?;

    let deleted_cache = conn
        .execute("DELETE FROM prompt_tree_cache", [])
        .map_err(|e| format!("Failed to wipe prompt_tree_cache: {}", e))?;

    Ok((deleted_projects, deleted_cache))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_repo_db_schema_initialization() {
        let conn = Connection::open_in_memory().unwrap();
        // Justification: test fixture on an in-memory database; the WAL pragma is a no-op here and init_tables is asserted below
        let _ = conn.pragma_update(None, "journal_mode", "WAL");
        assert!(init_tables(&conn).is_ok());

        // Insert mock project
        let res = conn.execute(
            "INSERT INTO running_projects 
             (id, instance_id, repo_name, repo_path, workspace_storage_path, is_running, last_detected_at, updated_at)
             VALUES ('test-proj', 'default', 'TestRepo', '/work/test', NULL, 1, 100, 100)",
            [],
        );
        assert!(res.is_ok());
    }
}
