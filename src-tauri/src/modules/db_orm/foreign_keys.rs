//! Split-DB Foreign Key Constraint Validator.
//! Validates `PRAGMA foreign_key_check` across split databases on demand, returning table-level orphan record diagnostics.

use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};

/// Detailed violation record returned by SQLite `PRAGMA foreign_key_check`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForeignKeyViolation {
    pub table_name: String,
    pub row_id: i64,
    pub parent_table: String,
    pub constraint_idx: i32,
}

/// Comprehensive report of foreign key validation across the connected database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForeignKeyCheckReport {
    pub is_valid: bool,
    pub violations: Vec<ForeignKeyViolation>,
    pub inspected_at_epoch: i64,
}

/// Runs `PRAGMA foreign_key_check` and collects any broken references or orphan rows.
pub fn check_foreign_key_constraints(conn: &Connection) -> Result<ForeignKeyCheckReport> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let mut stmt = conn.prepare("PRAGMA foreign_key_check")?;
    let rows = stmt.query_map([], |row| {
        let table_name: String = row.get(0)?;
        let row_id: i64 = row.get(1)?;
        let parent_table: String = row.get(2)?;
        let constraint_idx: i32 = row.get(3)?;
        Ok(ForeignKeyViolation {
            table_name,
            row_id,
            parent_table,
            constraint_idx,
        })
    })?;

    let mut violations = Vec::new();
    for v in rows {
        violations.push(v?);
    }

    let is_valid = violations.is_empty();

    Ok(ForeignKeyCheckReport {
        is_valid,
        violations,
        inspected_at_epoch: now,
    })
}

/// Enforces that foreign key constraints are enabled on this connection.
pub fn ensure_foreign_keys_enabled(conn: &Connection) -> Result<()> {
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    Ok(())
}
