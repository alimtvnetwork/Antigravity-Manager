use super::*;

pub(crate) async fn internal_refresh_account_quota(
    app: &tauri::AppHandle,
    account: &mut Account,
) -> Result<QuotaData, String> {
    modules::logger::log_info(&format!(
        "Automatically trigger quota refresh: {}",
        account.email
    ));

    // Using queries with retries (Shared logic)
    match modules::account::fetch_quota_with_retry(account).await {
        Ok(quota) => {
            // Update account quota
            // Justification: quota cache write is auxiliary; the fetched quota is returned to the caller
            crate::error::record_ignored(
                modules::update_account_quota(&account.id, quota.clone()),
                "cache refreshed account quota",
            );
            // Update tray menu
            crate::modules::tray::update_tray_menus(app);
            Ok(quota)
        }
        Err(e) => {
            modules::logger::log_warn(&format!(
                "Automatic quota refresh failed ( {} ): {}",
                account.email, e
            ));
            Err(e.to_string())
        }
    }
}

/// Query account quota
#[tauri::command]
pub async fn fetch_account_quota(
    app: tauri::AppHandle,
    proxy_state: tauri::State<'_, crate::commands::proxy::ProxyServiceState>,
    account_id: String,
) -> crate::error::AppResult<QuotaData> {
    modules::logger::log_info(&format!("Manual refresh quota request: {}", account_id));
    let mut account =
        modules::load_account(&account_id).map_err(crate::error::AppError::Account)?;

    // Using queries with retries (Shared logic)
    let mut quota = modules::account::fetch_quota_with_retry(&mut account).await?;

    // 4. Update account quota
    modules::update_account_quota(&account_id, quota.clone())
        .map_err(crate::error::AppError::Account)?;

    crate::modules::tray::update_tray_menus(&app);

    // 5. Synchronize to the running anti-generation service (if it is started)
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

/// Refresh all account quotas (implemented internally)
pub async fn refresh_all_quotas_internal(
    proxy_state: &crate::commands::proxy::ProxyServiceState,
    app_handle: Option<tauri::AppHandle>,
) -> Result<RefreshStats, String> {
    let stats = modules::account::refresh_all_quotas_logic().await?;

    // Sync to a running anti-generation service (if started)
    let instance_lock = proxy_state.instance.read().await;
    if let Some(instance) = instance_lock.as_ref() {
        // Justification: proxy reload is best-effort sync; the refresh stats are returned regardless
        crate::error::record_ignored(
            instance.token_manager.reload_all_accounts().await,
            "reload all accounts in proxy token manager",
        );
    }

    // Send global refresh events to the UI (if needed)
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

/// Refresh all account quotas (Tauri Command)
#[tauri::command]
pub async fn refresh_all_quotas(
    proxy_state: tauri::State<'_, crate::commands::proxy::ProxyServiceState>,
    app_handle: tauri::AppHandle,
) -> Result<RefreshStats, String> {
    refresh_all_quotas_internal(&proxy_state, Some(app_handle)).await
}
