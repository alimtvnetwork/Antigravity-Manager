use super::*;

#[tauri::command]
pub async fn load_config() -> Result<AppConfig, String> {
    modules::load_app_config()
}

/// 保存配置
#[tauri::command]
pub async fn save_config(
    app: tauri::AppHandle,
    proxy_state: tauri::State<'_, crate::commands::proxy::ProxyServiceState>,
    config: AppConfig,
) -> Result<(), String> {
    modules::save_app_config(&config)?;

    // 通知托盘配置已更新
    // Justification: UI event emission; the config was already saved
    crate::error::record_ignored(
        app.emit("config://updated", ()),
        "emit config updated event",
    );

    // Sync global in-memory config regardless of proxy runtime state
    crate::proxy::update_thinking_budget_config(config.proxy.thinking_budget.clone());
    crate::proxy::update_global_system_prompt_config(config.proxy.global_system_prompt.clone());
    crate::proxy::update_image_thinking_mode(config.proxy.image_thinking_mode.clone());
    crate::proxy::config::update_global_compression_level(
        config.proxy.experimental.compression_level.clone(),
        config.proxy.experimental.enable_usage_scaling,
    );
    crate::proxy::config::update_global_thresholds(
        config.proxy.experimental.context_compression_threshold_l1,
        config.proxy.experimental.context_compression_threshold_l2,
        config.proxy.experimental.context_compression_threshold_l3,
    );
    crate::proxy::config::update_global_audit_config(
        config.proxy.experimental.payload_storage_mode.clone(),
        config.proxy.experimental.log_retention_days,
        config.proxy.experimental.thinking_store_enabled,
        config.proxy.experimental.thinking_retention_days,
        Some(config.proxy.experimental.thinking_max_memory_turns),
    );

    // 同步健康检查日志捕获开关
    let monitor_lock = proxy_state.monitor.read().await;
    if let Some(monitor) = monitor_lock.as_ref() {
        monitor.set_capture_health_logs(config.proxy.capture_health_logs);
    }

    // 热更新正在运行的服务
    let instance_lock = proxy_state.instance.read().await;
    if let Some(instance) = instance_lock.as_ref() {
        // 更新模型映射
        instance.axum_server.update_mapping(&config.proxy).await;
        // 更新仅暴露真实配额模型开关
        instance
            .axum_server
            .update_only_raw_quota_models(config.proxy.only_raw_quota_models)
            .await;
        // 更新上游代理
        instance
            .axum_server
            .update_proxy(config.proxy.upstream_proxy.clone())
            .await;
        // 更新安全策略 (auth)
        instance.axum_server.update_security(&config.proxy).await;
        // 更新 z.ai 配置
        instance.axum_server.update_zai(&config.proxy).await;
        // 更新实验性配置
        instance
            .axum_server
            .update_experimental(&config.proxy)
            .await;
        // 更新调试日志配置
        instance
            .axum_server
            .update_debug_logging(&config.proxy)
            .await;
        // [NEW] 更新 User-Agent 配置
        instance.axum_server.update_user_agent(&config.proxy).await;
        // 更新 Thinking Budget 配置
        crate::proxy::update_thinking_budget_config(config.proxy.thinking_budget.clone());
        // [NEW] 更新全局系统提示词配置
        crate::proxy::update_global_system_prompt_config(config.proxy.global_system_prompt.clone());
        // [NEW] 更新全局图像思维模式配置
        crate::proxy::update_image_thinking_mode(config.proxy.image_thinking_mode.clone());
        // [NEW] 更新全局压缩等级配置
        crate::proxy::config::update_global_compression_level(
            config.proxy.experimental.compression_level.clone(),
            config.proxy.experimental.enable_usage_scaling,
        );
        crate::proxy::config::update_global_audit_config(
            config.proxy.experimental.payload_storage_mode.clone(),
            config.proxy.experimental.log_retention_days,
            config.proxy.experimental.thinking_store_enabled,
            config.proxy.experimental.thinking_retention_days,
            Some(config.proxy.experimental.thinking_max_memory_turns),
        );
        crate::proxy::config::update_global_thresholds(
            config.proxy.experimental.context_compression_threshold_l1,
            config.proxy.experimental.context_compression_threshold_l2,
            config.proxy.experimental.context_compression_threshold_l3,
        );
        // 更新代理池配置
        instance
            .axum_server
            .update_proxy_pool(config.proxy.proxy_pool.clone())
            .await;
        // 更新熔断配置
        instance
            .token_manager
            .update_circuit_breaker_config(config.circuit_breaker.clone())
            .await;
        tracing::debug!("已同步热更新反代服务配置");
    }

    if config.auto_profile_switcher.is_enabled {
        tauri::async_runtime::spawn(async move {
            // Justification: opportunistic rotation check runs detached; it logs its own errors
            crate::error::record_ignored(
                crate::modules::auto_switcher::check_and_rotate_if_needed().await,
                "trigger auto-switcher rotation check",
            );
        });
    }

    Ok(())
}
