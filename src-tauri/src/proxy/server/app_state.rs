//! Axum application state shared by all proxy/admin handlers.
use crate::proxy::TokenManager;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::proxy::server::image_scheduler::ImageScheduler;

/// Axum application state
#[derive(Clone)]
pub struct AppState {
    pub token_manager: Arc<TokenManager>,
    pub custom_mapping: Arc<tokio::sync::RwLock<std::collections::HashMap<String, String>>>,
    pub request_timeout: u64, // API request timeout (seconds)
    #[allow(dead_code)]
    pub thought_signature_map: Arc<tokio::sync::Mutex<std::collections::HashMap<String, String>>>, // Thinking chain signature mapping (ID -> Signature)
    #[allow(dead_code)]
    pub upstream_proxy: Arc<tokio::sync::RwLock<crate::proxy::config::UpstreamProxyConfig>>,
    pub upstream: Arc<crate::proxy::upstream::client::UpstreamClient>,
    pub zai: Arc<RwLock<crate::proxy::ZaiConfig>>,
    pub provider_rr: Arc<AtomicUsize>,
    pub zai_vision_mcp: Arc<crate::proxy::zai_vision_mcp::ZaiVisionMcpState>,
    pub monitor: Arc<crate::proxy::monitor::ProxyMonitor>,
    pub experimental: Arc<RwLock<crate::proxy::config::ExperimentalConfig>>,
    pub debug_logging: Arc<RwLock<crate::proxy::config::DebugLoggingConfig>>,
    pub switching: Arc<RwLock<bool>>, // [NEW] Account switching state lock to prevent concurrent switches
    pub integration: crate::modules::integration::SystemManager, // [NEW] System integration implementation
    pub account_service: Arc<crate::modules::account_service::AccountService>, // [NEW] Account management service layer
    pub security: Arc<RwLock<crate::proxy::ProxySecurityConfig>>, // [NEW] Security configuration state
    pub cloudflared_state: Arc<crate::commands::cloudflared::CloudflaredState>, // [NEW] Cloudflared plugin state
    pub is_running: Arc<RwLock<bool>>, // [NEW] Service running state flag
    pub port: u16,                     // [NEW] Local listening port
    pub proxy_pool_state: Arc<tokio::sync::RwLock<crate::proxy::config::ProxyPoolConfig>>, // [FIX Web Mode]
    pub proxy_pool_manager: Arc<crate::proxy::proxy_pool::ProxyPoolManager>, // [FIX Web Mode]
    pub only_raw_quota_models: Arc<tokio::sync::RwLock<bool>>, // [NEW] Whether to only expose raw quota models
    pub image_scheduler: Arc<ImageScheduler>,
}
