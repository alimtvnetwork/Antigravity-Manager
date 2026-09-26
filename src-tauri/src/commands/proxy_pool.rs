use crate::commands::proxy::ProxyServiceState;
use std::collections::HashMap;
use tauri::State;

/// Bind an account to a specific proxy
#[tauri::command]
pub async fn bind_account_proxy(
    state: State<'_, ProxyServiceState>,
    account_id: String,
    proxy_id: String,
) -> Result<(), String> {
    let instance_lock = state.instance.read().await;
    if let Some(instance) = instance_lock.as_ref() {
        instance
            .axum_server
            .proxy_pool_manager
            .bind_account_to_proxy(account_id, proxy_id)
            .await
    } else {
        // Fallback: persist binding to app configuration directly when proxy service is stopped
        let mut cfg = crate::modules::config::load_app_config().unwrap_or_default();
        cfg.proxy
            .proxy_pool
            .account_bindings
            .insert(account_id, proxy_id);
        crate::modules::config::save_app_config(&cfg).map_err(|e| e.to_string())?;
        Ok(())
    }
}

/// Unbind an account from its proxy
#[tauri::command]
pub async fn unbind_account_proxy(
    state: State<'_, ProxyServiceState>,
    account_id: String,
) -> Result<(), String> {
    let instance_lock = state.instance.read().await;
    if let Some(instance) = instance_lock.as_ref() {
        instance
            .axum_server
            .proxy_pool_manager
            .unbind_account_proxy(account_id)
            .await;
        Ok(())
    } else {
        // Fallback: remove binding from app configuration directly when proxy service is stopped
        let mut cfg = crate::modules::config::load_app_config().unwrap_or_default();
        cfg.proxy.proxy_pool.account_bindings.remove(&account_id);
        crate::modules::config::save_app_config(&cfg).map_err(|e| e.to_string())?;
        Ok(())
    }
}

/// Get the proxy binding for a specific account
#[tauri::command]
pub async fn get_account_proxy_binding(
    state: State<'_, ProxyServiceState>,
    account_id: String,
) -> Result<Option<String>, String> {
    let instance_lock = state.instance.read().await;
    if let Some(instance) = instance_lock.as_ref() {
        Ok(instance
            .axum_server
            .proxy_pool_manager
            .get_account_binding(&account_id))
    } else {
        let cfg = crate::modules::config::load_app_config().unwrap_or_default();
        Ok(cfg
            .proxy
            .proxy_pool
            .account_bindings
            .get(&account_id)
            .cloned())
    }
}

/// Get all account proxy bindings
#[tauri::command]
pub async fn get_all_account_bindings(
    state: State<'_, ProxyServiceState>,
) -> Result<HashMap<String, String>, String> {
    let instance_lock = state.instance.read().await;
    if let Some(instance) = instance_lock.as_ref() {
        // Since get_all_bindings returns a DashMap ref or clone, we need to convert it to HashMap for serialization
        Ok(instance
            .axum_server
            .proxy_pool_manager
            .get_all_bindings_snapshot())
    } else {
        let cfg = crate::modules::config::load_app_config().unwrap_or_default();
        Ok(cfg.proxy.proxy_pool.account_bindings)
    }
}
