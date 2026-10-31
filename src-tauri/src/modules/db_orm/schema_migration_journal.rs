// Schema Migration Append-Only Journal (Imp-525)
// Records immutable audit entries for schema migrations, tracking execution
// duration, version sequence, applied timestamp, and verification checksums.

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationJournalEntry {
    pub version_number: u32,
    pub description: String,
    pub checksum: String,
    pub applied_at_epoch_secs: u64,
    pub execution_duration_ms: u64,
}

pub struct SchemaMigrationJournal;

impl SchemaMigrationJournal {
    /// Ensures the SchemaMigrationJournal table exists.
    pub fn ensure_journal_table(conn: &Connection) -> Result<(), rusqlite::Error> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS SchemaMigrationJournal (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                version_number INTEGER NOT NULL,
                description TEXT NOT NULL,
                checksum TEXT NOT NULL,
                applied_at INTEGER NOT NULL,
                duration_ms INTEGER NOT NULL
            );",
            [],
        )?;
        Ok(())
    }

    /// Appends a new migration record to the journal.
    pub fn record_migration(
        conn: &Connection,
        entry: &MigrationJournalEntry,
    ) -> Result<(), rusqlite::Error> {
        Self::ensure_journal_table(conn)?;
        conn.execute(
            "INSERT INTO SchemaMigrationJournal (version_number, description, checksum, applied_at, duration_ms)
             VALUES (?1, ?2, ?3, ?4, ?5);",
            (
                entry.version_number,
                &entry.description,
                &entry.checksum,
                entry.applied_at_epoch_secs as i64,
                entry.execution_duration_ms as i64,
            ),
        )?;
        Ok(())
    }

    /// Retrieves the highest recorded migration version number.
    pub fn get_latest_version(conn: &Connection) -> Result<u32, rusqlite::Error> {
        Self::ensure_journal_table(conn)?;
        let mut stmt =
            conn.prepare("SELECT COALESCE(MAX(version_number), 0) FROM SchemaMigrationJournal;")?;
        let version: u32 = stmt.query_row([], |row| row.get(0))?;
        Ok(version)
    }
}
