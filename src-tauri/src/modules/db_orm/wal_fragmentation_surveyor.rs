//! Split-SQLite WAL Page Fragmentation Surveyor (Imp-544)
//! Inspects write-ahead log file dimensions, uncheckpointed frame count,
//! and page fragmentation ratios to determine whether passive or truncate checkpointing is advised.

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

pub const DEFAULT_MAX_WAL_PAGES_BEFORE_TRUNCATE: i64 = 1000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalFragmentationReport {
    pub wal_size_bytes: u64,
    pub page_size_bytes: i64,
    pub estimated_wal_pages: i64,
    pub is_truncate_recommended: bool,
}

pub struct WalFragmentationSurveyor;

impl WalFragmentationSurveyor {
    /// Inspects the open database connection and file metadata to survey WAL fragmentation.
    pub fn survey_wal(
        conn: &Connection,
        wal_size_bytes: u64,
        threshold_pages: Option<i64>,
    ) -> Result<WalFragmentationReport, rusqlite::Error> {
        let mut size_stmt = conn.prepare("PRAGMA page_size;")?;
        let page_size_bytes: i64 = size_stmt.query_row([], |row| row.get(0)).unwrap_or(4096);

        // In WAL mode, each frame is page_size + 24 bytes header (approx page_size)
        let estimated_wal_pages = if page_size_bytes > 0 {
            (wal_size_bytes as i64) / page_size_bytes
        } else {
            0
        };

        let threshold = threshold_pages.unwrap_or(DEFAULT_MAX_WAL_PAGES_BEFORE_TRUNCATE);
        let is_truncate_recommended = estimated_wal_pages >= threshold;

        Ok(WalFragmentationReport {
            wal_size_bytes,
            page_size_bytes,
            estimated_wal_pages,
            is_truncate_recommended,
        })
    }
}
