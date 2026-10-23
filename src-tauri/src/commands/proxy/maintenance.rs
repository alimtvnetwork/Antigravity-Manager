use super::*;

/// Generate API Key
#[tauri::command]
pub fn generate_api_key() -> String {
    format!("sk-{}", uuid::Uuid::new_v4().simple())
}

/// Reload account (called when the main application adds/delete an account)
#[tauri::command]
pub async fn reload_proxy_accounts(state: State<'_, ProxyServiceState>) -> Result<usize, String> {
    let instance_lock = state.instance.read().await;

    if let Some(instance) = instance_lock.as_ref() {
        // [FIX #820] Clear stale session bindings before reloading accounts
        // This ensures that after switching accounts in the UI, API requests
        // won't be routed to the previously bound (wrong) account
        instance.token_manager.clear_all_sessions();

        // Reload account
        let count = instance
            .token_manager
            .load_accounts()
            .await
            .map_err(|e| format!("Failed to reload account: {}", e))?;
        Ok(count)
    } else {
        Ok(0)
    }
}

/// Update model mapping table (hot update)
#[tauri::command]
pub async fn update_model_mapping(
    config: ProxyConfig,
    state: State<'_, ProxyServiceState>,
) -> Result<(), String> {
    let instance_lock = state.instance.read().await;

    // 1. If the service is running, immediately update the mapping in memory (currently only the RwLock of anthropic_mapping is updated here,
    // Later, you can let resolve_model_route directly read the full config as needed)
    if let Some(instance) = instance_lock.as_ref() {
        instance.axum_server.update_mapping(&config).await;
        tracing::debug!("The backend service has received the full model mapping configuration");
    }

    // 2. Regardless of whether it is running or not, it is saved to global configuration persistence.
    let mut app_config = crate::modules::config::load_app_config()?;
    app_config.proxy.custom_mapping = config.custom_mapping;
    crate::modules::config::save_app_config(&app_config)?;

    Ok(())
}

fn join_base_url(base: &str, path: &str) -> String {
    let base = base.trim_end_matches('/');
    let path = if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{}", path)
    };
    format!("{}{}", base, path)
}

fn extract_model_ids(value: &serde_json::Value) -> Vec<String> {
    let mut out = Vec::new();

    fn push_from_item(out: &mut Vec<String>, item: &serde_json::Value) {
        match item {
            serde_json::Value::String(s) => out.push(s.to_string()),
            serde_json::Value::Object(map) => {
                if let Some(id) = map.get("id").and_then(|v| v.as_str()) {
                    out.push(id.to_string());
                } else if let Some(name) = map.get("name").and_then(|v| v.as_str()) {
                    out.push(name.to_string());
                }
            }
            _ => {}
        }
    }

    match value {
        serde_json::Value::Array(arr) => {
            for item in arr {
                push_from_item(&mut out, item);
            }
        }
        serde_json::Value::Object(map) => {
            if let Some(serde_json::Value::Array(arr)) = map.get("data") {
                for item in arr {
                    push_from_item(&mut out, item);
                }
            }
            if let Some(models) = map.get("models") {
                match models {
                    serde_json::Value::Array(arr) => {
                        for item in arr {
                            push_from_item(&mut out, item);
                        }
                    }
                    other => push_from_item(&mut out, other),
                }
            }
        }
        _ => {}
    }

    out
}

/// Fetch available models from the configured z.ai Anthropic-compatible API (`/v1/models`).
#[tauri::command]
pub async fn fetch_zai_models(
    zai: crate::proxy::ZaiConfig,
    upstream_proxy: crate::proxy::config::UpstreamProxyConfig,
    request_timeout: u64,
) -> Result<Vec<String>, String> {
    if zai.base_url.trim().is_empty() {
        return Err("z.ai base_url is empty".to_string());
    }
    if zai.api_key.trim().is_empty() {
        return Err("z.ai api_key is not set".to_string());
    }

    let url = join_base_url(&zai.base_url, "/v1/models");

    let mut builder =
        reqwest::Client::builder().timeout(Duration::from_secs(request_timeout.max(5)));
    if upstream_proxy.enabled && !upstream_proxy.url.is_empty() {
        let proxy = reqwest::Proxy::all(&upstream_proxy.url)
            .map_err(|e| format!("Invalid upstream proxy url: {}", e))?;
        builder = builder.proxy(proxy);
    }
    let client = builder
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

    let resp = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", zai.api_key))
        .header("x-api-key", zai.api_key)
        .header("anthropic-version", "2023-06-01")
        .header("accept", "application/json")
        .send()
        .await
        .map_err(|e| format!("Upstream request failed: {}", e))?;

    let status = resp.status();
    let text = resp
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;

    if !status.is_success() {
        let preview = crate::proxy::mappers::common_utils::safe_truncate_str(&text, 4000);
        return Err(format!("Upstream returned {}: {}", status, preview));
    }

    let json: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| format!("Invalid JSON response: {}", e))?;
    let mut models = extract_model_ids(&json);
    models.retain(|s| !s.trim().is_empty());
    models.sort();
    models.dedup();
    Ok(models)
}

/// Get the current scheduling configuration
#[tauri::command]
pub async fn get_proxy_scheduling_config(
    state: State<'_, ProxyServiceState>,
) -> Result<crate::proxy::sticky_config::StickySessionConfig, String> {
    let instance_lock = state.instance.read().await;
    if let Some(instance) = instance_lock.as_ref() {
        Ok(instance.token_manager.get_sticky_config().await)
    } else {
        let cfg = crate::modules::config::load_app_config().unwrap_or_default();
        Ok(cfg.proxy.scheduling)
    }
}

/// Update scheduling configuration
#[tauri::command]
pub async fn update_proxy_scheduling_config(
    state: State<'_, ProxyServiceState>,
    config: crate::proxy::sticky_config::StickySessionConfig,
) -> Result<(), String> {
    let instance_lock = state.instance.read().await;
    if let Some(instance) = instance_lock.as_ref() {
        instance
            .token_manager
            .update_sticky_config(config.clone())
            .await;
    }

    let mut app_config = crate::modules::config::load_app_config()
        .map_err(|e| format!("Failed to load configuration: {}", e))?;
    app_config.proxy.scheduling = config;
    crate::modules::config::save_app_config(&app_config)
        .map_err(|e| format!("Failed to save configuration: {}", e))?;

    Ok(())
}

/// Clear all session sticky bindings
#[tauri::command]
pub async fn clear_proxy_session_bindings(
    state: State<'_, ProxyServiceState>,
) -> Result<(), String> {
    let instance_lock = state.instance.read().await;
    if let Some(instance) = instance_lock.as_ref() {
        instance.token_manager.clear_all_sessions();
    }
    Ok(())
}

// ===== [FIX #820] Fixed account mode command =====

/// Set priority account (fixed account mode)
/// Pass in account_id to enable fixed mode, pass in null/empty string to restore polling mode
#[tauri::command]
pub async fn set_preferred_account(
    state: State<'_, ProxyServiceState>,
    account_id: Option<String>,
) -> Result<(), String> {
    let instance_lock = state.instance.read().await;
    // Filter empty strings to None
    let cleaned_id = account_id.filter(|s| !s.trim().is_empty());

    if let Some(instance) = instance_lock.as_ref() {
        // 1. Update memory status
        instance
            .token_manager
            .set_preferred_account(cleaned_id.clone())
            .await;
    }

    // 2. Persistence to configuration file (fixes Issue #820 automatic shutdown problem)
    let mut app_config = crate::modules::config::load_app_config()
        .map_err(|e| format!("Failed to load configuration: {}", e))?;
    app_config.proxy.preferred_account_id = cleaned_id.clone();
    crate::modules::config::save_app_config(&app_config)
        .map_err(|e| format!("Failed to save configuration: {}", e))?;

    if let Some(ref id) = cleaned_id {
        tracing::info!(
            "🔒 [FIX #820] Fixed account mode enabled and persisted: {}",
            id
        );
    } else {
        tracing::info!("🔄 [FIX #820] Round-robin mode enabled and persisted");
    }

    Ok(())
}

/// Get the currently preferred account ID
#[tauri::command]
pub async fn get_preferred_account(
    state: State<'_, ProxyServiceState>,
) -> Result<Option<String>, String> {
    let instance_lock = state.instance.read().await;
    if let Some(instance) = instance_lock.as_ref() {
        Ok(instance.token_manager.get_preferred_account().await)
    } else {
        let cfg = crate::modules::config::load_app_config().unwrap_or_default();
        Ok(cfg.proxy.preferred_account_id)
    }
}

/// Clear the current limit record of the specified account
#[tauri::command]
pub async fn clear_proxy_rate_limit(
    state: State<'_, ProxyServiceState>,
    account_id: String,
) -> Result<bool, String> {
    let instance_lock = state.instance.read().await;
    if let Some(instance) = instance_lock.as_ref() {
        Ok(instance.token_manager.clear_rate_limit(&account_id))
    } else {
        Ok(false)
    }
}

/// Clear all throttling records
#[tauri::command]
pub async fn clear_all_proxy_rate_limits(
    state: State<'_, ProxyServiceState>,
) -> Result<(), String> {
    let instance_lock = state.instance.read().await;
    if let Some(instance) = instance_lock.as_ref() {
        instance.token_manager.clear_all_rate_limits();
    }
    Ok(())
}

/// Trigger a health check on all agents and return the updated configuration
#[tauri::command]
pub async fn check_proxy_health(
    state: State<'_, ProxyServiceState>,
) -> Result<ProxyPoolConfig, String> {
    let instance_lock = state.instance.read().await;
    if let Some(instance) = instance_lock.as_ref() {
        let pool_state = instance.axum_server.proxy_pool_state.clone();
        let manager = crate::proxy::proxy_pool::ProxyPoolManager::new(pool_state.clone());

        manager.health_check().await?;

        // Return the updated config from memory
        let config = pool_state.read().await;
        Ok(config.clone())
    } else {
        // Offline health check: check configured proxies even when proxy server is stopped
        let cfg = crate::modules::config::load_app_config().unwrap_or_default();
        let pool_state =
            std::sync::Arc::new(tokio::sync::RwLock::new(cfg.proxy.proxy_pool.clone()));
        let manager = crate::proxy::proxy_pool::ProxyPoolManager::new(pool_state.clone());

        manager.health_check().await?;

        let updated = pool_state.read().await.clone();
        if let Ok(mut app_cfg) = crate::modules::config::load_app_config() {
            app_cfg.proxy.proxy_pool = updated.clone();
            // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
            crate::error::record_ignored(
                crate::modules::config::save_app_config(&app_cfg),
                "save_app_config",
            );
        }
        Ok(updated)
    }
}
