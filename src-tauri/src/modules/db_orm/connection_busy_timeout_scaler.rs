// Dynamic Busy Timeout Scaler (Imp-524)
// Adjusts SQLite PRAGMA busy_timeout dynamically based on connection pool load
// and concurrent query contention to prevent premature SQLITE_BUSY aborts.

use rusqlite::Connection;
use tracing::debug;

pub const MIN_BUSY_TIMEOUT_MS: u32 = 3000;
pub const BASE_BUSY_TIMEOUT_MS: u32 = 5000;
pub const MAX_BUSY_TIMEOUT_MS: u32 = 15000;

pub struct ConnectionBusyTimeoutScaler;

impl ConnectionBusyTimeoutScaler {
    /// Computes recommended busy timeout milliseconds given active concurrent connections.
    pub fn compute_timeout_ms(active_concurrency: usize) -> u32 {
        if active_concurrency <= 1 {
            BASE_BUSY_TIMEOUT_MS
        } else {
            let scaled = BASE_BUSY_TIMEOUT_MS + (active_concurrency as u32 * 1000);
            scaled.clamp(MIN_BUSY_TIMEOUT_MS, MAX_BUSY_TIMEOUT_MS)
        }
    }

    /// Applies the computed busy timeout to an open SQLite connection.
    pub fn apply_scaled_timeout(
        conn: &Connection,
        active_concurrency: usize,
    ) -> Result<u32, rusqlite::Error> {
        let timeout_ms = Self::compute_timeout_ms(active_concurrency);
        conn.execute(&format!("PRAGMA busy_timeout = {};", timeout_ms), [])?;
        debug!(
            active_concurrency,
            timeout_ms, "Applied scaled busy_timeout pragma"
        );
        Ok(timeout_ms)
    }
}
