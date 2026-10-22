//! Connection Busy Handler Configurator.
//! Configures customized busy handler callbacks with progressive sleeping intervals for high-contention split database writes.

use rusqlite::{Connection, Result};
use std::thread::sleep;
use std::time::Duration;

/// Default retry count before returning SQLITE_BUSY to caller: 40 attempts.
pub const DEFAULT_MAX_BUSY_ATTEMPTS: i32 = 40;

/// Attaches a progressive sleeping busy handler to the connection.
pub fn configure_progressive_busy_handler(
    conn: &Connection,
    max_attempts: Option<i32>,
) -> Result<()> {
    let limit = max_attempts.unwrap_or(DEFAULT_MAX_BUSY_ATTEMPTS);

    conn.busy_handler(Some(move |attempts| {
        if attempts >= limit {
            return false;
        }

        // Progressive sleep: 5ms, 10ms, 15ms... capped at 50ms per tick
        let delay_ms = ((attempts + 1) * 5).min(50) as u64;
        sleep(Duration::from_millis(delay_ms));
        true
    }))?;

    Ok(())
}
