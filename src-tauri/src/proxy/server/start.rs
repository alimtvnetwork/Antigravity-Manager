//! AxumServer::start split into mechanical phases: state, routers, bind, serve loop.

use super::app_state::AppState;
use super::axum_server::AxumServer;
use super::image_scheduler::build_image_scheduler;
use crate::proxy::TokenManager;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error};

struct StartupParts {
    state: super::app_state::AppState,
    custom_mapping_state: Arc<tokio::sync::RwLock<std::collections::HashMap<String, String>>>,
    proxy_state: Arc<tokio::sync::RwLock<crate::proxy::config::UpstreamProxyConfig>>,
    proxy_pool_state: Arc<tokio::sync::RwLock<crate::proxy::config::ProxyPoolConfig>>,
    proxy_pool_manager: Arc<crate::proxy::proxy_pool::ProxyPoolManager>,
    security_state: Arc<tokio::sync::RwLock<crate::proxy::ProxySecurityConfig>>,
    zai_state: Arc<tokio::sync::RwLock<crate::proxy::ZaiConfig>>,
    experimental_state: Arc<tokio::sync::RwLock<crate::proxy::config::ExperimentalConfig>>,
    debug_logging_state: Arc<tokio::sync::RwLock<crate::proxy::config::DebugLoggingConfig>>,
    is_running_state: Arc<tokio::sync::RwLock<bool>>,
    only_raw_quota_models_state: Arc<tokio::sync::RwLock<bool>>,
    cloudflared_state: Arc<crate::commands::cloudflared::CloudflaredState>,
    token_manager: Arc<crate::proxy::TokenManager>,
}

async fn build_startup_parts(
    port: u16,
    token_manager: Arc<crate::proxy::TokenManager>,
    custom_mapping: std::collections::HashMap<String, String>,
    request_timeout: u64,
    upstream_proxy: crate::proxy::config::UpstreamProxyConfig,
    user_agent_override: Option<String>,
    security_config: crate::proxy::ProxySecurityConfig,
    zai_config: crate::proxy::ZaiConfig,
    monitor: Arc<crate::proxy::monitor::ProxyMonitor>,
    experimental_config: crate::proxy::config::ExperimentalConfig,
    debug_logging: crate::proxy::config::DebugLoggingConfig,
    integration: crate::modules::integration::SystemManager,
    cloudflared_state: Arc<crate::commands::cloudflared::CloudflaredState>,
    proxy_pool_config: crate::proxy::config::ProxyPoolConfig,
    only_raw_quota_models: bool,
    image_scheduler_config: crate::proxy::config::ImageSchedulerConfig,
) -> StartupParts {
    let custom_mapping_state = Arc::new(tokio::sync::RwLock::new(custom_mapping));
    let proxy_state = Arc::new(tokio::sync::RwLock::new(upstream_proxy.clone()));
    let proxy_pool_state = Arc::new(tokio::sync::RwLock::new(proxy_pool_config));
    let proxy_pool_manager =
        crate::proxy::proxy_pool::init_global_proxy_pool(proxy_pool_state.clone());

    // Start health check loop
    proxy_pool_manager.clone().start_health_check_loop();
    let security_state = Arc::new(RwLock::new(security_config));
    let zai_state = Arc::new(RwLock::new(zai_config));
    let provider_rr = Arc::new(AtomicUsize::new(0));
    let zai_vision_mcp_state = Arc::new(crate::proxy::zai_vision_mcp::ZaiVisionMcpState::new());
    let experimental_state = Arc::new(RwLock::new(experimental_config));
    let debug_logging_state = Arc::new(RwLock::new(debug_logging));
    let is_running_state = Arc::new(RwLock::new(false));

    let only_raw_quota_models_state = Arc::new(tokio::sync::RwLock::new(only_raw_quota_models));
    let image_account_ids = token_manager.enabled_account_ids();
    let image_account_count = image_account_ids.len();
    let image_scheduler = build_image_scheduler(
        image_account_ids,
        image_scheduler_config.per_account_concurrency,
    );
    token_manager.register_image_scheduler(&image_scheduler);
    tracing::info!(
        account_count = image_account_count,
        per_account_concurrency = image_scheduler_config.per_account_concurrency,
        capacity = image_scheduler.available_slots(),
        "Image scheduler initialized"
    );

    let state = AppState {
        token_manager: token_manager.clone(),
        custom_mapping: custom_mapping_state.clone(),
        request_timeout,
        thought_signature_map: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        upstream_proxy: proxy_state.clone(),
        upstream: {
            let u = Arc::new(crate::proxy::upstream::client::UpstreamClient::new(
                Some(upstream_proxy.clone()),
                Some(proxy_pool_manager.clone()),
            ));
            // Initialize User-Agent override
            if user_agent_override.is_some() {
                u.set_user_agent_override(user_agent_override).await;
            }
            u
        },
        zai: zai_state.clone(),
        provider_rr: provider_rr.clone(),
        zai_vision_mcp: zai_vision_mcp_state,
        monitor: monitor.clone(),
        experimental: experimental_state.clone(),
        debug_logging: debug_logging_state.clone(),
        switching: Arc::new(RwLock::new(false)),
        integration: integration.clone(),
        account_service: Arc::new(crate::modules::account_service::AccountService::new(
            integration.clone(),
        )),
        security: security_state.clone(),
        cloudflared_state: cloudflared_state.clone(),
        is_running: is_running_state.clone(),
        port,
        proxy_pool_state: proxy_pool_state.clone(),
        proxy_pool_manager: proxy_pool_manager.clone(),
        only_raw_quota_models: only_raw_quota_models_state.clone(),
        image_scheduler,
    };
    StartupParts {
        state,
        custom_mapping_state,
        proxy_state,
        proxy_pool_state,
        proxy_pool_manager,
        security_state,
        zai_state,
        experimental_state,
        debug_logging_state,
        is_running_state,
        only_raw_quota_models_state,
        cloudflared_state,
        token_manager,
    }
}

fn bind_listener(host: &str, port: u16) -> Result<tokio::net::TcpListener, String> {
    // Bind address (uses socket2 with SO_REUSEADDR and dual-stack IPv4/IPv6 support)
    let listener = super::bind::bind_tcp_listener(&host, port)?;
    let display_host = if host == "0.0.0.0" || host == "::" || host == "[::]" {
        "0.0.0.0 / [::] (IPv4/IPv6 Dual-Stack)".to_string()
    } else if host.contains(':') && !host.starts_with('[') {
        format!("[{}]", host)
    } else {
        host.to_string()
    };
    tracing::info!(
        "API proxy server started on http://{}:{}",
        display_host,
        port
    );
    Ok(listener)
}

fn spawn_serve_loop(
    app: axum::Router,
    listener: tokio::net::TcpListener,
    server_cancel_token: tokio_util::sync::CancellationToken,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        use hyper::server::conn::http1;
        use hyper_util::rt::TokioIo;
        use hyper_util::service::TowerToHyperService;

        loop {
            tokio::select! {
                _ = server_cancel_token.cancelled() => {
                    tracing::info!("Proxy server received termination signal, releasing port");
                    break;
                }
                res = listener.accept() => {
                    match res {
                        Ok((stream, remote_addr)) => {
                            let io = TokioIo::new(stream);

                            // If IPv4-mapped IPv6 address (e.g. ::ffff:192.168.1.1), normalize to native IPv4
                            let normalized_remote_addr = match remote_addr {
                                std::net::SocketAddr::V6(v6_addr) => {
                                    if let Some(v4) = v6_addr.ip().to_ipv4_mapped() {
                                        std::net::SocketAddr::V4(std::net::SocketAddrV4::new(v4, v6_addr.port()))
                                    } else {
                                        std::net::SocketAddr::V6(v6_addr)
                                    }
                                }
                                v4_addr => v4_addr,
                            };

                            // Inject ConnectInfo (for resolving real IP)
                            use tower::ServiceExt;
                            use hyper::body::Incoming;
                            let app_with_info = app.clone().map_request(move |mut req: axum::http::Request<Incoming>| {
                                req.extensions_mut().insert(axum::extract::ConnectInfo(normalized_remote_addr));
                                req
                            });

                            let service = TowerToHyperService::new(app_with_info);
                            let conn_cancel_token = server_cancel_token.clone();

                            tokio::task::spawn(async move {
                                let conn = http1::Builder::new()
                                    .serve_connection(io, service)
                                    .with_upgrades();
                                tokio::pin!(conn);

                                tokio::select! {
                                    res = conn.as_mut() => {
                                        if let Err(err) = res {
                                            debug!("Connection processing ended or encountered error: {:?}", err);
                                        }
                                    }
                                    _ = conn_cancel_token.cancelled() => {
                                        // Global shutdown signal: notify Hyper to gracefully terminate connections
                                        conn.as_mut().graceful_shutdown();
                                        // Allow up to 500ms grace period before closing socket
                                        // Justification: best-effort guarded wait; a timeout or inner failure is logged
                                        crate::error::record_ignored(tokio::time::timeout(std::time::Duration::from_millis(500), conn).await, "timeout wait");
                                    }
                                }
                            });
                        }
                        Err(e) => {
                            if server_cancel_token.is_cancelled() {
                                break;
                            }
                            error!("Failed to accept incoming connection: {:?}", e);
                        }
                    }
                }
            }
        }
    })
}

impl AxumServer {
    /// Start Axum server
    pub async fn start(
        host: String,
        port: u16,
        token_manager: Arc<TokenManager>,
        custom_mapping: std::collections::HashMap<String, String>,
        request_timeout: u64,
        upstream_proxy: crate::proxy::config::UpstreamProxyConfig,
        user_agent_override: Option<String>,
        security_config: crate::proxy::ProxySecurityConfig,
        zai_config: crate::proxy::ZaiConfig,
        monitor: Arc<crate::proxy::monitor::ProxyMonitor>,
        experimental_config: crate::proxy::config::ExperimentalConfig,
        debug_logging: crate::proxy::config::DebugLoggingConfig,

        integration: crate::modules::integration::SystemManager,
        cloudflared_state: Arc<crate::commands::cloudflared::CloudflaredState>,
        proxy_pool_config: crate::proxy::config::ProxyPoolConfig, // [NEW]
        only_raw_quota_models: bool,
        image_scheduler_config: crate::proxy::config::ImageSchedulerConfig,
    ) -> Result<(Self, tokio::task::JoinHandle<()>), String> {
        // Phase 1: shared state (scheduler, pools, upstream client)
        let parts = build_startup_parts(
            port,
            token_manager,
            custom_mapping,
            request_timeout,
            upstream_proxy,
            user_agent_override,
            security_config,
            zai_config,
            monitor,
            experimental_config,
            debug_logging,
            integration,
            cloudflared_state,
            proxy_pool_config,
            only_raw_quota_models,
            image_scheduler_config,
        )
        .await;

        // Phase 2: route tables
        let proxy_routes = super::router_proxy::build_proxy_routes(&parts.state);
        let admin_routes = super::router_admin::build_admin_routes(&parts.state);
        let app = super::router_proxy::assemble_app(&parts.state, proxy_routes, admin_routes);

        // Phase 3: bind listener
        let listener = bind_listener(&host, port)?;

        // Phase 4: serve loop
        let cancel_token = tokio_util::sync::CancellationToken::new();
        let server_instance = Self {
            cancel_token: cancel_token.clone(),
            custom_mapping: parts.custom_mapping_state.clone(),
            proxy_state: parts.proxy_state,
            upstream: parts.state.upstream.clone(),
            security_state: parts.security_state,
            zai_state: parts.zai_state,
            experimental: parts.experimental_state.clone(),
            debug_logging: parts.debug_logging_state.clone(),
            cloudflared_state: parts.cloudflared_state,
            is_running: parts.is_running_state,
            token_manager: parts.token_manager,
            proxy_pool_state: parts.proxy_pool_state,
            proxy_pool_manager: parts.proxy_pool_manager,
            only_raw_quota_models: parts.only_raw_quota_models_state,
        };
        let handle = spawn_serve_loop(app, listener, cancel_token);

        Ok((server_instance, handle))
    }
}
