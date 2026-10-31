//! Split Database Migration Version Stamp.
//! Manages read, write, and validation of `PRAGMA user_version;` across split SQLite databases
//! to ensure forward compatibility and prevent inadvertent schema downgrade drift.

use rusqlite::Connection;

/// Target schema versions across the split SQLite databases.
pub const REPO_PROMPTS_TARGET_SCHEMA_VERSION: u32 = 4;
pub const TASK_HISTORY_TARGET_SCHEMA_VERSION: u32 = 3;
pub const ERRORS_DB_TARGET_SCHEMA_VERSION: u32 = 2;
pub const PROMPT_BACKUPS_TARGET_SCHEMA_VERSION: u32 = 2;

/// Outcome of inspecting schema versions during startup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VersionStampStatus {
    MatchesTarget { version: u32 },
    UpgradeNeeded { current: u32, target: u32 },
    UnsupportedDowngrade { current: u32, target: u32 },
}

/// Reads the current schema version from `PRAGMA user_version;`.
pub fn get_user_version(conn: &Connection) -> Result<u32, rusqlite::Error> {
    conn.query_row("PRAGMA user_version;", [], |row| row.get(0))
}

/// Sets a new schema version stamp in `PRAGMA user_version;`.
pub fn set_user_version(conn: &Connection, new_version: u32) -> Result<(), rusqlite::Error> {
    conn.execute(&format!("PRAGMA user_version = {};", new_version), [])?;
    Ok(())
}

/// Validates current version against the expected target version.
pub fn validate_schema_version(
    conn: &Connection,
    target_version: u32,
) -> Result<VersionStampStatus, rusqlite::Error> {
    let current = get_user_version(conn)?;
    if current == target_version {
        Ok(VersionStampStatus::MatchesTarget { version: current })
    } else if current < target_version {
        Ok(VersionStampStatus::UpgradeNeeded {
            current,
            target: target_version,
        })
    } else {
        Ok(VersionStampStatus::UnsupportedDowngrade {
            current,
            target: target_version,
        })
    }
}
