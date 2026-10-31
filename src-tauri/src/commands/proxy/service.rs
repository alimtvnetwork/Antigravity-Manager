use super::*;

/// Start the anti-generation service (Tauri command)
#[tauri::command]
pub async fn start_proxy_service(
    config: ProxyConfig,
    state: State<'_, ProxyServiceState>,
    cf_state: State<'_, crate::commands::cloudflared::CloudflaredState>,
    app_handle: tauri::AppHandle,
) -> Result<ProxyStatus, String> {
    internal_start_proxy_service(
        config,
        &state,
        crate::modules::integration::SystemManager::Desktop(app_handle),
        Arc::new(cf_state.inner().clone()),
    )
    .await
}

struct StartingGuard(Arc<AtomicBool>);
impl Drop for StartingGuard {
    fn drop(&mut self) {
        self.0.store(false, Ordering::SeqCst);
    }
}

/// Internally start anti-generation service logic (decoupled version)
pub async fn internal_start_proxy_service(
    config: ProxyConfig,
    state: &ProxyServiceState,
    integration: crate::modules::integration::SystemManager,
    cloudflared_state: Arc<crate::commands::cloudflared::CloudflaredState>,
) -> Result<ProxyStatus, String> {
    // 1. Check status and lock
    {
        let instance_lock = state.instance.read().await;
        if instance_lock.is_some() {
            return Err("Service is already running".to_string());
        }
    }

    // 2. Check whether it is starting (to prevent deadlock & concurrent startup)
    if state
        .starting
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return Err("The service is starting, please wait...".to_string());
    }

    // Use a custom Drop guard to ensure that the starting state is reset regardless of success or failure
    let _starting_guard = StartingGuard(state.starting.clone());

    // Ensure monitor exists
    {
        let mut monitor_lock = state.monitor.write().await;
        if monitor_lock.is_none() {
            let app_handle =
                if let crate::modules::integration::SystemManager::Desktop(ref h) = integration {
                    Some(h.clone())
                } else {
                    None
                };
            *monitor_lock = Some(Arc::new(ProxyMonitor::new(1000, app_handle)));
        }
        // Sync enabled state from config
        if let Some(monitor) = monitor_lock.as_ref() {
            monitor.set_enabled(config.enable_logging);
            monitor.set_capture_health_logs(config.capture_health_logs);
        }
    }

    let _monitor = state.monitor.read().await.as_ref().unwrap().clone();

    // Check and start the management server if it is not already running
    ensure_admin_server(
        config.clone(),
        state,
        integration.clone(),
        cloudflared_state.clone(),
    )
    .await?;

    // 2. [FIX] Reuse the token manager of the management server (single instance to solve the problem of hot update synchronization)
    let token_manager = {
        let admin_lock = state.admin_server.read().await;
        admin_lock
            .as_ref()
            .unwrap()
            .axum_server
            .token_manager
            .clone()
    };

    // Synchronize configuration to running TokenManager
    token_manager.start_auto_cleanup().await;
    token_manager
        .update_sticky_config(config.scheduling.clone())
        .await;

    // [NEW] Load circuit breaker configuration (loaded from main configuration)
    let app_config = crate::modules::config::load_app_config()
        .unwrap_or_else(|_| crate::models::AppConfig::new());
    token_manager
        .update_circuit_breaker_config(app_config.circuit_breaker)
        .await;

    // 🆕 [FIX #820] Restore fixed account mode settings
    if let Some(ref account_id) = config.preferred_account_id {
        token_manager
            .set_preferred_account(Some(account_id.clone()))
            .await;
        tracing::info!("🔒 [FIX #820] Fixed account mode restored: {}", account_id);
    }

    // 3. Load account
    let active_accounts = token_manager.load_accounts().await.unwrap_or(0);

    if active_accounts == 0 {
        let zai_enabled = config.zai.enabled
            && !matches!(config.zai.dispatch_mode, crate::proxy::ZaiDispatchMode::Off);
        if !zai_enabled {
            tracing::warn!("If there is no available account, the anti-generation logic will be suspended. Please add it through the management interface.");
            return Ok(ProxyStatus {
                running: false,
                port: config.port,
                base_url: format!("http://127.0.0.1:{}", config.port),
                active_accounts: 0,
            });
        }
    }

    let mut instance_lock = state.instance.write().await;
    let admin_lock = state.admin_server.read().await;
    let axum_server = admin_lock
        .as_ref()
        .expect("admin server must exist after ensure_admin_server")
        .axum_server
        .clone();

    // Create a service instance (logical startup). No more saving fake server_handles:
    // The real handle to the listening task is already held by AdminServerInstance (see ensure_admin_server).
    let instance = ProxyServiceInstance {
        config: config.clone(),
        token_manager: token_manager.clone(),
        axum_server: axum_server.clone(),
    };

    // [FIX] Ensure the server is logically running
    axum_server.set_running(true).await;

    *instance_lock = Some(instance);

    // After successful startup, it is OK for guard to end here and reset starting
    // But in fact, we can directly turn it off manually, or trust guard
    Ok(ProxyStatus {
        running: true,
        port: config.port,
        base_url: format!("http://127.0.0.1:{}", config.port),
        active_accounts,
    })
}

/// Make sure the management server is running
pub async fn ensure_admin_server(
    config: ProxyConfig,
    state: &ProxyServiceState,
    integration: crate::modules::integration::SystemManager,
    cloudflared_state: Arc<crate::commands::cloudflared::CloudflaredState>,
) -> Result<(), String> {
    let mut admin_lock = state.admin_server.write().await;
    if admin_lock.is_some() {
        return Ok(());
    }

    crate::proxy::config::update_global_audit_config(
        config.experimental.payload_storage_mode.clone(),
        config.experimental.log_retention_days,
        config.experimental.thinking_store_enabled,
        config.experimental.thinking_retention_days,
        Some(config.experimental.thinking_max_memory_turns),
    );
    crate::proxy::config::update_global_compression_level(
        config.experimental.compression_level.clone(),
        config.experimental.enable_usage_scaling,
    );

    // Ensure monitor exists
    let monitor = {
        let mut monitor_lock = state.monitor.write().await;
        if monitor_lock.is_none() {
            let app_handle =
                if let crate::modules::integration::SystemManager::Desktop(ref h) = integration {
                    Some(h.clone())
                } else {
                    None
                };
            *monitor_lock = Some(Arc::new(ProxyMonitor::new(1000, app_handle)));
        }
        monitor_lock.as_ref().unwrap().clone()
    };

    // The default empty TokenManager is used for the management interface
    let app_data_dir = crate::modules::account::get_data_dir()?;
    let token_manager = Arc::new(TokenManager::new(app_data_dir));
    // [NEW] Load account data, otherwise the management interface statistics will be 0
    // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
    crate::error::record_ignored(token_manager.load_accounts().await, "load_accounts");

    let (axum_server, server_handle) = match crate::proxy::AxumServer::start(
        config.get_bind_address().to_string(),
        config.port,
        token_manager,
        config.custom_mapping.clone(),
        config.request_timeout,
        config.upstream_proxy.clone(),
        config.user_agent_override.clone(),
        crate::proxy::ProxySecurityConfig::from_proxy_config(&config),
        config.zai.clone(),
        monitor,
        config.experimental.clone(),
        config.debug_logging.clone(),
        integration.clone(),
        cloudflared_state,
        config.proxy_pool.clone(),
        config.only_raw_quota_models,
        config.image_scheduler.clone(),
    )
    .await
    {
        Ok((server, handle)) => (server, handle),
        Err(e) => return Err(format!("Failed to start management server: {}", e)),
    };

    *admin_lock = Some(AdminServerInstance {
        axum_server,
        server_handle,
    });

    // [NEW] Initialize global Thinking Budget configuration
    crate::proxy::update_thinking_budget_config(config.thinking_budget.clone());
    // [NEW] Initialize global system prompt word configuration
    crate::proxy::update_global_system_prompt_config(config.global_system_prompt.clone());
    // [NEW] Initialize global image thinking mode configuration
    crate::proxy::update_image_thinking_mode(config.image_thinking_mode.clone());
    // [NEW] Initialize global compression level configuration
    crate::proxy::config::update_global_compression_level(
        config.experimental.compression_level.clone(),
        config.experimental.enable_usage_scaling,
    );
    crate::proxy::config::update_global_audit_config(
        config.experimental.payload_storage_mode.clone(),
        config.experimental.log_retention_days,
        config.experimental.thinking_store_enabled,
        config.experimental.thinking_retention_days,
        Some(config.experimental.thinking_max_memory_turns),
    );

    Ok(())
}

/// Stop anti-generation service
#[tauri::command]
pub async fn stop_proxy_service(state: State<'_, ProxyServiceState>) -> Result<(), String> {
    let mut instance_lock = state.instance.write().await;

    if instance_lock.is_none() {
        return Ok(());
    }

    // Stop the Axum server (only logical stop, do not kill the process)
    if let Some(instance) = instance_lock.take() {
        instance.token_manager.abort_background_tasks().await;
        instance.axum_server.set_running(false).await;
        // The instance.axum_server.stop() call has been removed to prevent killing the Admin Server
    }

    Ok(())
}
