use super::*;

impl UpstreamClient {
    pub fn new(
        proxy_config: Option<crate::proxy::config::UpstreamProxyConfig>,
        proxy_pool: Option<Arc<crate::proxy::proxy_pool::ProxyPoolManager>>,
    ) -> Self {
        let default_client = match Self::build_client_internal(proxy_config.clone()) {
            Ok(client) => client,
            Err(err_with_proxy) => {
                tracing::error!(
                    error = %err_with_proxy,
                    "Failed to create default HTTP client with configured upstream proxy; retrying without proxy"
                );
                match Self::build_client_internal(None) {
                    Ok(client) => client,
                    Err(err_without_proxy) => {
                        tracing::error!(
                            error = %err_without_proxy,
                            "Failed to create default HTTP client without proxy; falling back to bare client"
                        );
                        Client::new()
                    }
                }
            }
        };

        Self {
            default_client: RwLock::new(default_client),
            proxy_pool,
            client_cache: DashMap::new(),
            user_agent_override: RwLock::new(None),
        }
    }

    /// [HOT-RELOAD] Rebuild the default HTTP client using the supplied upstream
    /// proxy config. Called from `update_proxy` so changes to the upstream proxy
    /// take effect without restarting the app.
    pub async fn rebuild_default_client(
        &self,
        proxy_config: Option<crate::proxy::config::UpstreamProxyConfig>,
    ) {
        let new_client = match Self::build_client_internal(proxy_config.clone()) {
            Ok(c) => c,
            Err(err_with_proxy) => {
                tracing::error!(
                    error = %err_with_proxy,
                    "Hot-reload: failed to rebuild default HTTP client with configured upstream proxy; retrying without proxy"
                );
                match Self::build_client_internal(None) {
                    Ok(c) => c,
                    Err(err_without_proxy) => {
                        tracing::error!(
                            error = %err_without_proxy,
                            "Hot-reload: failed to rebuild default HTTP client without proxy; keeping previous client"
                        );
                        return;
                    }
                }
            }
        };
        let mut guard = self.default_client.write().await;
        *guard = new_client;
        tracing::info!("UpstreamClient default_client rebuilt (upstream proxy hot-reloaded)");
    }

    /// [HOT-RELOAD] Drop all per-proxy cached clients. Call after the pool
    /// configuration changes (proxy URL/credentials edited, proxy removed,
    /// bindings changed) so the next request rebuilds with fresh settings.
    pub fn clear_client_cache(&self) {
        let size = self.client_cache.len();
        self.client_cache.clear();
        if size > 0 {
            tracing::info!("UpstreamClient cleared {} cached per-proxy clients", size);
        }
    }

    /// Internal helper to build a client with optional upstream proxy config
    fn build_client_internal(
        proxy_config: Option<crate::proxy::config::UpstreamProxyConfig>,
    ) -> Result<Client, rquest::Error> {
        let mut builder = Client::builder()
            .emulation(rquest_util::Emulation::Chrome123)
            // Connection settings (优化连接复用，减少建立开销)
            .connect_timeout(Duration::from_secs(20))
            .pool_max_idle_per_host(20) // 每主机最多 20 个空闲连接 (对齐官方指纹)
            .pool_idle_timeout(Duration::from_secs(90)) // 空闲连接保持 90 秒
            .tcp_keepalive(Duration::from_secs(60)) // TCP 保活探测 60 秒
            // 强制开启 HTTP/2 协议，并支持在 SOCKS/HTTPS 代理下通过 ALPN 强制降级/协商
            .timeout(Duration::from_secs(600));

        builder = Self::apply_default_user_agent(builder);

        if let Some(config) = proxy_config {
            if config.enabled && !config.url.is_empty() {
                let url = crate::proxy::config::normalize_proxy_url(&config.url);
                if let Ok(proxy) = rquest::Proxy::all(&url) {
                    builder = builder.proxy(proxy);
                    tracing::info!("UpstreamClient enabled proxy: {}", url);
                }
            }
        }

        builder.build()
    }

    /// Build a client with a specific PoolProxyConfig (from ProxyPool)
    fn build_client_with_proxy(
        &self,
        proxy_config: crate::proxy::proxy_pool::PoolProxyConfig,
    ) -> Result<Client, rquest::Error> {
        // Reuse base settings similar to default client but with specific proxy
        let builder = Client::builder()
            .emulation(rquest_util::Emulation::Chrome123)
            .connect_timeout(Duration::from_secs(20))
            .pool_max_idle_per_host(20)
            .pool_idle_timeout(Duration::from_secs(90))
            .tcp_keepalive(Duration::from_secs(60))
            .timeout(Duration::from_secs(600))
            .proxy(proxy_config.proxy); // Apply the specific proxy

        Self::apply_default_user_agent(builder).build()
    }

    fn apply_default_user_agent(builder: rquest::ClientBuilder) -> rquest::ClientBuilder {
        let ua = crate::constants::USER_AGENT.as_str();
        if header::HeaderValue::from_str(ua).is_ok() {
            builder.user_agent(ua)
        } else {
            tracing::warn!(
                user_agent = %ua,
                "Invalid default User-Agent value, using fallback"
            );
            builder.user_agent("antigravity")
        }
    }

    /// Set dynamic User-Agent override
    pub async fn set_user_agent_override(&self, ua: Option<String>) {
        let mut lock = self.user_agent_override.write().await;
        *lock = ua;
        tracing::debug!("UpstreamClient User-Agent override updated: {:?}", lock);
    }

    /// Get current User-Agent (sanitized with safety floor >= 4.3.0)
    pub async fn get_user_agent(&self) -> String {
        let ua_override = self.user_agent_override.read().await;
        match ua_override.as_ref() {
            Some(ua) => crate::constants::sanitize_egress_user_agent(ua),
            None => crate::constants::USER_AGENT.clone(),
        }
    }

    /// Get client for a specific account (or default if no proxy bound)
    pub async fn get_client(&self, account_id: Option<&str>) -> Client {
        if let Some(pool) = &self.proxy_pool {
            if let Some(acc_id) = account_id {
                // Try to get per-account proxy
                match pool.get_proxy_for_account(acc_id).await {
                    Ok(Some(proxy_cfg)) => {
                        // Check cache
                        if let Some(client) = self.client_cache.get(&proxy_cfg.entry_id) {
                            return client.clone();
                        }
                        // Build new client and cache it
                        match self.build_client_with_proxy(proxy_cfg.clone()) {
                            Ok(client) => {
                                self.client_cache
                                    .insert(proxy_cfg.entry_id.clone(), client.clone());
                                tracing::info!(
                                    "Using ProxyPool proxy ID: {} for account: {}",
                                    proxy_cfg.entry_id,
                                    acc_id
                                );
                                return client;
                            }
                            Err(e) => {
                                tracing::error!("Failed to build client for proxy {}: {}, falling back to default", proxy_cfg.entry_id, e);
                            }
                        }
                    }
                    Ok(None) => {
                        // No proxy found or required for this account, use default
                    }
                    Err(e) => {
                        tracing::error!(
                            "Error getting proxy for account {}: {}, falling back to default",
                            acc_id,
                            e
                        );
                    }
                }
            }
        }
        // Fallback to default client
        self.default_client.read().await.clone()
    }
}
