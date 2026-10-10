use super::*;

async fn internal_refresh_account_quota(
    app: &tauri::AppHandle,
    account: &mut Account,
) -> Result<QuotaData, String> {
    modules::logger::log_info(&format!("自动触发刷新配额: {}", account.email));

    // 使用带重试的查询 (Shared logic)
    match modules::account::fetch_quota_with_retry(account).await {
        Ok(quota) => {
            // 更新账号配额
            // Justification: quota cache write is auxiliary; the fetched quota is returned to the caller
            crate::error::record_ignored(
                modules::update_account_quota(&account.id, quota.clone()),
                "cache refreshed account quota",
            );
            // 更新托盘菜单
            crate::modules::tray::update_tray_menus(app);
            Ok(quota)
        }
        Err(e) => {
            modules::logger::log_warn(&format!("自动刷新配额失败 ({}): {}", account.email, e));
            Err(e.to_string())
        }
    }
}

/// 查询账号配额
#[tauri::command]
pub async fn fetch_account_quota(
    app: tauri::AppHandle,
    proxy_state: tauri::State<'_, crate::commands::proxy::ProxyServiceState>,
    account_id: String,
) -> crate::error::AppResult<QuotaData> {
    modules::logger::log_info(&format!("手动刷新配额请求: {}", account_id));
    let mut account =
        modules::load_account(&account_id).map_err(crate::error::AppError::Account)?;

    // 使用带重试的查询 (Shared logic)
    let mut quota = modules::account::fetch_quota_with_retry(&mut account).await?;

    // 4. 更新账号配额
    modules::update_account_quota(&account_id, quota.clone())
        .map_err(crate::error::AppError::Account)?;

    crate::modules::tray::update_tray_menus(&app);

    // 5. 同步到运行中的反代服务（如果已启动）
    let instance_lock = proxy_state.instance.read().await;
    if let Some(instance) = instance_lock.as_ref() {
        if quota.models.iter().any(|model| model.percentage > 0) {
            instance.token_manager.clear_rate_limit_memory(&account_id);
        }
        // Justification: proxy token-manager reload is best-effort sync; the quota was already fetched and stored
        crate::error::record_ignored(
            instance.token_manager.reload_account(&account_id).await,
            "reload account in proxy token manager",
        );

        // Blend TokenManager lockout state only for models that are still 0%
        if let Some(reset_secs) = instance
            .token_manager
            .get_rate_limit_reset_seconds(&account_id)
        {
            if reset_secs > 0 {
                let reset_iso = chrono::DateTime::<chrono::Utc>::from_timestamp(
                    chrono::Utc::now().timestamp() + reset_secs as i64,
                    0,
                )
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_default();

                for model in &mut quota.models {
                    if model.percentage == 0 {
                        model.reset_time = reset_iso.clone();
                    }
                }
            }
        }
    }

    Ok(quota)
}

pub use modules::account::RefreshStats;

/// 刷新所有账号配额 (内部实现)
pub async fn refresh_all_quotas_internal(
    proxy_state: &crate::commands::proxy::ProxyServiceState,
    app_handle: Option<tauri::AppHandle>,
) -> Result<RefreshStats, String> {
    let stats = modules::account::refresh_all_quotas_logic().await?;

    // 同步到运行中的反代服务（如果已启动）
    let instance_lock = proxy_state.instance.read().await;
    if let Some(instance) = instance_lock.as_ref() {
        // Justification: proxy reload is best-effort sync; the refresh stats are returned regardless
        crate::error::record_ignored(
            instance.token_manager.reload_all_accounts().await,
            "reload all accounts in proxy token manager",
        );
    }

    // 发送全局刷新事件给 UI (如果需要)
    if let Some(handle) = app_handle {
        use tauri::Emitter;
        // Justification: UI event emission; a missing listener is expected and harmless
        crate::error::record_ignored(
            handle.emit("accounts://refreshed", ()),
            "emit accounts refreshed event",
        );
    }

    Ok(stats)
}

/// 刷新所有账号配额 (Tauri Command)
#[tauri::command]
pub async fn refresh_all_quotas(
    proxy_state: tauri::State<'_, crate::commands::proxy::ProxyServiceState>,
    app_handle: tauri::AppHandle,
) -> Result<RefreshStats, String> {
    refresh_all_quotas_internal(&proxy_state, Some(app_handle)).await
}
