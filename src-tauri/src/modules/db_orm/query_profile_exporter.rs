//! Query Execution Profile Exporter.
//! Records and formats database query execution telemetry, capturing queries exceeding
//! the slow query threshold (>100ms) for forensic diagnosis.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Threshold for categorizing queries as slow.
pub const SLOW_QUERY_THRESHOLD: Duration = Duration::from_millis(100);

/// Detailed execution profile of an individual database query.
#[derive(Debug, Clone)]
pub struct QueryProfile {
    pub db_name: String,
    pub sql_fragment: String,
    pub execution_duration: Duration,
    pub rows_affected: usize,
    pub recorded_at: Instant,
}

/// In-memory ring buffer of slow query profiles.
#[derive(Debug)]
pub struct QueryProfileExporter {
    profiles: VecDeque<QueryProfile>,
    capacity: usize,
}

impl Default for QueryProfileExporter {
    fn default() -> Self {
        Self {
            profiles: VecDeque::with_capacity(100),
            capacity: 100,
        }
    }
}

impl QueryProfileExporter {
    /// Creates a new exporter with custom capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            profiles: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    /// Records query execution, storing only queries that exceed the slow threshold.
    pub fn record_query(
        &mut self,
        db_name: &str,
        sql: &str,
        duration: Duration,
        rows_affected: usize,
    ) {
        if duration >= SLOW_QUERY_THRESHOLD {
            if self.profiles.len() >= self.capacity {
                self.profiles.pop_front();
            }
            self.profiles.push_back(QueryProfile {
                db_name: db_name.to_string(),
                sql_fragment: sql.chars().take(200).collect(),
                execution_duration: duration,
                rows_affected,
                recorded_at: Instant::now(),
            });
        }
    }

    /// Returns all captured slow queries in reverse chronological order.
    pub fn slow_queries(&self) -> Vec<QueryProfile> {
        self.profiles.iter().rev().cloned().collect()
    }

    /// Total slow queries currently captured.
    pub fn count(&self) -> usize {
        self.profiles.len()
    }
}
