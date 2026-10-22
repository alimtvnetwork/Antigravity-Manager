//! Split-DB Page Allocation Fragmentation Estimator.
//! Computes freelist-to-page-count ratio to determine when incremental vacuum is recommended.

use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};

/// High fragmentation threshold: 20% free pages.
pub const FRAGMENTATION_THRESHOLD_RATIO: f64 = 0.20;
pub const MIN_PAGES_FOR_VACUUM: i64 = 500;

/// Diagnostic report on database page allocation and freelist ratio.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentationReport {
    pub page_count: i64,
    pub freelist_count: i64,
    pub fragmentation_ratio: f64,
    pub is_vacuum_recommended: bool,
}

/// Estimates fragmentation by comparing freelist pages against total allocated pages.
pub fn estimate_db_fragmentation(conn: &Connection) -> Result<FragmentationReport> {
    let page_count: i64 = conn.query_row("PRAGMA page_count", [], |r| r.get(0))?;
    let freelist_count: i64 = conn.query_row("PRAGMA freelist_count", [], |r| r.get(0))?;

    let fragmentation_ratio = if page_count > 0 {
        freelist_count as f64 / page_count as f64
    } else {
        0.0
    };

    let is_vacuum_recommended =
        fragmentation_ratio >= FRAGMENTATION_THRESHOLD_RATIO && freelist_count >= MIN_PAGES_FOR_VACUUM;

    Ok(FragmentationReport {
        page_count,
        freelist_count,
        fragmentation_ratio: (fragmentation_ratio * 1000.0).round() / 1000.0,
        is_vacuum_recommended,
    })
}
