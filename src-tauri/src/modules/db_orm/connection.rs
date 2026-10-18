//! Standardized SQLite connection and schema initializer for Split-DB architecture.

use crate::error::AppError;
use crate::modules::db_orm::generated::tables::ALL_TABLES;
use crate::modules::db_orm::generated::views::ALL_VIEWS;
use diesel::connection::Connection as DieselConnection;
use diesel::sqlite::SqliteConnection;
use rusqlite::Connection;
use std::fs;
use std::path::Path;

/// Opens a connection to a Split-DB SQLite file with WAL mode, busy timeout, and foreign keys.
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
         PRAGMA busy_timeout = 5000;
         PRAGMA foreign_keys = ON;",
    )
    .map_err(AppError::Database)?;

    Ok(conn)
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
