use super::*;

impl ProxyPoolManager {

    /// Build rquest::Proxy configuration
    fn build_proxy_config(&self, entry: &ProxyEntry) -> Result<PoolProxyConfig, String> {
        let raw_url = crate::proxy::config::normalize_proxy_url(&entry.url);

        // Parse URL and extract embedded username and password if present
        let (clean_url, parsed_auth) = match url::Url::parse(&raw_url) {
            Ok(mut u) => {
                let user = if !u.username().is_empty() {
                    Some(u.username().to_string())
                } else {
                    None
                };
                let pass = u.password().map(|p| p.to_string());

                // Strip credentials from URL to prevent underlying library parse errors
                // Justification: Url::set_* returns Result<(), ()> — the unit error carries no information to log; keep the discard.
                let _ = u.set_username("");
                // Justification: Url::set_* returns Result<(), ()> — the unit error carries no information to log; keep the discard.
                let _ = u.set_password(None);

                let auth = if let (Some(user), Some(pass)) = (user, pass) {
                    Some((user, pass))
                } else {
                    None
                };
                (u.to_string(), auth)
            }
            Err(_) => (raw_url.clone(), None),
        };

        let mut proxy = rquest::Proxy::all(&clean_url)
            .or_else(|_| rquest::Proxy::all(&raw_url))
            .map_err(|e| format!("Invalid proxy URL: {}", e))?;

        // Prefer structured auth, fallback to embedded credentials from URL
        if let Some(auth) = &entry.auth {
            if !auth.username.is_empty() {
                if auth.password.starts_with("ag_enc_") && parsed_auth.is_some() {
                    tracing::warn!(
                        "[ProxyPool] Proxy password decryption failed (retains ag_enc_ prefix); falling back to credentials embedded in URL"
                    );
                    let (user, pass) = parsed_auth.as_ref().unwrap();
                    proxy = proxy.basic_auth(user, pass);
                } else {
                    proxy = proxy.basic_auth(&auth.username, &auth.password);
                }
            } else if let Some((user, pass)) = parsed_auth {
                proxy = proxy.basic_auth(&user, &pass);
            }
        } else if let Some((user, pass)) = parsed_auth {
            proxy = proxy.basic_auth(&user, &pass);
        }

        Ok(PoolProxyConfig {
            proxy,
            entry_id: entry.id.clone(),
        })
    }

    /// Bind account to proxy
    pub async fn bind_account_to_proxy(
        &self,
        account_id: String,
        proxy_id: String,
    ) -> Result<(), String> {
        // Check if proxy exists
        {
            let config = self.config.read().await;
            if !config.proxies.iter().any(|p| p.id == proxy_id) {
                return Err(format!("Proxy {} not found", proxy_id));
            }

            // Check maximum account limit for proxy
            if let Some(entry) = config.proxies.iter().find(|p| p.id == proxy_id) {
                if let Some(max) = entry.max_accounts {
                    if max > 0 {
                        let current_count = self
                            .account_bindings
                            .iter()
                            .filter(|kv| *kv.value() == proxy_id)
                            .count();
                        if current_count >= max {
                            return Err(format!(
                                "Proxy {} has reached max accounts limit",
                                proxy_id
                            ));
                        }
                    }
                }
            }
        }

        // Update in-memory binding
        self.account_bindings
            .insert(account_id.clone(), proxy_id.clone());

        // Persist to configuration file
        self.persist_bindings().await;

        tracing::info!(
            "[ProxyPool] Bound account {} to proxy {}",
            account_id,
            proxy_id
        );
        Ok(())
    }

    /// Unbind account proxy
    pub async fn unbind_account_proxy(&self, account_id: String) {
        self.account_bindings.remove(&account_id);

        // Persist to configuration file
        self.persist_bindings().await;

        tracing::info!("[ProxyPool] Unbound account {}", account_id);
    }

    /// Get currently bound proxy ID for account
    pub fn get_account_binding(&self, account_id: &str) -> Option<String> {
        self.account_bindings
            .get(account_id)
            .map(|v| v.value().clone())
    }

    /// Get snapshot of all account bindings
    pub fn get_all_bindings_snapshot(&self) -> std::collections::HashMap<String, String> {
        self.account_bindings
            .iter()
            .map(|kv| (kv.key().clone(), kv.value().clone()))
            .collect()
    }

    /// [HOT-RELOAD] Re-sync the in-memory DashMap from `config.account_bindings`.
    /// Called after `update_proxy_pool` so that a wholesale ProxyPoolConfig
    /// replacement (e.g. via `save_config`) does not leave the in-memory
    /// bindings stale or empty.
    pub async fn sync_bindings_from_config(&self) {
        let config = self.config.read().await;
        let snapshot = config.account_bindings.clone();
        drop(config);

        // Reset the DashMap: clear old entries, then insert fresh ones.
        self.account_bindings.clear();
        for (account_id, proxy_id) in &snapshot {
            self.account_bindings
                .insert(account_id.clone(), proxy_id.clone());
        }
        tracing::info!(
            "[ProxyPool] Re-synced {} account bindings from config (hot-reload)",
            snapshot.len()
        );
    }

    /// Persist bindings to configuration file
    async fn persist_bindings(&self) {
        // Get current binding snapshot
        let bindings = self.get_all_bindings_snapshot();

        // Update bindings in configuration
        {
            let mut config = self.config.write().await;
            config.account_bindings = bindings;
        }

        // Save to disk
        if let Ok(mut app_config) = crate::modules::config::load_app_config() {
            let config = self.config.read().await;
            app_config.proxy.proxy_pool = config.clone();
            if let Err(e) = crate::modules::config::save_app_config(&app_config) {
                tracing::error!("[ProxyPool] Failed to persist bindings: {}", e);
            }
        }
    }

    /// Batch check proxy health status
    pub async fn health_check(&self) -> Result<(), String> {
        let proxies_to_check: Vec<ProxyEntry> = {
            let config = self.config.read().await;
            config
                .proxies
                .iter()
                .filter(|p| p.enabled)
                .cloned()
                .collect()
        };

        let concurrency_limit = 20usize;
        let results = stream::iter(proxies_to_check)
            .map(|proxy| async move {
                let (is_healthy, latency) = self.check_proxy_health(&proxy).await;

                let latency_msg = if let Some(ms) = latency {
                    format!("{}ms", ms)
                } else {
                    "-".to_string()
                };

                tracing::info!(
                    "Proxy {} ({}) health check: {} (Latency: {})",
                    proxy.name,
                    proxy.url,
                    if is_healthy { "✓ OK" } else { "✗ FAILED" },
                    latency_msg
                );

                (proxy.id, is_healthy, latency)
            })
            .buffer_unordered(concurrency_limit)
            .collect::<Vec<_>>()
            .await;

        // Update statuses in batch
        let mut config = self.config.write().await;
        for (id, is_healthy, latency) in results {
            if let Some(proxy) = config.proxies.iter_mut().find(|p| p.id == id) {
                proxy.is_healthy = is_healthy;
                proxy.latency = latency;
                proxy.last_check_time = Some(chrono::Utc::now().timestamp());
            }
        }

        Ok(())
    }

    /// Check health status for single proxy
    async fn check_proxy_health(&self, entry: &ProxyEntry) -> (bool, Option<u64>) {
        const DEFAULT_HEALTH_CHECK_URL: &str = "https://cp.cloudflare.com/generate_204";

        let check_url = if let Some(url) = &entry.health_check_url {
            if url.trim().is_empty() {
                DEFAULT_HEALTH_CHECK_URL
            } else {
                url.as_str()
            }
        } else {
            DEFAULT_HEALTH_CHECK_URL
        };

        // Attempt building Client; failure marks proxy unhealthy
        let proxy_res = self.build_proxy_config(entry);
        if let Err(e) = proxy_res {
            tracing::error!("Proxy {} build config failed: {}", entry.url, e);
            return (false, None);
        }
        let proxy_cfg = proxy_res.unwrap();

        let client_result = Client::builder()
            .proxy(proxy_cfg.proxy)
            .emulation(Emulation::Chrome123)
            .timeout(Duration::from_secs(10))
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/123.0.0.0 Safari/537.36")
            .build();

        let client = match client_result {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("Proxy {} build client failed: {}", entry.url, e);
                return (false, None);
            }
        };

        let start = std::time::Instant::now();
        match client.get(check_url).send().await {
            Ok(resp) => {
                let latency = start.elapsed().as_millis() as u64;
                if resp.status().is_success() {
                    (true, Some(latency))
                } else {
                    tracing::warn!(
                        "Proxy {} health check status error: {}",
                        entry.url,
                        resp.status()
                    );
                    (false, None)
                }
            }
            Err(e) => {
                tracing::warn!("Proxy {} health check request failed: {}", entry.url, e);
                (false, None)
            }
        }
    }

    /// Start background health check loop
    pub fn start_health_check_loop(self: Arc<Self>) {
        tokio::spawn(async move {
            tracing::info!("Starting proxy pool health check loop...");
            loop {
                // Perform check only if enabled
                let enabled = self.config.read().await.enabled;
                if enabled {
                    if let Err(e) = self.health_check().await {
                        tracing::error!("Proxy pool health check failed: {}", e);
                    }
                }

                // Get interval and sleep AFTER check
                let interval_secs = {
                    let cfg = self.config.read().await;
                    if !cfg.enabled {
                        60 // check every minute if disabled
                    } else {
                        cfg.health_check_interval.max(30) // Back to default min 30s
                    }
                };

                tokio::time::sleep(Duration::from_secs(interval_secs)).await;
            }
        });
    }

}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::proxy::config::ProxyAuth;

    #[test]
    fn test_build_proxy_config_with_explicit_auth() {
        let pool = ProxyPoolManager::new(Arc::new(RwLock::new(ProxyPoolConfig::default())));
        let entry = ProxyEntry {
            id: "p1".to_string(),
            name: "test".to_string(),
            url: "http://127.0.0.1:8080".to_string(),
            auth: Some(ProxyAuth {
                username: "user".to_string(),
                password: "pass".to_string(),
            }),
            enabled: true,
            priority: 1,
            tags: vec![],
            max_accounts: None,
            health_check_url: None,
            last_check_time: None,
            is_healthy: true,
            latency: None,
        };

        let res = pool.build_proxy_config(&entry);
        assert!(res.is_ok());
        assert_eq!(res.unwrap().entry_id, "p1");
    }

    #[test]
    fn test_build_proxy_config_with_url_auth() {
        let pool = ProxyPoolManager::new(Arc::new(RwLock::new(ProxyPoolConfig::default())));
        let entry = ProxyEntry {
            id: "p2".to_string(),
            name: "test_url_auth".to_string(),
            url: "http://user:pass@127.0.0.1:10080".to_string(),
            auth: None,
            enabled: true,
            priority: 1,
            tags: vec![],
            max_accounts: None,
            health_check_url: None,
            last_check_time: None,
            is_healthy: true,
            latency: None,
        };

        let res = pool.build_proxy_config(&entry);
        assert!(res.is_ok());
        assert_eq!(res.unwrap().entry_id, "p2");
    }

    #[test]
    fn test_build_proxy_config_fallback_to_url_auth_on_decrypt_failure() {
        let pool = ProxyPoolManager::new(Arc::new(RwLock::new(ProxyPoolConfig::default())));
        let entry = ProxyEntry {
            id: "p3".to_string(),
            name: "test_fallback_auth".to_string(),
            url: "http://url_user:url_pass@127.0.0.1:10080".to_string(),
            auth: Some(ProxyAuth {
                username: "struct_user".to_string(),
                password: "ag_enc_v2_failed_decrypt_payload".to_string(),
            }),
            enabled: true,
            priority: 1,
            tags: vec![],
            max_accounts: None,
            health_check_url: None,
            last_check_time: None,
            is_healthy: true,
            latency: None,
        };

        let res = pool.build_proxy_config(&entry);
        assert!(res.is_ok());
        assert_eq!(res.unwrap().entry_id, "p3");
    }
}
