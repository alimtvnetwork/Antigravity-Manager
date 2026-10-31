//! Incremental Auto-Vacuum Scheduler.
//! Schedules and executes non-blocking `PRAGMA incremental_vacuum(N);` calls
//! during application idle windows, reclaiming freed database pages incrementally.

use rusqlite::Connection;
use std::time::{Duration, Instant};

/// Configuration for the incremental vacuum scheduler.
#[derive(Debug, Clone, Copy)]
pub struct VacuumSchedulerConfig {
    /// Minimum interval between incremental vacuum runs.
    pub min_interval: Duration,
    /// Number of freelist pages to reclaim per incremental vacuum step.
    pub pages_per_step: u32,
    /// Minimum number of freelist pages before triggering a step.
    pub freelist_threshold: u32,
}

impl Default for VacuumSchedulerConfig {
    fn default() -> Self {
        Self {
            min_interval: Duration::from_secs(300), // 5 minutes
            pages_per_step: 100,
            freelist_threshold: 50,
        }
    }
}

/// State tracker for database auto-vacuum operations.
#[derive(Debug)]
pub struct VacuumScheduler {
    config: VacuumSchedulerConfig,
    last_vacuum_at: Option<Instant>,
    total_vacuums_performed: u64,
}

impl VacuumScheduler {
    /// Creates a scheduler with given or default configuration.
    pub fn new(config: Option<VacuumSchedulerConfig>) -> Self {
        Self {
            config: config.unwrap_or_default(),
            last_vacuum_at: None,
            total_vacuums_performed: 0,
        }
    }

    /// Queries the current freelist count (`PRAGMA freelist_count;`).
    pub fn get_freelist_count(conn: &Connection) -> Result<u32, rusqlite::Error> {
        conn.query_row("PRAGMA freelist_count;", [], |row| row.get(0))
    }

    /// Checks if conditions are met to run an incremental vacuum step.
    pub fn should_vacuum(&self, freelist_count: u32) -> bool {
        if freelist_count < self.config.freelist_threshold {
            return false;
        }

        if let Some(last) = self.last_vacuum_at {
            if last.elapsed() < self.config.min_interval {
                return false;
            }
        }

        true
    }

    /// Executes an incremental vacuum step if needed, returning pages reclaimed.
    pub fn maybe_vacuum(&mut self, conn: &Connection) -> Result<Option<u32>, rusqlite::Error> {
        let freelist = Self::get_freelist_count(conn)?;
        if !self.should_vacuum(freelist) {
            return Ok(None);
        }

        let pages_to_vacuum = self.config.pages_per_step.min(freelist);
        conn.execute(
            &format!("PRAGMA incremental_vacuum({});", pages_to_vacuum),
            [],
        )?;

        self.last_vacuum_at = Some(Instant::now());
        self.total_vacuums_performed += 1;

        Ok(Some(pages_to_vacuum))
    }

    /// Total vacuum steps executed since startup.
    pub fn total_vacuums(&self) -> u64 {
        self.total_vacuums_performed
    }
}
