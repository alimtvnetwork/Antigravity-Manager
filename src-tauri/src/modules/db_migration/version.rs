//! Schema version tracking and forward migration checks.

use crate::error::AppError;
use rusqlite::{Connection, OpenFlags};
use std::path::Path;

/// Canonical target schema version for PascalCase Split-DB architecture.
pub const TARGET_SCHEMA_VERSION: i64 = 1;

/// Fetches the current latest schema version number from the database.
pub fn get_schema_version(conn: &Connection) -> Result<i64, AppError> {
    // Check if SchemaVersion table exists
    let table_exists: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'SchemaVersion'",
            [],
            |row| row.get(0),
        )
        .map_err(AppError::Database)?;

    if table_exists == 0 {
        return Ok(0);
    }

    let version: Option<i64> = conn
        .query_row(
            "SELECT VersionNumber FROM SchemaVersion ORDER BY VersionNumber DESC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .map(Some)
        .unwrap_or(None);

    Ok(version.unwrap_or(0))
}

/// Fast boot check (<10ms) determining if a database is on the current schema version.
pub fn is_schema_current(conn: &Connection) -> bool {
    match get_schema_version(conn) {
        Ok(v) => v >= TARGET_SCHEMA_VERSION,
        Err(e) => {
            crate::error::record_ignored(
                Err::<(), _>(e),
                "is_schema_current check failed during fast boot probe",
            );
            // Justification: Fast-path boot probe safely returns false on uninitialized or corrupt databases.
            false
        }
    }
}

/// Checks whether the database has already been migrated to PascalCase schema (Version >= 1).
pub fn is_migrated_to_pascalcase(conn: &Connection) -> bool {
    is_schema_current(conn)
}

/// Records a new schema version migration in the database.
pub fn record_schema_version(
    conn: &Connection,
    version: i64,
    description: &str,
    checksum: &str,
) -> Result<(), AppError> {
    let now = chrono::Utc::now().timestamp();
    conn.execute(
        "INSERT INTO SchemaVersion (VersionNumber, AppliedAt, Description, Checksum)
         VALUES (?, ?, ?, ?)",
        rusqlite::params![version, now, description, checksum],
    )
    .map_err(AppError::Database)?;

    Ok(())
}

/// Seeds the initial SchemaVersion record (VersionNumber = 1) if not already present.
pub fn seed_initial_schema_version(
    conn: &Connection,
    description: Option<&str>,
    checksum: Option<&str>,
) -> Result<(), AppError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS SchemaVersion (
            SchemaVersionId INTEGER PRIMARY KEY AUTOINCREMENT,
            VersionNumber INTEGER NOT NULL,
            AppliedAt INTEGER NOT NULL,
            Description TEXT NOT NULL,
            Checksum TEXT NOT NULL
        );",
    )
    .map_err(AppError::Database)?;

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM SchemaVersion WHERE VersionNumber = ?",
            [TARGET_SCHEMA_VERSION],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if count == 0 {
        let desc = description.unwrap_or("PascalCase Split-DB Schema with Views");
        let chk = checksum.unwrap_or("v1_initial_pascalcase");
        record_schema_version(conn, TARGET_SCHEMA_VERSION, desc, chk)?;
    }

    Ok(())
}

/// Fast boot check (<10ms) inspecting all existing Split-DB files in the data directory.
/// Returns `true` if all existing databases are already migrated to `TARGET_SCHEMA_VERSION`.
pub fn is_all_schemas_current(data_dir: &Path) -> bool {
    let db_paths = [
        data_dir.join("errors.db"),
        data_dir.join("repo_prompts.db"),
        data_dir.join("backup-prompts").join("backup-prompts.db"),
    ];

    let mut existing_count = 0;
    for db_path in &db_paths {
        if db_path.exists() {
            existing_count += 1;
            match Connection::open_with_flags(
                db_path,
                OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
            ) {
                Ok(conn) => {
                    if !is_schema_current(&conn) {
                        return false;
                    }
                }
                Err(e) => {
                    crate::error::record_ignored(
                        Err::<(), _>(e),
                        "failed to open split db in read-only mode during boot check",
                    );
                    // Justification: Unopenable database must be treated as not current to trigger repair/migration.
                    return false;
                }
            }
        }
    }

    // If no databases exist yet, fresh installation is considered current (will be initialized on access)
    existing_count == 0 || true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_seeding_and_fast_check() {
        let conn = Connection::open_in_memory().unwrap();
        assert!(!is_schema_current(&conn));

        seed_initial_schema_version(&conn, None, None).unwrap();
        assert!(is_schema_current(&conn));
        assert_eq!(get_schema_version(&conn).unwrap(), 1);

        // Idempotency check: repeated seed shouldn't duplicate version 1
        seed_initial_schema_version(&conn, None, None).unwrap();
        assert_eq!(get_schema_version(&conn).unwrap(), 1);
    }
}
