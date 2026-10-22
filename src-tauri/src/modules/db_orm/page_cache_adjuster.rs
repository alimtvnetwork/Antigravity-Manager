//! SQLite page cache sizing adjuster.
//! Dynamically sets PRAGMA cache_size to optimize in-memory caching performance
//! without exhausting available RAM in multi-instance desktop environments.

use crate::error::AppError;
use rusqlite::Connection;

pub struct PageCacheAdjuster;

impl PageCacheAdjuster {
    /// Configures SQLite page cache size in KiB using negative pragma syntax (e.g. -4000 = 4MB).
    pub fn set_cache_size_kib(conn: &Connection, kib: i32) -> Result<(), AppError> {
        let safe_kib = kib.clamp(1024, 65536); // Between 1MB and 64MB
        let pragma_val = -safe_kib;

        conn.execute_batch(&format!("PRAGMA cache_size = {};", pragma_val))
            .map_err(|e| AppError::Database(format!("Failed to set PRAGMA cache_size: {}", e)))
    }

    /// Reads the current cache_size setting.
    pub fn get_current_cache_size(conn: &Connection) -> Result<i64, AppError> {
        let mut stmt = conn
            .prepare("PRAGMA cache_size;")
            .map_err(|e| AppError::Database(format!("Failed to query cache_size: {}", e)))?;

        let mut rows = stmt
            .query([])
            .map_err(|e| AppError::Database(format!("Failed to execute cache_size query: {}", e)))?;

        if let Some(row) = rows
            .next()
            .map_err(|e| AppError::Database(format!("Failed to read cache_size row: {}", e)))?
        {
            let val: i64 = row.get(0).unwrap_or(0);
            Ok(val)
        } else {
            Ok(0)
        }
    }
}
