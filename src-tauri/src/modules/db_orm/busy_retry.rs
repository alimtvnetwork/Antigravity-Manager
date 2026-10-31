//! Database Busy Timeout Retry Ladder.
//! Wraps contentious SQLite operations with an exponential backoff retry ladder (10ms, 50ms, 200ms, 500ms).

use rusqlite::{Connection, Error, ErrorCode, Result};
use std::thread::sleep;
use std::time::Duration;

/// Backoff intervals in milliseconds for contention resolution.
pub const RETRY_BACKOFF_LADDER_MS: &[u64] = &[10, 50, 200, 500];

/// Checks if a rusqlite error represents transient contention (`SQLITE_BUSY` or `SQLITE_LOCKED`).
pub fn is_busy_error(err: &Error) -> bool {
    match err {
        Error::SqliteFailure(ffi_err, _) => {
            ffi_err.code == ErrorCode::DatabaseBusy || ffi_err.code == ErrorCode::DatabaseLocked
        }
        _ => false,
    }
}

/// Executes a closure with automatic backoff retry on `SQLITE_BUSY` or `SQLITE_LOCKED` contention.
pub fn with_busy_retry<T, F>(mut operation: F) -> Result<T>
where
    F: FnMut() -> Result<T>,
{
    let mut attempt = 0;
    loop {
        match operation() {
            Ok(val) => return Ok(val),
            Err(err) if is_busy_error(&err) && attempt < RETRY_BACKOFF_LADDER_MS.len() => {
                let delay = Duration::from_millis(RETRY_BACKOFF_LADDER_MS[attempt]);
                sleep(delay);
                attempt += 1;
            }
            Err(err) => return Err(err),
        }
    }
}

/// Sets an explicit busy timeout on the given connection.
pub fn set_busy_timeout(conn: &Connection, timeout: Duration) -> Result<()> {
    conn.busy_timeout(timeout)?;
    Ok(())
}
