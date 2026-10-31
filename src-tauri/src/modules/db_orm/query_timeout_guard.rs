//! Query Execution Timeout Guard
//! Installs a progress handler on an SQLite connection to interrupt
//! long-running or runaway queries that exceed specified execution deadlines.

use rusqlite::Connection;
use std::time::{Duration, Instant};

pub struct QueryTimeoutGuard;

impl QueryTimeoutGuard {
    /// Installs a timeout guard on the connection for the duration of a query.
    /// Cancels query execution if it runs longer than timeout.
    pub fn install_timeout_guard(conn: &Connection, timeout: Duration) {
        let deadline = Instant::now() + timeout;

        // Callback every 1000 virtual machine opcodes
        conn.progress_handler(
            1000,
            Some(move || {
                // Return true to interrupt/abort query, false to continue
                Instant::now() >= deadline
            }),
        );
    }

    /// Clears any installed progress handler from the connection.
    pub fn clear_timeout_guard(conn: &Connection) {
        conn.progress_handler(0, None::<fn() -> bool>);
    }
}
