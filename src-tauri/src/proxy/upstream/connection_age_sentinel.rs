//! Upstream Connection Maximum Age Sentinel
//!
//! Tracks TCP/TLS connection creation timestamps and enforces a maximum connection
//! lifetime (default 300 seconds) to avoid silent NAT mapping expirations,
//! stale DNS resolution, and lingering half-open TCP connections with upstream gateways.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tracing::{debug, info};

/// Default maximum allowed age for an upstream connection before retirement (300 seconds).
pub const DEFAULT_MAX_CONNECTION_AGE: Duration = Duration::from_secs(300);

/// Metadata for a tracked connection.
#[derive(Debug, Clone)]
pub struct ConnectionAgeEntry {
    pub connection_id: String,
    pub endpoint: String,
    pub created_at: Instant,
    pub last_used_at: Instant,
    pub request_count: u64,
}

/// Sentinel managing connection ages and triggering graceful retirements.
pub struct ConnectionAgeSentinel {
    connections: Mutex<HashMap<String, ConnectionAgeEntry>>,
    max_age: Duration,
}

impl Default for ConnectionAgeSentinel {
    fn default() -> Self {
        Self::new(DEFAULT_MAX_CONNECTION_AGE)
    }
}

impl ConnectionAgeSentinel {
    pub fn new(max_age: Duration) -> Self {
        Self {
            connections: Mutex::new(HashMap::new()),
            max_age,
        }
    }

    /// Registers a newly established connection.
    pub fn register_connection(&self, connection_id: String, endpoint: String) {
        let now = Instant::now();
        let entry = ConnectionAgeEntry {
            connection_id: connection_id.clone(),
            endpoint,
            created_at: now,
            last_used_at: now,
            request_count: 0,
        };

        let mut map = match self.connections.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        map.insert(connection_id, entry);
    }

    /// Records request activity on an existing connection and checks if it should be retired.
    pub fn touch_connection(&self, connection_id: &str) -> bool {
        let mut map = match self.connections.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };

        if let Some(entry) = map.get_mut(connection_id) {
            entry.last_used_at = Instant::now();
            entry.request_count += 1;

            let age = entry.created_at.elapsed();
            if age > self.max_age {
                info!(
                    conn_id = connection_id,
                    age_secs = age.as_secs(),
                    requests = entry.request_count,
                    "Upstream connection exceeded maximum age, marking for retirement"
                );
                return true; // Should retire
            }
        }

        false
    }

    /// Removes a connection entry when explicitly closed or dropped.
    pub fn remove_connection(&self, connection_id: &str) {
        let mut map = match self.connections.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        map.remove(connection_id);
    }

    /// Sweeps all connections that have exceeded the maximum lifetime.
    pub fn sweep_stale_connections(&self) -> Vec<String> {
        let mut map = match self.connections.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };

        let max_age = self.max_age;
        let mut to_retire = Vec::new();

        map.retain(|id, entry| {
            if entry.created_at.elapsed() > max_age {
                to_retire.push(id.clone());
                false
            } else {
                true
            }
        });

        if !to_retire.is_empty() {
            debug!(
                count = to_retire.len(),
                "Swept expired upstream connections"
            );
        }

        to_retire
    }
}
