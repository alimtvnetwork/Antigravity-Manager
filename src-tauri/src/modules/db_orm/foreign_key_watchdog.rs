//! Foreign Key Watchdog Inspector
//! Periodically executes PRAGMA foreign_key_check to detect orphaned records
//! and dangling relations across split database tables.

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForeignKeyViolationDetail {
    pub table_name: String,
    pub row_id: i64,
    pub parent_table: String,
    pub fkid: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForeignKeyAuditResult {
    pub is_clean: bool,
    pub violation_count: usize,
    pub violations: Vec<ForeignKeyViolationDetail>,
}

pub struct ForeignKeyWatchdog;

impl ForeignKeyWatchdog {
    /// Runs PRAGMA foreign_key_check and parses all violation records.
    pub fn audit_foreign_keys(conn: &Connection) -> Result<ForeignKeyAuditResult, rusqlite::Error> {
        let mut stmt = conn.prepare("PRAGMA foreign_key_check;")?;
        let rows = stmt.query_map([], |row| {
            Ok(ForeignKeyViolationDetail {
                table_name: row.get(0)?,
                row_id: row.get(1)?,
                parent_table: row.get(2)?,
                fkid: row.get(3)?,
            })
        })?;

        let mut violations = Vec::new();
        for r in rows {
            violations.push(r?);
        }

        let is_clean = violations.is_empty();
        let violation_count = violations.len();

        Ok(ForeignKeyAuditResult {
            is_clean,
            violation_count,
            violations,
        })
    }
}
