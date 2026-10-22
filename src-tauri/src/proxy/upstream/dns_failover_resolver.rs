// In-Memory DNS Failover Resolver (Imp-516)
// Maintains a fast in-memory lookup cache for upstream hostnames, resolving
// IP address candidates and performing dual-stack IPv4/IPv6 fallback.

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

pub const DEFAULT_DNS_CACHE_TTL_SECS: u64 = 300; // 5 minutes

#[derive(Debug, Clone)]
struct CachedDnsRecord {
    addrs: Vec<IpAddr>,
    resolved_at: Instant,
    ttl: Duration,
}

#[derive(Debug, Clone)]
pub struct DnsFailoverResolver {
    cache: Arc<RwLock<HashMap<String, CachedDnsRecord>>>,
    default_ttl: Duration,
}

impl Default for DnsFailoverResolver {
    fn default() -> Self {
        Self::new(Duration::from_secs(DEFAULT_DNS_CACHE_TTL_SECS))
    }
}

impl DnsFailoverResolver {
    pub fn new(default_ttl: Duration) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            default_ttl,
        }
    }

    /// Stores resolved addresses for a host in the cache.
    pub fn cache_host(&self, host: &str, addrs: Vec<IpAddr>) {
        if let Ok(mut lock) = self.cache.write() {
            lock.insert(
                host.to_string(),
                CachedDnsRecord {
                    addrs,
                    resolved_at: Instant::now(),
                    ttl: self.default_ttl,
                },
            );
        }
    }

    /// Returns cached addresses for the host if not expired.
    pub fn resolve_cached(&self, host: &str) -> Option<Vec<IpAddr>> {
        let lock = self.cache.read().ok()?;
        let record = lock.get(host)?;

        if record.resolved_at.elapsed() < record.ttl {
            Some(record.addrs.clone())
        } else {
            None
        }
    }

    /// Clears expired DNS entries from the cache.
    pub fn purge_expired(&self) {
        if let Ok(mut lock) = self.cache.write() {
            lock.retain(|_, record| record.resolved_at.elapsed() < record.ttl);
        }
    }
}
