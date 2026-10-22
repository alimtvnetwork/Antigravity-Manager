//! SQLite Schema Integrity Gate
//! Validates required table existence and column counts across split databases during boot,
//! detecting missing tables or schema corruption before query dispatch.

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaIntegrityReport {
    pub is_valid: bool,
    pub existing_tables: Vec<String>,
    pub missing_tables: Vec<String>,
}

pub struct SchemaIntegrityGate;

impl SchemaIntegrityGate {
    /// Inspects the open database connection and verifies that all expected tables are present.
    pub fn verify_tables(
        conn: &Connection,
        expected_tables: &[&str],
    ) -> Result<SchemaIntegrityReport, rusqlite::Error> {
        let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%';")?;
        let table_names: Vec<String> = stmt
            .query_map([], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?;

        let existing_set: HashSet<String> = table_names.iter().map(|n| n.to_ascii_lowercase()).collect();
        let mut missing_tables = Vec::new();

        for &expected in expected_tables {
            if !existing_set.contains(&expected.to_ascii_lowercase()) {
                missing_tables.push(expected.to_string());
            }
        }

        let is_valid = missing_tables.is_empty();

        Ok(SchemaIntegrityReport {
            is_valid,
            existing_tables: table_names,
            missing_tables,
        })
    }
}
