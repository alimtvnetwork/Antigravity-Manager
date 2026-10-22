// Split Database Schema User Version Gate (Imp-495)
// Validates PRAGMA user_version against expected binary schema milestones,
// guarding against corrupt states or mismatched migration sequences.

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaVersionReport {
    pub current_version: u32,
    pub expected_version: u32,
    pub is_compatible: bool,
    pub requires_migration: bool,
}

pub struct SchemaVersionChecker;

impl SchemaVersionChecker {
    /// Reads the current schema user_version from the SQLite database.
    pub fn read_user_version(conn: &Connection) -> Result<u32, rusqlite::Error> {
        let mut stmt = conn.prepare("PRAGMA user_version;")?;
        let version: u32 = stmt.query_row([], |row| row.get(0))?;
        Ok(version)
    }

    /// Evaluates whether the database's user_version meets the compiled threshold.
    pub fn verify_version(conn: &Connection, expected_version: u32) -> Result<SchemaVersionReport, rusqlite::Error> {
        let current_version = Self::read_user_version(conn)?;
        let is_compatible = current_version >= expected_version;
        let requires_migration = current_version < expected_version;

        Ok(SchemaVersionReport {
            current_version,
            expected_version,
            is_compatible,
            requires_migration,
        })
    }

    /// Writes an updated user_version pragma after successful migration.
    pub fn set_user_version(conn: &Connection, new_version: u32) -> Result<(), rusqlite::Error> {
        conn.execute(&format!("PRAGMA user_version = {};", new_version), [])?;
        Ok(())
    }
}
