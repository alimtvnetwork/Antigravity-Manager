//! Connection Busy Handler Configurator.
//! Configures customized busy handler callbacks with progressive sleeping intervals for high-contention split database writes.

use rusqlite::{Connection, Result};
use std::sync::atomic::{AtomicI32, Ordering};
use std::thread::sleep;
use std::time::Duration;

/// Default retry count before returning SQLITE_BUSY to caller: 40 attempts.
pub const DEFAULT_MAX_BUSY_ATTEMPTS: i32 = 40;

/// Attempt limit read by the busy handler callback.
///
/// rusqlite's `busy_handler` accepts only a bare `fn(i32) -> bool` pointer,
/// so the configured limit cannot be captured by a closure; it is published
/// here before the handler is registered.
static MAX_BUSY_ATTEMPTS: AtomicI32 = AtomicI32::new(DEFAULT_MAX_BUSY_ATTEMPTS);

/// Progressive sleeping busy handler: retries until the configured attempt
/// limit is reached, then returns `false` so SQLite surfaces SQLITE_BUSY.
fn progressive_busy_handler(attempts: i32) -> bool {
    if attempts >= MAX_BUSY_ATTEMPTS.load(Ordering::Relaxed) {
        return false;
    }

    // Progressive sleep: 5ms, 10ms, 15ms... capped at 50ms per tick
    let delay_ms = ((attempts + 1) * 5).min(50) as u64;
    sleep(Duration::from_millis(delay_ms));
    true
}

/// Attaches a progressive sleeping busy handler to the connection.
pub fn configure_progressive_busy_handler(
    conn: &Connection,
    max_attempts: Option<i32>,
) -> Result<()> {
    let limit = max_attempts.unwrap_or(DEFAULT_MAX_BUSY_ATTEMPTS);
    MAX_BUSY_ATTEMPTS.store(limit, Ordering::Relaxed);

    conn.busy_handler(Some(progressive_busy_handler))?;

    Ok(())
}
