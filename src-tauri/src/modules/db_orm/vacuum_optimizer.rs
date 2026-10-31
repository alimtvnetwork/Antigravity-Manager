//! Freelist Threshold Vacuum Optimizer
//! Inspects SQLite freelist page counts and selectively triggers
//! incremental vacuum only when reclaimable storage exceeds defined thresholds.

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VacuumMetrics {
    pub page_count: i64,
    pub freelist_count: i64,
    pub freelist_ratio: f64,
    pub vacuum_executed: bool,
    pub pages_requested: i64,
}

pub struct VacuumOptimizer;

impl VacuumOptimizer {
    /// Inspects freelist metrics and conditionally executes incremental vacuum.
    pub fn optimize_if_needed(
        conn: &Connection,
        min_freelist_pages: i64,
        min_freelist_ratio: f64,
        max_vacuum_pages: i64,
    ) -> Result<VacuumMetrics, rusqlite::Error> {
        let page_count: i64 = conn.query_row("PRAGMA page_count;", [], |r| r.get(0))?;
        let freelist_count: i64 = conn.query_row("PRAGMA freelist_count;", [], |r| r.get(0))?;

        let freelist_ratio = if page_count > 0 {
            freelist_count as f64 / page_count as f64
        } else {
            0.0
        };

        let should_vacuum =
            freelist_count >= min_freelist_pages || freelist_ratio >= min_freelist_ratio;
        let mut vacuum_executed = false;
        let mut pages_requested = 0;

        if should_vacuum && freelist_count > 0 {
            pages_requested = freelist_count.min(max_vacuum_pages);
            conn.execute_batch(&format!("PRAGMA incremental_vacuum({});", pages_requested))?;
            vacuum_executed = true;
        }

        Ok(VacuumMetrics {
            page_count,
            freelist_count,
            freelist_ratio,
            vacuum_executed,
            pages_requested,
        })
    }
}
