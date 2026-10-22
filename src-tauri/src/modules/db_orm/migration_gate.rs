//! Split-DB Schema Migration Verification Gate.
//! Inspects table column definitions against expected schemas, verifying all required columns exist prior to app boot.

use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};

/// Expected table schema requirements for validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaRequirement {
    pub table_name: String,
    pub required_columns: Vec<String>,
}

impl SchemaRequirement {
    pub fn new(table_name: impl Into<String>, required_columns: Vec<impl Into<String>>) -> Self {
        Self {
            table_name: table_name.into(),
            required_columns: required_columns.into_iter().map(Into::into).collect(),
        }
    }
}

/// Verification report detailing missing tables or columns.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SchemaVerificationReport {
    pub passed: bool,
    pub missing_tables: Vec<String>,
    pub missing_columns: Vec<(String, String)>,
    pub checked_tables_count: usize,
}

/// Verifies that the connected SQLite database contains all required tables and columns.
pub fn verify_database_schema(
    conn: &Connection,
    requirements: &[SchemaRequirement],
) -> Result<SchemaVerificationReport> {
    let mut missing_tables = Vec::new();
    let mut missing_columns = Vec::new();

    for req in requirements {
        let table_exists: bool = conn.query_row(
            "SELECT count(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
            [&req.table_name],
            |row| {
                let cnt: i64 = row.get(0)?;
                Ok(cnt > 0)
            },
        )?;

        if !table_exists {
            missing_tables.push(req.table_name.clone());
            continue;
        }

        // Fetch existing columns for this table
        let pragma_sql = format!("PRAGMA table_info('{}')", req.table_name.replace('\'', "''"));
        let mut stmt = conn.prepare(&pragma_sql)?;
        let col_names: Vec<String> = stmt
            .query_map([], |row| row.get::<_, String>(1))?
            .filter_map(|r| r.ok())
            .collect();

        for required_col in &req.required_columns {
            let col_found = col_names
                .iter()
                .any(|c| c.eq_ignore_ascii_case(required_col));
            if !col_found {
                missing_columns.push((req.table_name.clone(), required_col.clone()));
            }
        }
    }

    let passed = missing_tables.is_empty() && missing_columns.is_empty();

    Ok(SchemaVerificationReport {
        passed,
        missing_tables,
        missing_columns,
        checked_tables_count: requirements.len(),
    })
}
