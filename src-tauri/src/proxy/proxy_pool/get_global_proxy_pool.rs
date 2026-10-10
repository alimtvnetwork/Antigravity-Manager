use super::*;

/// Global proxy pool manager singleton
pub static GLOBAL_PROXY_POOL: OnceLock<Arc<ProxyPoolManager>> = OnceLock::new();

/// Get global proxy pool manager
pub fn get_global_proxy_pool() -> Option<Arc<ProxyPoolManager>> {
    GLOBAL_PROXY_POOL.get().cloned()
}

/// Initialize global proxy pool manager
pub fn init_global_proxy_pool(config: Arc<RwLock<ProxyPoolConfig>>) -> Arc<ProxyPoolManager> {
    let manager = Arc::new(ProxyPoolManager::new(config));
    // Justification: OnceLock::set fails only if already initialized; double-init is a benign no-op by design.
    let _ = GLOBAL_PROXY_POOL.set(manager.clone());
    manager
}

/// Proxy configuration (used to construct reqwest/rquest Client)
/// Note: Renamed to PoolProxyConfig to avoid conflict with config::ProxyConfig
#[derive(Debug, Clone)]
pub struct PoolProxyConfig {
    pub proxy: rquest::Proxy,
    pub entry_id: String,
}

/// Proxy pool manager
pub struct ProxyPoolManager {
    pub(crate) config: Arc<RwLock<ProxyPoolConfig>>,

    /// Proxy usage count (proxy_id -> count)
    pub(crate) usage_counter: Arc<DashMap<String, usize>>,

    /// Account-to-proxy binding (account_id -> proxy_id)
    pub(crate) account_bindings: Arc<DashMap<String, String>>,

    /// Round-robin index (for RoundRobin strategy)
    pub(crate) round_robin_index: Arc<AtomicUsize>,
}

impl ProxyPoolManager {
    pub fn new(config: Arc<RwLock<ProxyPoolConfig>>) -> Self {
        // Load saved bindings from configuration
        let account_bindings = Arc::new(DashMap::new());

        // Use blocking read (since new is not async)
        // Note: use try_read to avoid deadlock
        if let Ok(cfg) = config.try_read() {
            for (account_id, proxy_id) in &cfg.account_bindings {
                account_bindings.insert(account_id.clone(), proxy_id.clone());
            }
            if !cfg.account_bindings.is_empty() {
                tracing::info!(
                    "[ProxyPool] Loaded {} account bindings from config",
                    cfg.account_bindings.len()
                );
            }
        }

        Self {
            config,
            usage_counter: Arc::new(DashMap::new()),
            account_bindings,
            round_robin_index: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// [NEW] Get effective HttpClient for specified account
    /// Logic:
    /// 1. Account-Proxy explicit binding takes priority (Dedicated IP)
    /// 2. If unbound and auto-global enabled, pick first node in pool
    /// 3. If none of above, check global Upstream Proxy [caller fallback]
    pub async fn get_effective_client(
        &self,
        account_id: Option<&str>,
        timeout_secs: u64,
    ) -> Client {
        let mut builder = Client::builder()
            .emulation(Emulation::Chrome123)
            .timeout(Duration::from_secs(timeout_secs));

        // Try to get proxy configuration
        let proxy_opt = if let Some(acc_id) = account_id {
            self.get_proxy_for_account(acc_id).await.ok().flatten()
        } else {
            // Generic request without account_id: pick node from pool if enabled
            let config = self.config.read().await;
            if config.enabled {
                let res = self.select_proxy_from_pool(&config).await.ok().flatten();
                if let Some(ref p) = res {
                    tracing::info!(
                        "[Proxy] Route: Generic Request -> Proxy {} (Pool)",
                        p.entry_id
                    );
                } else {
                    // [FIX #1583] Explicitly log when no available proxy exists in pool
                    tracing::warn!("[Proxy] Route: Generic Request -> No available proxy in pool, falling back to upstream or direct");
                }
                res
            } else {
                tracing::debug!("[Proxy] Route: Generic Request -> Proxy pool disabled");
                None
            }
        };

        if let Some(proxy_cfg) = proxy_opt {
            builder = builder.proxy(proxy_cfg.proxy);
            // Already logged more detail in get_proxy_for_account or pool selection
        } else {
            // Fallback to configured single upstream proxy
            if let Ok(app_cfg) = crate::modules::config::load_app_config() {
                let up = app_cfg.proxy.upstream_proxy;
                if up.enabled && !up.url.is_empty() {
                    if let Ok(p) = rquest::Proxy::all(&up.url) {
                        tracing::info!(
                            "[Proxy] Route: {:?} -> Upstream: {} (AppConfig)",
                            account_id.unwrap_or("Generic"),
                            up.url
                        );
                        builder = builder.proxy(p);
                    }
                } else {
                    tracing::info!(
                        "[Proxy] Route: {:?} -> Direct",
                        account_id.unwrap_or("Generic")
                    );
                }
            }
        }

        builder.build().unwrap_or_else(|_| Client::new())
    }

    /// [NEW] Get effective standard HttpClient without emulation (for pure OAuth flows)
    pub async fn get_effective_standard_client(
        &self,
        account_id: Option<&str>,
        timeout_secs: u64,
    ) -> Client {
        let mut builder = Client::builder()
            // No emulation: use base TLS fingerprint
            .timeout(Duration::from_secs(timeout_secs));

        // Try to get proxy configuration
        let proxy_opt = if let Some(acc_id) = account_id {
            self.get_proxy_for_account(acc_id).await.ok().flatten()
        } else {
            // Generic request without account_id: pick node from pool if enabled
            let config = self.config.read().await;
            if config.enabled {
                let res = self.select_proxy_from_pool(&config).await.ok().flatten();
                if let Some(ref p) = res {
                    tracing::info!(
                        "[Proxy] Route: Generic Request (Standard Client) -> Proxy {} (Pool)",
                        p.entry_id
                    );
                } else {
                    tracing::warn!("[Proxy] Route: Generic Request (Standard Client) -> No available proxy in pool, falling back to upstream or direct");
                }
                res
            } else {
                tracing::debug!(
                    "[Proxy] Route: Generic Request (Standard Client) -> Proxy pool disabled"
                );
                None
            }
        };

        if let Some(proxy_cfg) = proxy_opt {
            builder = builder.proxy(proxy_cfg.proxy);
        } else {
            // Fallback to configured single upstream proxy
            if let Ok(app_cfg) = crate::modules::config::load_app_config() {
                let up = app_cfg.proxy.upstream_proxy;
                if up.enabled && !up.url.is_empty() {
                    if let Ok(p) = rquest::Proxy::all(&up.url) {
                        tracing::info!(
                            "[Proxy] Route: {:?} (Standard Client) -> Upstream: {} (AppConfig)",
                            account_id.unwrap_or("Generic"),
                            up.url
                        );
                        builder = builder.proxy(p);
                    }
                } else {
                    tracing::info!(
                        "[Proxy] Route: {:?} (Standard Client) -> Direct",
                        account_id.unwrap_or("Generic")
                    );
                }
            }
        }

        builder.build().unwrap_or_else(|_| Client::new())
    }

    /// Get proxy for account
    pub async fn get_proxy_for_account(
        &self,
        account_id: &str,
    ) -> Result<Option<PoolProxyConfig>, String> {
        let config = self.config.read().await;

        if !config.enabled || config.proxies.is_empty() {
            return Ok(None);
        }

        // 1. Account binding priority (Dedicated IP)
        if let Some(proxy) = self.get_bound_proxy(account_id, &config).await? {
            tracing::info!(
                "[Proxy] Route: Account {} -> Proxy {} (Bound)",
                account_id,
                proxy.entry_id
            );
            return Ok(Some(proxy));
        }

        // 2. Otherwise select proxy from pool based on strategy (Shared pool)
        let res = self.select_proxy_from_pool(&config).await?;
        if let Some(ref p) = res {
            tracing::info!(
                "[Proxy] Route: Account {} -> Proxy {} (Pool)",
                account_id,
                p.entry_id
            );
        }
        Ok(res)
    }

    /// Get proxy bound to account
    async fn get_bound_proxy(
        &self,
        account_id: &str,
        config: &ProxyPoolConfig,
    ) -> Result<Option<PoolProxyConfig>, String> {
        if let Some(proxy_id) = self.account_bindings.get(account_id) {
            if let Some(entry) = config.proxies.iter().find(|p| p.id == *proxy_id.value()) {
                if entry.enabled {
                    // If auto failover enabled and proxy is unhealthy, return None
                    if config.auto_failover && !entry.is_healthy {
                        return Ok(None);
                    }
                    return Ok(Some(self.build_proxy_config(entry)?));
                }
            }
        }
        Ok(None)
    }

    /// Select proxy from pool
    async fn select_proxy_from_pool(
        &self,
        config: &ProxyPoolConfig,
    ) -> Result<Option<PoolProxyConfig>, String> {
        // [FIX] Dedicated isolation: exclude all bound proxies to protect dedicated IPs
        let bound_ids: std::collections::HashSet<String> = self
            .account_bindings
            .iter()
            .map(|kv| kv.value().clone())
            .collect();

        let healthy_proxies: Vec<_> = config
            .proxies
            .iter()
            .filter(|p| {
                if !p.enabled {
                    return false;
                }
                if config.auto_failover && !p.is_healthy {
                    return false;
                }
                // If proxy is dedicated-bound, exclude from shared round-robin
                if bound_ids.contains(&p.id) {
                    return false;
                }
                true
            })
            .collect();

        if healthy_proxies.is_empty() {
            // If all proxies are bound or pool is empty, try to return unbound active proxies
            // (Strict isolation maintained)
            return Ok(None);
        }

        let selected = match config.strategy {
            ProxySelectionStrategy::RoundRobin => self.select_round_robin(&healthy_proxies),
            ProxySelectionStrategy::Random => self.select_random(&healthy_proxies),
            ProxySelectionStrategy::Priority => self.select_by_priority(&healthy_proxies),
            ProxySelectionStrategy::LeastConnections => {
                self.select_least_connections(&healthy_proxies)
            }
            ProxySelectionStrategy::WeightedRoundRobin => self.select_weighted(&healthy_proxies),
        };

        if let Some(entry) = selected {
            // Update counter
            *self.usage_counter.entry(entry.id.clone()).or_insert(0) += 1;
            Ok(Some(self.build_proxy_config(entry)?))
        } else {
            Ok(None)
        }
    }

    fn select_round_robin<'a>(&self, proxies: &[&'a ProxyEntry]) -> Option<&'a ProxyEntry> {
        if proxies.is_empty() {
            return None;
        }
        let index = self.round_robin_index.fetch_add(1, Ordering::Relaxed);
        Some(proxies[index % proxies.len()])
    }

    fn select_random<'a>(&self, proxies: &[&'a ProxyEntry]) -> Option<&'a ProxyEntry> {
        if proxies.is_empty() {
            return None;
        }
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        proxies.choose(&mut rng).copied()
    }

    fn select_by_priority<'a>(&self, proxies: &[&'a ProxyEntry]) -> Option<&'a ProxyEntry> {
        // Lower priority value has higher precedence
        proxies.iter().min_by_key(|p| p.priority).copied()
    }

    fn select_least_connections<'a>(&self, proxies: &[&'a ProxyEntry]) -> Option<&'a ProxyEntry> {
        proxies
            .iter()
            .min_by_key(|p| self.usage_counter.get(&p.id).map(|v| *v).unwrap_or(0))
            .copied()
    }

    fn select_weighted<'a>(&self, proxies: &[&'a ProxyEntry]) -> Option<&'a ProxyEntry> {
        // Simple weighted implementation: using Priority as surrogate
        self.select_by_priority(proxies)
    }
}
