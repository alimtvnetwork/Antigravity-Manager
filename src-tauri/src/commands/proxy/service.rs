use super::*;

/// 启动反代服务 (Tauri 命令)
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

/// 内部启动反代服务逻辑 (解耦版本)
pub async fn internal_start_proxy_service(
    config: ProxyConfig,
    state: &ProxyServiceState,
    integration: crate::modules::integration::SystemManager,
    cloudflared_state: Arc<crate::commands::cloudflared::CloudflaredState>,
) -> Result<ProxyStatus, String> {
    // 1. 检查状态并加锁
    {
        let instance_lock = state.instance.read().await;
        if instance_lock.is_some() {
            return Err("服务已在运行中".to_string());
        }
    }

    // 2. 检查是否正在启动中 (防止死锁 & 并发启动)
    if state
        .starting
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return Err("服务正在启动中，请稍候...".to_string());
    }

    // 使用自定义 Drop guard 确保无论成功失败都会重置 starting 状态
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

    // 檢查並啟動管理服務器（如果尚未運行）
    ensure_admin_server(
        config.clone(),
        state,
        integration.clone(),
        cloudflared_state.clone(),
    )
    .await?;

    // 2. [FIX] 复用管理服务器的 Token 管理器 (单实例，解决热更新同步问题)
    let token_manager = {
        let admin_lock = state.admin_server.read().await;
        admin_lock
            .as_ref()
            .unwrap()
            .axum_server
            .token_manager
            .clone()
    };

    // 同步配置到运行中的 TokenManager
    token_manager.start_auto_cleanup().await;
    token_manager
        .update_sticky_config(config.scheduling.clone())
        .await;

    // [NEW] 加载熔断配置 (从主配置加载)
    let app_config = crate::modules::config::load_app_config()
        .unwrap_or_else(|_| crate::models::AppConfig::new());
    token_manager
        .update_circuit_breaker_config(app_config.circuit_breaker)
        .await;

    // 🆕 [FIX #820] 恢复固定账号模式设置
    if let Some(ref account_id) = config.preferred_account_id {
        token_manager
            .set_preferred_account(Some(account_id.clone()))
            .await;
        tracing::info!("🔒 [FIX #820] Fixed account mode restored: {}", account_id);
    }

    // 3. 加載賬號
    let active_accounts = token_manager.load_accounts().await.unwrap_or(0);

    if active_accounts == 0 {
        let zai_enabled = config.zai.enabled
            && !matches!(config.zai.dispatch_mode, crate::proxy::ZaiDispatchMode::Off);
        if !zai_enabled {
            tracing::warn!("沒有可用賬號，反代邏輯將暫停，請通過管理界面添加。");
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

    // 创建服务实例（逻辑启动）。不再保存假的 server_handle：
    // 监听任务的真实句柄已由 AdminServerInstance 持有（见 ensure_admin_server）。
    let instance = ProxyServiceInstance {
        config: config.clone(),
        token_manager: token_manager.clone(),
        axum_server: axum_server.clone(),
    };

    // [FIX] Ensure the server is logically running
    axum_server.set_running(true).await;

    *instance_lock = Some(instance);

    // 成功启动后，guard 在这里结束并重置 starting 是 OK 的
    // 但其实我们可以直接手动掉，或者相信 guard
    Ok(ProxyStatus {
        running: true,
        port: config.port,
        base_url: format!("http://127.0.0.1:{}", config.port),
        active_accounts,
    })
}

/// 确保管理服务器正在运行
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

    // 默认空 TokenManager 用于管理界面
    let app_data_dir = crate::modules::account::get_data_dir()?;
    let token_manager = Arc::new(TokenManager::new(app_data_dir));
    // [NEW] 加载账号数据，否则管理界面统计为 0
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
        Err(e) => return Err(format!("启动管理服务器失败: {}", e)),
    };

    *admin_lock = Some(AdminServerInstance {
        axum_server,
        server_handle,
    });

    // [NEW] 初始化全局 Thinking Budget 配置
    crate::proxy::update_thinking_budget_config(config.thinking_budget.clone());
    // [NEW] 初始化全局系统提示词配置
    crate::proxy::update_global_system_prompt_config(config.global_system_prompt.clone());
    // [NEW] 初始化全局图像思维模式配置
    crate::proxy::update_image_thinking_mode(config.image_thinking_mode.clone());
    // [NEW] 初始化全局压缩等级配置
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

/// 停止反代服务
#[tauri::command]
pub async fn stop_proxy_service(state: State<'_, ProxyServiceState>) -> Result<(), String> {
    let mut instance_lock = state.instance.write().await;

    if instance_lock.is_none() {
        return Ok(());
    }

    // 停止 Axum 服务器 (仅逻辑停止，不杀死进程)
    if let Some(instance) = instance_lock.take() {
        instance.token_manager.abort_background_tasks().await;
        instance.axum_server.set_running(false).await;
        // 已移除 instance.axum_server.stop() 调用，防止杀死 Admin Server
    }

    Ok(())
}
