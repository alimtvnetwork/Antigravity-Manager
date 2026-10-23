//! Standardized SQLite connection and schema initializer for Split-DB architecture.

use crate::error::AppError;
use crate::modules::db_orm::generated::tables::ALL_TABLES;
use crate::modules::db_orm::generated::views::ALL_VIEWS;
use diesel::connection::Connection as DieselConnection;
use diesel::sqlite::SqliteConnection;
use rusqlite::{Connection, OpenFlags};
use std::fs;
use std::path::Path;

/// Opens a connection to a Split-DB SQLite file with WAL mode, busy timeout, foreign keys, and incremental auto-vacuum.
pub fn open_split_db(path: &Path) -> Result<Connection, AppError> {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(AppError::Io)?;
        }
    }

    let conn = Connection::open(path).map_err(AppError::Database)?;

    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;
         PRAGMA cache_size = -64000;
         PRAGMA busy_timeout = 5000;
         PRAGMA foreign_keys = ON;
         PRAGMA auto_vacuum = INCREMENTAL;",
    )
    .map_err(AppError::Database)?;

    Ok(conn)
}

/// Imp-43: Opens a dedicated read-only connection to a Split-DB SQLite file for concurrent lock-free queries.
pub fn open_split_db_readonly(path: &Path) -> Result<Connection, AppError> {
    if !path.exists() {
        return Err(AppError::Config(format!(
            "Cannot open read-only database: file does not exist: {:?}",
            path
        )));
    }

    let flags = OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX;
    let conn = Connection::open_with_flags(path, flags).map_err(AppError::Database)?;

    conn.execute_batch(
        "PRAGMA busy_timeout = 5000;
         PRAGMA query_only = ON;",
    )
    .map_err(AppError::Database)?;

    Ok(conn)
}

/// Imp-48: Runs a fast non-blocking PRAGMA quick_check on the database and logs speed errors if corrupted.
pub fn verify_split_db_integrity(conn: &Connection, db_name: &str) -> Result<bool, AppError> {
    let result: String = conn
        .query_row("PRAGMA quick_check(1);", [], |row| row.get(0))
        .map_err(AppError::Database)?;

    if result.eq_ignore_ascii_case("ok") {
        Ok(true)
    } else {
        tracing::error!("SQLite integrity check failed for {db_name}: {result}");
        crate::modules::speed_errors::record_speed_error(
            "E2001",
            "error",
            &format!("SQLite database {db_name} integrity check failed: {result}"),
            "db_orm::connection",
            "verify_split_db_integrity",
            Some(&format!(
                r#"{{"database":"{db_name}","result":"{result}"}}"#
            )),
        );
        Ok(false)
    }
}

/// Imp-73: WAL Concurrency Checkpointer
/// Runs `PRAGMA wal_checkpoint(...)` to merge WAL log frames into the main database file without blocking active readers.
/// Returns (busy, log_frames, checkpointed_frames).
pub fn checkpoint_split_db(conn: &Connection, mode: &str) -> Result<(i32, i32, i32), AppError> {
    let mode_upper = mode.trim().to_uppercase();
    let valid_mode = match mode_upper.as_str() {
        "PASSIVE" | "FULL" | "RESTART" | "TRUNCATE" => mode_upper.as_str(),
        _ => "PASSIVE",
    };
    let sql = format!("PRAGMA wal_checkpoint({});", valid_mode);
    let mut stmt = conn.prepare(&sql).map_err(AppError::Database)?;
    let mut rows = stmt.query([]).map_err(AppError::Database)?;
    if let Some(row) = rows.next().map_err(AppError::Database)? {
        let busy: i32 = row.get(0).unwrap_or(0);
        let log: i32 = row.get(1).unwrap_or(0);
        let checkpointed: i32 = row.get(2).unwrap_or(0);
        Ok((busy, log, checkpointed))
    } else {
        Ok((0, 0, 0))
    }
}

/// Imp-74: PRAGMA Optimize Idle Maintenance
/// Executes `PRAGMA optimize;` to refresh SQLite query planner statistics and optimize table indices.
pub fn optimize_split_db(conn: &Connection) -> Result<(), AppError> {
    conn.execute_batch("PRAGMA optimize;")
        .map_err(AppError::Database)?;
    Ok(())
}

/// Opens a compile-time type-safe Diesel SQLite connection for Split-DB architecture.
pub fn open_diesel_split_db(path: &Path) -> Result<SqliteConnection, AppError> {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(AppError::Io)?;
        }
    }

    let url = path.to_str().ok_or_else(|| {
        AppError::Config("Invalid database path for Diesel connection".to_string())
    })?;
    SqliteConnection::establish(url).map_err(|e| {
        AppError::Config(format!(
            "Failed to establish Diesel SQLite connection to {url}: {e}"
        ))
    })
}

/// Initializes all PascalCase tables and views registered for a specific Split-DB file.
pub fn init_split_db_schema(conn: &Connection, db_file: &str) -> Result<(), AppError> {
    // 1. Ensure SchemaVersion table is created first
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

    let target_file = Path::new(db_file)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(db_file);

    // 2. Create registered tables for this db_file
    for table in ALL_TABLES {
        if table.db_file == target_file || table.db_file == db_file {
            conn.execute_batch(table.ddl).map_err(AppError::Database)?;
        }
    }

    // 3. Create registered views for this db_file
    for view in ALL_VIEWS {
        if view.db_file == target_file || view.db_file == db_file {
            conn.execute_batch(view.ddl).map_err(AppError::Database)?;
        }
    }

    // 4. Record initial SchemaVersion = 1 if none exists
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM SchemaVersion WHERE VersionNumber = 1",
            [],
            |row| row.get(0),
        )
        .map_err(AppError::Database)?;

    if count == 0 {
        let now = chrono::Utc::now().timestamp();
        conn.execute(
            "INSERT INTO SchemaVersion (VersionNumber, AppliedAt, Description, Checksum)
             VALUES (1, ?, 'PascalCase Split-DB Schema with Views', 'v1_initial_pascalcase')",
            [now],
        )
        .map_err(AppError::Database)?;
    }

    Ok(())
}
