//! Split-DB Schema Version Drift Detector.
//! Detects version mismatch between compiled `TARGET_SCHEMA_VERSION` and `SchemaVersion` records in SQLite files.

use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};

/// Target compile-time schema version across Split-DBs.
pub const DEFAULT_TARGET_SCHEMA_VERSION: i32 = 1;

/// Diagnostic report on schema version alignment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionDriftReport {
    pub is_current: bool,
    pub target_version: i32,
    pub current_version: Option<i32>,
    pub description: Option<String>,
    pub requires_migration: bool,
}

/// Inspects the `SchemaVersion` table to check if local database version matches expected target.
pub fn check_schema_version_drift(
    conn: &Connection,
    target_version: Option<i32>,
) -> Result<VersionDriftReport> {
    let target = target_version.unwrap_or(DEFAULT_TARGET_SCHEMA_VERSION);

    // Check if SchemaVersion table exists
    let table_exists: bool = conn.query_row(
        "SELECT count(*) FROM sqlite_master WHERE type = 'table' AND name = 'SchemaVersion'",
        [],
        |r| {
            let cnt: i64 = r.get(0)?;
            Ok(cnt > 0)
        },
    )?;

    if !table_exists {
        return Ok(VersionDriftReport {
            is_current: false,
            target_version: target,
            current_version: None,
            description: None,
            requires_migration: true,
        });
    }

    let version_row: Option<(i32, Option<String>)> = conn
        .query_row(
            "SELECT VersionNumber, Description FROM SchemaVersion ORDER BY VersionNumber DESC LIMIT 1",
            [],
            |r| {
                let ver: i32 = r.get(0)?;
                let desc: Option<String> = r.get(1)?;
                Ok((ver, desc))
            },
        )
        .ok();

    let (current_version, description) = match version_row {
        Some((v, d)) => (Some(v), d),
        None => (None, None),
    };

    let is_current = current_version == Some(target);
    let requires_migration = match current_version {
        Some(v) => v < target,
        None => true,
    };

    Ok(VersionDriftReport {
        is_current,
        target_version: target,
        current_version,
        description,
        requires_migration,
    })
}
