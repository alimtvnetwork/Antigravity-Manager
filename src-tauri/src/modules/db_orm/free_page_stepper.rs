//! Split database free page stepper.
//! Inspects freelist pages and executes bounded incremental vacuum steps
//! to reclaim physical storage without incurring large write stalls.

use crate::error::AppError;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteppedVacuumResult {
    pub initial_freelist_count: i64,
    pub pages_requested: i64,
    pub final_freelist_count: i64,
    pub pages_reclaimed: i64,
}

pub struct FreePageStepper;

impl FreePageStepper {
    /// Steps incremental vacuum by up to `max_step_pages` if freelist exceeds threshold.
    pub fn step_vacuum(
        conn: &Connection,
        threshold_pages: i64,
        max_step_pages: i64,
    ) -> Result<SteppedVacuumResult, AppError> {
        let initial_free = Self::get_freelist_count(conn)?;

        if initial_free < threshold_pages {
            return Ok(SteppedVacuumResult {
                initial_freelist_count: initial_free,
                pages_requested: 0,
                final_freelist_count: initial_free,
                pages_reclaimed: 0,
            });
        }

        let step = max_step_pages.min(initial_free).max(1);
        conn.execute_batch(&format!("PRAGMA incremental_vacuum({});", step))
            .map_err(|e| AppError::Database(format!("Incremental vacuum execution failed: {}", e)))?;

        let final_free = Self::get_freelist_count(conn)?;
        let reclaimed = (initial_free - final_free).max(0);

        Ok(SteppedVacuumResult {
            initial_freelist_count: initial_free,
            pages_requested: step,
            final_freelist_count: final_free,
            pages_reclaimed: reclaimed,
        })
    }

    fn get_freelist_count(conn: &Connection) -> Result<i64, AppError> {
        let mut stmt = conn
            .prepare("PRAGMA freelist_count;")
            .map_err(|e| AppError::Database(format!("Prepare freelist_count failed: {}", e)))?;

        let mut rows = stmt
            .query([])
            .map_err(|e| AppError::Database(format!("Query freelist_count failed: {}", e)))?;

        if let Some(row) = rows
            .next()
            .map_err(|e| AppError::Database(format!("Read freelist_count row failed: {}", e)))?
        {
            Ok(row.get(0).unwrap_or(0))
        } else {
            Ok(0)
        }
    }
}
