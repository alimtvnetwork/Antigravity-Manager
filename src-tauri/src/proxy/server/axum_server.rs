//! AxumServer handle: hot-reload updaters and lifecycle controls.
use crate::proxy::TokenManager;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;

/// Axum server instance
#[derive(Clone)]
pub struct AxumServer {
    cancel_token: tokio_util::sync::CancellationToken,
    custom_mapping: Arc<tokio::sync::RwLock<std::collections::HashMap<String, String>>>,
    proxy_state: Arc<tokio::sync::RwLock<crate::proxy::config::UpstreamProxyConfig>>,
    upstream: Arc<crate::proxy::upstream::client::UpstreamClient>,
    security_state: Arc<RwLock<crate::proxy::ProxySecurityConfig>>,
    zai_state: Arc<RwLock<crate::proxy::ZaiConfig>>,
    experimental: Arc<RwLock<crate::proxy::config::ExperimentalConfig>>,
    debug_logging: Arc<RwLock<crate::proxy::config::DebugLoggingConfig>>,
    #[allow(dead_code)] // Reserved for cloudflared status queries and future control
    pub cloudflared_state: Arc<crate::commands::cloudflared::CloudflaredState>,
    pub is_running: Arc<RwLock<bool>>,
    pub token_manager: Arc<TokenManager>, // [NEW] Expose TokenManager for proxy reuse
    pub proxy_pool_state: Arc<tokio::sync::RwLock<crate::proxy::config::ProxyPoolConfig>>, // [NEW] Proxy pool config state
    pub proxy_pool_manager: Arc<crate::proxy::proxy_pool::ProxyPoolManager>, // [NEW] Proxy pool manager instance
    pub only_raw_quota_models: Arc<tokio::sync::RwLock<bool>>,
}

impl AxumServer {
    pub async fn update_only_raw_quota_models(&self, only_raw: bool) {
        let mut r = self.only_raw_quota_models.write().await;
        *r = only_raw;
        tracing::debug!("only_raw_quota_models updated: {}", only_raw);
    }

    pub async fn update_mapping(&self, config: &crate::proxy::config::ProxyConfig) {
        {
            let mut m = self.custom_mapping.write().await;
            *m = config.custom_mapping.clone();
        }
        tracing::debug!("Model mapping (Custom) fully hot-reloaded");
    }

    /// Update proxy configuration
    pub async fn update_proxy(&self, new_config: crate::proxy::config::UpstreamProxyConfig) {
        {
            let mut proxy = self.proxy_state.write().await;
            *proxy = new_config.clone();
        }
        // [HOT-RELOAD] Rebuild default HTTP client with new upstream proxy
        self.upstream.rebuild_default_client(Some(new_config)).await;
        // Stale per-proxy clients may also be affected (e.g. fallback path)
        self.upstream.clear_client_cache();
        // 全局共享客户端（token 刷新 / 配额刷新 / 项目解析 / zai / MCP 等）同样是按
        // 构建时的代理配置定型的，必须一并失效，否则会带着旧代理继续跑。
        crate::utils::http::invalidate_shared_clients();
        tracing::info!("Upstream proxy config hot-reloaded");
    }

    /// Update proxy pool configuration
    pub async fn update_proxy_pool(&self, new_config: crate::proxy::config::ProxyPoolConfig) {
        {
            let mut pool = self.proxy_pool_state.write().await;
            *pool = new_config;
        }
        // [HOT-RELOAD] Re-sync in-memory account↔proxy bindings from the new config
        self.proxy_pool_manager.sync_bindings_from_config().await;
        // [HOT-RELOAD] Drop cached per-proxy HTTP clients so they rebuild with new URLs/credentials
        self.upstream.clear_client_cache();
        tracing::info!("Proxy pool config hot-reloaded");
    }

    pub async fn update_security(&self, config: &crate::proxy::config::ProxyConfig) {
        let mut sec = self.security_state.write().await;
        *sec = crate::proxy::ProxySecurityConfig::from_proxy_config(config);
        tracing::info!("Proxy service security configuration hot-reloaded");
    }

    pub async fn update_zai(&self, config: &crate::proxy::config::ProxyConfig) {
        let mut zai = self.zai_state.write().await;
        *zai = config.zai.clone();
        tracing::info!("z.ai configuration hot-reloaded");
    }

    pub async fn update_experimental(&self, config: &crate::proxy::config::ProxyConfig) {
        let mut exp = self.experimental.write().await;
        *exp = config.experimental.clone();
        tracing::info!("Experimental configuration hot-reloaded");
    }

    pub async fn update_debug_logging(&self, config: &crate::proxy::config::ProxyConfig) {
        let mut dbg_cfg = self.debug_logging.write().await;
        *dbg_cfg = config.debug_logging.clone();
        tracing::info!("Debug log configuration hot-reloaded");
    }

    pub async fn update_user_agent(&self, config: &crate::proxy::config::ProxyConfig) {
        self.upstream
            .set_user_agent_override(config.user_agent_override.clone())
            .await;
        tracing::info!(
            "User-Agent configuration hot-reloaded: {:?}",
            config.user_agent_override
        );
    }

    pub async fn set_running(&self, running: bool) {
        let mut r = self.is_running.write().await;
        *r = running;
        tracing::info!("Proxy service running status updated to: {}", running);
    }

    /// Stop server service
    pub fn stop(&self) {
        self.cancel_token.cancel();
        tracing::info!("Axum server stop signal sent");
    }

    /// Check if server has been stopped
    pub fn is_stopped(&self) -> bool {
        self.cancel_token.is_cancelled()
    }
}
