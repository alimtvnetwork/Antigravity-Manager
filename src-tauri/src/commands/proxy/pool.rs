use super::*;

/// 获取当前内存中的代理池状态
#[tauri::command]
pub async fn get_proxy_pool_config(
    state: State<'_, ProxyServiceState>,
) -> Result<ProxyPoolConfig, String> {
    let instance_lock = state.instance.read().await;
    if let Some(instance) = instance_lock.as_ref() {
        let config = instance.axum_server.proxy_pool_state.read().await;
        Ok(config.clone())
    } else {
        let cfg = crate::modules::config::load_app_config().unwrap_or_default();
        Ok(cfg.proxy.proxy_pool)
    }
}
