use super::*;

#[tauri::command]
pub async fn load_config() -> Result<AppConfig, String> {
    modules::load_app_config()
}

/// Save configuration
#[tauri::command]
pub async fn save_config(
    app: tauri::AppHandle,
    proxy_state: tauri::State<'_, crate::commands::proxy::ProxyServiceState>,
    config: AppConfig,
) -> Result<(), String> {
    modules::save_app_config(&config)?;

    // Notification tray configuration updated
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

    // Synchronous health check log capture switch
    let monitor_lock = proxy_state.monitor.read().await;
    if let Some(monitor) = monitor_lock.as_ref() {
        monitor.set_capture_health_logs(config.proxy.capture_health_logs);
    }

    // Hot update running services
    let instance_lock = proxy_state.instance.read().await;
    if let Some(instance) = instance_lock.as_ref() {
        // Update model mapping
        instance.axum_server.update_mapping(&config.proxy).await;
        // Updated to expose only real quota model switch
        instance
            .axum_server
            .update_only_raw_quota_models(config.proxy.only_raw_quota_models)
            .await;
        // Update upstream proxy
        instance
            .axum_server
            .update_proxy(config.proxy.upstream_proxy.clone())
            .await;
        // Update security policy (auth)
        instance.axum_server.update_security(&config.proxy).await;
        // Update z.ai configuration
        instance.axum_server.update_zai(&config.proxy).await;
        // Update experimental configuration
        instance
            .axum_server
            .update_experimental(&config.proxy)
            .await;
        // Update debug log configuration
        instance
            .axum_server
            .update_debug_logging(&config.proxy)
            .await;
        // [NEW] Update User-Agent configuration
        instance.axum_server.update_user_agent(&config.proxy).await;
        // Update Thinking Budget configuration
        crate::proxy::update_thinking_budget_config(config.proxy.thinking_budget.clone());
        // [NEW] Update global system prompt word configuration
        crate::proxy::update_global_system_prompt_config(config.proxy.global_system_prompt.clone());
        // [NEW] Update global image mindset configuration
        crate::proxy::update_image_thinking_mode(config.proxy.image_thinking_mode.clone());
        // [NEW] Update global compression level configuration
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
        // Update proxy pool configuration
        instance
            .axum_server
            .update_proxy_pool(config.proxy.proxy_pool.clone())
            .await;
        // Update circuit breaker configuration
        instance
            .token_manager
            .update_circuit_breaker_config(config.circuit_breaker.clone())
            .await;
        tracing::debug!("Hot update anti-generation service configuration synchronized");
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
