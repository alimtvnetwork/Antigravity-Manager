//! Repo DB: failed commands

use super::models::FailedCommandRecord;
use super::schema::connect_db;
use rusqlite::params;

pub fn log_failed_command(
    command: &str,
    full_args: &str,
    domain: &str,
    error_code: &str,
    message: &str,
    suggestions: &[String],
) -> Result<i64, String> {
    let conn = connect_db()?;
    let cmd_trimmed = command.trim();
    let domain_trimmed = if domain.is_empty() { "root" } else { domain };
    let err_code = if error_code.is_empty() {
        "E1001"
    } else {
        error_code
    };
    let sugg_str = suggestions.join(", ");
    let working_dir = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    let agm_version = env!("CARGO_PKG_VERSION");

    let mut stmt = conn
        .prepare("SELECT id, hit_count FROM failed_commands WHERE LOWER(command) = LOWER(?1) AND LOWER(domain) = LOWER(?2) LIMIT 1")
        .map_err(|e| e.to_string())?;

    let existing: Result<(i64, i64), _> = stmt
        .query_row(rusqlite::params![cmd_trimmed, domain_trimmed], |row| {
            Ok((row.get(0)?, row.get(1)?))
        });

    if let Ok((id, _)) = existing {
        conn.execute(
            "UPDATE failed_commands SET
                full_args = ?1,
                error_code = ?2,
                message = ?3,
                suggestions = ?4,
                hit_count = hit_count + 1,
                working_dir = ?5,
                agm_version = ?6,
                is_resolved = 0,
                last_seen_at = CURRENT_TIMESTAMP
            WHERE id = ?7",
            rusqlite::params![
                full_args,
                err_code,
                message,
                sugg_str,
                working_dir,
                agm_version,
                id
            ],
        )
        .map_err(|e| e.to_string())?;
        return Ok(id);
    }

    conn.execute(
        "INSERT INTO failed_commands (
            command, full_args, domain, error_code, message, suggestions, hit_count, working_dir, agm_version, is_resolved
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1, ?7, ?8, 0)",
        rusqlite::params![
            cmd_trimmed,
            full_args,
            domain_trimmed,
            err_code,
            message,
            sugg_str,
            working_dir,
            agm_version
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(conn.last_insert_rowid())
}

pub fn count_failed_commands() -> Result<(i64, i64), String> {
    let conn = connect_db()?;
    let mut stmt = conn
        .prepare("SELECT COUNT(*), COALESCE(SUM(hit_count), 0) FROM failed_commands")
        .map_err(|e| e.to_string())?;
    stmt.query_row([], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(|e| e.to_string())
}

pub fn list_failed_commands(limit: usize) -> Result<Vec<FailedCommandRecord>, String> {
    let conn = connect_db()?;
    let mut stmt = conn
        .prepare("SELECT id, command, full_args, domain, error_code, message, suggestions, hit_count, working_dir, agm_version, is_resolved, created_at, last_seen_at FROM failed_commands ORDER BY hit_count DESC, id DESC LIMIT ?1")
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(rusqlite::params![limit as i64], |row| {
            let is_res: i64 = row.get(10)?;
            Ok(FailedCommandRecord {
                id: row.get(0)?,
                command: row.get(1)?,
                full_args: row.get(2)?,
                domain: row.get(3)?,
                error_code: row.get(4)?,
                message: row.get(5)?,
                suggestions: row.get(6)?,
                hit_count: row.get(7)?,
                working_dir: row.get(8)?,
                agm_version: row.get(9)?,
                is_resolved: is_res != 0,
                created_at: row.get(11)?,
                last_seen_at: row.get(12)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut res = Vec::new();
    for r in rows {
        if let Ok(rec) = r {
            res.push(rec);
        }
    }
    Ok(res)
}

pub fn clear_failed_commands() -> Result<usize, String> {
    let conn = connect_db()?;
    let count = conn
        .execute("DELETE FROM failed_commands", [])
        .map_err(|e| e.to_string())?;
    Ok(count)
}

/// Decode file URI (e.g., file:///path/to/folder or file:///c%3A/path) to local path
pub fn decode_uri_to_path_pub(uri: &str) -> String {
    decode_uri_to_path(uri)
}

pub(crate) fn decode_uri_to_path(uri: &str) -> String {
    let stripped = uri
        .strip_prefix("file:///")
        .or_else(|| uri.strip_prefix("file://"))
        .unwrap_or(uri);

    let replaced = stripped
        .replace("%20", " ")
        .replace("%3A", ":")
        .replace("%3a", ":");

    #[cfg(target_os = "windows")]
    {
        replaced.replace('/', "\\")
    }
    #[cfg(not(target_os = "windows"))]
    {
        if replaced.starts_with('/') {
            replaced
        } else {
            format!("/{}", replaced)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uri_decoding() {
        let uri = "file:///d:/work/My%20Project";
        let path = decode_uri_to_path(uri);
        assert!(path.contains("My Project"));
    }
}
