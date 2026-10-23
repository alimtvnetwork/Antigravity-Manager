//! Upstream connection idle reaper.
//! Tracks and prunes stale pooled HTTP/2 and TCP connections that have
//! exceeded their idle keepalive threshold to prevent socket leaks and FD exhaustion.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Configuration for upstream connection reaping.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdleReaperConfig {
    pub max_idle_duration: Duration,
    pub max_connection_lifetime: Duration,
}

impl Default for IdleReaperConfig {
    fn default() -> Self {
        Self {
            max_idle_duration: Duration::from_secs(60),
            max_connection_lifetime: Duration::from_secs(300),
        }
    }
}

#[derive(Debug, Clone)]
struct TrackedConnection {
    endpoint_id: String,
    created_at: Instant,
    last_active_at: Instant,
}

pub struct IdleConnectionReaper {
    config: IdleReaperConfig,
    connections: Mutex<HashMap<String, TrackedConnection>>,
}

impl IdleConnectionReaper {
    pub fn new(config: Option<IdleReaperConfig>) -> Self {
        Self {
            config: config.unwrap_or_default(),
            connections: Mutex::new(HashMap::new()),
        }
    }

    /// Registers or updates activity on a pooled connection.
    pub fn touch_connection(
        &self,
        connection_id: impl Into<String>,
        endpoint_id: impl Into<String>,
    ) {
        let mut guard = match self.connections.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };

        let conn_id = connection_id.into();
        let now = Instant::now();

        if let Some(conn) = guard.get_mut(&conn_id) {
            conn.last_active_at = now;
        } else {
            guard.insert(
                conn_id,
                TrackedConnection {
                    endpoint_id: endpoint_id.into(),
                    created_at: now,
                    last_active_at: now,
                },
            );
        }
    }

    /// Scans for connections that have exceeded the idle or max lifetime limits.
    /// Returns the list of expired connection IDs that should be closed.
    pub fn reap_stale(&self) -> Vec<String> {
        let mut guard = match self.connections.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };

        let now = Instant::now();
        let mut stale_ids = Vec::new();

        guard.retain(|id, conn| {
            let is_idle_expired =
                now.duration_since(conn.last_active_at) > self.config.max_idle_duration;
            let is_lifetime_expired =
                now.duration_since(conn.created_at) > self.config.max_connection_lifetime;

            if is_idle_expired || is_lifetime_expired {
                stale_ids.push(id.clone());
                false // Evict from map
            } else {
                true // Retain
            }
        });

        stale_ids
    }

    /// Explicitly removes a connection when gracefully closed.
    pub fn remove_connection(&self, connection_id: &str) {
        let mut guard = match self.connections.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        guard.remove(connection_id);
    }
}

impl Default for IdleConnectionReaper {
    fn default() -> Self {
        Self::new(None)
    }
}
