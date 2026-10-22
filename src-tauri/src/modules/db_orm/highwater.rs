//! Auto-Increment Highwater Mark Inspector.
//! Inspects `sqlite_sequence` to alert when table autoincrement IDs approach 2 billion limit.

use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};

/// Highwater threshold: 1,800,000,000 (roughly 85% of standard signed 32-bit integer limit).
pub const DEFAULT_HIGHWATER_THRESHOLD: i64 = 1_800_000_000;
pub const SIGNED_32BIT_MAX: i64 = 2_147_483_647;

/// Highwater status for an individual table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SequenceHighwater {
    pub table_name: String,
    pub current_seq: i64,
    pub percentage_of_limit: f64,
    pub is_warning: bool,
}

/// Comprehensive report of autoincrement sequences.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighwaterReport {
    pub sequences: Vec<SequenceHighwater>,
    pub has_warnings: bool,
    pub inspected_at_secs: u64,
}

/// Inspects `sqlite_sequence` to determine if any autoincrement sequence is near capacity.
pub fn inspect_autoincrement_highwater(
    conn: &Connection,
    warning_threshold: Option<i64>,
) -> Result<HighwaterReport> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let threshold = warning_threshold.unwrap_or(DEFAULT_HIGHWATER_THRESHOLD);

    // Check if sqlite_sequence table exists
    let seq_exists: bool = conn.query_row(
        "SELECT count(*) FROM sqlite_master WHERE type = 'table' AND name = 'sqlite_sequence'",
        [],
        |row| {
            let cnt: i64 = row.get(0)?;
            Ok(cnt > 0)
        },
    )?;

    if !seq_exists {
        return Ok(HighwaterReport {
            sequences: Vec::new(),
            has_warnings: false,
            inspected_at_secs: now,
        });
    }

    let mut stmt = conn.prepare("SELECT name, seq FROM sqlite_sequence ORDER BY seq DESC")?;
    let rows = stmt.query_map([], |row| {
        let name: String = row.get(0)?;
        let seq: i64 = row.get(1)?;
        Ok((name, seq))
    })?;

    let mut sequences = Vec::new();
    let mut has_warnings = false;

    for item in rows {
        let (table_name, current_seq) = item?;
        let pct = (current_seq as f64 / SIGNED_32BIT_MAX as f64) * 100.0;
        let is_warning = current_seq >= threshold;
        if is_warning {
            has_warnings = true;
        }

        sequences.push(SequenceHighwater {
            table_name,
            current_seq,
            percentage_of_limit: (pct * 100.0).round() / 100.0,
            is_warning,
        });
    }

    Ok(HighwaterReport {
        sequences,
        has_warnings,
        inspected_at_secs: now,
    })
}
