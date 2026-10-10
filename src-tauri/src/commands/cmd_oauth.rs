use super::*;

#[tauri::command]
pub async fn start_oauth_login(
    app_handle: tauri::AppHandle,
    oauth_client_key: Option<String>,
) -> Result<Account, String> {
    modules::logger::log_info("开始 OAuth 授权流程...");
    let service = modules::account_service::AccountService::new(
        crate::modules::integration::SystemManager::Desktop(app_handle.clone()),
    );

    let mut account = service.start_oauth_login(oauth_client_key).await?;

    // 自动触发刷新额度
    // Justification: quota auto-refresh after OAuth login is opportunistic; the login already succeeded
    crate::error::record_ignored(
        internal_refresh_account_quota(&app_handle, &mut account).await,
        "auto-refresh quota after OAuth login",
    );

    // Reload token pool
    // Justification: proxy pool reload after OAuth login is best-effort sync; the login already succeeded
    crate::error::record_ignored(
        crate::commands::proxy::reload_proxy_accounts(
            app_handle.state::<crate::commands::proxy::ProxyServiceState>(),
        )
        .await,
        "reload proxy accounts after OAuth login",
    );

    Ok(account)
}

/// 完成 OAuth 授权（不自动打开浏览器）
#[tauri::command]
pub async fn complete_oauth_login(app_handle: tauri::AppHandle) -> Result<Account, String> {
    modules::logger::log_info("完成 OAuth 授权流程 (manual)...");
    let service = modules::account_service::AccountService::new(
        crate::modules::integration::SystemManager::Desktop(app_handle.clone()),
    );

    let mut account = service.complete_oauth_login().await?;

    // 自动触发刷新额度
    // Justification: quota auto-refresh after manual OAuth completion is opportunistic; the login already succeeded
    crate::error::record_ignored(
        internal_refresh_account_quota(&app_handle, &mut account).await,
        "auto-refresh quota after manual OAuth completion",
    );

    // Reload token pool
    // Justification: proxy pool reload after manual OAuth completion is best-effort sync; the login already succeeded
    crate::error::record_ignored(
        crate::commands::proxy::reload_proxy_accounts(
            app_handle.state::<crate::commands::proxy::ProxyServiceState>(),
        )
        .await,
        "reload proxy accounts after manual OAuth completion",
    );

    Ok(account)
}

/// 预生成 OAuth 授权链接 (不打开浏览器)
#[tauri::command]
pub async fn prepare_oauth_url(
    app_handle: tauri::AppHandle,
    oauth_client_key: Option<String>,
) -> Result<String, String> {
    let service = modules::account_service::AccountService::new(
        crate::modules::integration::SystemManager::Desktop(app_handle.clone()),
    );
    service.prepare_oauth_url(oauth_client_key).await
}

#[tauri::command]
pub async fn cancel_oauth_login() -> Result<(), String> {
    modules::oauth_server::cancel_oauth_flow();
    Ok(())
}

/// 手动提交 OAuth Code (用于 Docker/远程环境无法自动回调时)
#[tauri::command]
pub async fn submit_oauth_code(code: String, state: Option<String>) -> Result<(), String> {
    modules::logger::log_info("收到手动提交 OAuth Code 请求");
    modules::oauth_server::submit_oauth_code(code, state).await
}

#[tauri::command]
pub async fn list_oauth_clients(
) -> Result<Vec<crate::modules::oauth::OAuthClientDescriptor>, String> {
    crate::modules::oauth::list_oauth_clients()
}

#[tauri::command]
pub async fn get_active_oauth_client() -> Result<String, String> {
    crate::modules::oauth::get_active_oauth_client_key()
}

#[tauri::command]
pub async fn set_active_oauth_client(client_key: String) -> Result<(), String> {
    crate::modules::oauth::set_active_oauth_client_key(&client_key)
}

// --- 导入命令 ---
