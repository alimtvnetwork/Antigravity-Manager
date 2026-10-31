//! SQLite lock escalation guard.
//! Prevents SQLITE_BUSY deadlocks during shared-to-exclusive lock upgrades
//! in WAL mode through bounded jittered retries and busy timeout backoff.

use crate::error::AppError;
use rusqlite::Connection;
use std::time::Duration;

/// Configuration for lock escalation retry policy.
#[derive(Debug, Clone)]
pub struct LockEscalationConfig {
    pub max_retries: u32,
    pub initial_backoff_ms: u64,
    pub max_backoff_ms: u64,
}

impl Default for LockEscalationConfig {
    fn default() -> Self {
        Self {
            max_retries: 5,
            initial_backoff_ms: 20,
            max_backoff_ms: 300,
        }
    }
}

pub struct LockEscalationGuard;

impl LockEscalationGuard {
    /// Executes a transaction block with exponential backoff if a busy/locked error occurs.
    pub fn execute_with_guard<F, T>(
        conn: &Connection,
        config: Option<&LockEscalationConfig>,
        operation: F,
    ) -> Result<T, AppError>
    where
        F: Fn(&Connection) -> Result<T, rusqlite::Error>,
    {
        let default_cfg = LockEscalationConfig::default();
        let cfg = config.unwrap_or(&default_cfg);

        let mut backoff = cfg.initial_backoff_ms;
        let mut last_err = None;

        for attempt in 0..=cfg.max_retries {
            match operation(conn) {
                Ok(result) => return Ok(result),
                Err(err) => {
                    let is_busy = matches!(
                        err,
                        rusqlite::Error::SqliteFailure(ref f, _)
                            if f.extended_code == 5 || f.extended_code == 517 || f.extended_code == 261
                    );

                    last_err = Some(err);

                    if !is_busy || attempt == cfg.max_retries {
                        break;
                    }

                    // Apply bounded backoff sleep
                    std::thread::sleep(Duration::from_millis(backoff));
                    backoff = (backoff * 2).min(cfg.max_backoff_ms);
                }
            }
        }

        match last_err {
            Some(err) => Err(AppError::Database(err)),
            // The retry loop always runs at least once, so reaching this
            // return means an error was recorded; None is defensive only.
            None => Err(AppError::Config(format!(
                "SQLite lock escalation failed after {} retries",
                cfg.max_retries
            ))),
        }
    }
}
