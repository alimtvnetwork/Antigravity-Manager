//! Upstream Dynamic DNS Resolver & Cache
//! Mitigates transient NXDOMAIN and DNS query stalls by caching resolved IP addresses
//! for upstream AI API endpoints with configurable TTL.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
struct CachedDnsEntry {
    addrs: Vec<SocketAddr>,
    expires_at: Instant,
}

#[derive(Debug, Clone)]
pub struct DynamicDnsResolver {
    cache: Arc<RwLock<HashMap<String, CachedDnsEntry>>>,
    default_ttl: Duration,
}

impl Default for DynamicDnsResolver {
    fn default() -> Self {
        Self::new(Duration::from_secs(300))
    }
}

impl DynamicDnsResolver {
    pub fn new(default_ttl: Duration) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            default_ttl,
        }
    }

    /// Resolves host and port, returning cached results if fresh, or performing DNS resolution.
    pub async fn resolve(&self, host: &str, port: u16) -> Result<Vec<SocketAddr>, std::io::Error> {
        let key = format!("{}:{}", host.to_ascii_lowercase(), port);

        // 1. Check read cache
        {
            let guard = self.cache.read().await;
            if let Some(entry) = guard.get(&key) {
                if Instant::now() < entry.expires_at && !entry.addrs.is_empty() {
                    return Ok(entry.addrs.clone());
                }
            }
        }

        // 2. Perform DNS lookup
        let resolved: Vec<SocketAddr> = match tokio::net::lookup_host(&key).await {
            Ok(iter) => iter.collect(),
            Err(e) => {
                // If resolution fails but stale cache exists, fallback to stale cache
                let guard = self.cache.read().await;
                if let Some(entry) = guard.get(&key) {
                    if !entry.addrs.is_empty() {
                        return Ok(entry.addrs.clone());
                    }
                }
                return Err(e);
            }
        };

        if !resolved.is_empty() {
            let mut guard = self.cache.write().await;
            guard.insert(
                key,
                CachedDnsEntry {
                    addrs: resolved.clone(),
                    expires_at: Instant::now() + self.default_ttl,
                },
            );
        }

        Ok(resolved)
    }

    /// Clears expired entries from the cache.
    pub async fn sweep_expired(&self) {
        let mut guard = self.cache.write().await;
        let now = Instant::now();
        guard.retain(|_, entry| entry.expires_at > now);
    }
}
