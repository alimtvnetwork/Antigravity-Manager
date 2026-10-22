// Database Page Freelist Reclaim Analyzer (Imp-528)
// Inspects PRAGMA freelist_count and evaluates whether incremental_vacuum
// is needed to return deleted page capacity to the OS filesystem.

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

pub const DEFAULT_VACUUM_FREELIST_THRESHOLD: i64 = 100;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreelistReport {
    pub freelist_pages: i64,
    pub page_size_bytes: i64,
    pub reclaimable_bytes: i64,
    pub is_vacuum_recommended: bool,
}

pub struct PageFreelistAnalyzer;

impl PageFreelistAnalyzer {
    /// Inspects the number of free pages currently in the SQLite database freelist.
    pub fn inspect_freelist(conn: &Connection, threshold: Option<i64>) -> Result<FreelistReport, rusqlite::Error> {
        let mut free_stmt = conn.prepare("PRAGMA freelist_count;")?;
        let freelist_pages: i64 = free_stmt.query_row([], |row| row.get(0))?;

        let mut size_stmt = conn.prepare("PRAGMA page_size;")?;
        let page_size_bytes: i64 = size_stmt.query_row([], |row| row.get(0)).unwrap_or(4096);

        let limit = threshold.unwrap_or(DEFAULT_VACUUM_FREELIST_THRESHOLD);
        let is_vacuum_recommended = freelist_pages >= limit;
        let reclaimable_bytes = freelist_pages * page_size_bytes;

        Ok(FreelistReport {
            freelist_pages,
            page_size_bytes,
            reclaimable_bytes,
            is_vacuum_recommended,
        })
    }

    /// Executes incremental vacuum to reclaim up to max_pages.
    pub fn execute_incremental_vacuum(conn: &Connection, max_pages: i64) -> Result<(), rusqlite::Error> {
        conn.execute(&format!("PRAGMA incremental_vacuum({});", max_pages), [])?;
        Ok(())
    }
}
