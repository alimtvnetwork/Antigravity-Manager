//! SQLite PRAGMA diagnostic reporter.
//! Collects and inspects low-level SQLite configuration state to diagnose
//! performance, locking modes, and WAL status across split databases.

use crate::error::AppError;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PragmaDiagnosticReport {
    pub journal_mode: String,
    pub synchronous: i64,
    pub cache_size: i64,
    pub wal_autocheckpoint: i64,
    pub busy_timeout: i64,
    pub page_size: i64,
    pub freelist_count: i64,
}

pub struct PragmaDiagnosticReporter;

impl PragmaDiagnosticReporter {
    /// Queries core diagnostic pragmas on the given SQLite connection.
    pub fn collect_report(conn: &Connection) -> Result<PragmaDiagnosticReport, AppError> {
        let journal_mode = Self::query_string_pragma(conn, "journal_mode")?;
        let synchronous = Self::query_int_pragma(conn, "synchronous")?;
        let cache_size = Self::query_int_pragma(conn, "cache_size")?;
        let wal_autocheckpoint = Self::query_int_pragma(conn, "wal_autocheckpoint")?;
        let busy_timeout = Self::query_int_pragma(conn, "busy_timeout")?;
        let page_size = Self::query_int_pragma(conn, "page_size")?;
        let freelist_count = Self::query_int_pragma(conn, "freelist_count")?;

        Ok(PragmaDiagnosticReport {
            journal_mode,
            synchronous,
            cache_size,
            wal_autocheckpoint,
            busy_timeout,
            page_size,
            freelist_count,
        })
    }

    fn query_string_pragma(conn: &Connection, pragma: &str) -> Result<String, AppError> {
        let mut stmt = conn
            .prepare(&format!("PRAGMA {};", pragma))
            .map_err(AppError::Database)?;

        let mut rows = stmt
            .query([])
            .map_err(AppError::Database)?;

        if let Some(row) = rows
            .next()
            .map_err(AppError::Database)?
        {
            Ok(row.get(0).unwrap_or_default())
        } else {
            Ok(String::new())
        }
    }

    fn query_int_pragma(conn: &Connection, pragma: &str) -> Result<i64, AppError> {
        let mut stmt = conn
            .prepare(&format!("PRAGMA {};", pragma))
            .map_err(AppError::Database)?;

        let mut rows = stmt
            .query([])
            .map_err(AppError::Database)?;

        if let Some(row) = rows
            .next()
            .map_err(AppError::Database)?
        {
            Ok(row.get(0).unwrap_or(0))
        } else {
            Ok(0)
        }
    }
}
